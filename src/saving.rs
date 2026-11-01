//! Save through WordCraft; only DOCX and ODT are exposed by this prototype.
use serde_json::json;
use std::path::Path;
use wordcraft_engine::Session;

/// Prevent silent loss through the pinned upstream ODT exporter. The recovery
/// copy is engine-native; this guard applies to document files and autosave.
pub fn requires_docx(session: &Session) -> bool {
    !session.doc.comments.is_empty() || !session.doc.core.category.is_empty()
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
    (!session.doc.core.category.is_empty()).then(|| wordcraft_gnome::t!("Save as DOCX to preserve the document category. The current ODT exporter cannot keep it."))
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
    let result = session
        .run("file.saveAs", &json!({"path": path}))
        .map_err(|e| e.to_string())?;
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
