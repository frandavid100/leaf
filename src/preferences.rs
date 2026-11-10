//! Host preferences, separate from the pinned WordCraft engine.
use std::path::PathBuf;
pub fn config_root() -> PathBuf {
    #[cfg(windows)]
    return windows_data_root("APPDATA").join("Leaf");
    #[cfg(not(windows))]
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        })
        .join("wordcraft-gnome")
}
pub fn dictionary_folder() -> Option<PathBuf> {
    read()["dictionary_folder"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
}
pub fn set_dictionary_folder(folder: Option<&std::path::Path>) -> Result<(), String> {
    let value = match folder {
        Some(p) => serde_json::Value::String(
            p.to_str()
                .ok_or(wordcraft_gnome::t!(
                    "The folder path cannot be represented as UTF-8."
                ))?
                .to_owned(),
        ),
        None => serde_json::Value::Null,
    };
    update("dictionary_folder", value)
}
pub fn recovery_root() -> PathBuf {
    #[cfg(windows)]
    return windows_data_root("LOCALAPPDATA").join("Leaf/recovery");
    #[cfg(not(windows))]
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("FLATPAK_ID")
                .and_then(|_| std::env::var_os("XDG_DATA_HOME"))
                .map(|p| PathBuf::from(p).join("state"))
        })
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/state")
        })
        .join("wordcraft-gnome/recovery")
}
/// 0 disabled, 1 recovery (default), 2 document file.
pub fn autosave_mode() -> u32 {
    std::fs::read(config_root().join("preferences.json"))
        .ok()
        .and_then(|v| serde_json::from_slice::<serde_json::Value>(&v).ok())
        .and_then(|v| v["autosave"].as_u64())
        .filter(|v| *v <= 2)
        .unwrap_or(1) as u32
}
pub fn set_autosave_mode(mode: u32) -> Result<(), String> {
    if mode > 2 {
        return Err(wordcraft_gnome::t!("Unknown Autosave Mode").into());
    }
    update("autosave", serde_json::json!(mode))
}
fn read() -> serde_json::Value {
    std::fs::read(config_root().join("preferences.json"))
        .ok()
        .and_then(|v| serde_json::from_slice(&v).ok())
        .filter(|v: &serde_json::Value| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}))
}
fn update(key: &str, value: serde_json::Value) -> Result<(), String> {
    let dir = config_root();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut settings = read();
    settings[key] = value;
    let temp = dir.join(format!("preferences-{}.tmp", std::process::id()));
    std::fs::write(&temp, settings.to_string()).map_err(|e| e.to_string())?;
    std::fs::rename(temp, dir.join("preferences.json")).map_err(|e| e.to_string())
}
pub fn default_language() -> Option<String> {
    read()["language"].as_str().map(str::to_owned)
}
pub fn set_default_language(tag: &str) -> Result<(), String> {
    let tag = wordcraft_engine::proof::lang::normalize_tag(tag).ok_or(wordcraft_gnome::t!(
        "Use a language tag such as es-ES or en-GB."
    ))?;
    update("language", serde_json::json!(tag))
}
pub fn check_spelling() -> bool {
    read()["spelling"].as_bool().unwrap_or(true)
}
pub fn set_check_spelling(on: bool) -> Result<(), String> {
    update("spelling", serde_json::json!(on))
}

#[cfg(windows)]
fn windows_data_root(variable: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("USERPROFILE").map(|p| {
                PathBuf::from(p).join(if variable == "APPDATA" {
                    "AppData/Roaming"
                } else {
                    "AppData/Local"
                })
            })
        })
        .unwrap_or_else(std::env::temp_dir)
}

