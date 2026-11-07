//! Personal templates are ordinary files, opened by the engine as untitled copies.
use std::path::{Path, PathBuf};
use wordcraft_engine::Session;
pub fn folder() -> Option<PathBuf> {
    if let Some(path) = crate::preferences::template_folder() {
        return Some(path);
    }
    // Parse only the single XDG assignment; never evaluate shell configuration.
    let home = PathBuf::from(std::env::var_os("HOME")?);
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let dirs = std::fs::read_to_string(config.join("user-dirs.dirs")).ok();
    let value = dirs
        .as_deref()
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("XDG_TEMPLATES_DIR="))
        })
        .map(|v| {
            v.trim()
                .trim_matches('"')
                .replace("$HOME", &home.to_string_lossy())
        });
    let path = value
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("Templates"));
    (path.is_absolute() && path != home).then_some(path)
}
pub fn list(folder: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(folder)
        .map_err(|e| e.to_string())?
        .take(4096)
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file()
            && path.extension().and_then(|v| v.to_str()).is_some_and(|v| {
                ["docx", "odt", "dotx", "ott"]
                    .iter()
                    .any(|ext| v.eq_ignore_ascii_case(ext))
            })
        {
            files.push(path)
        }
    }
    files.sort_by_key(|p| p.file_name().map(|v| v.to_string_lossy().to_lowercase()));
    Ok(files)
}
pub fn open(path: &Path) -> Result<Session, String> {
    let path = path
        .to_str()
        .ok_or_else(|| wordcraft_gnome::t!("The path is not UTF-8").to_owned())?;
    let mut session = Session::new(wordcraft_doc::Document::new());
    session
        .run("file.newFromTemplate", &serde_json::json!({"path":path}))
        .map_err(|e| e.to_string())?;
    Ok(session)
}
