use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{document_properties::DocumentInfo, open_document, saving, selection};

fn info() -> DocumentInfo {
    DocumentInfo {
        title: "Tutoría <entre> iguales 🦊".into(),
        subject: "Inglés & tutoría".into(),
        author: "David Prieto".into(),
        keywords: "inglés, gramática".into(),
        category: "Trabajo".into(),
    }
}

#[test]
fn metadata_is_undoable_preserves_other_properties_and_does_not_change_session_author() {
    let mut s = Session::new(Document::from_text("Texto del documento"));
    s.run(
        "file.properties",
        &json!({"comments":"Description", "custom":{"Unrelated":"Keep"}}),
    )
    .unwrap();
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
    let (original, sel, depth, author) = (
        s.doc.clone(),
        s.sel.clone(),
        s.undo_depth(),
        s.author.clone(),
    );
    assert!(info().apply(&mut s).unwrap());
    assert_eq!(DocumentInfo::current(&s), info());
    assert_eq!(s.doc.core.description, "Description");
    assert_eq!(s.doc.custom_prop("Unrelated"), Some("Keep"));
    assert_eq!(s.sel, sel);
    assert_eq!(s.author, author);
    assert_eq!(s.undo_depth(), depth + 1);
    let rev = s.rev();
    assert!(!info().apply(&mut s).unwrap());
    assert_eq!(s.rev(), rev);
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, original);
    s.run("edit.redo", &json!({})).unwrap();
    assert_eq!(DocumentInfo::current(&s), info());
    assert!(DocumentInfo::default().apply(&mut s).unwrap());
    assert_eq!(DocumentInfo::current(&s), DocumentInfo::default());
    s.doc.settings.protection = Some("readOnly".into());
    let before = s.doc.clone();
    assert!(info().apply(&mut s).is_err());
    assert_eq!(s.doc, before);
}

#[test]
fn docx_preserves_all_fields_and_odt_saves_are_protected_when_category_would_be_lost() {
    let dir = std::env::temp_dir().join(format!("leaf-metadata-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = Session::new(Document::from_text("Original"));
    info().apply(&mut s).unwrap();
    let docx = dir.join("metadata.docx");
    saving::save(&mut s, &docx).unwrap();
    assert_eq!(
        DocumentInfo::current(&open_document(&docx).unwrap()),
        info()
    );
    let odt = dir.join("metadata.odt");
    std::fs::write(&odt, b"keep original file").unwrap();
    let path = s.path.clone();
    assert!(saving::save(&mut s, &odt).is_err());
    assert_eq!(s.path, path);
    assert_eq!(std::fs::read(&odt).unwrap(), b"keep original file");
    // Reproduce the pinned upstream limit without changing its exporter.
    std::fs::write(
        &odt,
        wordcraft_engine::io::save_bytes("upstream.odt", &s.doc).unwrap(),
    )
    .unwrap();
    assert!(
        DocumentInfo::current(&open_document(&odt).unwrap())
            .category
            .is_empty()
    );
    let mut supported = info();
    supported.category.clear();
    supported.apply(&mut s).unwrap();
    saving::save(&mut s, &odt).unwrap();
    assert_eq!(
        DocumentInfo::current(&open_document(&odt).unwrap()),
        supported
    );
    std::fs::remove_dir_all(dir).unwrap();
}
