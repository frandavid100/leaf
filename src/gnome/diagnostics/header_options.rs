use super::*;

pub(in crate::gnome) fn page_options_ui_test(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    app: &Application,
) {
    let mut d = wordcraft_doc::Document::from_text("First section\nSecond section\nThird section");
    for i in 0..2 {
        if let wordcraft_doc::Block::Para(p) = std::sync::Arc::make_mut(&mut d.body[i]) {
            p.section = Some(Box::new(wordcraft_doc::section::SectionProps::default()));
        }
    }
    opened(ui, Session::new(d), "Sections");
    let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let dialog = show_document_properties(ui, parent);
    wait_dialog_presented(&dialog.window);
    for (row, value) in dialog.metadata.iter().zip([
        "Tutoría entre iguales",
        "Tutoría",
        "David",
        "inglés, clase",
        "Trabajo",
    ]) {
        row.set_text(value);
    }
    dialog.headers.odd_even.set_active(true);
    dialog.apply.emit_clicked();
    let after_metadata = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let dialog = show_section_properties(ui, parent, 1);
    wait_dialog_presented(&dialog.window);
    let switcher = find_widget(&dialog.window, "document-properties-switcher").unwrap();
    let expected = if adw::minor_version() >= 7 {
        "AdwInlineViewSwitcher"
    } else {
        "AdwViewSwitcher"
    };
    assert_eq!(switcher.type_().name(), expected);
    assert_eq!(dialog.headers.section.selected(), 1);
    dialog.headers.first.set_active(true);
    dialog.headers.header.set_value(7.0);
    dialog.headers.footer.set_value(9.0);
    dialog.headers.format.set_selected(2);
    dialog.headers.restart.set_selected(1);
    dialog.headers.start.set_value(8.0);
    dialog.headers.headers.set_selected(2);
    dialog.headers.footers.set_selected(2);
    dialog.apply.emit_clicked();
    assert_dialog_closed(&dialog.window);
    {
        let v = ui.current.borrow();
        let s = &v.as_ref().unwrap().session;
        let p = s.doc.sections()[1].1;
        assert_eq!(s.doc.core.title, "Tutoría entre iguales");
        assert_eq!(s.doc.core.subject, "Tutoría");
        assert_eq!(s.doc.core.creator, "David");
        assert_eq!(s.doc.core.keywords, "inglés, clase");
        assert_eq!(s.doc.core.category, "Trabajo");
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
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.doc,
        after_metadata
    );
    apply_user_edit(ui, Edit::Undo);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, original);
    apply_user_edit(ui, Edit::Redo);
    apply_user_edit(ui, Edit::Redo);
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let cancelled = show_document_properties(ui, parent);
    wait_dialog_presented(&cancelled.window);
    assert_eq!(cancelled.metadata[0].text(), "Tutoría entre iguales");
    cancelled.metadata[0].set_text("Cancelled title");
    cancelled.headers.first.set_active(false);
    cancelled.window.close();
    assert_dialog_closed(&cancelled.window);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
    let stale = show_document_properties(ui, parent);
    wait_dialog_presented(&stale.window);
    opened(ui, Session::new(before.clone()), "Replacement");
    stale.metadata[0].set_text("Wrong document");
    stale.apply.emit_clicked();
    assert!(stale.window.root().is_some());
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
    stale.window.close();
    assert_dialog_closed(&stale.window);
    eprintln!("DOCUMENT_METADATA_UI_OK fields=5 undo=true cancel=true stale=true");
    eprintln!("PAGE_OPTIONS_UI_OK");
    app.quit();
}
