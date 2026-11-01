//! Save through WordCraft; only DOCX and ODT are exposed by this prototype.
use std::path::Path;
use serde_json::json;
use wordcraft_engine::Session;

pub fn save(session: &mut Session, path: &Path) -> Result<(), String> {
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("");
    if !["docx", "odt"].iter().any(|x| ext.eq_ignore_ascii_case(x)) {
        return Err(wordcraft_gnome::t!("Use a .docx or .odt extension").into());
    }
    let path = path.to_str().ok_or(wordcraft_gnome::t!("The path is not UTF-8"))?;
    let result = session.run("file.saveAs", &json!({"path": path}))
        .map_err(|e| e.to_string())?;
    if result["saved"] != true { return Err(wordcraft_gnome::t!("WordCraft did not confirm that the document was saved").into()); }
    Ok(())
}

/// Do not silently choose a format from a dialog filter: the filename is explicit.
pub fn destination(path: &Path) -> Result<std::path::PathBuf, String> {
    if path.extension().is_none() { return Ok(path.with_extension("docx")); }
    let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("");
    if ["docx", "odt"].iter().any(|x| ext.eq_ignore_ascii_case(x)) {
        Ok(path.to_path_buf())
    } else { Err(wordcraft_gnome::t!("The file name must end in .docx or .odt").into()) }
}
