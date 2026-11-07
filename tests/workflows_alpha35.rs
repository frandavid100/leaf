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

#[test]
fn section_move_keeps_nested_table_image_and_note_references_intact() {
    let mut s = session("A\nTable location\nA child\nChild body\nB\nB body");
    for (i, style) in [(0, "Heading1"), (2, "Heading2"), (4, "Heading1")] {
        selection::set_caret(&mut s, &Pos::body(i, 0), false).unwrap();
        s.run("para.style", &json!({"style":style})).unwrap();
    }
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    s.run("insert.table", &json!({"rows":2,"cols":2})).unwrap();
    s.run("text.insert", &json!({"text":"Inside table"}))
        .unwrap();
    wordcraft_gnome::references::insert_footnote(&mut s).unwrap();
    s.run("text.insert", &json!({"text":"Travel with reference"}))
        .unwrap();
    wordcraft_gnome::references::return_to_reference(&mut s).unwrap();
    let png =
        wordcraft_engine::cmd::insert::base64_decode(include_str!("fixtures/picture.b64")).unwrap();
    pictures::insert_data(&mut s, &png).unwrap();
    let original = s.doc.clone();
    let headings = navigation::headings(&s.doc);
    let b = headings.iter().find(|h| h.text == "B").unwrap().pos.path.0[0] as usize;
    let mut expected = original.clone();
    expected.body.rotate_left(b);
    let depth = s.undo_depth();
    navigation::move_section(&mut s, 0, headings.len()).unwrap();
    assert_eq!(s.doc, expected);
    assert_eq!(s.undo_depth(), depth + 1);
    assert_eq!(wordcraft_gnome::references::footnote_refs(&s).len(), 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, original);
    s.run("edit.redo", &json!({})).unwrap();
    assert_eq!(s.doc, expected);
    let dir = folder();
    let path = dir.join("Moved.docx");
    saving::save(&mut s, &path).unwrap();
    let reopened = open_document(&path).unwrap();
    assert_eq!(
        wordcraft_gnome::references::footnote_refs(&reopened).len(),
        1
    );
    assert!(!reopened.doc.media.is_empty());
    assert!(reopened.doc.body.iter().any(|b| b.as_table().is_some()));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn structural_refusals_preserve_document_selection_and_history() {
    let mut s = session("A\nA body\nB\nB body");
    for i in [0, 2] {
        selection::set_caret(&mut s, &Pos::body(i, 0), false).unwrap();
        s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    }
    s.run("review.trackChanges", &json!({"value":true}))
        .unwrap();
    let before = s.doc.clone();
    let selection = s.sel.clone();
    let depth = s.undo_depth();
    let rev = s.rev();
    assert!(navigation::move_section(&mut s, 0, 2).is_err());
    assert_eq!(s.doc, before);
    assert_eq!(s.sel, selection);
    assert_eq!(s.rev(), rev);
    assert_eq!(s.undo_depth(), depth);
    assert!(document_fonts::substitute(&mut s, "Aptos", "DejaVu Sans").is_err());
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), depth);
    s.run("review.trackChanges", &json!({"value":false}))
        .unwrap();
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    s.run("layout.break", &json!({"kind":"nextPage"})).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    assert!(navigation::move_section(&mut s, 0, 2).is_err());
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), depth);
}

