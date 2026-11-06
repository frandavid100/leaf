use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, references, saving, selection};

#[test]
fn footnote_edit_navigation_history_and_docx_roundtrip() {
    let mut s = Session::new(Document::from_text("First reference.\nSecond reference."));
    selection::set_caret(&mut s, &Pos::body(0, 5), false).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    references::insert_footnote(&mut s).unwrap();
    assert!(matches!(s.sel.focus.story, StoryRef::Part(_)));
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("text.insert", &json!({"text":"A real footnote."}))
        .unwrap();
    let note = s.sel.focus.story;
    references::return_to_reference(&mut s).unwrap();
    assert_eq!(s.sel.focus.story, StoryRef::Body);
    references::open_footnote(&mut s, false).unwrap();
    assert_eq!(s.sel.focus.story, note);
    let layout = s.layout();
    assert!(layout.caret_on(&s.sel.focus, 0).is_some());
    references::return_to_reference(&mut s).unwrap();
    let path = std::env::temp_dir().join(format!("leaf-footnote-{}.docx", std::process::id()));
    saving::save(&mut s, &path).unwrap();
    let mut reopened = open_document(&path).unwrap();
    assert_eq!(references::footnote_refs(&reopened).len(), 1);
    references::open_footnote(&mut reopened, true).unwrap();
    assert!(
        reopened
            .doc
            .plain_text(reopened.sel.focus.story)
            .contains("A real footnote.")
    );
    references::return_to_reference(&mut reopened).unwrap();
    reopened.run("text.backspace", &json!({})).unwrap();
    assert!(references::footnote_refs(&reopened).is_empty());
    reopened.run("edit.undo", &json!({})).unwrap();
    assert_eq!(references::footnote_refs(&reopened).len(), 1);
    assert!(saving::format_preservation_error(&reopened, &path.with_extension("odt")).is_some());
    std::fs::remove_file(path).unwrap();
    while s.can_undo() {
        s.run("edit.undo", &json!({})).unwrap();
    }
    assert_eq!(s.doc, before);
}

#[test]
fn toc_and_cross_reference_remain_structured_and_updatable() {
    let mut s = Session::new(Document::from_text(
        "Chapter One\nA paragraph\nSee the heading.",
    ));
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    selection::set_caret(&mut s, &Pos::body(2, 16), false).unwrap();
    s.run(
        "insert.crossReference",
        &json!({"to":"heading","target":"Chapter One","show":"text"}),
    )
    .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    s.run("references.toc", &json!({"title":"Contents","levels":3}))
        .unwrap();
    references::update(&mut s).unwrap();
    let path = std::env::temp_dir().join(format!("leaf-references-{}.docx", std::process::id()));
    saving::save(&mut s, &path).unwrap();
    let mut reopened = open_document(&path).unwrap();
    let (_, mut heading) = wordcraft_engine::cmd::citations::ref_targets(&reopened, "heading")
        .into_iter()
        .find(|(text, _)| text == "Chapter One")
        .unwrap();
    heading.off = reopened
        .doc
        .para_at(&heading)
        .unwrap()
        .text
        .find("Chapter One")
        .unwrap();
    let mut end = heading.clone();
    end.off += "Chapter One".len();
    selection::select_range(&mut reopened, &heading, &end).unwrap();
    reopened
        .run("text.insert", &json!({"text":"Renamed heading"}))
        .unwrap();
    references::update(&mut reopened).unwrap();
    let mut refreshed = false;
    reopened.doc.objects_in_reading_order(&mut |o| {
        if let wordcraft_doc::para::InlineObject::Field { instr, result, .. } = o {
            if instr.starts_with("REF ") {
                refreshed |= result.contains("Renamed heading");
            }
        }
    });
    assert!(
        refreshed,
        "cross-reference must update after the target heading is edited"
    );
    let value = serde_json::to_string(&reopened.doc).unwrap();
    assert!(value.contains("TOC "));
    assert!(value.contains("REF _Ref"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn long_notes_paginate_and_export_with_editable_story() {
    let mut s = Session::new(Document::from_text("An annotated document."));
    references::insert_footnote(&mut s).unwrap();
    s.run("text.insert",&json!({"text":"A long explanatory note with enough text to continue across pages. ".repeat(400)})).unwrap();
    let layout = s.layout();
    assert!(layout.pages.len() > 1);
    assert!(layout.caret_on(&s.sel.focus, 0).is_some());
    let pdf = std::env::temp_dir().join(format!("leaf-long-note-{}.pdf", std::process::id()));
    wordcraft_gnome::pdf_export::export(&mut s, &pdf).unwrap();
    assert!(std::fs::metadata(&pdf).unwrap().len() > 1000);
    std::fs::remove_file(pdf).unwrap();
}

#[test]
fn toc_destinations_use_distinct_stored_bookmarks_and_navigation_is_read_only() {
    let mut s = Session::new(Document::from_text(
        "Same title\nBody one\nSame title\nBody two",
    ));
    for n in [0, 2] {
        selection::set_caret(&mut s, &Pos::body(n, 0), false).unwrap();
        s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    }
    selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
    references::insert_toc(
        &mut s,
        &json!({"title":"Contents","levels":3,"hyperlinks":true}),
    )
    .unwrap();
    let entries = s
        .doc
        .para_paths(StoryRef::Body)
        .into_iter()
        .filter_map(|path| {
            let p = s.doc.para(StoryRef::Body, &path)?;
            (p.props.style.as_deref() == Some("TOC1")).then_some(Pos {
                story: StoryRef::Body,
                path,
                off: 0,
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 2);
    let before = s.doc.clone();
    let depth = s.undo_depth();
    let destinations = entries
        .iter()
        .map(|p| references::toc_destination(&s, p).unwrap())
        .collect::<Vec<_>>();
    assert_ne!(destinations[0], destinations[1]);
    for (pos, url) in entries.iter().zip(destinations) {
        assert!(references::toc_span_at(&s, pos).is_some());
        wordcraft_gnome::annotations::goto_bookmark(&mut s, url.trim_start_matches('#')).unwrap();
        assert!(
            !s.doc
                .para_at(&s.sel.focus)
                .unwrap()
                .props
                .style
                .as_deref()
                .unwrap_or("")
                .starts_with("TOC")
        );
    }
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), depth);
    assert!(references::toc_span_at(&s, &s.sel.focus).is_none());
    let index = wordcraft_gnome::navigation::elements(&s.doc);
    assert!(index.iter().any(|e| e.kind == 5));
    assert!(index.iter().any(|e| e.kind == 10));
    assert!(index.windows(2).all(|p| p[0].pos <= p[1].pos));
}
