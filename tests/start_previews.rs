use std::sync::Arc;
use wordcraft_doc::{Block, Document};
use wordcraft_engine::Session;
use wordcraft_gnome::{saving, templates};

#[test]
fn whole_first_page_pixels_and_geometry_survive_cache_and_file_replacement() {
    let folder = std::env::temp_dir().join(format!("leaf-preview-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let file = folder.join("A deliberately long template filename.docx");
    for (width, height, text) in [
        (595., 842., "Portrait page"),
        (842., 595., "Landscape replacement"),
    ] {
        let mut document = Document::from_text(text);
        if let Block::Para(paragraph) = Arc::make_mut(&mut document.body[0]) {
            let mut props = wordcraft_doc::section::SectionProps::default();
            props.page_w = width;
            props.page_h = height;
            paragraph.section = Some(Box::new(props));
        }
        let mut source = Session::new(document);
        saving::save(&mut source, &file).unwrap();
        let original = std::fs::read(&file).unwrap();
        let image = templates::preview(&file).unwrap();
        let cached = templates::preview(&file).unwrap();
        assert!(Arc::ptr_eq(&image, &cached));
        let mut opened = wordcraft_gnome::open_document(&file).unwrap();
        let before = (
            opened.doc.clone(),
            opened.sel.clone(),
            opened.undo_depth(),
            opened.rev(),
            opened.dirty,
        );
        let actual = templates::page_preview(&mut opened).unwrap();
        assert_eq!((image.width, image.height), (actual.width, actual.height));
        assert_eq!(image.rgba, actual.rgba);
        assert!((image.width as f32 / image.height as f32 - width / height).abs() < 0.01);
        assert_eq!(
            (
                opened.doc.clone(),
                opened.sel.clone(),
                opened.undo_depth(),
                opened.rev(),
                opened.dirty
            ),
            before
        );
        assert_eq!(std::fs::read(&file).unwrap(), original);
    }
    std::fs::remove_dir_all(folder).unwrap();
}
