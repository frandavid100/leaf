use std::sync::Arc;
use wordcraft_doc::{Block, Document, Paragraph, PartKind, StoryRef, section::NumFormat};

#[test]
fn independent_standard_odf_geometry_needs_no_leaf_metadata() {
    let bytes = include_bytes!("fixtures/odt-geometry-standard.odt");
    let doc = wordcraft_engine::io::open_bytes("independent.odt", bytes).unwrap();
    let p = &doc.last_section;
    for (actual, expected) in [
        (p.header, 21.0),
        (p.footer, 27.0),
        (p.margin_top, 72.0),
        (p.margin_bottom, 72.0),
    ] {
        assert!((actual - expected).abs() < 0.02, "{actual} != {expected}");
    }
    assert_eq!(p.page_num_format, NumFormat::UpperRoman);
    assert_eq!(
        doc.plain_text(StoryRef::Part(p.headers.default.unwrap())),
        "Independent header"
    );
    let saved = wordcraft_engine::io::save_bytes("saved.odt", &doc).unwrap();
    let next = wordcraft_engine::io::open_bytes("saved.odt", &saved).unwrap();
    assert!((next.last_section.header - 21.0).abs() < 0.02);
    assert!((next.last_section.margin_top - 72.0).abs() < 0.02);
}

#[test]
fn inactive_header_styles_do_not_enlarge_body_margins() {
    let bytes = include_bytes!("fixtures/odt-geometry-disabled.odt");
    let doc = wordcraft_engine::io::open_bytes("disabled.odt", bytes).unwrap();
    assert_eq!(doc.last_section.margin_top, 72.0);
    assert_eq!(doc.last_section.margin_bottom, 72.0);
    assert!(doc.last_section.headers.default.is_none());
    assert!(doc.last_section.footers.default.is_none());
}

#[test]
fn stale_leaf_attributes_cannot_override_standard_geometry_or_number_format() {
    let doc = wordcraft_engine::io::open_bytes(
        "externally-edited.odt",
        include_bytes!("fixtures/odt-geometry-stale.odt"),
    )
    .unwrap();
    let p = &doc.last_section;
    assert_eq!(p.page_num_format, NumFormat::UpperRoman);
    for (actual, expected) in [
        (p.header, 21.0),
        (p.footer, 27.0),
        (p.margin_top, 72.0),
        (p.margin_bottom, 72.0),
    ] {
        assert!((actual - expected).abs() < 0.02);
    }
}

#[test]
fn overlapping_nominal_margins_still_roundtrip_without_changing_live_document() {
    let mut doc = Document::from_text("Body");
    let h = doc.add_part(
        PartKind::Header,
        vec![Arc::new(Block::Para(Paragraph::with_text(
            "Header",
            Default::default(),
        )))],
    );
    let f = doc.add_part(
        PartKind::Footer,
        vec![Arc::new(Block::Para(Paragraph::with_text(
            "Footer",
            Default::default(),
        )))],
    );
    let p = &mut doc.last_section;
    p.headers.default = Some(h);
    p.footers.default = Some(f);
    p.header = 90.0;
    p.footer = 80.0;
    p.margin_top = 20.0;
    p.margin_bottom = 15.0;
    let before = doc.clone();
    let bytes = wordcraft_engine::io::save_bytes("overlap.odt", &doc).unwrap();
    assert_eq!(doc, before);
    let next = wordcraft_engine::io::open_bytes("overlap.odt", &bytes).unwrap();
    for (actual, expected) in [
        (next.last_section.header, 90.0),
        (next.last_section.footer, 80.0),
        (next.last_section.margin_top, 20.0),
        (next.last_section.margin_bottom, 15.0),
    ] {
        assert!((actual - expected).abs() < 0.02);
    }
}