#[cfg(windows)]
pub fn system_user_name() -> String {
    // A Windows login is not necessarily the author's full name. Ask in Preferences.
    String::new()
}
#[cfg(not(windows))]
pub fn system_user_name() -> String {
    let uid = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines().find_map(|l| {
                l.strip_prefix("Uid:")
                    .and_then(|v| v.split_whitespace().next())
                    .map(str::to_owned)
            })
        });
    if let (Some(uid), Ok(passwd)) = (uid, std::fs::read_to_string("/etc/passwd")) {
        if let Some(name) = real_name_from_passwd(&passwd, &uid) {
            return name;
        }
    }
    String::new()
}
#[cfg(any(not(windows), test))]
fn real_name_from_passwd(passwd: &str, uid: &str) -> Option<String> {
    passwd.lines().find_map(|line| {
        let f: Vec<_> = line.split(':').collect();
        (f.len() > 4 && f[2] == uid)
            .then(|| f[4].split(',').next().unwrap_or("").trim().to_owned())
            .filter(|s| !s.is_empty())
    })
}
#[cfg(test)]
mod names {
    #[test]
    fn default_uses_full_name_without_turning_login_into_a_real_name() {
        assert_eq!(
            super::real_name_from_passwd(
                "david:x:1000:1000:David Prieto,Room:/home/david:/bin/bash",
                "1000"
            )
            .as_deref(),
            Some("David Prieto")
        );
        assert_eq!(
            super::real_name_from_passwd("david:x:1000:1000::/home/david:/bin/bash", "1000"),
            None
        );
        assert_eq!(super::real_name_from_passwd("broken", "1000"), None);
    }
}
pub fn user_name() -> String {
    read()["user_name"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(system_user_name)
}
pub fn set_user_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err(wordcraft_gnome::t!("Enter your name.").into());
    }
    update("user_name", serde_json::json!(name.trim()))
}
/// 0 follows the system, 1 light, 2 dark.
pub fn theme() -> u32 {
    read()["theme"].as_u64().filter(|n| *n <= 2).unwrap_or(0) as u32
}
pub fn set_theme(mode: u32) -> Result<(), String> {
    if mode > 2 {
        return Err("Unknown theme".into());
    }
    update("theme", serde_json::json!(mode))
}

pub fn template_folder() -> Option<PathBuf> {
    read()["template_folder"]
        .as_str()
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
}
pub fn set_template_folder(path: &std::path::Path) -> Result<(), String> {
    update(
        "template_folder",
        serde_json::json!(path.to_str().ok_or("The path is not UTF-8")?),
    )
}
pub fn reset_template_folder() -> Result<(), String> {
    update("template_folder", serde_json::Value::Null)
}

pub fn toolbar(group: &str) -> Vec<(String, bool)> {
    crate::toolbar::normalize(group, &read()["toolbar"][group])
}
pub fn toolbar_placements(group: &str) -> Vec<(String, crate::toolbar::Placement)> {
    crate::toolbar::placements(group, &read()["toolbar"][group])
}
pub fn set_toolbar(group: &str, order: &[(String, bool)]) -> Result<(), String> {
    set_toolbar_placements(
        group,
        &order
            .iter()
            .map(|(id, on)| {
                (
                    id.clone(),
                    if *on {
                        crate::toolbar::Placement::Adaptive
                    } else {
                        crate::toolbar::Placement::More
                    },
                )
            })
            .collect::<Vec<_>>(),
    )
}
pub fn set_toolbar_placements(
    group: &str,
    order: &[(String, crate::toolbar::Placement)],
) -> Result<(), String> {
    if crate::toolbar::tools(group).is_empty() {
        return Err("Unknown toolbar group".into());
    }
    let value = serde_json::Value::Array(
        order
            .iter()
            .map(|(id, mode)| serde_json::json!({"id":id,"placement":mode.key()}))
            .collect(),
    );
    let normalized = crate::toolbar::placements(group, &value);
    let mut settings = read()["toolbar"].as_object().cloned().unwrap_or_default();
    settings.insert(
        group.into(),
        serde_json::Value::Array(
            normalized
                .iter()
                .map(|(id, mode)| serde_json::json!({"id":id,"placement":mode.key()}))
                .collect(),
        ),
    );
    update("toolbar", serde_json::Value::Object(settings))
}
