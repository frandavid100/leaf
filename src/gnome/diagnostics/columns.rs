use super::*;

pub(super) fn columns_ui_test(ui:&Rc<Ui>,parent:&ApplicationWindow,app:&Application){
    let session=Session::new(wordcraft_doc::Document::from_text("Antes\nTexto para distribuir entre columnas\nDespués"));opened(ui,session,"Prueba de columnas");
    {let mut d=ui.current.borrow_mut();d.as_mut().unwrap().session.run("select.range",&serde_json::json!({"anchor":Pos::body(1,0),"focus":Pos::body(2,0)})).unwrap();}refresh_selection(ui);
    let (doc,depth,rev,text)={let d=ui.current.borrow();let s=&d.as_ref().unwrap().session;(s.doc.clone(),s.undo_depth(),s.rev(),s.selected_text())};
    let dialog=show_columns(ui,parent);assert_eq!(dialog.controls.scopes[dialog.controls.scope.selected()as usize],ColumnScope::Selection);dialog.controls.count.set_value(2.0);dialog.controls.gap.set_value(7.0);dialog.controls.separator.set_active(true);assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),rev);dialog.apply.emit_clicked();assert!(!dialog.window.is_visible());
    {let d=ui.current.borrow();let s=&d.as_ref().unwrap().session;assert_eq!(s.selected_text(),text);assert_eq!(s.undo_depth(),depth+1);let counts:Vec<_>=s.doc.sections().iter().map(|(_,p)|p.columns.count).collect();assert_eq!(counts,vec![1,2,1]);}
    apply_user_edit(ui,Edit::Undo);assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,doc);apply_user_edit(ui,Edit::Redo);
    let rev=ui.current.borrow().as_ref().unwrap().session.rev();
    let paragraph=ui.paragraph.borrow().as_ref().unwrap().clone();paragraph.page("columns");assert!(find_widget(&paragraph.detail,"columns-count").is_some());paragraph.home();assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),rev);
    let dialog=show_columns(ui,parent);dialog.controls.count.set_value(3.0);dialog.window.close();assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),rev);
    for menu in ui.format_buttons.borrow().iter(){if let Some(name)=menu.icon_name(){assert!(gtk::IconTheme::for_display(&gtk::prelude::WidgetExt::display(parent)).has_icon(&name));}}
    eprintln!("COLUMNS_UI_OK");app.quit();
}