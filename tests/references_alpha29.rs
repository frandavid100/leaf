use serde_json::json;
use wordcraft_doc::{Document, InlineObject, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{references, selection};
fn fields(s: &Session, name: &str) -> Vec<String> {
    let mut result = Vec::new();
    s.doc.objects_in_reading_order(&mut |o| {
        if let InlineObject::Field {
            instr, result: r, ..
        } = o
        {
            if wordcraft_engine::cmd::references::FieldCode::parse(instr).name == name {
                result.push(r.clone())
            }
        }
    });
    result
}
#[test]
fn derived_updates_preserve_dates_caret_and_source_history_including_redo() {
    let mut s = Session::new(Document::from_text("Chapter One\nA paragraph\nSee: "));
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    selection::set_caret(&mut s, &Pos::body(2, 5), false).unwrap();
    s.run(
        "insert.crossReference",
        &json!({"to":"heading","target":"Chapter One","show":"text"}),
    )
    .unwrap();
    s.run(
        "insert.field",
        &json!({"instr":"DATE","result":"unchanged"}),
    )
    .unwrap();
    // Imported dynamic fields keep their cached value while unrelated text changes.
    for path in s.doc.para_paths(StoryRef::Body) {
        let p = s.doc.para_mut(StoryRef::Body, &path).unwrap();
        for o in &mut p.objects {
            if let InlineObject::Field { instr, result, .. } = o {
                if instr == "DATE" {
                    *result = "frozen date".into();
                }
            }
        }
        p.touch();
    }
    s.touch();
    let mut target = wordcraft_engine::cmd::citations::ref_targets(&s, "heading")[0]
        .1
        .clone();
    target.off = s
        .doc
        .para_at(&target)
        .unwrap()
        .text
        .find("Chapter One")
        .unwrap();
    selection::set_caret(&mut s, &target, false).unwrap();
    s.run("text.insert", &json!({"text":"New "})).unwrap();
    let caret = s.sel.clone();
    let depth = s.undo_depth();
    assert!(references::refresh_derived(&mut s).unwrap());
    assert_eq!(s.sel, caret);
    assert_eq!(s.undo_depth(), depth);
    assert!(fields(&s, "REF")[0].contains("New Chapter"));
    assert_eq!(fields(&s, "DATE"), ["frozen date"]);
    let rev = s.rev();
    assert!(!references::refresh_derived(&mut s).unwrap());
    assert_eq!(s.rev(), rev);
    s.run("edit.undo", &json!({})).unwrap();
    references::refresh_derived(&mut s).unwrap();
    assert!(s.can_redo());
    assert!(fields(&s, "REF")[0].contains("Chapter One"));
    assert!(!fields(&s, "REF")[0].contains("New Chapter"));
    s.run("edit.redo", &json!({})).unwrap();
    references::refresh_derived(&mut s).unwrap();
    assert!(fields(&s, "REF")[0].contains("New Chapter"));
}
#[test]
fn captions_require_actual_targets_and_start_with_an_editable_description() {
    let mut s = Session::new(Document::from_text("Text without a selected object"));
    let before = s.doc.clone();
    assert!(references::insert_caption(&mut s).is_err());
    assert_eq!(s.doc, before);
    s.run("insert.table", &json!({"rows":2,"cols":2})).unwrap();
    assert_eq!(references::caption_kind(&s), Some("Table"));
    let before = s.doc.clone();
    let depth = s.undo_depth();
    references::insert_caption(&mut s).unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    assert!(s.doc.para_at(&s.sel.focus).unwrap().text.ends_with(": "));
    s.run("text.insert", &json!({"text":"A table description"}))
        .unwrap();
    assert!(
        s.doc
            .para_at(&s.sel.focus)
            .unwrap()
            .text
            .ends_with("A table description")
    );
    s.run("edit.undo", &json!({})).unwrap();
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
}
#[test]
fn toc_edit_changes_the_existing_table_and_preserves_custom_switches() {
    let mut s = Session::new(Document::from_text("Heading\nParagraph"));
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    s.run(
        "references.toc",
        &json!({"title":"Contents","levels":2,"hyperlinks":true}),
    )
    .unwrap();
    let (at, code) = references::toc_field(&s).unwrap();
    assert!(references::toc_at(&s, &at));
    references::edit_toc(
        &mut s,
        &format!("{code} \\t \"CustomStyle,4\""),
        Some("Index"),
    )
    .unwrap();
    let (_, code) = references::toc_field(&s).unwrap();
    assert!(code.contains("CustomStyle,4"));
    assert_eq!(fields(&s, "TOC").len(), 1);
    assert!(
        s.doc
            .body
            .iter()
            .filter_map(|b| b.as_para())
            .any(|p| p.text.starts_with("Index"))
    );
}
