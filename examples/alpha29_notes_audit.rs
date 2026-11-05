//! Small reproduction of note labels and bottom-of-page placement using public APIs.
use serde_json::json;
use wordcraft_doc::{Document, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{references, selection};
fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or("/tmp/leaf29-notes".into());
    std::fs::create_dir_all(&dir).unwrap();
    for (name, text) in [
        ("new", "Hola".to_string()),
        (
            "full",
            (0..38)
                .map(|i| format!("Paragraph {i}. A populated page with a note near its end.\n"))
                .collect::<String>(),
        ),
    ] {
        let mut s = Session::new(Document::from_text(&text));
        let at = s.doc.end_of(StoryRef::Body);
        selection::set_caret(&mut s, &at, false).unwrap();
        references::insert_footnote(&mut s).unwrap();
        s.run(
            "text.insert",
            &json!({"text":"Probando los pies de página. ".repeat(if name=="new" {1}else{12})}),
        )
        .unwrap();
        let note = s.sel.focus.clone();
        let l = s.layout();
        let c = l.caret_on(&note, 0).unwrap();
        println!(
            "{name}: pages={} note caret page={} bottom={} sheet={}",
            l.pages.len(),
            c.page,
            c.top + c.height,
            l.pages[c.page].h
        );
        for (i, p) in l.pages.iter().enumerate() {
            std::fs::write(
                format!("{dir}/{name}-{i}.png"),
                wordcraft_render::render_page(&s.doc, p, 1.5, &Default::default()).to_png(),
            )
            .unwrap();
        }
    }
}
