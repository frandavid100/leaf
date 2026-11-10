use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{history::RepeatHistory, selection, text_properties};
fn target(s: &mut Session, a: usize, b: usize) {
    selection::select_range(s, &Pos::body(0, a), &Pos::body(0, b)).unwrap();
}
#[test]
fn cumulative_repeat_is_chronological_explicit_and_one_undo_without_recursive_growth() {
    let mut s = Session::new(Document::from_text("first second third"));
    let mut h = RepeatHistory::default();
    target(&mut s, 0, 5);
    s.run("format.bold", &json!({"value":true})).unwrap();
    h.record(&s);
    s.run("format.italic", &json!({"value":true})).unwrap();
    h.record(&s);
    assert!(h.descriptions()[0].contains("Italic"));
    assert!(h.descriptions()[1].contains("Bold"));
    target(&mut s, 6, 12);
    let depth = s.undo_depth();
    assert!(h.repeat_recent(&mut s, 2).unwrap());
    assert_eq!(s.undo_depth(), depth + 1);
    assert!(
        text_properties::selected_resolved(&s)
            .iter()
            .all(|p| p.bold && p.italic)
    );
    s.run("edit.undo", &json!({})).unwrap();
    assert!(
        text_properties::selected_resolved(&s)
            .iter()
            .all(|p| !p.bold && !p.italic)
    );
    s.run("edit.redo", &json!({})).unwrap();
    assert!(
        text_properties::selected_resolved(&s)
            .iter()
            .all(|p| p.bold && p.italic)
    );
    target(&mut s, 13, 18);
    assert!(h.repeat_recent(&mut s, 1).unwrap());
    assert_eq!(h.descriptions().len(), 1);
    let depth = s.undo_depth();
    assert!(!h.repeat_recent(&mut s, 1).unwrap());
    assert_eq!(s.undo_depth(), depth);
    assert!(
        text_properties::selected_resolved(&s)
            .iter()
            .all(|p| p.bold && p.italic)
    );
}
#[test]
fn range_freezes_parameter_order_and_rejects_another_document() {
    let mut s = Session::new(Document::from_text("first second"));
    let mut h = RepeatHistory::default();
    target(&mut s, 0, 5);
    for size in [14., 22., 18.] {
        s.run("format.size", &json!({"size":size})).unwrap();
        h.record(&s);
    }
    target(&mut s, 6, 12);
    assert!(h.repeat_recent(&mut s, 3).unwrap());
    assert!(
        text_properties::selected_resolved(&s)
            .iter()
            .all(|p| p.size == 18.)
    );
    let mut other = Session::new(Document::from_text("other"));
    assert!(h.repeat_recent(&mut other, 1).is_err());
}
#[test]
fn explicit_removal_repeats_without_toggling_or_replaying_io() {
    let mut s = Session::new(Document::from_text("first second"));
    let mut h = RepeatHistory::default();
    target(&mut s, 0, 5);
    s.run("format.bold", &json!({"value":false})).unwrap();
    h.record(&s);
    target(&mut s, 6, 12);
    s.run("format.bold", &json!({"value":true})).unwrap();
    assert!(h.repeat_recent(&mut s, 1).unwrap());
    assert!(
        text_properties::selected_resolved(&s)
            .iter()
            .all(|p| !p.bold)
    );
    assert!(!h.repeat_recent(&mut s, 1).unwrap());
}
