//! Public-engine contracts checked when Leaf changes its pinned WordCraft base.
use serde_json::json;
use wordcraft_doc::{Document, InlineObject, StoryRef};
use wordcraft_engine::Session;

#[test]
fn odt_keeps_newly_supported_editable_objects() {
    for kind in ["equation", "note", "toc", "image"] {
        let mut session = Session::new(Document::from_text("Heading\nBody text."));
        match kind {
            "equation" => {
                session
                    .run(
                        "insert.equation",
                        &json!({"linear":"(a+b)/c+x^2+√x","edit":false}),
                    )
                    .unwrap();
            }
            "note" => {
                session
                    .run("references.footnote", &json!({"text":"Footnote text"}))
                    .unwrap();
            }
            "toc" => {
                session.run("para.heading1", &json!({})).unwrap();
                session.run("references.toc", &json!({"levels":3})).unwrap();
            }
            "image" => {
                session
                    .run(
                        "insert.picture",
                        &json!({"data":include_str!("fixtures/picture.b64").trim()}),
                    )
                    .unwrap();
                session
                    .run("arrange.wrap", &json!({"wrap":"square"}))
                    .unwrap();
            }
            _ => unreachable!(),
        }
        let original = session.doc.clone();
        let bytes = wordcraft_engine::io::save_bytes("object.odt", &original).unwrap();
        assert_eq!(
            session.doc, original,
            "{kind}: export must not mutate the live document"
        );
        let out = wordcraft_engine::io::open_bytes("object.odt", &bytes).unwrap();
        let mut found = false;
        out.objects_in_reading_order(&mut |o| {
            found |= match kind {
                "equation" => matches!(o, InlineObject::Equation { linear, .. } if linear.contains("x")),
                "note" => matches!(o, InlineObject::NoteRef { id, .. } if out.plain_text(StoryRef::Part(*id)).contains("Footnote text")),
                "toc" => matches!(o, InlineObject::Field { instr, .. } | InlineObject::FieldStart { instr, .. } if instr.trim_start().starts_with("TOC")),
                "image" => matches!(o, InlineObject::Image { float, .. } if float.wrap == wordcraft_doc::para::Wrap::Square),
                _ => false,
            };
        });
        assert!(
            found,
            "{kind}: the object must remain structured after ODT reopen"
        );
    }
}

#[test]
fn legacy_unchecked_text_keeps_its_original_language() {
    let source = br#"<?xml version="1.0"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0" xmlns:leaf="https://github.com/frandavid100/leaf/ns/odt/1.0" office:version="1.3">
<office:styles><style:default-style style:family="paragraph"><style:text-properties fo:language="en" fo:country="GB"/></style:default-style></office:styles>
<office:automatic-styles><style:style style:name="Unchecked" style:family="text"><style:text-properties fo:language="zxx" leaf:no-proof="true" leaf:language="es-ES"/></style:style></office:automatic-styles>
<office:body><office:text><text:p><text:span text:style-name="Unchecked">hola</text:span></text:p></office:text></office:body></office:document>"#;
    let original = wordcraft_engine::io::open_bytes("legacy.fodt", source).unwrap();
    for doc in [
        &original,
        &wordcraft_engine::io::open_bytes(
            "again.odt",
            &wordcraft_engine::io::save_bytes("again.odt", &original).unwrap(),
        )
        .unwrap(),
    ] {
        assert_eq!(doc.styles.default_chr.lang.as_deref(), Some("en-GB"));
        let p = doc.body[0].as_para().unwrap();
        let props = doc
            .styles
            .resolve_char(p.props.style.as_deref(), p.props_of_char(0));
        assert!(props.no_proof);
        assert_eq!(props.lang.as_deref(), Some("es-ES"));
    }
}
