use wordcraft_doc::{Document,Pos,StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{accessible_text::{TextIndex,Change},selection,header_footer::{self,Region}};

#[test]
fn unicode_character_offsets_roundtrip_to_engine_utf8_positions(){
    let doc=Document::from_text("Café 👩‍💻\n\nFinal");
    let index=TextIndex::new(&doc,StoryRef::Body);
    assert_eq!(index.text(),"Café 👩‍💻\n\nFinal");
    assert_eq!(index.len(),index.text().chars().count()as u32);
    for offset in 0..=index.len(){let pos=index.position(offset).unwrap();assert_eq!(index.offset(&pos),Some(offset));}
    assert_eq!(index.offset(&Pos::body(0,"Café".len())),Some(4));
    assert_eq!(index.contents(3,4),Some("é"));
    assert_eq!(index.contents(0,u32::MAX),Some(index.text()));
    assert!(index.contents(4,3).is_none());assert!(index.position(index.len()+1).is_none());
    assert!(index.offset(&Pos::body(0,4)).is_none(),"middle of UTF-8 character must be rejected");
}

#[test]
fn empty_stories_paragraphs_and_embedded_control_characters_are_safe(){
    assert_eq!(TextIndex::default().contents(0,u32::MAX),Some(""));
    let doc=Document::from_text("\n\n");let index=TextIndex::new(&doc,StoryRef::Body);
    assert_eq!(index.text(),"\n\n");assert_eq!(index.paragraph(1),Some((1,2)));assert_eq!(index.paragraph(2),Some((2,2)));
    let doc=Document::from_text("a\0b\u{b}c");let index=TextIndex::new(&doc,StoryRef::Body);
    assert_eq!(index.text(),"a�b\nc");assert_eq!(index.len(),5);
    assert_eq!(index.position(3).unwrap().off,3,"offsets map to original engine bytes");
}

#[test]
fn words_and_sentences_use_unicode_boundaries_and_keep_paragraph_separators(){
    let doc=Document::from_text("Café isn't cold. Next sentence!\nFinal");let index=TextIndex::new(&doc,StoryRef::Body);
    let(a,b)=index.word(1).unwrap();assert_eq!(index.contents(a,b),Some("Café "));
    let(a,b)=index.word(7).unwrap();assert_eq!(index.contents(a,b),Some("isn't "));
    let(a,b)=index.sentence(2).unwrap();assert_eq!(index.contents(a,b),Some("Café isn't cold. "));
    let(a,b)=index.paragraph(0).unwrap();assert!(index.contents(a,b).unwrap().ends_with('\n'));
}

#[test]
fn replacement_notifications_use_character_units_in_both_versions(){
    let before=TextIndex::new(&Document::from_text("ab😀cd"),StoryRef::Body);
    let after=TextIndex::new(&Document::from_text("abéxd"),StoryRef::Body);
    assert_eq!(before.change(&after),Some(Change{start:2,old_end:4,new_end:4}));
    assert_eq!(before.contents(2,4),Some("😀c"));assert_eq!(after.contents(2,4),Some("éx"));
    assert_eq!(before.change(&before),None);
}

#[test]
fn selection_readback_and_wrapped_lines_do_not_mutate_document_or_undo(){
    let text=format!("{}\nFinal","Café words and a longer line. ".repeat(80));let mut s=Session::new(Document::from_text(&text));
    selection::select_range(&mut s,&Pos::body(0,0),&Pos::body(0,"Café".len())).unwrap();
    let before=s.doc.clone();let rev=s.rev();let depth=s.undo_depth();let sel=s.sel.clone();
    let index=TextIndex::new(&s.doc,StoryRef::Body);assert_eq!(index.selection(&s),Some((0,4)));
    let(a,b)=index.line(&mut s,5).unwrap();assert_eq!(a,0);assert!(b>5&&b<text.chars().count()as u32);
    assert!(index.contents(a,b).unwrap().starts_with("Café"));
    assert_eq!(s.doc,before);assert_eq!(s.rev(),rev);assert_eq!(s.undo_depth(),depth);assert_eq!(s.sel,sel);
}

#[test]
fn active_page_story_offsets_are_separate_from_body_offsets(){
    let mut s=Session::new(Document::from_text("Body paragraph"));header_footer::enter(&mut s,0,Region::Header).unwrap();
    s.run("text.insert",&serde_json::json!({"text":"En-tête é"})).unwrap();
    let story=s.sel.focus.story;let index=TextIndex::new(&s.doc,story);
    assert_eq!(index.text(),"En-tête é");assert_eq!(index.offset(&s.sel.focus),Some(9));
    assert!(index.offset(&Pos::body(0,0)).is_none());
}
