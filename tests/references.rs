use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, references, saving, selection};

#[test]
fn footnote_edit_navigation_history_and_docx_roundtrip() {
    let mut s = Session::new(Document::from_text("First reference.\nSecond reference."));
    selection::set_caret(&mut s, &Pos::body(0, 5), false).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    references::insert_footnote(&mut s).unwrap();
    assert!(matches!(s.sel.focus.story, StoryRef::Part(_)));
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("text.insert", &json!({"text":"A real footnote."}))
        .unwrap();
    let note = s.sel.focus.story;
    references::return_to_reference(&mut s).unwrap();
    assert_eq!(s.sel.focus.story, StoryRef::Body);
    references::open_footnote(&mut s, false).unwrap();
    assert_eq!(s.sel.focus.story, note);
    let layout = s.layout();
    assert!(layout.caret_on(&s.sel.focus, 0).is_some());
    references::return_to_reference(&mut s).unwrap();
    let path = std::env::temp_dir().join(format!("leaf-footnote-{}.docx", std::process::id()));
    saving::save(&mut s, &path).unwrap();
    let mut reopened = open_document(&path).unwrap();
    assert_eq!(references::footnote_refs(&reopened).len(), 1);
    references::open_footnote(&mut reopened, true).unwrap();
    assert!(
        reopened
            .doc
            .plain_text(reopened.sel.focus.story)
            .contains("A real footnote.")
    );
    references::return_to_reference(&mut reopened).unwrap();
    reopened.run("text.backspace", &json!({})).unwrap();
    assert!(references::footnote_refs(&reopened).is_empty());
    reopened.run("edit.undo", &json!({})).unwrap();
    assert_eq!(references::footnote_refs(&reopened).len(), 1);
    assert!(saving::format_preservation_error(&reopened, &path.with_extension("odt")).is_some());
    std::fs::remove_file(path).unwrap();
    while s.can_undo() {
        s.run("edit.undo", &json!({})).unwrap();
    }
    assert_eq!(s.doc, before);
}

#[test]
fn toc_and_cross_reference_remain_structured_and_updatable() {
    let mut s = Session::new(Document::from_text(
        "Chapter One\nA paragraph\nSee the heading.",
    ));
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    selection::set_caret(&mut s, &Pos::body(2, 16), false).unwrap();
    s.run(
        "insert.crossReference",
        &json!({"to":"heading","target":"Chapter One","show":"text"}),
    )
    .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    s.run("references.toc", &json!({"title":"Contents","levels":3}))
        .unwrap();
    references::update(&mut s).unwrap();
    let path = std::env::temp_dir().join(format!("leaf-references-{}.docx", std::process::id()));
    saving::save(&mut s, &path).unwrap();
    let mut reopened = open_document(&path).unwrap();
    references::update(&mut reopened).unwrap();
    let value = serde_json::to_string(&reopened.doc).unwrap();
    assert!(value.contains("TOC "));
    assert!(value.contains("REF _Ref"));
    std::fs::remove_file(path).unwrap();
}
