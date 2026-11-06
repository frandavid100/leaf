use wordcraft_doc::Document;
use wordcraft_engine::Session;
use wordcraft_gnome::printing::PrintFile;

#[test]
fn printer_pdf_omits_links_but_pdf_export_preserves_them() {
    let mut session = Session::new(Document::new());
    session
        .run(
            "insert.link",
            &serde_json::json!({"url": "https://example.org", "text": "A printable link"}),
        )
        .unwrap();
    let original = session.doc.clone();
    let print = PrintFile::prepare(session.doc.clone()).unwrap();
    let exported = print.directory().join("export.pdf");
    wordcraft_gnome::pdf_export::export(&mut session, &exported).unwrap();
    let exported = std::fs::read(exported).unwrap();
    let exported = String::from_utf8_lossy(&exported);
    assert!(exported.contains("/Annots") && exported.contains("/Link"));
    let job = std::fs::read(print.path()).unwrap();
    let job = String::from_utf8_lossy(&job);
    assert!(
        !job.contains("/Annots"),
        "printer jobs must omit annotations"
    );
    assert!(!job.contains("/Link"));
    assert_eq!(session.doc, original);
}

#[test]
fn printing_snapshots_unsaved_changes_and_cleans_private_files() {
    let mut session = Session::new(Document::from_text("Original"));
    session
        .run(
            "text.insert",
            &serde_json::json!({"text":"Unsaved changes "}),
        )
        .unwrap();
    let document = session.doc.clone();
    let selection = session.sel.clone();
    let (revision, history, dirty, path) = (
        session.rev(),
        session.undo_depth(),
        session.dirty,
        session.path.clone(),
    );
    let first = PrintFile::prepare(session.doc.clone()).unwrap();
    let second = PrintFile::prepare(session.doc.clone()).unwrap();
    assert_ne!(first.directory(), second.directory());
    assert!(std::fs::read(first.path()).unwrap().starts_with(b"%PDF-"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(first.directory())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
    assert_eq!(session.doc, document);
    assert_eq!(session.sel, selection);
    assert_eq!(
        (
            session.rev(),
            session.undo_depth(),
            session.dirty,
            session.path.clone()
        ),
        (revision, history, dirty, path)
    );
    let directory = first.directory().to_owned();
    drop(first);
    assert!(!directory.exists());
    assert!(second.path().exists());
}
