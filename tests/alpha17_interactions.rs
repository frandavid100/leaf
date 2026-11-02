use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{links, selection, text_properties, viewer_layout};

#[test]
fn engine_word_boundaries_trim_spaces_and_reverse_drag_keeps_full_words() {
    for suffix in [" ", "   ", "\t", "\n", ""] {
        let mut s = Session::new(Document::from_text(&if suffix.is_empty() {
            "café".into()
        } else {
            format!("café{suffix}beta gamma")
        }));
        let range = selection::word_range(&s, &Pos::body(0, 2)).unwrap();
        assert_eq!((range.0.off, range.1.off), (0, 5));
        selection::select_range(&mut s, &range.0, &range.1).unwrap();
        assert_eq!(selection::copy_text(&mut s).unwrap(), "café");
        assert_eq!(s.undo_depth(), 0);
        assert!(!s.dirty);
    }
    let mut s = Session::new(Document::from_text("alpha beta gamma delta"));
    let start = selection::word_range(&s, &Pos::body(0, 14)).unwrap();
    let (a, b) = selection::word_drag_range(&s, &start, &Pos::body(0, 2)).unwrap();
    selection::select_range(&mut s, &a, &b).unwrap();
    assert_eq!(selection::copy_text(&mut s).unwrap(), "alpha beta gamma");
    let (a, b) = selection::word_drag_range(&s, &start, &Pos::body(0, 19)).unwrap();
    selection::select_range(&mut s, &a, &b).unwrap();
    assert_eq!(selection::copy_text(&mut s).unwrap(), "gamma delta");
    assert_eq!(s.undo_depth(), 0);
}

#[test]
fn arbitrary_page_grid_has_stable_slots_and_exact_hit_geometry() {
    let sizes = vec![(400.0, 600.0); 17];
    for count in 1..=6 {
        let width = 400.0 * count as f64 + 22.0 * (count + 1) as f64;
        let a = viewer_layout::arrange(&sizes, 1.0, width);
        assert_eq!(a.columns, count);
        for (i, p) in a.pages.iter().enumerate() {
            assert_eq!(p.x, a.pages[i % count].x);
            assert_eq!(a.page_at(p.x + 30.0, p.y + 30.0), i);
        }
    }
    let a = viewer_layout::arrange(
        &[
            (400., 600.),
            (200., 400.),
            (300., 800.),
            (250., 500.),
            (300., 600.),
        ],
        1.0,
        1100.0,
    );
    assert_eq!(a.columns, 3);
    assert_eq!(a.pages[3].x, a.pages[0].x);
    for pair in a.pages[..3].windows(2) {
        assert!(pair[0].x + pair[0].width < pair[1].x);
    }
}

#[test]
fn ordinary_new_link_is_visibly_blue_underlined_and_undoable() {
    let mut s = Session::new(Document::from_text("linked text"));
    selection::select_all(&mut s).unwrap();
    let before = s.doc.clone();
    links::apply(&mut s, "https://example.org").unwrap();
    for c in text_properties::selected_resolved(&s) {
        assert_eq!(
            c.color,
            wordcraft_doc::props::TextColor::Rgb(
                wordcraft_doc::props::Rgb::parse("0066CC").unwrap()
            )
        );
        assert_eq!(c.underline, wordcraft_doc::props::Underline::Single);
    }
    assert_eq!(s.doc.plain_text(StoryRef::Body), "linked text");
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
}
