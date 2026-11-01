use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    annotations,
    links::{self, LinkState},
    open_document, saving, selection, styles, text_properties, viewer_layout,
};

#[test]
fn odd_pages_share_left_slot_at_every_zoom_and_single_pages_remain_centered() {
    for zoom in [0.5, 0.85, 1.0, 2.0] {
        let sizes = [
            (612., 792.),
            (600., 792.),
            (420., 500.),
            (612., 600.),
            (500., 792.),
        ];
        let spread = viewer_layout::arrange(&sizes, zoom, 5000.);
        assert_eq!(spread.columns, 2);
        for i in [0, 2, 4] {
            assert_eq!(spread.pages[i].x, spread.pages[0].x);
        }
        let p = &spread.pages[4];
        assert_eq!(spread.page_at(p.x + 10., p.y + 10.), 4);
        let single = viewer_layout::arrange(&sizes, zoom, 100.);
        assert_eq!(single.columns, 1);
        for p in &single.pages {
            assert!((p.x - (single.width - p.width) / 2.).abs() < 0.001);
        }
    }
}

#[test]
fn links_create_edit_mixed_ranges_preserve_text_and_formatting_and_roundtrip() {
    let mut s = Session::new(Document::from_text("Café 🦊 beta gamma"));
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, "Café 🦊".len())).unwrap();
    s.run("format.bold", &json!({"value":true})).unwrap();
    s.run("format.color", &json!({"color":"CC1133"})).unwrap();
    let before = s.doc.clone();
    let sel = s.sel.clone();
    let depth = s.undo_depth();
    assert!(links::apply(&mut s, "https://example.org/one").unwrap());
    assert_eq!(s.undo_depth(), depth + 1);
    assert_eq!(s.sel, sel);
    assert_eq!(
        links::state(&s),
        LinkState::Uniform("https://example.org/one".into())
    );
    let props = text_properties::selected_resolved(&s);
    assert!(props.iter().all(|p| p.bold));
    assert!(
        props
            .iter()
            .all(|p| matches!(p.color,wordcraft_doc::props::TextColor::Rgb(c)if c.hex()=="CC1133"))
    );
    let text = s.doc.plain_text(StoryRef::Body);
    let rev = s.rev();
    assert!(!links::apply(&mut s, "https://example.org/one").unwrap());
    assert_eq!(s.rev(), rev);
    assert!(links::apply(&mut s, "mailto:teacher@example.org").unwrap());
    assert_eq!(s.doc.plain_text(StoryRef::Body), text);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(
        links::state(&s),
        LinkState::Uniform("https://example.org/one".into())
    );
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    selection::select_all(&mut s).unwrap();
    assert_eq!(links::state(&s), LinkState::Mixed);
    links::apply(&mut s, "https://example.org/all").unwrap();
    let dir = std::env::temp_dir().join(format!("leaf-issue-links-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for ext in ["docx", "odt"] {
        let path = dir.join(format!("links.{ext}"));
        saving::save(&mut s, &path).unwrap();
        let mut reopened = open_document(&path).unwrap();
        selection::select_all(&mut reopened).unwrap();
        assert_eq!(
            links::state(&reopened),
            LinkState::Uniform("https://example.org/all".into())
        );
        assert_eq!(reopened.doc.plain_text(StoryRef::Body), text);
    }
    std::fs::remove_dir_all(dir).unwrap();
    s.doc.settings.protection = Some("readOnly".into());
    let before = s.doc.clone();
    assert!(links::apply(&mut s, "https://example.org/protected").is_err());
    assert_eq!(s.doc, before);
}

