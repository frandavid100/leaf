//! Save through WordCraft; only DOCX and ODT are exposed by this prototype.
use serde_json::json;
use std::path::Path;
use wordcraft_engine::Session;

/// Prevent silent loss through the pinned upstream ODT exporter. The recovery
/// copy is engine-native; this guard applies to document files and autosave.
pub fn requires_docx(session: &Session) -> bool {
    !session.doc.core.category.is_empty()
        || crate::proofing_defaults::odt_loses_inheritance(session)
        || crate::styles::odt_loses_next(session)
        || odt_structure_error(session).is_some()
}

pub fn format_preservation_error(session: &Session, path: &Path) -> Option<&'static str> {
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("odt"))
    {
        return None;
    }
    if crate::proofing_defaults::odt_loses_inheritance(session) {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve inherited language and review settings. The current ODT exporter flattens them."
        ));
    }
    if crate::styles::odt_loses_next(session) {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve the following paragraph style. The current ODT exporter cannot keep it."
        ));
    }
    if let Some(error) = odt_structure_error(session) {
        return Some(error);
    }
    (!session.doc.core.category.is_empty()).then(|| wordcraft_gnome::t!("Save as DOCX to preserve the document category. The current ODT exporter cannot keep it."))
}

fn odt_structure_error(session: &Session) -> Option<&'static str> {
    use wordcraft_doc::{InlineObject, StoryRef, para::COLUMN_BREAK, section::SectionStart};
    if crate::captions::requires_docx(&session.doc) {
        return Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve caption associations and hidden caption text."
        ));
    }
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
    let (mut equation, mut effects, mut reference, mut toc) = (false, false, false, false);
    for story in
        std::iter::once(StoryRef::Body).chain(session.doc.parts.keys().copied().map(StoryRef::Part))
    {
        for path in session.doc.para_paths(story) {
            for o in &session.doc.para(story, &path).unwrap().objects {
                match o {
                    InlineObject::Equation { linear, math, .. } => {
                        let text = if math.nodes.is_empty() {
                            linear.clone()
                        } else {
                            wordcraft_doc::math::to_linear(&math.nodes)
                        };
                        equation |= !math.omml.is_empty()
                            || math.linear
                            || (!math.nodes.is_empty()
                                && wordcraft_doc::math::parse_linear(&text) != math.nodes);
                    }
                    InlineObject::FieldStart { instr, .. } | InlineObject::Field { instr, .. } => {
                        toc |= instr
                            .split_whitespace()
                            .next()
                            .is_some_and(|name| name.eq_ignore_ascii_case("TOC"));
                        reference |= instr.split_whitespace().next().is_some_and(|s| {
                            [
                                "REF", "PAGEREF", "SEQ", "AUTHOR", "TITLE", "FILENAME", "DATE",
                                "TIME",
                            ]
                            .iter()
                            .any(|name| s.eq_ignore_ascii_case(name))
                        });
                    }
                    InlineObject::Image { fx, .. } => effects |= *fx != Default::default(),
                    _ => {}
                }
            }
        }
    }
    if equation {
        Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve the original equation structure and formatting."
        ))
    } else if reference || toc {
        Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve all table of contents and automatic field options."
        ))
    } else if effects {
        Some(wordcraft_gnome::t!(
            "Save as DOCX to preserve picture effects. The current ODT exporter cannot keep them."
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
