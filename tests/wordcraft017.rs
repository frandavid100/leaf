//! Verify adopted upstream behavior through the same public operations as Leaf.
use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    editing::{self, Edit},
    selection, styles,
};
#[test]
fn deleting_an_empty_preceding_paragraph_preserves_heading_style_and_undo_redo() {
    for backward in [false, true] {
        let mut s = Session::new(Document::from_text("\nHeading"));
        selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
        styles::edit(&mut s, styles::Edit::Apply("Heading1".into())).unwrap();
        let expected = s.doc.para_at(&Pos::body(1, 0)).unwrap().props.clone();
        selection::set_caret(&mut s, &Pos::body(if backward { 1 } else { 0 }, 0), false).unwrap();
        let before = s.doc.clone();
        editing::apply(
            &mut s,
            if backward {
                Edit::Backspace
            } else {
                Edit::Delete
            },
        )
        .unwrap();
        assert_eq!(s.doc.para_at(&Pos::body(0, 0)).unwrap().props, expected);
        let after = s.doc.clone();
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc, before);
        s.run("edit.redo", &json!({})).unwrap();
        assert_eq!(s.doc, after);
        for ext in ["docx", "odt"] {
            let bytes =
                wordcraft_engine::io::save_bytes(&format!("heading.{ext}"), &s.doc).unwrap();
            let d = wordcraft_engine::io::open_bytes(&format!("heading.{ext}"), &bytes).unwrap();
            assert_eq!(
                d.para_at(&Pos::body(0, 0)).unwrap().props.style.as_deref(),
                Some("Heading1")
            );
        }
    }
}
