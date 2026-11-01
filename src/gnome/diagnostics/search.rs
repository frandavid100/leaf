use super::*;

pub(super) fn search_ui_test(ui:&Rc<Ui>,s:&Rc<SearchPanel>,app:&Application,host:&Rc<Workspace>){
    opened(ui,Session::new(wordcraft_doc::Document::from_text("Hola hola hola\nSecond Hola")),"Prueba de búsqueda");
    let selection=ui.current.borrow().as_ref().unwrap().session.sel.clone();
    s.show(true);s.query.set_text("hola");s.refresh();assert_eq!(s.hits.borrow().len(),4);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel,selection);
    assert!(!ui.search_highlights.borrow().is_empty());
    s.match_case.set_active(true);assert_eq!(s.hits.borrow().len(),2);
    s.match_case.set_active(false);s.replacement.set_text("Adiós");
    find_widget(&s.list,"search-replace-1").unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc.plain_text(wordcraft_doc::StoryRef::Body),"Hola Adiós hola\nSecond Hola");
    apply_user_edit(ui,Edit::Undo);s.refresh();assert_eq!(s.hits.borrow().len(),4);
    s.replacement.set_text("");s.replace_hit(0);assert_eq!(s.hits.borrow().len(),3);
    apply_user_edit(ui,Edit::Undo);s.refresh();assert_eq!(s.hits.borrow().len(),4);
    s.replacement.set_text("$1 ^p");s.replace_all.emit_clicked();assert!(s.hits.borrow().is_empty());
    apply_user_edit(ui,Edit::Undo);s.refresh();assert_eq!(s.hits.borrow().len(),4);assert!(!s.status.is_visible());
    s.toggle.set_active(false);assert!(ui.search_highlights.borrow().is_empty());s.show(false);
    let original=host.tabs.selected_page().unwrap();blank_tab(app,host);
    let new=host.docs.borrow().last().unwrap().1.clone();
    assert!(new.search.borrow().as_ref().unwrap().query.text().is_empty());
    host.tabs.set_selected_page(&original);assert_eq!(s.query.text(),"hola");
    assert_eq!(app.windows().len(),1);eprintln!("SEARCH_SIDEBAR_OK");app.quit();
}