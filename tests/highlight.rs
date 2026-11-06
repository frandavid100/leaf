use serde_json::json;
use wordcraft_doc::{Document, Pos, props::Rgb};
use wordcraft_engine::Session;
use wordcraft_gnome::{highlight, selection};

fn session() -> Session {
    let mut s = Session::new(Document::from_text("Alpha Beta"));
    highlight::register(&mut s);
    selection::select_all(&mut s).unwrap();
    s
}
#[test]
fn arbitrary_background_is_one_undoable_docx_edit_and_repeats() {
    let mut s = session();
    let original = s.doc.clone();
    let depth = s.undo_depth();
    s.run("leaf.highlight", &json!({"color":"234567"})).unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    assert!(highlight::matches(&s, "234567"));
    let repeat = wordcraft_gnome::history::RepeatAction::capture(&s).unwrap();
    let path = std::env::temp_dir().join(format!("leaf-rgb-highlight-{}.docx", std::process::id()));
    wordcraft_gnome::saving::save(&mut s, &path).unwrap();
    let mut reopened = wordcraft_gnome::open_document(&path).unwrap();
    selection::select_all(&mut reopened).unwrap();
    assert!(highlight::matches(&reopened, "234567"));
    std::fs::remove_file(path).unwrap();
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, original);
    s.run("edit.redo", &json!({})).unwrap();
    assert!(highlight::matches(&s, "234567"));
    s.run("edit.undo", &json!({})).unwrap();
    assert!(repeat.apply(&mut s).unwrap());
    assert!(highlight::matches(&s, "234567"));
}
#[test]
fn caret_background_changes_typing_without_formatting_the_word() {
    let mut s = session();
    selection::set_caret(&mut s, &Pos::body(0, 3), false).unwrap();
    let original = s.doc.clone();
    s.run("leaf.highlight", &json!({"color":"234567"})).unwrap();
    assert_eq!(s.doc, original);
    assert_eq!(s.typing_props().shading, Some(Rgb(0x23, 0x45, 0x67)));
    s.run("text.insert", &json!({"text":"X"})).unwrap();
    selection::select_range(&mut s, &Pos::body(0, 3), &Pos::body(0, 4)).unwrap();
    assert!(highlight::matches(&s, "234567"));
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, original);
}
#[test]
fn native_palette_and_no_highlight_clear_custom_shading() {
    let mut s = session();
    for color in ["234567", "yellow", "none"] {
        s.run("leaf.highlight", &json!({"color":color})).unwrap();
        assert!(highlight::matches(&s, color));
    }
    assert!(wordcraft_gnome::text_properties::selected_resolved(&s).iter().all(|p| p.shading.is_none() && p.highlight.is_none()));
    let before = s.doc.clone();
    assert!(s.run("leaf.highlight", &json!({"color":"invalid"})).is_err());
    assert_eq!(s.doc, before);
}
