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
    xdg_folder(&home, dirs.as_deref())
}
fn xdg_folder(home: &Path, config: Option<&str>) -> Option<PathBuf> {
    // Missing or disabled XDG entries do not turn $HOME into a template library.
    let value = config?
        .lines()
        .find_map(|line| line.trim_start().strip_prefix("XDG_TEMPLATES_DIR="))?;
    let value = value.trim().strip_prefix('"')?.strip_suffix('"')?;
    let path = if value == "$HOME" || value == "${HOME}" {
        home.to_path_buf()
    } else if let Some(rest) = value
        .strip_prefix("$HOME/")
        .or_else(|| value.strip_prefix("${HOME}/"))
    {
        home.join(rest)
    } else {
        PathBuf::from(value)
    };
    (path.is_absolute() && path != home).then_some(path)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xdg_templates_respects_localized_custom_and_disabled_locations() {
        let home = Path::new("/home/test");
        assert_eq!(
            xdg_folder(
                home,
                Some("XDG_TEMPLATES_DIR=\"$HOME/Plantillas personales\"")
            ),
            Some(home.join("Plantillas personales"))
        );
        assert_eq!(
            xdg_folder(home, Some("XDG_TEMPLATES_DIR=\"/srv/Plantillas\"")),
            Some(PathBuf::from("/srv/Plantillas"))
        );
        for value in [
            None,
            Some(""),
            Some("XDG_TEMPLATES_DIR=\"$HOME\""),
            Some("XDG_TEMPLATES_DIR=\"relative\""),
        ] {
            assert_eq!(xdg_folder(home, value), None);
        }
    }
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

/// A small process cache lives outside the user's template directory.
/// Keying by size/mtime avoids stale previews after replacing a template file.
pub fn preview(path: &Path) -> Result<std::sync::Arc<crate::PageImage>, String> {
    use std::sync::{Arc, Mutex, OnceLock};
    type Key = (PathBuf, u64, Option<std::time::SystemTime>);
    type Entry = (Key, Arc<crate::PageImage>);
    static CACHE: OnceLock<Mutex<std::collections::VecDeque<Entry>>> = OnceLock::new();
    let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
    if metadata.len() > 32 * 1024 * 1024 {
        return Err("Template exceeds the preview size limit".into());
    }
    let key = (path.to_path_buf(), metadata.len(), metadata.modified().ok());
    let cache = CACHE.get_or_init(Default::default);
    if let Some((_, image)) = cache.lock().unwrap().iter().find(|(k, _)| *k == key) {
        return Ok(image.clone());
    }
    let mut session = open(path)?;
    let image = Arc::new(crate::render_page_premultiplied(&mut session, 0, 0.18)?);
    let mut cache = cache.lock().unwrap();
    cache.push_back((key, image.clone()));
    while cache.len() > 24 {
        cache.pop_front();
    }
    Ok(image)
}
