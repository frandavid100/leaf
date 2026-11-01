use std::sync::Arc;
use serde_json::json;
use wordcraft_doc::{Document,Block,Paragraph,PartKind,StoryRef,Pos,section::{SectionProps,NumFormat}};
use wordcraft_engine::Session;
use wordcraft_gnome::{page_options::{self,Options,Links},header_footer::{self,Region},selection};
fn fixture()->Session{
    let mut d=Document::from_text("First section\nSecond section\nThird section");
    let id=d.add_part(PartKind::Header,vec![Arc::new(Block::Para(Paragraph::with_text("Shared header",Default::default())))]);
    let mut first=SectionProps::default();first.headers.default=Some(id);
    if let Block::Para(p)=Arc::make_mut(&mut d.body[0]){p.section=Some(Box::new(first));}
    if let Block::Para(p)=Arc::make_mut(&mut d.body[1]){p.section=Some(Box::new(SectionProps::default()));}
    Session::new(d)
}
#[test]
fn independent_headers_keep_content_and_options_are_atomic(){
    let mut s=fixture();selection::set_caret(&mut s,&Pos::body(1,3),false).unwrap();let before=s.doc.clone();let sel=s.sel.clone();let depth=s.undo_depth();
    let mut o=Options::current(&s,1);o.header=18.0;o.footer=24.0;o.first=true;o.odd_even=true;o.format=NumFormat::LowerRoman;o.start=Some(7);o.headers=Links::Independent;o.footers=Links::Independent;
    page_options::apply(&mut s,&o).unwrap();assert_eq!(s.undo_depth(),depth+1);assert_eq!(s.sel,sel);assert_eq!(s.doc.plain_text(StoryRef::Body),before.plain_text(StoryRef::Body));
    let sections=s.doc.sections();assert_eq!(sections[0].1,before.sections()[0].1);let p=sections[1].1;let inherited=sections[0].1.headers.default.unwrap();let own=p.headers.default.unwrap();assert_ne!(own,inherited);assert_eq!(s.doc.plain_text(StoryRef::Part(own)),"Shared header");
    assert!(p.title_page);assert!(s.doc.settings.even_odd_headers);assert_eq!(p.page_num_start,Some(7));assert_eq!(p.page_num_format,NumFormat::LowerRoman);
    assert_eq!(s.layout().pages.iter().find(|p|p.section==1).unwrap().number,7);
    let applied=s.doc.clone();s.run("edit.undo",&json!({})).unwrap();assert_eq!(s.doc,before);s.run("edit.redo",&json!({})).unwrap();assert_eq!(s.doc,applied);
    let end=s.doc.end_of(StoryRef::Part(own));selection::set_caret(&mut s,&end,false).unwrap();s.run("text.insert",&json!({"text":" edited"})).unwrap();assert_eq!(s.doc.plain_text(StoryRef::Part(inherited)),"Shared header");
    let current=Options::current(&s,1);assert!(!page_options::apply(&mut s,&current).unwrap());
}
#[test]
fn page_options_links_and_inactive_variants_survive_both_formats(){
    let mut s=fixture();let mut o=Options::current(&s,1);o.header=21.0;o.footer=27.0;o.format=NumFormat::UpperRoman;o.start=Some(8);o.headers=Links::Independent;o.footers=Links::Independent;o.first=true;o.odd_even=true;
    page_options::apply(&mut s,&o).unwrap();
    for ext in ["docx","odt"]{
        let name=format!("options.{ext}");let bytes=wordcraft_engine::io::save_bytes(&name,&s.doc).unwrap();let out=wordcraft_engine::io::open_bytes(&name,&bytes).unwrap();let sections=out.sections();assert_eq!(sections.len(),3);let p=sections[1].1;
        assert!(p.title_page);assert!(out.settings.even_odd_headers);assert_eq!(p.page_num_start,Some(8));assert_eq!(p.page_num_format,NumFormat::UpperRoman);assert!((p.header-21.0).abs()<0.1);assert!((p.footer-27.0).abs()<0.1);
        assert!(p.headers.default.is_some());assert!(sections[2].1.headers.default.is_none(),"{ext}: later section remains linked");
        assert_eq!(out.plain_text(StoryRef::Part(p.headers.default.unwrap())),"Shared header");
    }
    let mut o=Options::current(&s,1);o.first=false;o.odd_even=false;page_options::apply(&mut s,&o).unwrap();
    let bytes=wordcraft_engine::io::save_bytes("inactive.odt",&s.doc).unwrap();let out=wordcraft_engine::io::open_bytes("inactive.odt",&bytes).unwrap();assert!(!out.sections()[1].1.title_page);assert!(out.sections()[1].1.headers.first.is_some());assert!(out.sections()[1].1.headers.even.is_some());assert!(!out.settings.even_odd_headers);
}
#[test]
fn relinking_and_header_editing_target_the_displayed_section(){
    let mut s=fixture();header_footer::enter(&mut s,1,Region::Header).unwrap();assert_eq!(page_options::current_section(&mut s),1);
    let mut o=Options::current(&s,1);o.headers=Links::Independent;page_options::apply(&mut s,&o).unwrap();let old=s.doc.sections()[1].1.headers.default.unwrap();
    let mut o=Options::current(&s,1);o.headers=Links::Previous;page_options::apply(&mut s,&o).unwrap();assert!(s.doc.sections()[1].1.headers.default.is_none());assert!(s.doc.parts.contains_key(&old));
    assert_eq!(s.layout().pages[1].header_story,s.doc.sections()[0].1.headers.default);
    s.run("review.restrict",&json!({"mode":"readOnly"})).unwrap();let doc=s.doc.clone();let depth=s.undo_depth();o.header=0.0;assert!(page_options::apply(&mut s,&o).is_err());assert_eq!(s.doc,doc);assert_eq!(s.undo_depth(),depth);
}
