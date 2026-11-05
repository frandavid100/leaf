use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    ruler::{self, Lease, Marker},
    selection,
};

#[test]
fn indent_commits_once_to_the_original_selection_and_undoes() {
    let mut s = Session::new(Document::from_text("First paragraph\nSecond paragraph"));
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(1, 6)).unwrap();
    let selected = s.sel.clone();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    let state = ruler::state(&mut s).unwrap();
    let lease = Lease::capture(&s);
    assert!(ruler::edit(&mut s, &lease, &state, Marker::Left, 36.0, false).unwrap());
    assert_eq!(s.sel, selected);
    assert_eq!(s.undo_depth(), depth + 1);
    for n in 0..2 {
        let p = s.doc.para_at(&Pos::body(n, 0)).unwrap();
        assert_eq!(s.doc.styles.resolve_para(&p.props).indent_left, 36.0);
    }
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    assert_eq!(ruler::state(&mut s).unwrap().left, 36.0);
}
#[test]
fn moving_a_tab_retains_its_kind_and_leader_and_drop_is_one_edit() {
    let mut s = Session::new(Document::from_text("Text\tTab"));
    s.run(
        "para.tabs",
        &json!({"set":[{"pos":72.0,"align":"right","leader":"dot"}]}),
    )
    .unwrap();
    let state = ruler::state(&mut s).unwrap();
    let old = state.tabs[0];
    let lease = Lease::capture(&s);
    assert!(ruler::edit(&mut s, &lease, &state, Marker::Tab(0), 108.0, false).unwrap());
    let state = ruler::state(&mut s).unwrap();
    assert_eq!(state.tabs.len(), 1);
    assert_eq!(state.tabs[0].pos, 108.0);
    assert_eq!(state.tabs[0].align, old.align);
    assert_eq!(state.tabs[0].leader, old.leader);
    let depth = s.undo_depth();
    let lease = Lease::capture(&s);
    assert!(ruler::edit(&mut s, &lease, &state, Marker::Tab(0), 108.0, true).unwrap());
    assert!(ruler::state(&mut s).unwrap().tabs.is_empty());
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(ruler::state(&mut s).unwrap().tabs[0].pos, 108.0);
}
#[test]
fn gestures_reject_changed_selection_and_revision_without_mutation() {
    let mut s = Session::new(Document::from_text("Text\nOther target"));
    let state = ruler::state(&mut s).unwrap();
    let lease = Lease::capture(&s);
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    let before = s.doc.clone();
    assert!(ruler::edit(&mut s, &lease, &state, Marker::Left, 24.0, false).is_err());
    assert_eq!(s.doc, before);
    let lease = Lease::capture(&s);
    s.run("text.insert", &json!({"text":"Changed"})).unwrap();
    let before = s.doc.clone();
    assert!(ruler::edit(&mut s, &lease, &state, Marker::Left, 24.0, false).is_err());
    assert_eq!(s.doc, before);
}

#[test]
fn pointer_magnets_use_document_centimetres_and_hold_near_a_boundary() {
    use wordcraft_gnome::ruler::{HALF_CM, snap_drag};
    assert!((snap_drag(18.0, None, 0., 500.) - HALF_CM).abs() < 0.0001);
    assert!((snap_drag(29.0, None, 0., 500.) - HALF_CM * 2.).abs() < 0.0001);
    let first = snap_drag(HALF_CM * 1.49, None, 0., 500.);
    assert_eq!(snap_drag(HALF_CM * 1.54, Some(first), 0., 500.), first);
    assert!((snap_drag(HALF_CM * 1.60, Some(first), 0., 500.) - HALF_CM * 2.).abs() < 0.0001);
    let constrained = snap_drag(95., None, 0., 93.);
    assert!(
        constrained <= 93.
            && (constrained / HALF_CM - (constrained / HALF_CM).round()).abs() < 0.0001
    );
    assert!((snap_drag(-19., None, -100., 100.) + HALF_CM).abs() < 0.0001);
}

#[test]
fn marker_grid_uses_each_property_origin_and_numeric_edits_keep_exact_values() {
    let mut s = Session::new(Document::from_text("First line\tTab"));
    s.run(
        "para.indents",
        &json!({"left":19.25,"right":27.75,"firstLine":3.125}),
    )
    .unwrap();
    let state = ruler::state(&mut s).unwrap();
    assert_eq!(state.left, 19.25);
    for (marker, input, property) in [
        (Marker::Left, 29., ruler::HALF_CM * 2.),
        (Marker::First, state.left + 18., state.left + ruler::HALF_CM),
        (
            Marker::Right,
            state.width - 18.,
            state.width - ruler::HALF_CM,
        ),
        (Marker::NewTab, 29., ruler::HALF_CM * 2.),
    ] {
        assert!((ruler::snap_marker(&state, marker, input, None) - property).abs() < 0.001);
    }
    assert_eq!(ruler::state(&mut s).unwrap().first, 3.125);
    assert_eq!(ruler::state(&mut s).unwrap().right, 27.75);
}
