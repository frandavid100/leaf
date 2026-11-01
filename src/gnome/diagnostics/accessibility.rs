use super::*;

pub(super) fn keyboard_ui_test(ui:&Rc<Ui>, parent:&ApplicationWindow, main:&gtk::MenuButton, app:&Application) {
    let ui=ui.clone();let parent=parent.clone();let main=main.clone();let app=app.clone();
    opened(&ui, Session::new(wordcraft_doc::Document::from_text("Keyboard test")), "Teclado");
    let document=ui.current.borrow().as_ref().unwrap().session.doc.clone();
    ui.document_box.grab_focus(); eprintln!("KEYBOARD_READY 0");
    let stage=Rc::new(Cell::new(0));let ticks=Rc::new(Cell::new(0));
    glib::timeout_add_local(std::time::Duration::from_millis(150),move || {
        ticks.set(ticks.get()+1);assert!(ticks.get()<250,"keyboard test timed out");
        let marker=format!("keyboard-stage-{}",stage.get());
        if !Path::new(&marker).exists(){return glib::ControlFlow::Continue;}
        std::fs::remove_file(marker).unwrap();
        match stage.get() {
            0=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.sidebar.sidebar().is_some_and(|s|contains_focus(&s,&f)));},
            1=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.format_buttons.borrow().iter().any(|m|contains_focus(m,&f)));},
            2=>{assert!(ui.document_box.has_focus());},
            3=>{assert!(main.popover().unwrap().is_visible());},
            4=>{assert!(!main.popover().unwrap().is_visible());show_search(&ui,&parent,false);},
            5=>{assert!(ui.document_box.has_focus());assert!(!ui.search.borrow().as_ref().unwrap().toggle.is_active());},
            6=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.format_buttons.borrow().iter().any(|m|contains_focus(m,&f)));},
            7=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.sidebar.sidebar().is_some_and(|side|contains_focus(&side,&f)));},
            8=>{assert!(ui.document_box.has_focus());},
            _=>{assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,document);eprintln!("KEYBOARD_UI_OK");app.quit();return glib::ControlFlow::Break;}
        }
        stage.set(stage.get()+1);eprintln!("KEYBOARD_READY {}",stage.get());glib::ControlFlow::Continue
    });
}