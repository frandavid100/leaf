use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    history::{self, RepeatAction},
    selection, text_properties,
};
fn select(s: &mut Session, n: usize) {
    selection::select_range(s, &Pos::body(n, 0), &Pos::body(n, 5)).unwrap();
}
#[test]
fn repeat_size_uses_new_target_and_ordinary_undo_redo() {
    let mut s = Session::new(Document::from_text("Alpha\nBravo"));
    select(&mut s, 0);
    s.run("format.size", &json!({"size":14})).unwrap();
    let action = RepeatAction::capture(&s).unwrap();
    select(&mut s, 1);
    let before = s.doc.clone();
    let depth = s.undo_depth();
    assert!(action.apply(&mut s).unwrap());
    assert_eq!(s.undo_depth(), depth + 1);
    assert_eq!(text_properties::selected_resolved(&s)[0].size, 14.0);
    let repeated = s.doc.clone();
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    assert_eq!(s.doc, repeated);
    let depth = s.undo_depth();
    assert!(!action.apply(&mut s).unwrap());
    assert_eq!(s.undo_depth(), depth);
    assert!(!action.applies_to(&Session::new(s.doc.clone())));
}
#[test]
fn repeat_captures_toggle_result_instead_of_replaying_a_toggle() {
    let mut s = Session::new(Document::from_text("Alpha\nBravo"));
    select(&mut s, 0);
    s.run("format.bold", &json!({})).unwrap();
    let on = RepeatAction::capture(&s).unwrap();
    select(&mut s, 1);
    assert!(on.apply(&mut s).unwrap());
    assert!(text_properties::selected_resolved(&s)[0].bold);
    assert!(!on.apply(&mut s).unwrap());
    assert!(text_properties::selected_resolved(&s)[0].bold);
    select(&mut s, 0);
    s.run("format.bold", &json!({})).unwrap();
    let off = RepeatAction::capture(&s).unwrap();
    select(&mut s, 1);
    assert!(off.apply(&mut s).unwrap());
    assert!(!text_properties::selected_resolved(&s)[0].bold);
}
#[test]
fn engine_redo_snapshot_inspection_is_nonmutating_and_traversal_is_cumulative() {
    let mut s = Session::new(Document::from_text("Alpha\nBravo"));
    select(&mut s, 0);
    s.run("format.bold", &json!({})).unwrap();
    let first = s.doc.clone();
    s.run("format.size", &json!({"size":18})).unwrap();
    s.run("format.italic", &json!({})).unwrap();
    let last = s.doc.clone();
    assert_eq!(history::travel(&mut s, false, 2).unwrap(), 2);
    assert_eq!(s.doc, first);
    let before = (
        s.doc.clone(),
        s.sel.clone(),
        s.rev(),
        s.dirty,
        s.undo_depth(),
        s.last_command.clone(),
    );
    let labels = history::labels(&s, true);
    assert_eq!(labels.len(), 2);
    assert_eq!(labels[0], s.redo_label().unwrap());
    assert_eq!(
        (
            s.doc.clone(),
            s.sel.clone(),
            s.rev(),
            s.dirty,
            s.undo_depth(),
            s.last_command.clone()
        ),
        before
    );
    assert_eq!(history::travel(&mut s, true, 2).unwrap(), 2);
    assert_eq!(s.doc, last);
    history::travel(&mut s, false, 2).unwrap();
    s.run("format.size", &json!({"size":20})).unwrap();
    assert!(!s.can_redo());
    assert!(history::labels(&s, true).is_empty());
}
#[test]
fn typing_group_is_one_history_entry_and_structural_commands_do_not_repeat() {
    let mut s = Session::new(Document::new());
    for text in ["a", "b", "c"] {
        s.run("text.insert", &json!({"text":text})).unwrap();
    }
    assert_eq!(history::labels(&s, false).len(), 1);
    assert!(RepeatAction::capture(&s).is_none());
    let text = s.doc.plain_text(StoryRef::Body);
    history::travel(&mut s, false, 1).unwrap();
    assert_ne!(s.doc.plain_text(StoryRef::Body), text);
    history::travel(&mut s, true, 1).unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Body), text);
}
