//! Discover readable Hunspell sources; WordCraft still loads and checks dictionaries.
//! A private index combines imports, an explicitly chosen folder and bundled data.
//! It never changes the source files or widens the Flatpak sandbox.
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::SystemTime,
};
use wordcraft_engine::proof::{self, lang::ProofLang};

const MAX_ENTRIES: usize = 4096;
static NEXT_INDEX: AtomicU64 = AtomicU64::new(0);
static CURRENT: OnceLock<Mutex<Current>> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileStamp {
    path: PathBuf,
    len: u64,
    modified: Option<SystemTime>,
}
impl FileStamp {
    fn read(path: PathBuf) -> Option<Self> {
        let m = path.metadata().ok().filter(|m| m.is_file())?;
        Some(Self {
            path,
            len: m.len(),
            modified: m.modified().ok(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Pair {
    aff: FileStamp,
    dic: FileStamp,
}

fn pairs(dir: &Path) -> Result<BTreeMap<String, Pair>, String> {
    let mut result = BTreeMap::new();
    for (i, entry) in std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .enumerate()
    {
        if i == MAX_ENTRIES {
            return Err(
                wordcraft_gnome::t!("Choose a folder containing only dictionary files.").into(),
            );
        }
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_none_or(|s| s != "dic") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()).filter(|s| {
            !s.is_empty()
                && s.len() <= 128
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        }) else {
            continue;
        };
        let Some(aff) = FileStamp::read(path.with_extension("aff")) else {
            continue;
        };
        let Some(dic) = FileStamp::read(path.clone()) else {
            continue;
        };
        result.insert(stem.to_owned(), Pair { aff, dic });
    }
    Ok(result)
}

#[derive(Default, PartialEq, Eq)]
struct Inventory {
    folder: Option<PathBuf>,
    preferred: BTreeMap<String, Pair>,
    system: Vec<Pair>,
}
fn inventory(
    imports: &Path,
    folder: Option<&Path>,
    bundled: &Path,
    index: Option<&Path>,
) -> Inventory {
    let mut out = Inventory {
        folder: folder.map(Path::to_path_buf),
        ..Inventory::default()
    };
    // Explicit imports win; the authorized folder supplements them. Bundled
    // dictionaries remain available when a folder is removed or disappears.
    for dir in [Some(imports), folder, Some(bundled)].into_iter().flatten() {
        for (name, pair) in pairs(dir).unwrap_or_default() {
            out.preferred.entry(name).or_insert(pair);
        }
    }
    // Let the engine search its system locations itself. Fingerprinting these
    // files also notices newly installed dictionaries without restarting Leaf.
    for dir in proof::hunspell::search_dirs(None) {
        if dir == imports
            || Some(dir.as_path()) == folder
            || dir == bundled
            || Some(dir.as_path()) == index
        {
            continue;
        }
        out.system
            .extend(pairs(&dir).unwrap_or_default().into_values());
    }
    out.system.sort_by(|a, b| a.dic.path.cmp(&b.dic.path));
    out
}

struct Index(PathBuf);
impl Drop for Index {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
impl Index {
    fn build(files: &BTreeMap<String, Pair>) -> Result<Self, String> {
        let seq = NEXT_INDEX.fetch_add(1, Ordering::Relaxed);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "leaf-dictionaries-{}-{now}-{seq}",
            std::process::id()
        ));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&dir).map_err(|e| e.to_string())?;
        let index = Self(dir);
        for (name, pair) in files {
            for (ext, file) in [("aff", &pair.aff), ("dic", &pair.dic)] {
                let to = index.0.join(format!("{name}.{ext}"));
                let from = std::path::absolute(&file.path).map_err(|e| e.to_string())?;
                #[cfg(unix)]
                std::os::unix::fs::symlink(from, to).map_err(|e| e.to_string())?;
                #[cfg(not(unix))]
                std::fs::copy(from, to).map_err(|e| e.to_string())?;
            }
        }
        Ok(index)
    }
}
#[derive(Default)]
struct Current {
    inventory: Option<Inventory>,
    index: Option<Index>,
}

pub fn bundled() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent()?.parent().map(|p| p.join("share/hunspell")))
        .filter(|p| p.is_dir())
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/dictionaries"))
}
pub fn imports() -> PathBuf {
    crate::preferences::config_root().join("hunspell")
}
pub fn folder() -> Option<PathBuf> {
    crate::preferences::dictionary_folder()
}

fn update(chosen: Option<Option<&Path>>, force: bool) -> Result<bool, String> {
    let mut current = CURRENT
        .get_or_init(|| Mutex::new(Current::default()))
        .lock()
        .map_err(|e| e.to_string())?;
    let existing = folder();
    let selected = chosen.unwrap_or(existing.as_deref());
    if let Some(Some(path)) = chosen {
        if !path.is_absolute() || pairs(path)?.is_empty() {
            return Err(wordcraft_gnome::t!(
                "No matching Hunspell .aff and .dic pairs were found in this folder."
            )
            .into());
        }
    }
    let inventory = inventory(
        &imports(),
        selected,
        &bundled(),
        current.index.as_ref().map(|i| i.0.as_path()),
    );
    if chosen.is_none() && !force && current.inventory.as_ref() == Some(&inventory) {
        return Ok(false);
    }
    let index = Index::build(&inventory.preferred)?;
    if chosen.is_some() {
        crate::preferences::set_dictionary_folder(selected)?;
    }
    proof::set_dictionaries_folder(Some(index.0.clone()));
    current.inventory = Some(inventory);
    current.index = Some(index);
    Ok(true)
}
/// Rescan available sources; a forced refresh also reloads unchanged-size files
/// and engine-specific fallback locations such as LibreOffice language folders.
pub fn refresh(force: bool) -> Result<bool, String> {
    update(None, force)
}
/// Store only the folder returned by the native/portal chooser. Build the new
/// index before writing settings so a failed selection keeps the previous one.
pub fn use_folder(path: Option<&Path>) -> Result<(), String> {
    update(Some(path), true).map(|_| ())
}
pub fn shutdown() {
    if let Some(mut state) = CURRENT.get().and_then(|s| s.lock().ok()) {
        state.index.take();
        state.inventory = None;
    }
}

