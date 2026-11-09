//! Blocking desktop recovery adapters; native callers run these off the GTK thread.
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use wordcraft_engine::{Session, recovery::RecoveryInfo};

pub struct Inventory {
    pub documents: Vec<RecoveryInfo>,
    pub warning: Option<String>,
}

fn ready(root: &Path) -> Result<Session, String> {
    let mut session = Session::new(wordcraft_doc::Document::new());
    session.enable_recovery(root.to_path_buf(), Arc::new(|| {}))?;
    wait(&mut session, |s| s.recovery_ready() && !s.recovery_busy())?;
    Ok(session)
}

fn wait(session: &mut Session, done: impl Fn(&Session) -> bool) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        session.recovery_tick();
        if done(session) {
            return Ok(());
        }
        if let Some(error) = session.recovery_error() {
            return Err(error.into());
        }
        if Instant::now() >= deadline {
            return Err(wordcraft_gnome::t!("Recovery did not finish in time.").into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn inventory(root: &Path) -> Result<Inventory, String> {
    let session = ready(root)?;
    Ok(Inventory {
        documents: session.recovery_documents(),
        warning: session.recovery_error().map(str::to_owned),
    })
}

/// Restore into an isolated canonical Session. A stale UI result can be dropped
/// without overwriting live work or discarding the retained recovery record.
pub fn restore(root: &Path, id: &str) -> Result<Session, String> {
    let mut session = ready(root)?;
    let before = session.document_id();
    session
        .run("file.recover", &serde_json::json!({"restore":id}))
        .map_err(|e| e.to_string())?;
    wait(&mut session, |s| s.document_id() != before)?;
    Ok(session)
}

/// Discard only a listed, unlocked record through the engine's public command.
/// The caller must obtain confirmation before requesting this irreversible action.
pub fn discard(root: &Path, id: &str, saved_at: &str) -> Result<(), String> {
    let mut session = ready(root)?;
    if !session
        .recovery_documents()
        .iter()
        .any(|record| record.id == id && record.saved_at == saved_at)
    {
        return Err("Recovery record changed".into());
    }
    session
        .run("file.recover", &serde_json::json!({"discard":id}))
        .map_err(|e| e.to_string())?;
    wait(&mut session, |s| {
        !s.recovery_busy() && !s.recovery_documents().iter().any(|record| record.id == id)
    })
}

/// Preview a private byte-for-byte copy through the public recovery API.
/// The original record is never opened for writing, locked, restored or pruned.
/// Copying opaque snapshot files avoids depending on the engine's JSON schema.
pub fn preview(root: &Path, id: &str, saved_at: &str) -> Result<crate::PageImage, String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err("Invalid recovery record".into());
    }
    struct Temporary(std::path::PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let temporary = loop {
        let path = std::env::temp_dir().join(format!(
            "leaf-recovery-preview-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        match builder.create(&path) {
            Ok(()) => break Temporary(path),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.to_string()),
        }
    };
    let source = root.join(id);
    if !std::fs::symlink_metadata(&source)
        .map_err(|e| e.to_string())?
        .is_dir()
    {
        return Err("Recovery record is unavailable".into());
    }
    let destination = temporary.0.join(id);
    std::fs::create_dir(&destination).map_err(|e| e.to_string())?;
    let mut total = 0u64;
    let mut count = 0;
    for entry in std::fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().is_none_or(|e| e != "json") {
            continue;
        }
        let metadata = entry.path().symlink_metadata().map_err(|e| e.to_string())?;
        total = total.saturating_add(metadata.len());
        count += 1;
        if !metadata.is_file() || count > 16 || total > 32 * 1024 * 1024 {
            return Err("Recovery exceeds the preview size limit".into());
        }
        std::fs::copy(entry.path(), destination.join(entry.file_name()))
            .map_err(|e| e.to_string())?;
    }
    let inventory = inventory(&temporary.0)?;
    if !inventory
        .documents
        .iter()
        .any(|d| d.id == id && d.saved_at == saved_at)
    {
        return Err("Recovery record changed".into());
    }
    let mut session = restore(&temporary.0, id)?;
    crate::templates::page_preview(&mut session)
}
