//! Optional first-page comparison for a local document; never uploads its bytes.
use std::path::Path;
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let input = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    std::fs::create_dir_all(output).unwrap();
    let preview = wordcraft_gnome::templates::preview(input).unwrap();
    let mut session = wordcraft_gnome::open_document(input).unwrap();
    let page = wordcraft_gnome::templates::page_preview(&mut session).unwrap();
    println!(
        "preview={}x{} opened={}x{} pixels_equal={}",
        preview.width,
        preview.height,
        page.width,
        page.height,
        preview.rgba == page.rgba
    );
    for (name, image) in [("preview", &*preview), ("opened", &page)] {
        std::fs::write(output.join(format!("{name}.rgba")), &image.rgba).unwrap();
    }
    let layout = session.layout();
    let page = &layout.pages[0];
    println!("logical={}x{} pages={}", page.w, page.h, layout.pages.len());
    std::fs::write(
        output.join("opened.png"),
        wordcraft_render::render_page(&session.doc, page, 1.0, &Default::default()).to_png(),
    )
    .unwrap();
}
