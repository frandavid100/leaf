use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    document_fonts, navigation, open_document, pictures, saving,
    search::{self, Options, Range, Scope},
    selection, styles, templates,
};
fn session(text: &str) -> Session {
    Session::new(Document::from_text(text))
}
fn text(s: &Session) -> String {
    s.doc.plain_text(StoryRef::Body)
}
fn folder() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "leaf35-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}
#[test]
fn whole_words_case_patterns_and_single_undo() {
    let mut s = session("cat catch CAT Cat cAt");
    let o = Options {
        whole_word: true,
        preserve_case: true,
        ..Default::default()
    };
    let hits = search::query_with(&mut s, "cat", &o, None).unwrap();
    assert_eq!(hits.len(), 4);
    let depth = s.undo_depth();
    search::replace_with(&mut s, &hits, "cat", "dog", &o, None).unwrap();
    assert_eq!(text(&s), "dog catch DOG Dog dog");
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(text(&s), "cat catch CAT Cat cAt");
    s.run("edit.redo", &json!({})).unwrap();
    assert_eq!(text(&s), "dog catch DOG Dog dog");
}
#[test]
fn selection_scope_survives_replacement_but_never_foreign_edits() {
    let mut s = session("cat cat cat");
    selection::select_range(&mut s, &Pos::body(0, 4), &Pos::body(0, 7)).unwrap();
    let mut range = Range::capture(&s).unwrap();
    let o = Options {
        scope: Scope::Selection,
        ..Default::default()
    };
    let hits = search::query_with(&mut s, "cat", &o, Some(&range)).unwrap();
    assert_eq!(hits.len(), 1);
    search::replace_with(&mut s, &hits, "cat", "elephant", &o, Some(&mut range)).unwrap();
    assert_eq!(text(&s), "cat elephant cat");
    assert_eq!(
        search::query_with(&mut s, "elephant", &o, Some(&range))
            .unwrap()
            .len(),
        1
    );
    assert!(
        search::query_with(&mut s, "cat", &o, Some(&range))
            .unwrap()
            .is_empty()
    );
    s.run("text.insert", &json!({"text":"x"})).unwrap();
    let before = s.doc.clone();
    assert!(search::query_with(&mut s, "cat", &o, Some(&range)).is_err());
    assert_eq!(s.doc, before);
}
#[test]
fn special_codes_and_regex_captures_remain_literal_by_default() {
    let mut s = session("cat\t12\nsecond");
    let o = Options {
        special: true,
        ..Default::default()
    };
    let hits = search::query_with(&mut s, "^t", &o, None).unwrap();
    assert_eq!(hits.len(), 1);
    search::replace_with(&mut s, &hits, "^t", "^l", &o, None).unwrap();
    assert!(text(&s).contains('\n'));
    let hits = search::query_with(&mut s, "^p", &o, None).unwrap();
    assert_eq!(hits.len(), 1);
    search::replace_with(&mut s, &hits, "^p", " ", &o, None).unwrap();
    assert_eq!(s.doc.para_paths(StoryRef::Body).len(), 1);
    let o = Options {
        regex: true,
        ..Default::default()
    };
    let hits = search::query_with(&mut s, "([0-9]+)", &o, None).unwrap();
    search::replace_with(&mut s, &hits, "([0-9]+)", "[$1]", &o, None).unwrap();
    assert!(text(&s).contains("[12]"));
    let before = s.doc.clone();
    assert!(search::query_with(&mut s, "(", &o, None).is_err());
    assert_eq!(before, s.doc);
    let mut s = session("^p $1");
    assert_eq!(search::replace_all(&mut s, "^p", "$1", false).unwrap(), 1);
    assert_eq!(text(&s), "$1 $1");
}
#[test]
fn search_notes_and_headers_once_without_leaking_caret_or_history() {
    let mut s = session("needle body");
    wordcraft_gnome::references::insert_footnote(&mut s).unwrap();
    s.run("text.insert", &json!({"text":"needle note"}))
        .unwrap();
    s.run("insert.header", &json!({})).unwrap();
    s.run("text.insert", &json!({"text":"needle header"}))
        .unwrap();
    let caret = s.sel.clone();
    let rev = s.rev();
    let depth = s.undo_depth();
    let hits = search::query_with(&mut s, "needle", &Options::default(), None).unwrap();
    assert_eq!(hits.len(), 3);
    assert_eq!(s.sel, caret);
    assert_eq!(s.rev(), rev);
    assert_eq!(s.undo_depth(), depth);
    for scope in [Scope::Body, Scope::Notes, Scope::HeadersFooters] {
        let hits = search::query_with(
            &mut s,
            "needle",
            &Options {
                scope,
                ..Default::default()
            },
            None,
        )
        .unwrap();
        assert_eq!(hits.len(), 1);
    }
}
#[test]
fn zero_width_and_stale_hits_never_partially_replace() {
    let mut s = session("alpha alpha");
    let o = Options {
        regex: true,
        ..Default::default()
    };
    let mut hits = search::query_with(&mut s, "a*", &o, None).unwrap();
    assert!(!hits.is_empty());
    let mut zero = hits[0].clone();
    zero.end = zero.start.clone();
    hits.push(zero);
    let before = s.doc.clone();
    let depth = s.undo_depth();
    assert!(search::replace_with(&mut s, &hits, "a*", "x", &o, None).is_err());
    assert_eq!(before, s.doc);
    assert_eq!(depth, s.undo_depth());
    let hits = search::query_with(&mut s, "alpha", &Options::default(), None).unwrap();
    s.run("text.insert", &json!({"text":"new"})).unwrap();
    let before = s.doc.clone();
    assert!(search::replace_with(&mut s, &hits, "alpha", "x", &Options::default(), None).is_err());
    assert_eq!(before, s.doc);
}
#[test]
fn following_style_controls_enter_and_survives_docx_and_odt() {
    let mut s = session("Title");
    styles::edit(&mut s, styles::Edit::Apply("Heading1".into())).unwrap();
    styles::edit(
        &mut s,
        styles::Edit::Next("Heading1".into(), "Normal".into()),
    )
    .unwrap();
    let dir = folder();
    for ext in ["docx", "odt"] {
        let path = dir.join(format!("next.{ext}"));
        saving::save(&mut s, &path).unwrap();
        let reopened = open_document(&path).unwrap();
        assert_eq!(
            reopened.doc.styles.get("Heading1").unwrap().next.as_deref(),
            Some("Normal")
        );
    }
    selection::set_caret(&mut s, &Pos::body(0, 5), false).unwrap();
    s.run("text.newParagraph", &json!({})).unwrap();
    assert_eq!(
        s.doc.para_at(&s.sel.focus).unwrap().props.style.as_deref(),
        Some("Normal")
    );
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn templates_are_untitled_copies_and_source_bytes_never_change() {
    let dir = folder();
    let path = dir.join("Template.docx");
    let mut original = session("Authored template");
    saving::save(&mut original, &path).unwrap();
    let before = std::fs::read(&path).unwrap();
    assert_eq!(templates::list(&dir).unwrap(), vec![path.clone()]);
    let mut copy = templates::open(&path).unwrap();
    assert!(copy.dirty);
    assert!(copy.file_name.is_none());
    assert_eq!(text(&copy), "Authored template");
    copy.run("text.insert", &json!({"text":"Changed"})).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn moving_heading_subtree_preserves_blocks_and_one_undo() {
    let mut s = session("A\na body\nA child\nchild body\nB\nb body");
    for (i, style) in [(0, "Heading1"), (2, "Heading2"), (4, "Heading1")] {
        selection::set_caret(&mut s, &Pos::body(i, 0), false).unwrap();
        s.run("para.style", &json!({"style":style})).unwrap();
    }
    let original = s.doc.clone();
    let depth = s.undo_depth();
    navigation::move_section(&mut s, 0, 3).unwrap();
    assert_eq!(text(&s), "B\nb body\nA\na body\nA child\nchild body");
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, original);
    s.run("edit.redo", &json!({})).unwrap();
    assert!(text(&s).starts_with("B"));
}
#[test]
fn font_inventory_is_pure_and_substitution_is_document_wide_and_undoable() {
    let mut s = session("Font example\nOther font");
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 4)).unwrap();
    s.run("format.font", &json!({"name":"Leaf35 Missing Font"}))
        .unwrap();
    s.run(
        "styles.modify",
        &json!({"style":"Heading1","chr":{"font":"Leaf35 Missing Font"}}),
    )
    .unwrap();
    let before = s.doc.clone();
    let rev = s.rev();
    let depth = s.undo_depth();
    let fonts = document_fonts::inventory(&s);
    assert_eq!(before, s.doc);
    assert_eq!(rev, s.rev());
    assert!(
        fonts
            .iter()
            .any(|f| f.name == "Leaf35 Missing Font" && !f.uses.is_empty())
    );
    document_fonts::substitute(&mut s, "Leaf35 Missing Font", "DejaVu Sans").unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    assert!(
        !document_fonts::inventory(&s)
            .iter()
            .any(|f| f.name == "Leaf35 Missing Font")
    );
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
}
#[test]
fn image_bytes_use_engine_sizing_and_rejected_image_is_atomic() {
    let mut s = session("Insert here");
    let before = s.doc.clone();
    let depth = s.undo_depth();
    assert!(pictures::insert_data(&mut s, b"not image").is_err());
    assert_eq!(before, s.doc);
    assert_eq!(s.undo_depth(), depth);
    let bytes =
        wordcraft_engine::cmd::insert::base64_decode(include_str!("fixtures/picture.b64")).unwrap();
    pictures::insert_data(&mut s, &bytes).unwrap();
    let (_, w, h) = pictures::selected(&s).unwrap();
    assert!((w / h - 2.).abs() < 0.01);
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
}