#[test]
fn custom_following_style_and_font_substitution_roundtrip_with_history() {
    let mut s = session("Custom heading");
    let id = styles::edit(&mut s, styles::Edit::Create("Personal heading".into()))
        .unwrap()
        .unwrap();
    let original = s.doc.clone();
    let depth = s.undo_depth();
    styles::edit(&mut s, styles::Edit::Next(id.clone(), "Heading2".into())).unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, original);
    s.run("edit.redo", &json!({})).unwrap();
    s.run("styles.modify",&json!({"style":id,"chr":{"font":"Leaf35 Missing Font","fontCs":"Leaf35 Missing Font","fontEa":"Leaf35 Missing Font"}})).unwrap();
    document_fonts::substitute(&mut s, "Leaf35 Missing Font", "DejaVu Sans").unwrap();
    let dir = folder();
    for ext in ["docx"] {
        let path = dir.join(format!("Styles.{ext}"));
        let mut copy = Session::new(s.doc.clone());
        saving::save(&mut copy, &path).unwrap();
        let reopened = open_document(&path).unwrap();
        let style = reopened.doc.styles.get(&id).unwrap();
        assert_eq!(style.next.as_deref(), Some("Heading2"), "{ext}");
        assert_eq!(style.chr.font.as_deref(), Some("DejaVu Sans"));
        assert!(
            !document_fonts::inventory(&reopened)
                .iter()
                .any(|f| f.name == "Leaf35 Missing Font")
        );
    }
    let odt = dir.join("Guarded.odt");
    let original = s.doc.clone();
    let selection = s.sel.clone();
    let depth = s.undo_depth();
    assert!(saving::requires_docx(&s));
    assert!(saving::save(&mut s, &odt).is_err());
    assert!(!odt.exists());
    assert_eq!(s.doc, original);
    assert_eq!(s.sel, selection);
    assert_eq!(s.undo_depth(), depth);
    // The font substitution also round-trips through ODT when no next-style choice is lost.
    let mut copy = Session::new(s.doc.clone());
    copy.run("styles.modify", &json!({"style":id,"next":id}))
        .unwrap();
    assert!(!styles::odt_loses_next(&copy));
    saving::save(&mut copy, &odt).unwrap();
    let reopened = open_document(&odt).unwrap();
    assert_eq!(
        reopened.doc.styles.get(&id).unwrap().chr.font.as_deref(),
        Some("DejaVu Sans")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn regex_capture_expansion_keeps_original_anchor_and_deleted_text_context() {
    let mut s = session("xfoo foo");
    let o = Options {
        regex: true,
        ..Default::default()
    };
    let pattern = "(^foo)|(foo)";
    let hits = search::query_with(&mut s, pattern, &o, None).unwrap();
    assert_eq!(hits.len(), 2);
    search::replace_with(&mut s, &hits, pattern, "$1[$2]", &o, None).unwrap();
    assert_eq!(text(&s), "x[foo] [foo]");
    let mut s = session("xxfoo");
    s.run("review.trackChanges", &json!({"value":true}))
        .unwrap();
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 2)).unwrap();
    s.run("text.delete", &json!({})).unwrap();
    let hits = search::query_with(&mut s, pattern, &o, None).unwrap();
    assert_eq!(hits.len(), 1);
    search::replace_with(&mut s, &hits, pattern, "$1-$2", &o, None).unwrap();
    let live = s
        .doc
        .para_at(&Pos::body(0, 0))
        .unwrap()
        .run_ranges()
        .filter(|(_, chr)| chr.del.is_none())
        .map(|(range, _)| &s.doc.para_at(&Pos::body(0, 0)).unwrap().text[range])
        .collect::<String>();
    assert_eq!(live, "foo-");
}

#[test]
fn painter_copies_inherited_appearance_and_does_not_copy_content_or_links() {
    let mut s = session("Heading source\nNormal target words");
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    let source = s
        .doc
        .styles
        .resolve_char(Some("Heading1"), &Default::default());
    let before = s.doc.clone();
    let rev = s.rev();
    let depth = s.undo_depth();
    wordcraft_gnome::format_painter::capture(&mut s, false).unwrap();
    assert_eq!(s.doc, before);
    assert_eq!(s.rev(), rev);
    assert_eq!(s.undo_depth(), depth);
    selection::select_range(&mut s, &Pos::body(1, 7), &Pos::body(1, 13)).unwrap();
    s.run("edit.pasteFormat", &json!({})).unwrap();
    let p = s.doc.para_at(&Pos::body(1, 7)).unwrap();
    let actual = s
        .doc
        .styles
        .resolve_char(p.props.style.as_deref(), p.props_of_char(7));
    assert_eq!(actual.size, source.size);
    assert_eq!(actual.bold, source.bold);
    assert_eq!(actual.color, source.color);
    assert_eq!(text(&s), "Heading source\nNormal target words");
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    wordcraft_gnome::format_painter::capture(&mut s, true).unwrap();
    selection::select_range(&mut s, &Pos::body(0, 8), &Pos::body(0, 14)).unwrap();
    s.run("edit.pasteFormat", &json!({})).unwrap();
    let p = s.doc.para_at(&Pos::body(0, 8)).unwrap();
    let actual = s
        .doc
        .styles
        .resolve_char(p.props.style.as_deref(), p.props_of_char(8));
    assert!(!actual.bold);
    assert_eq!(
        actual.size,
        s.doc.styles.resolve_char(None, &Default::default()).size
    );
    assert!(s.painter.is_some());
}
