use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{paragraph_borders::change, selection};
#[test]
fn edge_and_property_edits_preserve_other_edges_shading_and_one_undo() {
    let mut s = Session::new(Document::from_text("First\nSecond"));
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    s.run(
        "para.borders",
        &json!({"kind":"top","width":2.,"color":"FF0000"}),
    )
    .unwrap();
    s.run("para.shading", &json!({"color":"FFFF00","paragraph":true}))
        .unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    let rev = s.rev();
    change(
        &mut s,
        Some("bottom"),
        None,
        &json!({"width":0.5,"color":"1C71D8"}),
    )
    .unwrap();
    assert!(s.rev() > rev);
    assert_eq!(s.undo_depth(), depth + 1);
    let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
    let b = p.props.borders.unwrap();
    assert_eq!(b.top.unwrap().width, 2.);
    assert_eq!(b.bottom.unwrap().width, 0.5);
    assert!(p.props.shading.is_some());
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    change(&mut s, None, Some(&("color", json!("000000"))), &json!({})).unwrap();
    let b = s
        .doc
        .para_at(&Pos::body(0, 0))
        .unwrap()
        .props
        .borders
        .unwrap();
    assert_eq!(b.top.unwrap().width, 2.);
    assert_eq!(b.bottom.unwrap().width, 0.5);
    assert_eq!(b.left, None);
    change(&mut s, Some("none"), None, &json!({})).unwrap();
    let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
    assert!(!p.props.borders.unwrap().any_visible());
    assert!(p.props.shading.is_some());
}
