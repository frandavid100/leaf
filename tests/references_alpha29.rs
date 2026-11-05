use serde_json::json;
use wordcraft_doc::{Document, InlineObject, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{references, selection};
fn fields(s: &Session, name: &str) -> Vec<String> {
    let mut result = Vec::new();
    for story in
        std::iter::once(StoryRef::Body).chain(s.doc.parts.keys().copied().map(StoryRef::Part))
    {
        for path in s.doc.para_paths(story) {
            for o in &s.doc.para(story, &path).unwrap().objects {
                if let InlineObject::Field {
                    instr, result: r, ..
                } = o
                {
                    if wordcraft_engine::cmd::references::FieldCode::parse(instr).name == name {
                        result.push(r.clone());
                    }
                }
            }
        }
    }
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
    references::insert_field(&mut s, r#"AUTHOR "Do not overwrite the user""#).unwrap();
    s.run("file.properties", &json!({"author":"Actual user metadata"}))
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
    assert_eq!(s.doc.core.creator, "Actual user metadata");
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

#[test]
fn save_as_refreshes_its_actual_filename_and_a_failed_save_preserves_the_session() {
    let mut s = Session::new(Document::from_text("File: "));
    references::insert_field(&mut s, "FILENAME").unwrap();
    let root = std::env::temp_dir().join(format!("leaf29-fields-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("a-real-document.docx");
    wordcraft_gnome::saving::save(&mut s, &path).unwrap();
    let copy = wordcraft_gnome::open_document(&path).unwrap();
    assert_eq!(fields(&copy, "FILENAME"), ["a-real-document.docx"]);
    let before = s.doc.clone();
    let old_path = s.path.clone();
    let depth = s.undo_depth();
    assert!(wordcraft_gnome::saving::save(&mut s, &root.join("missing-dir/other.docx")).is_err());
    assert_eq!(s.doc, before);
    assert_eq!(s.path, old_path);
    assert_eq!(s.undo_depth(), depth);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn automatic_fields_keep_their_stories_and_metadata_after_docx_roundtrip() {
    use wordcraft_gnome::header_footer::{self as hf, Region};
    let mut s = Session::new(Document::from_text("Body survives.\nSecond page."));
    s.run(
        "file.properties",
        &json!({"author":"Test author","title":"Test title"}),
    )
    .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    for instr in [
        "AUTHOR",
        "TITLE",
        "FILENAME",
        r#"DATE \@ "yyyy-MM-dd""#,
        r#"TIME \@ "HH:mm""#,
    ] {
        references::insert_field(&mut s, instr).unwrap();
    }
    hf::enter(&mut s, 0, Region::Header).unwrap();
    s.run("text.insert", &json!({"text":"Existing header "}))
        .unwrap();
    let header = s.sel.focus.story;
    let depth = s.undo_depth();
    references::insert_field(&mut s, "PAGE").unwrap();
    assert_eq!(s.sel.focus.story, header);
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert!(s.doc.plain_text(header).contains("Existing header"));
    s.run("edit.redo", &json!({})).unwrap();
    hf::close(&mut s).unwrap();
    hf::enter(&mut s, 0, Region::Footer).unwrap();
    references::insert_field(&mut s, "NUMPAGES").unwrap();
    let footer = s.sel.focus.story;
    hf::close(&mut s).unwrap();
    s.run(
        "file.properties",
        &json!({"author":"Changed author","title":"Changed title"}),
    )
    .unwrap();
    let path = std::env::temp_dir().join(format!("leaf29-all-fields-{}.docx", std::process::id()));
    saving_field_roundtrip(&mut s, &path, header, footer);
    std::fs::remove_file(path).unwrap();
}

fn saving_field_roundtrip(
    s: &mut Session,
    path: &std::path::Path,
    header: StoryRef,
    footer: StoryRef,
) {
    wordcraft_gnome::saving::save(s, path).unwrap();
    let mut copy = wordcraft_gnome::open_document(path).unwrap();
    assert_eq!(fields(&copy, "AUTHOR"), ["Changed author"]);
    assert_eq!(fields(&copy, "TITLE"), ["Changed title"]);
    for name in ["PAGE", "NUMPAGES", "FILENAME", "DATE", "TIME"] {
        assert_eq!(fields(&copy, name).len(), 1, "structured field {name}");
    }
    assert!(
        copy.doc
            .plain_text(StoryRef::Body)
            .contains("Body survives.")
    );
    assert!(copy.doc.plain_text(header).contains("Existing header"));
    assert!(copy.doc.para_paths(header).into_iter().any(|p| {
        copy.doc
            .para(header, &p)
            .unwrap()
            .objects
            .iter()
            .any(|o| matches!(o,InlineObject::Field{instr,..}if instr=="PAGE"))
    }));
    assert!(copy.doc.para_paths(footer).into_iter().any(|p| {
        copy.doc
            .para(footer, &p)
            .unwrap()
            .objects
            .iter()
            .any(|o| matches!(o,InlineObject::Field{instr,..}if instr=="NUMPAGES"))
    }));
    let pdf = path.with_extension("pdf");
    wordcraft_gnome::pdf_export::export(&mut copy, &pdf).unwrap();
    assert!(std::fs::metadata(&pdf).unwrap().len() > 1000);
    std::fs::remove_file(pdf).unwrap();
}

#[test]
fn metadata_fields_only_in_a_header_refresh_and_retain_the_odt_loss_guard() {
    use wordcraft_gnome::header_footer::{self as hf, Region};
    let mut s = Session::new(Document::from_text("No fields in the body"));
    hf::enter(&mut s, 0, Region::Header).unwrap();
    references::insert_field(&mut s, "AUTHOR").unwrap();
    hf::close(&mut s).unwrap();
    s.run("file.properties", &json!({"author":"Header-only author"}))
        .unwrap();
    assert!(references::refresh_derived(&mut s).unwrap());
    assert_eq!(fields(&s, "AUTHOR"), ["Header-only author"]);
    assert!(
        wordcraft_gnome::saving::format_preservation_error(&s, std::path::Path::new("lossy.odt"))
            .is_some()
    );
}
