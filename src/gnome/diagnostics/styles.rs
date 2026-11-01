use super::*;

pub(super) fn style_click(s:&StylePopover,name:&str){find_widget(&s.stack,name).unwrap_or_else(||panic!("missing widget {name}")).downcast::<gtk::Button>().unwrap().emit_clicked();}

pub(super) fn style_ui_test(s:&Rc<StylePopover>,menu:&gtk::MenuButton,app:&Application){
    menu.popup();let ui=s.ui.upgrade().unwrap();
    let revision=ui.current.borrow().as_ref().unwrap().session.rev();
    style_click(s,"style-manage-Heading1");assert_eq!(s.stack.visible_child_name().as_deref(),Some("detail"));
    assert!(find_widget(&s.detail,"style-preview").is_some());assert!(!find_widget(&s.detail,"style-delete").unwrap().is_sensitive());
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),revision);
    s.header.button.emit_clicked();
    style_click(s,"style-new");let name=find_widget(&s.form,"style-name").unwrap().downcast::<gtk::Entry>().unwrap();name.set_text("Prueba del desplegable");style_click(s,"style-commit");
    let id=ui.current.borrow().as_ref().unwrap().session.doc.styles.find("Prueba del desplegable").unwrap().id.clone();
    assert!(find_widget(&s.list,&format!("style-manage-{id}")).is_some());
    style_click(s,&format!("style-manage-{id}"));style_click(s,"style-rename");find_widget(&s.form,"style-name").unwrap().downcast::<gtk::Entry>().unwrap().set_text("Renombrado en desplegable");style_click(s,"style-commit");
    assert!(ui.current.borrow().as_ref().unwrap().session.doc.styles.find("Renombrado en desplegable").is_some());
    style_click(s,&format!("style-manage-{id}"));style_click(s,"style-update");style_click(s,"style-delete");
    assert!(ui.current.borrow().as_ref().unwrap().session.doc.styles.get(&id).is_none());assert_eq!(s.stack.visible_child_name().as_deref(),Some("list"));
    s.search.set_text("Heading 9");s.refresh_list();assert!(find_widget(&s.list,"style-apply-Heading9").is_some());assert!(find_widget(&s.list,"style-apply-Normal").is_none());
    style_click(s,"style-apply-Heading9");assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc.para_at(&ui.current.borrow().as_ref().unwrap().session.sel.focus).unwrap().props.style.as_deref(),Some("Heading9"));
    assert_eq!(app.windows().len(),1,"style actions must stay inside the popover");eprintln!("STYLES_POPOVER_OK");app.quit();
}