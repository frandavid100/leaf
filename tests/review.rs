use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    editing::{self, Edit},
    open_document, review, saving, selection,
};
fn proposed() -> Session {
    let mut s = Session::new(Document::from_text("Original text."));
    s.author = "Reviewer".into();
    s.run("review.trackChanges", &json!({"value":true}))
        .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 9), false).unwrap();
    editing::apply(&mut s, Edit::Insert("proposed ".into())).unwrap();
    s
}
#[test]
fn proposals_keep_tracking_mode_and_resolve_with_source_history() {
    let mut s = proposed();
    let before = s.doc.clone();
    let sel = s.sel.clone();
    let rev = s.rev();
    let depth = s.undo_depth();
    let change = review::changes(&mut s)
        .unwrap()
        .into_iter()
        .find(|c| c.kind == "insert")
        .unwrap();
    assert_eq!(change.author, "Reviewer");
    assert!(change.text.contains("proposed"));
    assert_eq!(s.doc, before);
    assert_eq!(s.sel, sel);
    assert_eq!(s.rev(), rev);
    assert_eq!(s.undo_depth(), depth);
    review::resolve(&mut s, &change, false).unwrap();
    assert!(s.doc.settings.track_changes);
    assert!(review::changes(&mut s).unwrap().is_empty());
    assert_eq!(s.doc.plain_text(StoryRef::Body), "Original text.");
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    assert!(review::changes(&mut s).unwrap().is_empty());
}
#[test]
fn proposals_survive_docx_and_odt_and_remain_resolvable() {
    for ext in ["docx", "odt"] {
        let mut s = proposed();
        let path = std::env::temp_dir().join(format!("leaf29-review-{}.{ext}", std::process::id()));
        saving::save(&mut s, &path).unwrap();
        let mut copy = open_document(&path).unwrap();
        let change = review::changes(&mut copy)
            .unwrap()
            .into_iter()
            .find(|c| c.kind == "insert")
            .unwrap();
        assert_eq!(change.author, "Reviewer");
        review::resolve(&mut copy, &change, true).unwrap();
        assert!(review::changes(&mut copy).unwrap().is_empty());
        assert!(copy.doc.plain_text(StoryRef::Body).contains("proposed"));
        std::fs::remove_file(path).unwrap();
    }
}
#[test]
fn stale_review_rows_cannot_resolve_another_passage() {
    let mut s = proposed();
    let change = review::changes(&mut s).unwrap().remove(0);
    s.run("review.trackChanges", &json!({"value":false}))
        .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    editing::apply(&mut s, Edit::Insert("Different ".into())).unwrap();
    let before = s.doc.clone();
    assert!(review::resolve(&mut s, &change, true).is_err());
    assert_eq!(s.doc, before);
}
