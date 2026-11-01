use super::*;

pub(super) fn accessible_document_ui_test(ui: &Rc<Ui>, app: &Application) {
    let fixture = "First paragraph for Orca.\nCafé, español y 😀.\nFinal paragraph.";
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(fixture)),
        "Orca test",
    );
    let doc = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    ui.document_box.grab_focus();
    let weak = Rc::downgrade(ui);
    let app = app.clone();
    let ticks = Rc::new(Cell::new(0));
    eprintln!("ACCESSIBLE_DOCUMENT_READY");
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        ticks.set(ticks.get() + 1);
        assert!(ticks.get() < 1400, "AT-SPI document test timed out");
        if !Path::new("a11y-finish").exists() {
            return glib::ControlFlow::Continue;
        }
        std::fs::remove_file("a11y-finish").unwrap();
        let ui = weak.upgrade().unwrap();
        let current = ui.current.borrow();
        let s = &current.as_ref().unwrap().session;
        assert_eq!(
            s.doc, doc,
            "read, selection and undo leave canonical document unchanged"
        );
        assert_eq!(s.sel.focus, Pos::body(0, 0));
        eprintln!("ACCESSIBLE_DOCUMENT_ENGINE_OK");
        app.quit();
        glib::ControlFlow::Break
    });
}
