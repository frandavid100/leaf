use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{document_fonts, open_document, saving, selection};

#[test]
fn docx_and_odt_keep_two_unavailable_requested_families() {
    let mut source = Session::new(Document::from_text(
        "First missing font\nSecond missing font",
    ));
    for (index, family) in [(0, "Leaf279 First Missing"), (1, "Leaf279 Second Missing")] {
        let pos = Pos::body(index, 0);
        let end = source.doc.para_at(&pos).unwrap().text.len();
        selection::select_range(&mut source, &pos, &Pos::body(index, end)).unwrap();
        source.run("format.font", &json!({"name":family})).unwrap();
    }
    let dir = std::env::temp_dir().join(format!("leaf279-roundtrip-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for extension in ["docx", "odt"] {
        let mut disposable = Session::new(source.doc.clone());
        let file = dir.join(format!("two-fonts.{extension}"));
        saving::save(&mut disposable, &file).unwrap();
        let reopened = open_document(&file).unwrap();
        for family in ["Leaf279 First Missing", "Leaf279 Second Missing"] {
            assert!(
                document_fonts::inventory(&reopened)
                    .iter()
                    .any(|f| f.name == family && !f.uses.is_empty()),
                "{extension} lost {family}"
            );
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}
