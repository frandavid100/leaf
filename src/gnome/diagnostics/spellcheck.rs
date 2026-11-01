use super::*;

pub(super) fn proofing_ui_test(ui:&Rc<Ui>,parent:&ApplicationWindow,app:&Application){
    opened(ui,Session::new(wordcraft_doc::Document::from_text("El qqqgtkwordzz examen.")),"Proofing");
    let language=show_text_language(ui,parent);language.tag.set_text("es-ES");language.scope.set_selected(1);language.no_proof.set_active(false);language.apply.emit_clicked();assert!(!language.window.is_visible());
    show_spelling(ui,parent);let weak=Rc::downgrade(ui);let a=app.clone();let w=parent.clone();let attempts=Rc::new(Cell::new(0));
    glib::timeout_add_local(std::time::Duration::from_millis(100),move||{
        let ui=weak.upgrade().unwrap();attempts.set(attempts.get()+1);assert!(attempts.get()<200,"proofing scan did not finish");
        let s=ui.spelling.borrow().as_ref().unwrap().clone();if s.source.borrow().is_some(){return glib::ControlFlow::Continue;}
        assert_eq!(s.item().unwrap().text,"qqqgtkwordzz");assert!(s.add.is_sensitive());
        let context=show_document_context(&ui,&w,120,180,DocumentContext::Text);assert!(context_model_row(&context,wordcraft_gnome::t!("Add to Dictionary")).is_some());context.popdown();
        s.replacement.set_text("profesor");s.change.emit_clicked();assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc.plain_text(wordcraft_doc::StoryRef::Body),"El profesor examen.");
        let window=s.window.clone();window.close();assert!(ui.spelling.borrow().is_none());eprintln!("PROOFING_UI_OK");a.quit();glib::ControlFlow::Break
    });
}