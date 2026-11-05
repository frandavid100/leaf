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

#[test]
fn deletions_resolve_in_both_editing_modes_and_leave_comments_independent() {
    for tracking in [false, true] {
        for accept in [false, true] {
            let mut s = Session::new(Document::from_text("Keep remove final."));
            selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 4)).unwrap();
            wordcraft_gnome::annotations::add_comment(&mut s, "Separate conversation").unwrap();
            s.run("review.trackChanges", &json!({"value":true}))
                .unwrap();
            let text = s.doc.para_at(&Pos::body(0, 0)).unwrap().text.clone();
            let start = text.find("remove").unwrap();
            selection::select_range(&mut s, &Pos::body(0, start), &Pos::body(0, start + 7))
                .unwrap();
            editing::apply(&mut s, Edit::Backspace).unwrap();
            s.run("review.trackChanges", &json!({"value":tracking}))
                .unwrap();
            let change = review::changes(&mut s)
                .unwrap()
                .into_iter()
                .find(|c| c.kind == "delete")
                .unwrap();
            let before = s.doc.clone();
            let depth = s.undo_depth();
            review::resolve(&mut s, &change, accept).unwrap();
            assert_eq!(s.doc.settings.track_changes, tracking);
            assert_eq!(s.undo_depth(), depth + 1);
            assert_eq!(wordcraft_gnome::annotations::comments(&s).len(), 1);
            assert!(review::changes(&mut s).unwrap().is_empty());
            assert_eq!(s.doc.plain_text(StoryRef::Body).contains("remove"), !accept);
            s.run("edit.undo", &json!({})).unwrap();
            assert_eq!(s.doc, before);
        }
    }
}
#[test]
fn formatting_proposals_survive_docx_and_odt_refuses_silent_loss() {
    let mut s = Session::new(Document::from_text("Bold proposal"));
    s.run("review.trackingOptions", &json!({"trackFormatting":true}))
        .unwrap();
    s.run("review.trackChanges", &json!({"value":true}))
        .unwrap();
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 4)).unwrap();
    s.run("format.bold", &json!({"value":true})).unwrap();
    assert!(wordcraft_engine::cmd::review::has_format_changes(&s.doc));
    let change = review::changes(&mut s)
        .unwrap()
        .into_iter()
        .find(|c| c.kind == "format")
        .unwrap();
    let path =
        std::env::temp_dir().join(format!("leaf29-format-review-{}.odt", std::process::id()));
    let before = s.doc.clone();
    assert!(saving::save(&mut s, &path).is_err());
    assert_eq!(s.doc, before);
    let path = path.with_extension("docx");
    saving::save(&mut s, &path).unwrap();
    let mut copy = open_document(&path).unwrap();
    let reopened = review::changes(&mut copy)
        .unwrap()
        .into_iter()
        .find(|c| c.kind == "format")
        .unwrap();
    review::resolve(&mut copy, &reopened, false).unwrap();
    assert!(review::changes(&mut copy).unwrap().is_empty());
    review::resolve(&mut s, &change, true).unwrap();
    assert!(review::changes(&mut s).unwrap().is_empty());
    std::fs::remove_file(path).unwrap();
}
