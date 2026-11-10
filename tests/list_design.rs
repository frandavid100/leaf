use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef, numbering::Counters};
use wordcraft_engine::Session;
use wordcraft_gnome::{list_design, selection};
fn number(s: &Session, index: usize) -> String {
    let mut counter = Counters::default();
    let mut value = String::new();
    for (i, path) in s.doc.para_paths(StoryRef::Body).iter().enumerate() {
        let n = s
            .doc
            .para(StoryRef::Body, path)
            .and_then(|p| s.doc.styles.resolve_para(&p.props).numbering);
        if let Some(n) = n {
            value = counter
                .next_label(&s.doc.numbering, n.num, n.level)
                .unwrap()
                .0;
        }
        if i == index {
            return value;
        }
    }
    value
}
#[test]
fn explicit_reapply_never_toggles_or_restarts_the_current_numbered_design() {
    let mut s = Session::new(Document::from_text("First\nSecond\nThird"));
    s.run("select.all", &json!({})).unwrap();
    list_design::apply(&mut s, "para.numbering", &json!({"kind":"numbered"})).unwrap();
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    assert_eq!(number(&s, 1), "2.");
    list_design::apply(&mut s, "para.numbering", &json!({"kind":"numbered"})).unwrap();
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), depth);
    assert_eq!(number(&s, 1), "2.");
    assert!(!list_design::can_apply(&s, "upperLetter"));
    assert!(list_design::apply(&mut s, "para.numbering", &json!({"kind":"upperLetter"})).is_err());
    assert_eq!(
        s.doc, before,
        "unsupported design conversion must preserve the existing sequence"
    );
}
#[test]
fn starting_number_uses_one_undo_and_does_not_edit_the_preceding_item() {
    let mut s = Session::new(Document::from_text("First\nSecond\nThird"));
    s.run("select.all", &json!({})).unwrap();
    s.run("para.numbering", &json!({"kind":"numbered"}))
        .unwrap();
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    list_design::start_at(&mut s, 7).unwrap();
    assert_eq!(number(&s, 0), "1.");
    assert_eq!(number(&s, 1), "7.");
    assert_eq!(number(&s, 2), "8.");
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
}

#[test]
fn removing_list_keeps_authored_indents_style_shading_and_text() {
    let mut s = Session::new(Document::from_text("Keep this text"));
    s.run("select.all", &json!({})).unwrap();
    s.run("para.bullets", &json!({"kind":"•"})).unwrap();
    s.run("para.indents", &json!({"left":48.,"firstLine":-12.}))
        .unwrap();
    s.run("para.shading", &json!({"color":"FFFF00","paragraph":true}))
        .unwrap();
    let before = s.doc.clone();
    let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
    let (style, shading) = (p.props.style.clone(), p.props.shading);
    list_design::apply(&mut s, "para.bullets", &json!({"off":true})).unwrap();
    let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
    assert_eq!(p.props.indent_left, Some(48.));
    assert_eq!(p.props.indent_first, Some(-12.));
    assert_eq!(p.props.style, style);
    assert_eq!(p.props.shading, shading);
    assert_eq!(s.doc.plain_text(StoryRef::Body), "Keep this text");
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
}