#[test]
fn anchor_navigation_finds_overlapping_roots_without_editing_or_reply_duplicates() {
    let mut s = Session::new(Document::from_text("Alpha beta gamma outside"));
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 10)).unwrap();
    annotations::add_comment(&mut s, "First").unwrap();
    let first = annotations::comments(&s)[0].id;
    let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
    let start = p.text.find("beta").unwrap();
    let end = p.text.find("outside").unwrap() - 1;
    selection::select_range(&mut s, &Pos::body(0, start), &Pos::body(0, end)).unwrap();
    annotations::add_comment(&mut s, "Second").unwrap();
    let second = annotations::comments(&s)
        .iter()
        .find(|c| c.id != first)
        .unwrap()
        .id;
    annotations::reply(&mut s, first, "Reply").unwrap();
    let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
    let beta = Pos::body(0, p.text.find("beta").unwrap() + 1);
    let gamma = Pos::body(0, p.text.find("gamma").unwrap() + 1);
    let outside = Pos::body(0, p.text.find("outside").unwrap() + 1);
    let snapshot = (
        s.doc.clone(),
        s.sel.clone(),
        s.rev(),
        s.undo_depth(),
        s.dirty,
    );
    assert_eq!(annotations::threads_at(&s, &beta), [first, second]);
    assert_eq!(annotations::threads_at(&s, &gamma), [second]);
    assert!(annotations::threads_at(&s, &outside).is_empty());
    assert_eq!(
        (
            s.doc.clone(),
            s.sel.clone(),
            s.rev(),
            s.undo_depth(),
            s.dirty
        ),
        snapshot
    );
}

#[test]
fn font_sizes_include_effective_styles_and_other_stories_but_not_unused_styles() {
    let mut s = Session::new(Document::from_text("Heading\nBody"));
    s.run("para.heading1", &json!({})).unwrap();
    let heading = s
        .doc
        .styles
        .resolve_char(Some("Heading1"), &Default::default())
        .size as f64;
    selection::select_range(&mut s, &Pos::body(1, 0), &Pos::body(1, 4)).unwrap();
    s.run("format.set", &json!({"props":{"size":13.25}}))
        .unwrap();
    let mut unused = s.doc.styles.get("Normal").unwrap().clone();
    unused.id = "Unused".into();
    unused.name = "Unused".into();
    unused.chr.size = Some(77.5);
    s.doc.styles.styles.push(unused);
    let header = s.doc.add_part(
        wordcraft_doc::PartKind::Header,
        vec![wordcraft_doc::para_block(
            wordcraft_doc::Paragraph::with_text(
                "Header",
                wordcraft_doc::CharProps {
                    size: Some(18.5),
                    ..Default::default()
                },
            ),
        )],
    );
    assert!(s.doc.parts.contains_key(&header));
    s.doc.add_part(
        wordcraft_doc::PartKind::TextBox,
        vec![wordcraft_doc::para_block(
            wordcraft_doc::Paragraph::with_text(
                "مرحبا",
                wordcraft_doc::CharProps {
                    size: Some(11.0),
                    size_cs: Some(20.25),
                    ..Default::default()
                },
            ),
        )],
    );

    let sizes = text_properties::used_sizes(&s);
    assert!(sizes.contains(&heading));
    assert!(sizes.contains(&13.25));
    assert!(sizes.contains(&18.5));
    assert!(sizes.contains(&20.25));
    assert!(!sizes.contains(&77.5));
    assert!(sizes.windows(2).all(|w| w[0] < w[1]));
    selection::select_all(&mut s).unwrap();
    assert_eq!(styles::current(&s), None);
    assert!(!text_properties::formatting_matches(
        &s,
        "format.size",
        &json!({"size":13.25})
    ));
    selection::select_range(&mut s, &Pos::body(1, 0), &Pos::body(1, 4)).unwrap();
    assert!(text_properties::formatting_matches(
        &s,
        "format.size",
        &json!({"size":13.25})
    ));
}

#[test]
fn ordinary_paragraph_spacing_changes_real_layout_and_saved_docx_geometry() {
    let mut s = Session::new(Document::from_text("First paragraph\nSecond paragraph"));
    selection::select_all(&mut s).unwrap();
    s.run("para.spacing", &json!({"before":0,"after":0}))
        .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    let baseline = s.layout().caret(&Pos::body(1, 0)).unwrap().top;
    s.run("para.spacing", &json!({"after":24})).unwrap();
    let after = s.layout().caret(&Pos::body(1, 0)).unwrap().top;
    assert!((after - baseline - 24.0).abs() < 0.1);
    let path = std::env::temp_dir().join(format!("leaf-spacing-{}.docx", std::process::id()));
    saving::save(&mut s, &path).unwrap();
    let mut reopened = open_document(&path).unwrap();
    assert!((reopened.layout().caret(&Pos::body(1, 0)).unwrap().top - after).abs() < 0.2);
    std::fs::remove_file(path).unwrap();
}
