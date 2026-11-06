use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    captions::{self, Target},
    pictures, selection,
};

fn picture(s: &mut Session) -> Pos {
    static IMAGE: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    let path = IMAGE.get_or_init(|| {
        let path = std::env::temp_dir().join(format!("leaf-caption-{}.png", std::process::id()));
        std::fs::write(
            &path,
            wordcraft_engine::cmd::insert::base64_decode(include_str!("fixtures/picture.b64"))
                .unwrap(),
        )
        .unwrap();
        path
    });
    pictures::insert(s, path).unwrap();
    pictures::selected(s).unwrap().0
}
fn owner(s: &Session) -> Pos {
    match &captions::all(&s.doc)[0].owner {
        Target::Image(p) => p.clone(),
        _ => panic!("expected picture"),
    }
}
fn roundtrip(s: &Session) -> Session {
    let bytes = wordcraft_engine::io::save_bytes("captions.docx", &s.doc).unwrap();
    Session::new(wordcraft_engine::io::open_bytes("captions.docx", &bytes).unwrap())
}
#[test]
fn caption_toggle_preserves_authored_text_formatting_and_native_history_after_docx() {
    let mut s = Session::new(Document::from_text("\nTail"));
    picture(&mut s);
    let before = s.doc.clone();
    let depth = s.undo_depth();
    captions::set_visible(&mut s, true).unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("text.insert", &json!({"text":"Authored description"}))
        .unwrap();
    assert!(captions::text(&s.doc, &captions::all(&s.doc)[0]).ends_with("Authored description"));
    s.run("edit.undo", &json!({})).unwrap();
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    s.run("edit.redo", &json!({})).unwrap();
    let pos = owner(&s);
    pictures::select(&mut s, &pos).unwrap();
    captions::set_visible(&mut s, false).unwrap();
    assert!(!captions::all(&s.doc)[0].visible);
    let hidden = s.doc.clone();
    let depth = s.undo_depth();
    captions::set_visible(&mut s, false).unwrap();
    assert_eq!(s.doc, hidden);
    assert_eq!(s.undo_depth(), depth);
    let mut s = roundtrip(&s);
    assert_eq!(captions::all(&s.doc).len(), 1);
    assert!(!captions::all(&s.doc)[0].visible);
    let pos = owner(&s);
    pictures::select(&mut s, &pos).unwrap();
    captions::set_visible(&mut s, true).unwrap();
    assert_eq!(captions::all(&s.doc).len(), 1);
    assert!(captions::all(&s.doc)[0].visible);
    assert!(captions::text(&s.doc, &captions::all(&s.doc)[0]).ends_with("Authored description"));
    assert!(
        s.doc
            .custom_props
            .iter()
            .all(|p| !p.name.starts_with("Leaf.Caption.Hidden."))
    );
    let same = s.doc.clone();
    let depth = s.undo_depth();
    let pos = owner(&s);
    pictures::select(&mut s, &pos).unwrap();
    captions::set_visible(&mut s, true).unwrap();
    assert_eq!(s.doc, same);
    assert_eq!(
        s.undo_depth(),
        depth,
        "showing an existing caption does not duplicate it"
    );
}
#[test]
fn identical_pictures_keep_distinct_owners_when_dragged_and_reopened() {
    let mut s = Session::new(Document::from_text("\n\nTail"));
    picture(&mut s);
    captions::set_visible(&mut s, true).unwrap();
    selection::set_caret(&mut s, &Pos::body(2, 0), false).unwrap();
    picture(&mut s);
    captions::set_visible(&mut s, true).unwrap();
    let pairs = captions::all(&s.doc);
    assert_eq!(pairs.len(), 2);
    assert_ne!(pairs[0].id, pairs[1].id);
    let first = match &pairs[0].owner {
        Target::Image(p) => p.clone(),
        _ => unreachable!(),
    };
    pictures::select(&mut s, &first).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    pictures::drop_at(&mut s, 0, Some(&Pos::body(4, 0)), 100., 200.).unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    let moved = captions::all(&s.doc);
    assert_eq!(moved.len(), 2);
    assert_eq!(moved[0].id, pairs[0].id);
    assert_ne!(moved[0].owner, pairs[0].owner);
    assert_eq!(moved[1].owner, pairs[1].owner);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    let reopened = roundtrip(&s);
    let after = captions::all(&reopened.doc);
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].owner, moved[0].owner);
}
#[test]
fn whole_table_owner_survives_removing_its_first_row_and_docx() {
    let mut s = Session::new(Document::from_text("Tail"));
    s.run("insert.table", &json!({"rows":3,"cols":2})).unwrap();
    let target = captions::target(&s).unwrap();
    captions::set_visible(&mut s, true).unwrap();
    let pair = captions::all(&s.doc)[0].clone();
    assert_eq!(pair.owner, target);
    let Target::Table { story, path } = target else {
        unreachable!()
    };
    let cell = s
        .doc
        .para_paths(story)
        .into_iter()
        .find(|p| p.cell().is_some_and(|(t, _, _)| t == path))
        .unwrap();
    selection::set_caret(
        &mut s,
        &Pos {
            story,
            path: cell,
            off: 0,
        },
        false,
    )
    .unwrap();
    s.run("table.deleteRow", &json!({})).unwrap();
    assert_eq!(captions::all(&s.doc)[0].owner, pair.owner);
    let reopened = roundtrip(&s);
    assert_eq!(captions::all(&reopened.doc)[0].owner, pair.owner);
}
#[test]
fn no_target_and_deletion_never_reassociate_a_caption_by_proximity() {
    let mut s = Session::new(Document::from_text("First\nSecond"));
    let before = s.doc.clone();
    assert!(captions::set_visible(&mut s, true).is_err());
    assert_eq!(s.doc, before);
    picture(&mut s);
    captions::set_visible(&mut s, true).unwrap();
    let first = owner(&s);
    pictures::select(&mut s, &first).unwrap();
    pictures::delete(&mut s).unwrap();
    assert!(captions::all(&s.doc).is_empty());
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    picture(&mut s);
    assert!(
        captions::all(&s.doc).is_empty(),
        "a new adjacent picture does not own the old caption"
    );
    assert!(s.doc.plain_text(StoryRef::Body).contains("Figure 1"));
}

