//! Reproduce the maintainer's whole-paragraph deletion versus two-step control.
use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    editing::{self, Edit},
    selection, styles,
};
fn main() {
    let mut base = Session::new(Document::from_text("Heading\nFollowing list item"));
    styles::edit(&mut base, styles::Edit::Apply("Heading2".into())).unwrap();
    selection::set_caret(&mut base, &Pos::body(1, 0), false).unwrap();
    base.run("para.bullets", &json!({})).unwrap();
    let doc = base.doc.clone();
    let expected = doc.para_at(&Pos::body(1, 0)).unwrap().props.clone();
    let mut whole = Session::new(doc.clone());
    whole.run("select.paragraph", &json!({})).unwrap();
    println!(
        "WHOLE_SELECTION {:?} -> {:?}",
        whole.sel.anchor, whole.sel.focus
    );
    editing::apply(&mut whole, Edit::Delete).unwrap();
    println!(
        "WHOLE_PRESERVES_FOLLOWING={}",
        whole.doc.para_at(&Pos::body(0, 0)).unwrap().props == expected
    );
    let mut control = Session::new(doc);
    selection::select_range(&mut control, &Pos::body(0, 0), &Pos::body(0, 7)).unwrap();
    editing::apply(&mut control, Edit::Delete).unwrap();
    editing::apply(&mut control, Edit::Delete).unwrap();
    println!(
        "TWO_STEP_PRESERVES_FOLLOWING={}",
        control.doc.para_at(&Pos::body(0, 0)).unwrap().props == expected
    );
}
