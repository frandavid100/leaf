use serde_json::json;
use wordcraft_engine::{Session, layout::Placed};

#[test]
fn inherited_spacing_keeps_wrapped_baselines_separate_with_and_without_emoji() {
    for bytes in [
        include_bytes!("fixtures/wrapped-emoji.docx").as_slice(),
        include_bytes!("fixtures/wrapped-control.docx").as_slice(),
    ] {
        for width in [540., 575., 595.3, 615.] {
            let doc = wordcraft_engine::io::open_bytes("wrapped.docx", bytes).unwrap();
            let mut s = Session::new(doc);
            let mut section = s.doc.sections()[0].1.clone();
            section.page_w = width;
            s.run("layout.pageSetup", &json!({"section":section}))
                .unwrap();
            let original = s.doc.clone();
            let mut measured = 0;
            for p in &s.layout().pages {
                for item in &p.items {
                    if let Placed::Lines { para, l0, l1, .. } = item {
                        let lines = &para.lines[*l0..*l1];
                        for pair in lines.windows(2) {
                            assert!(pair[1].baseline - pair[0].baseline > 5., "{lines:?}");
                            assert!(pair[1].top >= pair[0].top + pair[0].height - 0.01);
                        }
                        measured += lines.len();
                    }
                }
            }
            assert!(measured >= 2, "control must wrap");
            for zoom in [0.75, 1., 2.] {
                let image = wordcraft_gnome::render_page_premultiplied(&mut s, 0, zoom).unwrap();
                assert!(!image.rgba.is_empty());
            }
            assert_eq!(s.doc, original);
            let saved = wordcraft_engine::io::save_bytes("wrapped.docx", &s.doc).unwrap();
            let reopened = wordcraft_engine::io::open_bytes("wrapped.docx", &saved).unwrap();
            assert_eq!(
                reopened.plain_text(wordcraft_doc::StoryRef::Body),
                original.plain_text(wordcraft_doc::StoryRef::Body)
            );
        }
    }
}