#[test]
fn protection_rejects_caption_changes_without_partial_anchors_or_history() {
    let mut s = Session::new(Document::from_text("\nTail"));
    picture(&mut s);
    s.doc.settings.protection = Some("readOnly".into());
    let before = s.doc.clone();
    let depth = s.undo_depth();
    assert!(captions::set_visible(&mut s, true).is_err());
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), depth);
    assert!(captions::all(&s.doc).is_empty());
}

#[test]
fn nested_tables_receive_their_caption_in_the_actual_parent_cell() {
    let mut s = Session::new(Document::from_text("Tail"));
    s.run("insert.table", &json!({"rows":2,"cols":2})).unwrap();
    s.run("insert.table", &json!({"rows":2,"cols":2})).unwrap();
    let target = captions::table_target(&s).unwrap();
    let Target::Table { path, .. } = &target else {
        unreachable!()
    };
    assert!(
        !path.0.is_empty() && path.cell().is_some(),
        "fixture must contain a nested table"
    );
    captions::set_visible(&mut s, true).unwrap();
    let caption = &captions::all(&s.doc)[0];
    assert_eq!(caption.owner, target);
    assert_eq!(caption.pos.path, path.with_last(path.last() + 1));
    assert_eq!(captions::all(&roundtrip(&s).doc)[0].owner, target);
}
#[test]
fn typed_navigation_selects_the_exact_table_or_picture_without_an_edit() {
    let mut s = Session::new(Document::from_text("Tail"));
    s.run("insert.table", &json!({"rows":2,"cols":2})).unwrap();
    picture(&mut s);
    captions::set_visible(&mut s, true).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    let entries = wordcraft_gnome::navigation::elements(&s.doc);
    let image = entries.iter().find(|e| e.kind == 3).unwrap();
    let table = entries.iter().find(|e| e.kind == 4).unwrap();
    wordcraft_gnome::navigation::activate_element(&mut s, table).unwrap();
    assert_eq!(s.cell_range().unwrap().table, table.table.clone().unwrap());
    assert!(wordcraft_gnome::navigation::element_contains(
        &s,
        table,
        &s.sel.focus
    ));
    assert!(!wordcraft_gnome::navigation::element_contains(
        &s,
        image,
        &s.sel.focus
    ));
    wordcraft_gnome::navigation::activate_element(&mut s, image).unwrap();
    assert_eq!(pictures::selected(&s).unwrap().0, image.pos);
    let caption = entries.iter().find(|e| e.kind == 6).unwrap();
    assert_eq!(caption.owner.as_ref(), Some(&image.pos));
    assert!(image.label.starts_with("Figure 1"));
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), depth);
}
#[test]
fn caption_cues_are_read_only_and_hidden_captions_have_no_visual_geometry() {
    let mut s = Session::new(Document::from_text("\nTail"));
    picture(&mut s);
    captions::set_visible(&mut s, true).unwrap();
    s.run(
        "text.insert",
        &json!({"text":"A long caption that is directly edited in the document"}),
    )
    .unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    let cues = captions::cues(&mut s);
    assert_eq!(cues.len(), 1);
    assert!(!cues[0].object.is_empty());
    assert!(!cues[0].text.is_empty());
    assert!(cues[0].bridge.iter().all(|r| r.h <= 24.
        && cues[0].object.iter().any(|o| o.page == r.page)
        && cues[0].text.iter().any(|t| t.page == r.page)));
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), depth);
    let pos = owner(&s);
    pictures::select(&mut s, &pos).unwrap();
    captions::set_visible(&mut s, false).unwrap();
    assert!(captions::cues(&mut s).is_empty());
    let mut reopened = roundtrip(&s);
    assert!(captions::cues(&mut reopened).is_empty());
}
