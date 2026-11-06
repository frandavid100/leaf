use serde_json::json;
use wordcraft_doc::Document;
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, references, saving, selection};
#[test]
fn imported_toc_edits_the_existing_automatic_index_with_undo_and_docx() {
    let mut s = Session::new(Document::from_text("First heading\nA paragraph"));
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    s.run(
        "references.toc",
        &json!({"title":"Contents","levels":2,"hyperlinks":true}),
    )
    .unwrap();
    let file = std::env::temp_dir().join("leaf29-imported-toc.docx");
    saving::save(&mut s, &file).unwrap();
    let mut s = open_document(&file).unwrap();
    let (at, code) = references::toc_field(&s).unwrap();
    assert!(references::toc_at(&s, &at));
    selection::set_caret(&mut s, &at, false).unwrap();
    let before = s.doc.clone();
    references::edit_toc(&mut s, &format!("{code} \\n \"1-2\""), None).unwrap();
    assert!(references::toc_field(&s).unwrap().1.contains("\\n"));
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    saving::save(&mut s, &file).unwrap();
    let copy = open_document(&file).unwrap();
    assert!(references::toc_field(&copy).unwrap().1.contains("\\n"));
    std::fs::remove_file(file).unwrap();
}

#[test]
fn explicit_creation_never_replaces_an_existing_index_even_with_caret_elsewhere() {
    use wordcraft_doc::Pos;
    let mut s = Session::new(Document::from_text("Heading\nBody"));
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    references::insert_toc(&mut s, &json!({"title":"First contents","levels":2})).unwrap();
    let last = s.doc.body.len() - 1;
    selection::set_caret(&mut s, &Pos::body(last, 0), false).unwrap();
    let before = s.doc.clone();
    let selected = s.sel.clone();
    let rev = s.rev();
    let depth = s.undo_depth();
    assert!(references::insert_toc(&mut s, &json!({"title":"New contents","levels":4})).is_err());
    assert_eq!(s.doc, before);
    assert_eq!(s.sel, selected);
    assert_eq!(s.rev(), rev);
    assert_eq!(s.undo_depth(), depth);
}