/// The actual selected dictionary, without exposing the disposable index path.
pub fn dictionary_path(tag: &str) -> Option<PathBuf> {
    let lang = ProofLang::from_tag(Some(tag));
    let path = if lang.is_english() {
        match proof::english_source(lang)? {
            proof::EnglishSource::Hunspell(p) => Some(p),
            _ => None,
        }
    } else {
        proof::installed_dictionaries()
            .into_iter()
            .find_map(|(l, p)| (l == lang).then_some(p).flatten())
            .or_else(|| proof::pack_dictionary_file(lang))
    }?;
    let indexed = CURRENT.get().and_then(|s| s.lock().ok()).is_some_and(|s| {
        s.index
            .as_ref()
            .is_some_and(|i| path.parent() == Some(i.0.as_path()))
    });
    Some(if indexed {
        std::fs::read_link(&path).unwrap_or(path)
    } else {
        path
    })
}

pub fn available(tag: &str) -> bool {
    !matches!(ProofLang::from_tag(Some(tag)), ProofLang::Other)
        && proof::missing_dictionary(Some(tag)).is_none()
        && proof::unreadable_dictionary(Some(tag)).is_none()
}

/// Add installed languages beyond the frontend's usual choices. A filename is
/// advertised only when the public engine actually selects that dictionary.
pub fn additional_languages() -> Vec<String> {
    let names = CURRENT
        .get()
        .and_then(|s| s.lock().ok())
        .and_then(|s| {
            s.inventory.as_ref().map(|i| {
                i.preferred
                    .values()
                    .chain(i.system.iter())
                    .filter_map(|p| p.dic.path.file_stem()?.to_str().map(str::to_owned))
                    .collect::<Vec<_>>()
            })
        })
        .unwrap_or_default();
    let mut tags = std::collections::BTreeSet::new();
    for name in names {
        if let Some(tag) = proof::lang::normalize_tag(&name.replace('_', "-")) {
            if dictionary_path(&tag)
                .is_some_and(|p| p.file_stem().is_some_and(|s| s == name.as_str()))
                && available(&tag)
            {
                tags.insert(tag);
            }
        }
    }
    tags.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        let p = std::env::temp_dir().join(format!(
            "leaf-dictionary-test-{}-{}",
            std::process::id(),
            NEXT_INDEX.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
    fn pair(dir: &Path, name: &str, text: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(format!("{name}.aff")), "SET UTF-8\n").unwrap();
        std::fs::write(dir.join(format!("{name}.dic")), format!("1\n{text}\n")).unwrap();
    }
    #[test]
    fn chosen_folder_supplements_imports_and_bundled_data_without_modifying_sources() {
        let r = root();
        let (i, f, b) = (r.join("imports"), r.join("folder"), r.join("bundled"));
        pair(&i, "it_IT", "imported");
        pair(&f, "it_IT", "folder");
        pair(&f, "fr_FR", "francais");
        pair(&b, "es_ES", "espanol");
        let inv = inventory(&i, Some(&f), &b, None);
        let index = Index::build(&inv.preferred).unwrap();
        assert!(index.0.join("es_ES.dic").is_file());
        assert!(index.0.join("fr_FR.aff").is_file());
        assert!(
            std::fs::read_to_string(index.0.join("it_IT.dic"))
                .unwrap()
                .contains("imported")
        );
        assert!(
            std::fs::read_to_string(f.join("it_IT.dic"))
                .unwrap()
                .contains("folder")
        );
        drop(index);
        std::fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn changes_and_removed_folders_are_detected_and_builtin_fallback_survives() {
        let r = root();
        let (i, f, b) = (r.join("i"), r.join("f"), r.join("b"));
        pair(&f, "it_IT", "prova");
        pair(&b, "es_ES", "hola");
        let first = inventory(&i, Some(&f), &b, None);
        pair(&f, "it_IT", "nuovaparola");
        assert!(first != inventory(&i, Some(&f), &b, None));
        std::fs::remove_dir_all(&f).unwrap();
        let missing = inventory(&i, Some(&f), &b, None);
        assert!(!missing.preferred.contains_key("it_IT"));
        assert!(missing.preferred.contains_key("es_ES"));
        std::fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn incomplete_pairs_and_unrelated_files_are_not_indexed() {
        let r = root();
        std::fs::write(r.join("it_IT.dic"), "1\nprova\n").unwrap();
        std::fs::write(r.join("not-a-dictionary.txt"), "private").unwrap();
        assert!(pairs(&r).unwrap().is_empty());
        pair(&r, "it_IT", "prova");
        assert_eq!(pairs(&r).unwrap().len(), 1);
        std::fs::remove_dir_all(r).unwrap();
    }
}
