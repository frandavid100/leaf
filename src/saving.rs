//! Save through WordCraft; only DOCX and ODT are exposed by this prototype.
use serde_json::json;
use std::path::Path;
use wordcraft_engine::Session;

/// Prevent silent loss through the pinned upstream ODT exporter. The recovery
/// copy is engine-native; this guard applies to document files and autosave.
pub fn requires_docx(session: &Session) -> bool {
    !session.doc.comments.is_empty()
        || !session.doc.core.category.is_empty()
        || crate::proofing_defaults::odt_loses_inheritance(session)
        || odt_structure_error(session).is_some()
}

pub fn format_preservation_error(session: &Session, path: &Path) -> Option<&'static str> {
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("odt"))
    {
        return None;
    }
    if !session.doc.comments.is_empty() {
        return Some(wordcraft_gnome::t!(
            "ODT cannot preserve comments with the current WordCraft exporter. Save as DOCX to keep them."
        ));
    }
    if crate::proofing_defaults::odt_loses_inheritance(session) {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve inherited language and review settings. The current ODT exporter flattens them."
        ));
    }
    if let Some(error) = odt_structure_error(session) {
        return Some(error);
    }
    (!session.doc.core.category.is_empty()).then(|| wordcraft_gnome::t!("Save as DOCX to preserve the document category. The current ODT exporter cannot keep it."))
}

fn odt_structure_error(session: &Session) -> Option<&'static str> {
    use wordcraft_doc::{
        InlineObject, PartKind, StoryRef,
        para::{COLUMN_BREAK, Wrap},
        section::SectionStart,
    };
    if session.doc.sections().iter().any(|(_, section)| {
        section.gutter != 0.0
            || section.valign != wordcraft_doc::section::PageVAlign::Top
            || section.rtl
            || section.text_direction != wordcraft_doc::props::TextDirection::Horizontal
            || section.line_numbers.is_some()
    }) {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve the section gutter, alignment, direction or line numbering."
        ));
    }
    if session.doc.sections().iter().skip(1).any(|(_, section)| {
        matches!(
            section.start,
            SectionStart::Continuous
                | SectionStart::EvenPage
                | SectionStart::OddPage
                | SectionStart::NextColumn
        )
    }) || session
        .doc
        .plain_text(StoryRef::Body)
        .contains(COLUMN_BREAK)
    {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve column breaks and continuous, even-page or odd-page sections."
        ));
    }
    if session
        .doc
        .parts
        .values()
        .any(|p| p.kind == PartKind::Footnote)
    {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve footnotes. The current ODT exporter cannot keep their structure."
        ));
    }
    if wordcraft_engine::cmd::review::has_format_changes(&session.doc) {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve tracked formatting changes. The current ODT exporter cannot keep them."
        ));
    }
    if !wordcraft_engine::cmd::review::move_changes(session).is_empty() {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve tracked moves. The current ODT exporter cannot keep their association."
        ));
    }
    let (mut equation, mut toc, mut floating, mut reference) = (false, false, false, false);
    for story in
        std::iter::once(StoryRef::Body).chain(session.doc.parts.keys().copied().map(StoryRef::Part))
    {
        for path in session.doc.para_paths(story) {
            for o in &session.doc.para(story, &path).unwrap().objects {
                match o {
                    InlineObject::Equation { .. } => equation = true,
                    InlineObject::FieldStart { instr, .. } | InlineObject::Field { instr, .. } => {
                        reference |= instr.split_whitespace().next().is_some_and(|s| {
                            [
                                "REF", "PAGEREF", "SEQ", "AUTHOR", "TITLE", "FILENAME", "DATE",
                                "TIME",
                            ]
                            .iter()
                            .any(|name| s.eq_ignore_ascii_case(name))
                        });
                        toc |= instr
                            .split_whitespace()
                            .next()
                            .is_some_and(|s| s.eq_ignore_ascii_case("TOC"));
                    }
                    InlineObject::Image { float, .. } if float.wrap != Wrap::Inline => {
                        floating = true
                    }
                    _ => {}
                }
            }
        }
    }
    if equation {
        Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve editable equations. The current ODT exporter cannot keep their structure."
        ))
    } else if toc {
        Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve the automatic table of contents. The current ODT exporter only keeps static entries."
        ))
    } else if reference {
        Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve automatic fields, cross-references and caption numbering. The current ODT exporter only keeps their text."
        ))
    } else if floating {
        Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve floating image placement and text wrapping. The current ODT exporter makes these images inline."
        ))
    } else {
        None
    }
}

pub fn save(session: &mut Session, path: &Path) -> Result<(), String> {
    if let Some(error) = format_preservation_error(session, path) {
        return Err(error.into());
    }
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("");
    if !["docx", "odt"].iter().any(|x| ext.eq_ignore_ascii_case(x)) {
        return Err(wordcraft_gnome::t!("Use a .docx or .odt extension").into());
    }
    let path = path
        .to_str()
        .ok_or(wordcraft_gnome::t!("The path is not UTF-8"))?;
    // Prepare derived values in a disposable session. A failed write leaves the
    // live document, source history and revision exactly as they were.
    let mut prepared = Session::new(session.doc.clone());
    prepared.sel = session.sel.clone();
    prepared.page_hint = session.page_hint;
    prepared.path = Some(path.into());
    crate::references::refresh_derived(&mut prepared)?;
    let changed = prepared.doc != session.doc;
    let old_doc = session.doc.clone();
    let old_selection = session.sel.clone();
    let old_path = session.path.clone();
    let old_dirty = session.dirty;
    if changed {
        session.doc = prepared.doc;
        session.sel = prepared.sel;
        session.relayout();
    }
    let result = match session.run("file.saveAs", &json!({"path":path})) {
        Ok(value) => value,
        Err(error) => {
            if changed {
                session.doc = old_doc;
                session.sel = old_selection;
                session.relayout();
            }
            session.path = old_path;
            session.dirty = old_dirty;
            return Err(error.to_string());
        }
    };
    if changed {
        session.touch();
        session.dirty = false;
    }
    if result["saved"] != true {
        return Err(
            wordcraft_gnome::t!("WordCraft did not confirm that the document was saved").into(),
        );
    }
    Ok(())
}

/// Do not silently choose a format from a dialog filter: the filename is explicit.
pub fn destination(path: &Path) -> Result<std::path::PathBuf, String> {
    if path.extension().is_none() {
        return Ok(path.with_extension("docx"));
    }
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("");
    if ["docx", "odt"].iter().any(|x| ext.eq_ignore_ascii_case(x)) {
        Ok(path.to_path_buf())
    } else {
        Err(wordcraft_gnome::t!("The file name must end in .docx or .odt").into())
    }
}
