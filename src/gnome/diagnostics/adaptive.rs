use super::*;

pub(super) fn adaptive_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let a = ui.adaptive.borrow().as_ref().unwrap().clone();
    assert!(!a.narrow.get());
    assert!(!ui.format_toolbar.is_visible());
    let buttons = ui.format_buttons.borrow().clone();
    assert_eq!(
        a.history.parent().as_ref(),
        Some(a.top_history.upcast_ref())
    );
    assert_eq!(
        a.formats.parent().as_ref(),
        Some(a.top_formats.upcast_ref())
    );
    let style = ui.style.borrow().as_ref().unwrap().clone();
    buttons[0].popup();
    style.show_detail("Heading1");
    popover_pin(&buttons[0].popover().unwrap()).set_active(true);
    let rev = ui.current.borrow().as_ref().unwrap().session.rev();
    let selection = ui.current.borrow().as_ref().unwrap().session.sel.clone();
    window.set_default_size(520, 800);
    let ui = ui.clone();
    let window = window.clone();
    let app = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
        assert!(a.narrow.get());
        assert!(ui.format_toolbar.is_visible());
        assert_eq!(a.formats.parent().as_ref(), Some(a.bottom.upcast_ref()));
        assert_eq!(*ui.format_buttons.borrow(), buttons);
        assert!(buttons[0].popover().unwrap().is_mapped());
        assert_eq!(style.stack.visible_child_name().as_deref(), Some("detail"));
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(), rev);
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel, selection);
        eprintln!("ADAPTIVE_NARROW_OK");
        window.set_default_size(1200, 850);
        glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
            assert!(!a.narrow.get());
            assert!(!ui.format_toolbar.is_visible());
            assert!(buttons[0].popover().unwrap().is_mapped());
            assert_eq!(style.stack.visible_child_name().as_deref(), Some("detail"));
            buttons[0].popdown();
            eprintln!("ADAPTIVE_WIDE_OK");
            let origin = buttons[0].popover().unwrap();
            let body = origin.child().unwrap();
            let context = show_document_context(&ui, &window, 100, 150, DocumentContext::Text);
            context_activate_row(&context, wordcraft_gnome::t!("Style"));
            assert_eq!(
                context.visible_submenu().as_deref(),
                Some(wordcraft_gnome::t!("Style"))
            );
            assert!(origin.child().is_none());
            assert!(!origin.is_visible());
            assert!(style.stack.interpolates_size());
            assert!(native_menu_stack(&context).unwrap().interpolates_size());
            style_click(&style, "style-apply-Normal");
            assert!(context.is_visible(), "pinned context must stay open");
            style.main();
            style.header.button.emit_clicked();
            assert_eq!(context.visible_submenu().as_deref(), Some("main"));
            buttons[1].popup();
            assert!(!context.is_visible());
            assert_eq!(origin.child().as_ref(), Some(&body));
            assert!(buttons[1].popover().unwrap().is_visible());
            buttons[1].popdown();
            popover_pin(&origin).set_active(false);
            let context = show_document_context(&ui, &window, 100, 150, DocumentContext::Text);
            context_activate_row(&context, wordcraft_gnome::t!("Text"));
            let text = ui.text_tools.borrow().as_ref().unwrap().clone();
            text.page("options");
            text.page("character");
            text.header.button.emit_clicked();
            assert_eq!(text.stack.visible_child_name().as_deref(), Some("detail"));
            text.header.button.emit_clicked();
            assert_eq!(text.stack.visible_child_name().as_deref(), Some("main"));
            text.header.button.emit_clicked();
            assert_eq!(context.visible_submenu().as_deref(), Some("main"));
            assert_eq!(app.windows().len(), 1);
            context.popdown();
            let settings = ui.document_box.settings();
            let old = settings.property::<bool>("gtk-enable-animations");
            settings.set_property("gtk-enable-animations", false);
            buttons[2].popup();
            text.page("options");
            assert!(!text.stack.is_transition_running());
            assert!(text.stack.interpolates_size());
            buttons[2].popdown();
            settings.set_property("gtk-enable-animations", old);
            eprintln!("ADAPTIVE_UI_OK");
            app.quit();
        });
    });
}
