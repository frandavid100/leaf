use serde_json::json;
use std::sync::atomic::AtomicBool;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    font_install as fonts, selection,
    viewer_layout::{self, ReadingAnchor},
};

#[test]
fn reading_location_survives_zoom_and_a_different_number_of_pages_per_row() {
    let sizes = vec![(600., 800.); 12];
    let old = viewer_layout::arrange(&sizes, 1., 680.);
    let p = &old.pages[7];
    let anchor = ReadingAnchor::capture(&old, 0., p.y + 230., 680., 500.).unwrap();
    assert_eq!(anchor.page, 7);
    for (zoom, width) in [(0.5, 1400.), (1.75, 680.), (1., 2200.)] {
        let next = viewer_layout::arrange(&sizes, zoom, width);
        let (x, y) = anchor.position(&next, width, 500.).unwrap();
        let restored = ReadingAnchor::capture(&next, x, y, width, 500.).unwrap();
        assert_eq!(restored.page, 7);
        assert!((restored.y - anchor.y).abs() < 0.00001);
    }
}
#[test]
fn a_reading_anchor_clamps_to_the_last_remaining_page() {
    let layout = viewer_layout::arrange(&[(600., 800.)], 1., 680.);
    let anchor = ReadingAnchor {
        page: 9,
        x: 0.5,
        y: 0.8,
        at_top: false,
    };
    assert!(anchor.position(&layout, 680., 500.).is_some());
    assert!(
        anchor
            .position(&viewer_layout::arrange(&[], 1., 680.), 680., 500.)
            .is_none()
    );
}
#[test]
fn a_view_at_the_document_top_stays_at_the_top_when_zoom_changes() {
    let sizes = [(600., 800.); 3];
    let old = viewer_layout::arrange(&sizes, 1., 680.);
    let anchor = ReadingAnchor::capture(&old, 0., 0., 680., 500.).unwrap();
    for (zoom, width) in [(0.5, 1400.), (1.75, 680.), (1.1, 700.)] {
        let next = viewer_layout::arrange(&sizes, zoom, width);
        assert_eq!(anchor.position(&next, width, 500.).unwrap().1, 0.);
    }
}
#[test]
fn missing_font_notice_uses_drawn_text_and_is_not_a_document_edit() {
    let mut s = Session::new(Document::from_text("Used font\nBody"));
    s.run(
        "styles.modify",
        &json!({"style":"Heading1","chr":{"font":"Leaf45 Unused Font"}}),
    )
    .unwrap();
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 9)).unwrap();
    s.run("format.font", &json!({"name":"Leaf45 Missing Font"}))
        .unwrap();
    let before = (
        s.doc.clone(),
        s.sel.clone(),
        s.rev(),
        s.undo_depth(),
        s.dirty,
    );
    let missing = fonts::missing(&mut s);
    assert!(missing.iter().any(|f| f.font == "Leaf45 Missing Font"));
    assert!(!missing.iter().any(|f| f.font == "Leaf45 Unused Font"));
    assert_eq!(
        before,
        (
            s.doc.clone(),
            s.sel.clone(),
            s.rev(),
            s.undo_depth(),
            s.dirty
        )
    );
}
#[test]
fn download_catalog_cannot_interpolate_arbitrary_paths_or_request_urls() {
    assert_eq!(
        fonts::catalog_file("Poppins", "Bold").as_deref(),
        Some("Poppins-Bold.ttf")
    );
    for family in [
        "Arial",
        "../Poppins",
        "Poppins/../../",
        "https://example.com",
    ] {
        assert!(fonts::catalog_file(family, "Regular").is_none());
    }
    assert!(fonts::catalog_file("Poppins", "../../font").is_none());
    let temp = std::env::temp_dir().join(format!("leaf45-cancel-{}", std::process::id()));
    assert!(fonts::download("Poppins", "Regular", &temp, &AtomicBool::new(true)).is_err());
    assert!(!temp.exists());
}
#[test]
fn validation_rejects_html_and_truncated_fonts_without_global_registration() {
    for bytes in [
        b"<html>not a font</html>".as_slice(),
        b"\0\x01\0\0garbage".as_slice(),
        &[],
    ] {
        assert!(fonts::validate(bytes, None).is_err());
    }
    let bytes = wordcraft_fonts::bundled()[0];
    assert!(fonts::validate(bytes, None).is_ok());
    assert!(fonts::validate(bytes, Some("Poppins")).is_err());
    assert!(!fonts::available("Leaf45 Validation Font"));
}
