use super::*;

pub(super) fn page_options_ui_test(ui: &Rc<Ui>, parent: &ApplicationWindow, app: &Application) {
    let mut d = wordcraft_doc::Document::from_text("First section\nSecond section\nThird section");
    for i in 0..2 {
        if let wordcraft_doc::Block::Para(p) = std::sync::Arc::make_mut(&mut d.body[i]) {
            p.section = Some(Box::new(wordcraft_doc::section::SectionProps::default()));
        }
    }
    opened(ui, Session::new(d), "Sections");
    let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let dialog = show_document_properties(ui, parent);
    dialog.headers.section.set_selected(1);
    dialog.headers.first.set_active(true);
    dialog.headers.odd_even.set_active(true);
    dialog.headers.header.set_value(7.0);
    dialog.headers.footer.set_value(9.0);
    dialog.headers.format.set_selected(2);
    dialog.headers.restart.set_selected(1);
    dialog.headers.start.set_value(8.0);
    dialog.headers.headers.set_selected(2);
    dialog.headers.footers.set_selected(2);
    dialog.apply.emit_clicked();
    assert!(!dialog.window.is_visible());
    {
        let v = ui.current.borrow();
        let s = &v.as_ref().unwrap().session;
        let p = s.doc.sections()[1].1;
        assert!(p.title_page);
        assert!(s.doc.settings.even_odd_headers);
        assert_eq!(p.page_num_start, Some(8));
        assert_eq!(
            p.page_num_format,
            wordcraft_doc::section::NumFormat::UpperRoman
        );
        assert!((p.header - 7.0 * PT_PER_MM as f32).abs() < 0.02);
        assert!(p.headers.default.is_some());
    }
    apply_user_edit(ui, Edit::Undo);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, original);
    apply_user_edit(ui, Edit::Redo);
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let cancelled = show_document_properties(ui, parent);
    cancelled.headers.first.set_active(false);
    cancelled.window.close();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
    eprintln!("PAGE_OPTIONS_UI_OK");
    app.quit();
}
