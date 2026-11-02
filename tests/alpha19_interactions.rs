use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::{Selection, Session};
use wordcraft_gnome::{
    formatting::{self, Format},
    links, navigation, saving, selection,
};

#[test]
fn link_target_spans_formatting_runs_but_not_adjacent_links_or_plain_text() {
    let mut s = Session::new(Document::from_text("plain café beta other tail"));
    selection::select_range(&mut s, &Pos::body(0, 6), &Pos::body(0, 16)).unwrap();
    links::apply(&mut s, "https://example.org/first").unwrap();
    selection::select_range(&mut s, &Pos::body(0, 12), &Pos::body(0, 16)).unwrap();
    formatting::apply(&mut s, Format::Bold).unwrap();
    selection::select_range(&mut s, &Pos::body(0, 16), &Pos::body(0, 21)).unwrap();
    links::apply(&mut s, "https://example.org/second").unwrap();
    for offset in [6, 8, 12, 15] {
        let (a, b, url) = links::range_at(&s, &Pos::body(0, offset)).unwrap();
        assert_eq!((a.off, b.off), (6, 16));
        assert_eq!(url, "https://example.org/first");
    }
    assert_eq!(
        links::range_at(&s, &Pos::body(0, 16)).unwrap().2,
        "https://example.org/second"
    );
    assert!(links::range_at(&s, &Pos::body(0, 0)).is_none());
    let old = s.doc.clone();
    let depth = s.undo_depth();
    let (a, b, _) = links::range_at(&s, &Pos::body(0, 13)).unwrap();
    links::apply_to(
        &mut s,
        &Selection {
            anchor: a,
            focus: b,
        },
        "https://example.org/edited",
        "",
    )
    .unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    assert_eq!(
        links::at(&s, &Pos::body(0, 7)).as_deref(),
        Some("https://example.org/edited")
    );
    assert_eq!(
        links::at(&s, &Pos::body(0, 17)).as_deref(),
        Some("https://example.org/second")
    );
    assert_eq!(
        s.doc.plain_text(StoryRef::Body),
        old.plain_text(StoryRef::Body)
    );
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, old);
}

#[test]
fn inserting_a_link_at_the_caret_is_one_undo_step_and_survives_formats() {
    let mut s = Session::new(Document::from_text("before after"));
    selection::set_caret(&mut s, &Pos::body(0, 7), false).unwrap();
    let before = s.doc.clone();
    let original = s.sel.clone();
    links::apply_to(
        &mut s,
        &original,
        "https://example.org/?a=1&b=<value>",
        "café ",
    )
    .unwrap();
    assert_eq!(s.undo_depth(), 1);
    assert_eq!(s.doc.plain_text(StoryRef::Body), "before café after");
    assert_eq!(
        links::at(&s, &Pos::body(0, 8)).as_deref(),
        Some("https://example.org/?a=1&b=<value>")
    );
    let dir = std::env::temp_dir().join(format!("leaf19-links-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for ext in ["docx", "odt"] {
        let path = dir.join(format!("link.{ext}"));
        saving::save(&mut s, &path).unwrap();
        let again = wordcraft_gnome::open_document(&path).unwrap();
        assert_eq!(again.doc.plain_text(StoryRef::Body), "before café after");
        assert_eq!(
            links::at(&again, &Pos::body(0, 8)).as_deref(),
            Some("https://example.org/?a=1&b=<value>")
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Body), "before café after");
}

#[test]
fn invalid_link_insertion_does_not_edit_or_add_history() {
    let mut s = Session::new(Document::from_text("intact"));
    let before = s.doc.clone();
    let target = s.sel.clone();
    for (url, text) in [("", "text"), ("https://example.org", "")] {
        assert!(links::apply_to(&mut s, &target, url, text).is_err());
        assert_eq!(s.doc, before);
        assert_eq!(s.undo_depth(), 0);
        assert!(!s.dirty);
    }
}

#[test]
fn paragraph_drag_reverses_on_document_boundaries_without_editing() {
    let mut s = Session::new(Document::from_text(
        "first paragraph\nsecond paragraph\nlast paragraph",
    ));
    let initial = wordcraft_engine::cmd::caret::unit_range(
        &mut s,
        &Pos::body(1, 4),
        wordcraft_engine::cmd::caret::Unit::Paragraph,
    );
    assert_eq!(initial, (Pos::body(1, 0), Pos::body(2, 0)));
    for (target, expected) in [
        (Pos::body(2, 4), "second paragraph\nlast paragraph"),
        (Pos::body(0, 4), "first paragraph\nsecond paragraph\n"),
        (Pos::body(1, 7), "second paragraph\n"),
    ] {
        let (a, b) = selection::paragraph_drag_range(&mut s, &initial, &target).unwrap();
        selection::select_range(&mut s, &a, &b).unwrap();
        assert_eq!(selection::copy_text(&mut s).unwrap(), expected);
        assert_eq!(s.undo_depth(), 0);
        assert!(!s.dirty);
    }
}

#[test]
fn heading_context_uses_body_line_heights_instead_of_heading_size() {
    let mut s = Session::new(Document::from_text("first\nsecond\nthird\nHeading"));
    selection::set_caret(&mut s, &Pos::body(3, 0), false).unwrap();
    formatting::apply(&mut s, Format::FontSize(32)).unwrap();
    let layout = s.layout();
    let caret = layout.caret_on(&Pos::body(3, 0), 0).unwrap();
    let mut expected = 0.;
    for item in &layout.pages[0].items {
        if let wordcraft_engine::layout::Placed::Lines {
            path, para, l0, l1, ..
        } = item
        {
            if path < &Pos::body(3, 0).path {
                expected += para.lines[*l0..*l1].iter().map(|l| l.height).sum::<f32>();
            }
        }
    }
    let (page, y) = navigation::heading_view_target(&mut s, &Pos::body(3, 0)).unwrap();
    assert_eq!(page, 0);
    assert!((caret.top - y - expected).abs() < 0.01);
}
