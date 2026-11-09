use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use wordcraft_doc::{Document, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::recovery;

fn seed(root: &Path) {
    seed_named(root, "Notas.docx");
}
fn seed_named(root: &Path, name: &str) {
    let before = copies(root).len();
    let mut session = Session::new(Document::new());
    session
        .enable_recovery(root.into(), Arc::new(|| {}))
        .unwrap();
    session.path = Some(PathBuf::from(name));
    session
        .run(
            "text.insert",
            &serde_json::json!({"text":"Recovered text ñ"}),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        session.recovery_tick();
        if session.recovery_ready() && !session.recovery_busy() && copies(root).len() > before {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn copies(root: &Path) -> Vec<Vec<u8>> {
    let mut files = std::fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|e| std::fs::read_dir(e.path()).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|e| e == "json"))
        .map(|e| e.path())
        .collect::<Vec<_>>();
    files.sort();
    files.iter().map(|p| std::fs::read(p).unwrap()).collect()
}
#[test]
fn pending_listing_and_dropped_restore_preserve_original_copies() {
    let root = std::env::temp_dir().join(format!("leaf-start-recovery-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    assert!(recovery::inventory(&root).unwrap().documents.is_empty());
    seed(&root);
    let before = copies(&root);
    assert!(!before.is_empty());
    let inventory = recovery::inventory(&root).unwrap();
    assert!(inventory.warning.is_none());
    assert_eq!(inventory.documents.len(), 1);
    assert_eq!(inventory.documents[0].name, "Notas.docx");
    let id = &inventory.documents[0].id;
    let preview = recovery::preview(&root, id, &inventory.documents[0].saved_at).unwrap();
    assert_eq!(copies(&root), before);
    assert_eq!(recovery::inventory(&root).unwrap().documents.len(), 1);
    assert!(recovery::preview(&root, id, "stale timestamp").is_err());
    assert!(recovery::preview(&root, "../escape", "").is_err());
    assert!(recovery::restore(&root, "missing").is_err());
    assert_eq!(copies(&root), before);
    let mut restored = recovery::restore(&root, id).unwrap();
    let rendered = wordcraft_gnome::templates::page_preview(&mut restored).unwrap();
    assert_eq!(
        (preview.width, preview.height),
        (rendered.width, rendered.height)
    );
    assert_eq!(preview.rgba, rendered.rgba, "unsaved snapshot first page");
    assert_eq!(restored.doc.plain_text(StoryRef::Body), "Recovered text ñ");
    assert!(restored.dirty && restored.path.is_none());
    assert!(
        recovery::inventory(&root).unwrap().documents.is_empty(),
        "live recovered copy is locked"
    );
    assert_eq!(copies(&root), before);
    drop(restored); // A stale asynchronous UI result must retain the recovery data.
    assert_eq!(recovery::inventory(&root).unwrap().documents.len(), 1);
    assert_eq!(copies(&root), before);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn confirmed_discard_rejects_stale_and_live_records_and_preserves_other_drafts() {
    let root = std::env::temp_dir().join(format!(
        "leaf-discard-recovery-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    seed(&root);
    seed_named(&root, "Other draft.docx");
    let records = recovery::inventory(&root).unwrap().documents;
    assert_eq!(records.len(), 2);
    let (first, second) = (&records[0], &records[1]);
    let before = copies(&root);
    assert!(recovery::discard(&root, &first.id, "stale snapshot").is_err());
    assert!(recovery::discard(&root, "../escape", &first.saved_at).is_err());
    assert_eq!(copies(&root), before);
    let live = recovery::restore(&root, &first.id).unwrap();
    assert!(recovery::discard(&root, &first.id, &first.saved_at).is_err());
    assert_eq!(copies(&root), before);
    drop(live);
    recovery::discard(&root, &first.id, &first.saved_at).unwrap();
    let remaining = recovery::inventory(&root).unwrap().documents;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, second.id);
    assert!(!root.join(&first.id).exists());
    let restored = recovery::restore(&root, &second.id).unwrap();
    assert_eq!(restored.doc.plain_text(StoryRef::Body), "Recovered text ñ");
    drop(restored);
    assert_eq!(recovery::inventory(&root).unwrap().documents.len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}
