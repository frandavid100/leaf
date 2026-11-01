use serde_json::json;
use wordcraft_doc::{Document,StoryRef,Pos,InlineObject};
use wordcraft_engine::Session;
use wordcraft_gnome::{header_footer::{self as hf,Region,NumberPosition},selection};

fn session()->Session {
    Session::new(Document::from_text(&(0..150).map(|i|format!("Paragraph {i}: body text remains unchanged.\n")).collect::<String>()))
}
fn fields(s:&Session,story:StoryRef)->Vec<String>{
    s.doc.para_paths(story).iter().flat_map(|path|s.doc.para(story,path).unwrap().objects.iter()).filter_map(|o|match o{InlineObject::Field{instr,..}=>Some(instr.clone()),_=>None}).collect()
}
#[test]
fn navigation_direct_editing_hit_testing_and_return_on_later_page(){
    let mut s=session();assert!(s.layout().pages.len()>2);
    selection::set_caret(&mut s,&Pos::body(100,5),false).unwrap();let body=s.sel.clone();
    assert!(hf::enter(&mut s,1,Region::Header).unwrap());
    assert_eq!(hf::active(&s),Some(Region::Header));assert_eq!(s.page_hint,1);
    s.run("text.insert",&json!({"text":"Café — shared heading"})).unwrap();
    let story=s.sel.focus.story;
    let caret=s.layout().caret_on(&s.sel.focus,1).unwrap();assert_eq!(caret.page,1);
    let hit=hf::hit(&mut s,1,caret.x,caret.top+caret.height/2.0).unwrap();assert_eq!(hit.story,story);
    selection::set_caret(&mut s,&hit,false).unwrap();assert_eq!(selection::geometry(&mut s).caret.unwrap().page,1);
    assert!(!hf::close(&mut s).unwrap());assert_eq!(s.sel,body);
    s.dirty=false;let doc=s.doc.clone();let depth=s.undo_depth();
    assert!(!hf::enter(&mut s,2,Region::Header).unwrap());assert!(!s.dirty);assert_eq!(s.doc,doc);assert_eq!(s.undo_depth(),depth);
    hf::close(&mut s).unwrap();assert_eq!(s.sel,body);assert!(!s.dirty);
    let untouched=s.sel.clone();hf::close(&mut s).unwrap();assert_eq!(s.sel,untouched);
}
#[test]
fn numbering_preserves_existing_content_is_automatic_and_undoes_atomically(){
    let mut s=session();hf::enter(&mut s,0,Region::Footer).unwrap();
    s.run("text.insert",&json!({"text":"Existing footer"})).unwrap();hf::close(&mut s).unwrap();
    let before=s.doc.clone();let body=s.sel.clone();let depth=s.undo_depth();
    hf::insert_number(&mut s,1,NumberPosition::Footer,"right",true).unwrap();
    let story=s.sel.focus.story;let text=s.doc.plain_text(story);
    assert!(text.starts_with("Existing footer"));assert_eq!(fields(&s,story),["PAGE","NUMPAGES"]);
    assert_eq!(s.doc.plain_text(StoryRef::Body),before.plain_text(StoryRef::Body));
    assert_eq!(s.undo_depth(),depth+1);assert_eq!(s.page_hint,1);
    let numbered=s.doc.clone();
    s.run("edit.undo",&json!({})).unwrap();assert_eq!(s.doc,before);assert_eq!(s.sel,body);
    s.run("edit.redo",&json!({})).unwrap();assert_eq!(s.doc,numbered);
    for ext in ["docx","odt"] {
        let bytes=wordcraft_engine::io::save_bytes(&format!("test.{ext}"),&s.doc).unwrap();
        let doc=wordcraft_engine::io::open_bytes(&format!("test.{ext}"),&bytes).unwrap();
        let id=doc.last_section.footers.default.expect("footer must survive save");
        assert!(doc.plain_text(StoryRef::Part(id)).contains("Existing footer"));
        let mut reopened=Session::new(doc);assert_eq!(fields(&reopened,StoryRef::Part(id)),["PAGE","NUMPAGES"]);
        assert!(reopened.layout().pages.len()>1);hf::enter(&mut reopened,1,Region::Footer).unwrap();
        let c=reopened.layout().caret_on(&reopened.sel.focus,1).unwrap();assert_eq!(c.page,1);
        assert_eq!(hf::hit(&mut reopened,1,c.x,c.top+c.height/2.0).unwrap().story,StoryRef::Part(id));
    }
}
#[test]
fn current_position_keeps_the_surrounding_header_text_and_empty_close_cleans_up(){
    let mut s=session();let body=s.sel.clone();
    hf::enter(&mut s,1,Region::Header).unwrap();assert!(hf::close(&mut s).unwrap());assert_eq!(s.sel,body);
    assert!(s.doc.last_section.headers.default.is_none());
    hf::enter(&mut s,1,Region::Header).unwrap();s.run("text.insert",&json!({"text":"Before After"})).unwrap();
    let mut pos=s.sel.focus.clone();pos.off=7;selection::set_caret(&mut s,&pos,false).unwrap();
    hf::insert_number(&mut s,1,NumberPosition::Current,"center",false).unwrap();
    let story=s.sel.focus.story;let text=s.doc.plain_text(story);assert!(text.starts_with("Before "));assert!(text.ends_with("After"));assert_eq!(fields(&s,story),["PAGE"]);
    let before=s.doc.clone();assert!(hf::insert_number(&mut s,usize::MAX,NumberPosition::Footer,"left",false).is_err());assert_eq!(s.doc,before);
}
#[test]
fn first_even_default_headers_use_the_displayed_story_and_survive_roundtrips(){
    let mut s=session();s.run("layout.differentFirstPage",&json!({"value":true})).unwrap();s.run("layout.differentOddEven",&json!({"value":true})).unwrap();
    let mut stories=Vec::new();
    for page in 0..3 {
        hf::enter(&mut s,page,Region::Header).unwrap();let story=s.sel.focus.story;stories.push(story);
        s.run("text.insert",&json!({"text":format!("Variant {page}")})).unwrap();hf::close(&mut s).unwrap();
    }
    assert_ne!(stories[0],stories[1]);assert_ne!(stories[1],stories[2]);
    for ext in ["docx","odt"]{
        let name=format!("variants.{ext}");let bytes=wordcraft_engine::io::save_bytes(&name,&s.doc).unwrap();let mut out=Session::new(wordcraft_engine::io::open_bytes(&name,&bytes).unwrap());
        for page in 0..3 {hf::enter(&mut out,page,Region::Header).unwrap();assert!(out.doc.plain_text(out.sel.focus.story).contains(&format!("Variant {page}")));hf::close(&mut out).unwrap();}
    }
}

#[test]
fn protected_documents_refuse_creation_and_number_insertion_without_partial_edits(){
    let mut s=session();hf::enter(&mut s,0,Region::Header).unwrap();
    s.run("text.insert",&json!({"text":"Protected header"})).unwrap();hf::close(&mut s).unwrap();
    s.run("review.restrict",&json!({"mode":"readOnly"})).unwrap();
    let before=s.doc.clone();let sel=s.sel.clone();let depth=s.undo_depth();
    assert!(hf::enter(&mut s,0,Region::Footer).is_err());assert_eq!(s.doc,before);assert_eq!(s.sel,sel);
    assert!(hf::insert_number(&mut s,0,NumberPosition::Footer,"center",true).is_err());
    assert_eq!(s.doc,before);assert_eq!(s.sel,sel);assert_eq!(s.undo_depth(),depth);
    assert!(!hf::enter(&mut s,0,Region::Header).unwrap());assert_eq!(hf::active(&s),Some(Region::Header));
    let sel=s.sel.clone();let depth=s.undo_depth();
    assert!(hf::insert_number(&mut s,0,NumberPosition::Current,"left",false).is_err());assert_eq!(s.doc,before);assert_eq!(s.sel,sel);assert_eq!(s.undo_depth(),depth);
}
