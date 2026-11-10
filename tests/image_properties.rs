use serde_json::json;
use wordcraft_doc::{Document, StoryRef, para::InlineObject};
use wordcraft_engine::Session;
use wordcraft_gnome::pictures;
#[test]
fn crop_rotation_and_alternative_metadata_keep_source_data_and_round_trip_docx() {
    let mut s = Session::new(Document::new());
    let data =
        wordcraft_engine::cmd::insert::base64_decode(include_str!("fixtures/picture.b64")).unwrap();
    pictures::insert_data(&mut s, &data).unwrap();
    let media = s.doc.media.clone();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    s.run(
        "object.altText",
        &json!({"text":"A blue rectangle","decorative":true}),
    )
    .unwrap();
    s.run("arrange.rotate", &json!({"direction":"right"}))
        .unwrap();
    s.run("picture.crop", &json!({"left":0.1,"right":0.15}))
        .unwrap();
    assert_eq!(
        s.doc.media, media,
        "crop and rotation preserve original image bytes"
    );
    let bytes = wordcraft_engine::io::save_bytes("image.docx", &s.doc).unwrap();
    let reopened = wordcraft_engine::io::open_bytes("image.docx", &bytes).unwrap();
    let p = reopened
        .para_paths(StoryRef::Body)
        .into_iter()
        .find_map(|path| {
            let para = reopened.para(StoryRef::Body, &path)?;
            para.text.char_indices().find_map(|(off, _)| {
                para.object_at(off)
                    .filter(|o| matches!(o, InlineObject::Image { .. }))
                    .cloned()
            })
        })
        .unwrap();
    let InlineObject::Image {
        alt, float, crop, ..
    } = p
    else {
        panic!("image")
    };
    assert_eq!(alt, "A blue rectangle");
    assert!(float.alt.decorative);
    assert_eq!(float.rot, 90.);
    assert!((crop[0] - 0.1).abs() < 0.001 && (crop[2] - 0.15).abs() < 0.001);
    for _ in depth..s.undo_depth() {
        s.run("edit.undo", &json!({})).unwrap();
    }
    assert_eq!(s.doc, before);
}
