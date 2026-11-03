use serde_json::json;
use wordcraft_doc::{Document, InlineObject, PartKind};
use wordcraft_engine::Session;
use wordcraft_gnome::saving;
fn fixture(kind: &str) -> Session {
    let mut s = Session::new(Document::from_text(
        "Heading\nBody text for the format audit.",
    ));
    match kind {
        "math" => {
            s.run(
                "insert.equation",
                &json!({"linear":"(a+b)/c+x^2+√x","edit":false}),
            )
            .unwrap();
        }
        "note" => {
            s.run("references.footnote", &json!({"text":"Footnote text"}))
                .unwrap();
        }
        "toc" => {
            s.run("para.heading1", &json!({})).unwrap();
            s.run("references.toc", &json!({"levels":3})).unwrap();
        }
        "image" => {
            s.run(
                "insert.picture",
                &json!({"data":include_str!("fixtures/picture.b64").trim()}),
            )
            .unwrap();
            s.run("arrange.wrap", &json!({"wrap":"square"})).unwrap();
        }
        _ => unreachable!(),
    }
    s
}
fn check(kind: &str) {
    let dir = std::env::temp_dir().join(format!("leaf20-preserve-{}-{kind}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = fixture(kind);
    let odt = dir.join("existing.odt");
    std::fs::write(&odt, b"original bytes").unwrap();
    assert!(saving::requires_docx(&s));
    let before = (
        s.doc.clone(),
        s.sel.clone(),
        s.rev(),
        s.dirty,
        s.path.clone(),
        s.undo_depth(),
    );
    assert!(saving::save(&mut s, &odt).unwrap_err().contains("DOCX"));
    assert_eq!(std::fs::read(&odt).unwrap(), b"original bytes");
    assert_eq!(
        (
            s.doc.clone(),
            s.sel.clone(),
            s.rev(),
            s.dirty,
            s.path.clone(),
            s.undo_depth()
        ),
        before
    );
    let docx = dir.join("safe.docx");
    saving::save(&mut s, &docx).unwrap();
    let copy = wordcraft_gnome::open_document(&docx).unwrap();
    assert!(saving::requires_docx(&copy));
    if kind == "note" {
        assert!(
            copy.doc
                .parts
                .values()
                .any(|p| p.kind == PartKind::Footnote)
        );
    }
    let mut expected = false;
    copy.doc.objects_in_reading_order(&mut |o|{expected|=match kind {
        "math"=>matches!(o,InlineObject::Equation{..}),
        "toc"=>matches!(o,InlineObject::FieldStart{instr,..}|InlineObject::Field{instr,..} if instr.starts_with("TOC")),
        "image"=>matches!(o,InlineObject::Image{float,..} if float.wrap!=wordcraft_doc::para::Wrap::Inline),
        "note"=>matches!(o,InlineObject::NoteRef{..}), _=>false,
    };});
    assert!(expected);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn editable_math_is_not_silently_flattened_on_odt_save() {
    check("math");
}
#[test]
fn footnotes_are_not_silently_lost_on_odt_save() {
    check("note");
}
#[test]
fn automatic_toc_is_not_silently_saved_as_static_odt_text() {
    check("toc");
}
#[test]
fn floating_pictures_are_not_silently_saved_as_inline_odt_images() {
    check("image");
}

#[test]
fn incompatible_breaks_preserve_docx_and_are_blocked_for_odt() {
    for kind in ["column", "continuous", "evenPage", "oddPage"] {
        let mut s = Session::new(Document::from_text("Before\nAfter"));
        s.run("layout.break", &json!({"kind": kind})).unwrap();
        assert!(saving::requires_docx(&s), "{kind}");
        let name = "breaks.docx";
        let bytes = wordcraft_engine::io::save_bytes(name, &s.doc).unwrap();
        let reopened = Session::new(wordcraft_engine::io::open_bytes(name, &bytes).unwrap());
        assert!(saving::requires_docx(&reopened), "{kind}");
        if kind != "column" {
            assert_eq!(
                wordcraft_gnome::navigation::section_starts(&reopened.doc).len(),
                1
            );
        }
    }
}
