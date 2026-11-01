//! Host preferences, separate from the pinned WordCraft engine.
use std::path::PathBuf;
pub fn config_root() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"))
        .join("wordcraft-gnome")
}
pub fn recovery_root() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("FLATPAK_ID").and_then(|_| std::env::var_os("XDG_DATA_HOME")).map(|p|PathBuf::from(p).join("state")))
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state"))
        .join("wordcraft-gnome/recovery")
}
/// 0 disabled, 1 recovery (default), 2 document file.
pub fn autosave_mode() -> u32 {
    std::fs::read(config_root().join("preferences.json")).ok()
        .and_then(|v| serde_json::from_slice::<serde_json::Value>(&v).ok())
        .and_then(|v| v["autosave"].as_u64()).filter(|v| *v <= 2).unwrap_or(1) as u32
}
pub fn set_autosave_mode(mode: u32) -> Result<(), String> {
    if mode > 2 { return Err(wordcraft_gnome::t!("Unknown Autosave Mode").into()); }
    update("autosave",serde_json::json!(mode))
}
fn read()->serde_json::Value{std::fs::read(config_root().join("preferences.json")).ok().and_then(|v|serde_json::from_slice(&v).ok()).filter(|v:&serde_json::Value|v.is_object()).unwrap_or_else(||serde_json::json!({}))}
fn update(key:&str,value:serde_json::Value)->Result<(),String>{
    let dir=config_root();std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let mut settings=read();settings[key]=value;let temp=dir.join(format!("preferences-{}.tmp",std::process::id()));
    std::fs::write(&temp,settings.to_string()).map_err(|e|e.to_string())?;std::fs::rename(temp,dir.join("preferences.json")).map_err(|e|e.to_string())
}
pub fn default_language()->Option<String>{read()["language"].as_str().map(str::to_owned)}
pub fn set_default_language(tag:&str)->Result<(),String>{let tag=wordcraft_engine::proof::lang::normalize_tag(tag).ok_or(wordcraft_gnome::t!("Use a language tag such as es-ES or en-GB."))?;update("language",serde_json::json!(tag))}
pub fn check_spelling()->bool{read()["spelling"].as_bool().unwrap_or(true)}
pub fn set_check_spelling(on:bool)->Result<(),String>{update("spelling",serde_json::json!(on))}
