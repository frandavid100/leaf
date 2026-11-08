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
