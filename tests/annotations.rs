use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    annotations as a,
    editing::{self, Edit},
    saving, selection,
};

fn session() -> Session {
    Session::new(Document::from_text(
        "Café con emoji 🦊.\nOtro párrafo para comentar.",
    ))
}

#[test]
fn bookmarks_preserve_selection_history_and_reject_silent_replacement() {
    let mut s = session();
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(1, 5)).unwrap();
    let text = selection::copy_text(&mut s).unwrap();
    a::add_bookmark(&mut s, "Examen_1").unwrap();
    a::goto_bookmark(&mut s, "Examen_1").unwrap();
    assert_eq!(selection::copy_text(&mut s).unwrap(), text);
    let rev = s.rev();
    assert!(a::add_bookmark(&mut s, "Examen_1").is_err());
    assert!(a::add_bookmark(&mut s, "_hidden").is_err());
    assert!(a::add_bookmark(&mut s, "dos palabras").is_err());
    assert_eq!(s.rev(), rev);
    a::delete_bookmark(&mut s, "Examen_1").unwrap();
    assert!(a::bookmarks(&s).is_empty());
    editing::apply(&mut s, Edit::Undo).unwrap();
    assert_eq!(a::bookmarks(&s), ["Examen_1"]);
    editing::apply(&mut s, Edit::Redo).unwrap();
    assert!(a::bookmarks(&s).is_empty());
    // WordCraft replaces names across all stories, including an unused header.
    // A body-only sidebar must not accidentally move a mark it does not show.
    let part = s.doc.add_part(
        wordcraft_doc::PartKind::Header,
        vec![wordcraft_doc::para_block(
            wordcraft_doc::Paragraph::with_text("Header", Default::default()),
        )],
    );
    let at = s.doc.start_of(StoryRef::Part(part));
    selection::set_caret(&mut s, &at, false).unwrap();
    s.run(
        "insert.bookmark",
        &serde_json::json!({"name": "HeaderMark"}),
    )
    .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    let rev = s.rev();
    assert!(a::add_bookmark(&mut s, "HeaderMark").is_err());
    assert_eq!(s.rev(), rev);
}

#[test]
fn comment_threads_edit_resolve_navigate_and_delete_with_undo() {
    let mut s = session();
    s.author = "David".into();
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
    a::add_comment(&mut s, "Revisa el café 🦊").unwrap();
    let root = a::comments(&s)[0].id;
    a::reply(&mut s, root, "Primera respuesta").unwrap();
    let reply = a::comments(&s)
        .iter()
        .find(|c| c.parent == Some(root))
        .unwrap()
        .id;
    a::edit_comment(&mut s, reply, "Respuesta corregida\nSegunda línea").unwrap();
    a::resolve(&mut s, root, true).unwrap();
    assert!(
        a::comments(&s)
            .iter()
            .find(|c| c.id == root)
            .unwrap()
            .resolved
    );
    let rev = s.rev();
    a::goto_comment(&mut s, reply).unwrap();
    assert_eq!(s.rev(), rev, "navigation must not dirty the document");
    assert_eq!(
        Some(s.sel.anchor.off),
        a::comment_anchor(&s, root).map(|p| p.off + 3)
    );
    assert_eq!(selection::copy_text(&mut s).unwrap(), "Café");
    assert_eq!(a::comments(&s)[0].author, "David");
    a::resolve(&mut s, root, false).unwrap();
    a::delete_comment(&mut s, root).unwrap();
    assert!(a::comments(&s).is_empty());
    editing::apply(&mut s, Edit::Undo).unwrap();
    assert_eq!(a::comments(&s).len(), 2);
    assert!(
        a::comments(&s)
            .iter()
            .any(|c| c.text == "Respuesta corregida\nSegunda línea")
    );
}

#[test]
fn annotation_queries_are_read_only_and_protection_is_respected() {
    let mut s = session();
    let rev = s.rev();
    assert!(a::add_comment(&mut s, " \n").is_err());
    assert!(a::bookmarks(&s).is_empty());
    assert!(a::comments(&s).is_empty());
    assert_eq!(s.rev(), rev);
    s.doc.settings.protection = Some("readOnly".into());
    assert!(!a::can_bookmark(&s));
    assert!(!a::can_comment(&s));
    assert!(a::add_comment(&mut s, "Blocked").is_err());
    s.doc.settings.protection = Some("comments".into());
    assert!(!a::can_bookmark(&s));
    assert!(a::can_comment(&s));
    a::add_comment(&mut s, "Allowed").unwrap();
}

#[test]
fn bookmarks_and_comment_threads_roundtrip_docx() {
    let dir = std::env::temp_dir().join(format!("leaf-annotations-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for ext in ["docx"] {
        let mut s = session();
        selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
        a::add_bookmark(&mut s, "Café_1").unwrap();
        a::goto_bookmark(&mut s, "Café_1").unwrap();
        a::add_comment(&mut s, "Comentario principal 🦊").unwrap();
        let root = a::comments(&s)[0].id;
        a::reply(&mut s, root, "Respuesta con acento").unwrap();
        a::resolve(&mut s, root, true).unwrap();
        let path = dir.join(format!("annotations.{ext}"));
        saving::save(&mut s, &path).unwrap();
        let mut back = wordcraft_gnome::open_document(&path).unwrap();
        assert_eq!(a::bookmarks(&back), ["Café_1"], "{ext}");
        let list = a::comments(&back);
        assert_eq!(list.len(), 2, "{ext}: {list:?}");
        let main = list.iter().find(|c| c.parent.is_none()).unwrap();
        assert_eq!(main.text, "Comentario principal 🦊", "{ext}");
        assert!(main.resolved, "{ext}");
        let response = list.iter().find(|c| c.parent == Some(main.id)).unwrap();
        assert_eq!(response.text, "Respuesta con acento", "{ext}");
        a::goto_comment(&mut back, response.id).unwrap();
        a::goto_bookmark(&mut back, "Café_1").unwrap();
        assert!(
            back.doc
                .plain_text(StoryRef::Body)
                .contains("Café con emoji 🦊.")
        );
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn odt_bookmark_locations_survive_but_comments_must_not_be_silently_saved() {
    let dir = std::env::temp_dir().join(format!("leaf-annotations-odt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("bookmarks.odt");
    let mut s = session();
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
    a::add_bookmark(&mut s, "Café_1").unwrap();
    saving::save(&mut s, &path).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut back = wordcraft_gnome::open_document(&path).unwrap();
    assert_eq!(a::bookmarks(&back), ["Café_1"]);
    a::goto_bookmark(&mut back, "Café_1").unwrap();
    // The pinned upstream exporter stores a point bookmark, not its original range.
    assert!(back.sel.is_collapsed());
    a::add_comment(&mut s, "Must not disappear").unwrap();
    assert!(saving::annotation_format_error(&s, &path).is_some());
    assert!(saving::save(&mut s, &path).unwrap_err().contains("DOCX"));
    assert!(s.dirty);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    // Reproduction for upstream, deliberately bypassing Leaf's save guard.
    let bytes = wordcraft_engine::io::save_bytes("upstream.odt", &s.doc).unwrap();
    let raw = dir.join("upstream.odt");
    std::fs::write(&raw, bytes).unwrap();
    assert!(a::comments(&wordcraft_gnome::open_document(&raw).unwrap()).is_empty());
    std::fs::remove_dir_all(dir).unwrap();
}
