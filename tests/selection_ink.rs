use serde_json::json;
use wordcraft_doc::{Document, props::Rgb};
use wordcraft_engine::Session;
#[test]
fn native_selection_ink_recolors_colored_highlighted_text_without_editing() {
    let mut s = Session::new(Document::from_text(
        "Colored hyperlink and highlighted text wrap across the page.",
    ));
    s.run("select.all", &json!({})).unwrap();
    s.run("format.color", &json!({"color":"FF0000"})).unwrap();
    s.run("format.highlight", &json!({"color":"yellow"}))
        .unwrap();
    let before = (
        s.doc.clone(),
        s.sel.clone(),
        s.undo_depth(),
        s.rev(),
        s.dirty,
    );
    for scale in [0.5, 1.0, 2.0, 8.0] {
        let image =
            wordcraft_gnome::selection_ink::render(&mut s, 0, scale, Rgb(17, 201, 93)).unwrap();
        assert!((image.width as u64) * (image.height as u64) < 12_000_000);
        let visible = image
            .rgba
            .chunks_exact(4)
            .filter(|p| p[3] > 20)
            .collect::<Vec<_>>();
        assert!(!visible.is_empty(), "selected ink is present at {scale}");
        assert!(
            visible.iter().all(
                |p| [17.0, 201.0, 93.0]
                    .iter()
                    .enumerate()
                    .all(|(i, c)| (p[i] as f32 - c * p[3] as f32 / 255.0).abs() <= 2.0)
            ),
            "antialiased selected ink uses native foreground at {scale}"
        );
        assert!(image.rgba.chunks_exact(4).any(|p| p[3] == 0));
    }
    assert_eq!(
        (
            s.doc.clone(),
            s.sel.clone(),
            s.undo_depth(),
            s.rev(),
            s.dirty
        ),
        before
    );
}
