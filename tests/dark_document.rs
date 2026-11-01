use wordcraft_doc::Document;
use wordcraft_engine::Session;
use wordcraft_gnome::{render_page_screen,render_page_premultiplied,pictures,saving,open_document};
#[test]
fn screen_appearance_preserves_document_images_history_and_saved_colors(){
 let folder=std::env::temp_dir().join(format!("leaf-dark-{}",std::process::id()));std::fs::create_dir_all(&folder).unwrap();
 let image=folder.join("sample.png");std::fs::write(&image,wordcraft_engine::cmd::insert::base64_decode(include_str!("fixtures/picture.b64")).unwrap()).unwrap();
 let mut session=Session::new(Document::from_text("Dark document: café, blue text and an image."));pictures::insert(&mut session,&image).unwrap();
 let original=session.doc.clone();let selection=session.sel.clone();let rev=session.rev();let undo=session.undo_depth();
 let light=render_page_premultiplied(&mut session,0,0.5).unwrap();let dark=render_page_screen(&mut session,0,0.5,true).unwrap();
 assert_eq!((light.width,light.height),(dark.width,dark.height));assert_eq!(&light.rgba[..4],&[255,255,255,255]);assert_eq!(&dark.rgba[..4],&[51,51,51,255]);assert_ne!(light.rgba,dark.rgba);
 // Image-colored pixels remain present without channel inversion.
 let colored=|p:&&[u8]| p[3]==255&&p[0]==100&&p[1]==180&&p[2]==130;
 let red_light=light.rgba.chunks_exact(4).filter(colored).count();let red_dark=dark.rgba.chunks_exact(4).filter(colored).count();assert!(red_light>0);assert_eq!(red_light,red_dark);
 assert_eq!(session.doc,original);assert_eq!(session.sel,selection);assert_eq!(session.rev(),rev);assert_eq!(session.undo_depth(),undo);
 for ext in ["docx","odt"]{let file=folder.join(format!("copy.{ext}"));saving::save(&mut session,&file).unwrap();let mut reopened=open_document(&file).unwrap();let paper=render_page_premultiplied(&mut reopened,0,0.25).unwrap();assert_eq!(&paper.rgba[..4],&[255,255,255,255]);}
 std::fs::remove_dir_all(folder).unwrap();
}
