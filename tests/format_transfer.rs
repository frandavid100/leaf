use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{format_painter::FormatTransfer, selection, text_properties};
fn range(s: &mut Session, a: usize, b: usize) {
    selection::select_range(s, &Pos::body(0, a), &Pos::body(0, b)).unwrap();
}
#[test]
fn ordinary_copy_appearance_transfers_across_documents_without_text_or_source_list_ids() {
    let mut source = Session::new(Document::from_text("Source"));
    range(&mut source, 0, 6);
    source.run("format.bold", &json!({"value":true})).unwrap();
    source
        .run("format.color", &json!({"color":"1C71D8"}))
        .unwrap();
    source
        .run("para.numbering", &json!({"kind":"upperLetter"}))
        .unwrap();
    source
        .run("para.align", &json!({"value":"center"}))
        .unwrap();
    let original = source.doc.clone();
    let depth = source.undo_depth();
    let payload = FormatTransfer::capture(&source).unwrap();
    assert_eq!(source.doc, original);
    assert_eq!(source.undo_depth(), depth);
    assert!(source.painter.is_none());
    let mut destination = Session::new(Document::from_text("Destination"));
    range(&mut destination, 0, 11);
    destination
        .run("para.numbering", &json!({"kind":"numbered"}))
        .unwrap();
    let numbering = destination
        .doc
        .para_at(&Pos::body(0, 0))
        .unwrap()
        .props
        .numbering;
    let before = destination.doc.clone();
    let selected = destination.sel.clone();
    let text = destination.doc.plain_text(StoryRef::Body);
    let depth = destination.undo_depth();
    payload.apply(&mut destination).unwrap();
    assert_eq!(destination.doc.plain_text(StoryRef::Body), text);
    assert_eq!(destination.sel, selected);
    assert_eq!(destination.undo_depth(), depth + 1);
    assert!(
        text_properties::selected_resolved(&destination)
            .iter()
            .all(|p| p.bold)
    );
    assert_eq!(
        destination
            .doc
            .para_at(&Pos::body(0, 0))
            .unwrap()
            .props
            .numbering,
        numbering,
        "preserve destination list identity"
    );
    assert_eq!(
        wordcraft_engine::cmd::para::selection_para(&destination).align,
        wordcraft_doc::props::Align::Center
    );
    assert!(destination.painter.is_none());
    destination.run("edit.undo", &json!({})).unwrap();
    assert_eq!(destination.doc, before);
    destination.run("edit.redo", &json!({})).unwrap();
    assert!(
        text_properties::selected_resolved(&destination)
            .iter()
            .all(|p| p.bold)
    );
}
#[test]
fn ordinary_copy_does_not_replace_an_active_format_painter() {
    let mut s = Session::new(Document::from_text("Alpha Beta"));
    range(&mut s, 0, 5);
    wordcraft_gnome::format_painter::capture(&mut s, true).unwrap();
    let painter = s.painter.clone();
    let payload = FormatTransfer::capture(&s).unwrap();
    range(&mut s, 6, 10);
    payload.apply(&mut s).unwrap();
    assert_eq!(s.painter, painter);
}

#[test]
fn source_without_a_text_border_clears_destination_border_without_resetting_paragraph_style() {
    let source = Session::new(Document::from_text("source"));
    let transfer = FormatTransfer::capture(&source).unwrap();
    let mut target = Session::new(Document::from_text("target"));
    range(&mut target, 0, 6);
    target.run("format.border", &json!({"value":true})).unwrap();
    target.run("para.indents", &json!({"left":36.})).unwrap();
    let style = target
        .doc
        .para_at(&Pos::body(0, 0))
        .unwrap()
        .props
        .style
        .clone();
    transfer.apply(&mut target).unwrap();
    assert!(
        text_properties::selected_resolved(&target)
            .iter()
            .all(|p| p.border.is_none())
    );
    assert_eq!(
        target.doc.para_at(&Pos::body(0, 0)).unwrap().props.style,
        style
    );
}
