use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    saving,
    search::{self, Options, Range, Scope},
    selection, templates,
};
fn lines() -> Session {
    let mut s = Session::new(Document::from_text("one\tTwo\nTHREE\nfour"));
    let o = Options {
        special: true,
        ..Default::default()
    };
    let hits = search::query_with(&mut s, "^t", &o, None).unwrap();
    search::replace_with(&mut s, &hits, "^t", "^l", &o, None).unwrap();
    s
}
#[test]
fn pasted_line_endings_match_soft_and_paragraph_boundaries_and_replace_atomically() {
    let mut s = lines();
    let before = s.doc.clone();
    let o = Options::default();
    assert_eq!(search::query_with(&mut s, "\n", &o, None).unwrap().len(), 3);
    let hit = search::query_with(&mut s, "ONE\nTwo\nthree", &o, None).unwrap();
    assert_eq!(hit.len(), 1);
    assert_eq!(hit[0].start, Pos::body(0, 0));
    assert_eq!(hit[0].end, Pos::body(1, 5));
    assert_eq!(
        search::query_with(&mut s, "one\r\nTwo", &o, None)
            .unwrap()
            .len(),
        1
    );
    let hits = search::query_with(&mut s, "\n", &o, None).unwrap();
    search::replace_with(&mut s, &hits, "\n", " ", &o, None).unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Body), "one Two THREE four");
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Body), "one Two THREE four");
}
#[test]
fn multiline_selection_scope_and_literal_replacement_preserve_history() {
    let mut s = lines();
    selection::select_range(&mut s, &Pos::body(1, 0), &Pos::body(2, 4)).unwrap();
    let mut range = Range::capture(&s).unwrap();
    let o = Options {
        scope: Scope::Selection,
        ..Default::default()
    };
    let hits = search::query_with(&mut s, "\n", &o, Some(&range)).unwrap();
    assert_eq!(hits.len(), 1);
    search::replace_with(&mut s, &hits, "\n", "\n^p $1\t", &o, Some(&mut range)).unwrap();
    assert!(s.doc.plain_text(StoryRef::Body).contains("^p $1\t"));
    assert_eq!(s.doc.para_paths(StoryRef::Body).len(), 3);
    assert_eq!(
        search::query_with(&mut s, "^p $1\t", &Options::default(), None)
            .unwrap()
            .len(),
        1
    );
}
#[test]
fn multiline_queries_do_not_join_across_section_boundaries() {
    let mut s = lines();
    s.run("caret.set", &json!({"pos":Pos::body(1,2)})).unwrap();
    s.run("layout.break", &json!({"kind":"continuous"}))
        .unwrap();
    let before = s.doc.clone();
    let hits = search::query_with(&mut s, "\n", &Options::default(), None).unwrap();
    assert!(
        hits.iter()
            .all(|h| s.doc.para_at(&h.start).unwrap().section.is_none())
    );
    assert_eq!(s.doc, before);
}
#[test]
fn template_preview_is_cached_without_modifying_or_adding_source_files() {
    let folder = std::env::temp_dir().join(format!("leaf36-templates-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("Plantilla personal.docx");
    let mut s = Session::new(Document::from_text("An actual template page"));
    saving::save(&mut s, &path).unwrap();
    let before = std::fs::read(&path).unwrap();
    let a = templates::preview(&path).unwrap();
    let b = templates::preview(&path).unwrap();
    assert!(std::sync::Arc::ptr_eq(&a, &b));
    assert!(a.width > 0 && a.height > a.width && !a.rgba.is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn multiline_queries_do_not_bridge_tracked_deleted_text() {
    let mut s = lines();
    s.doc.settings.track_changes = true;
    selection::select_range(&mut s, &Pos::body(0, 1), &Pos::body(0, 2)).unwrap();
    s.run("text.delete", &json!({})).unwrap();
    let before = s.doc.clone();
    assert!(
        search::query_with(&mut s, "oe\nTwo", &Options::default(), None)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        search::query_with(&mut s, "e\nTwo", &Options::default(), None)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(s.doc, before);
}
