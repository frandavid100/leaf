use super::*;

pub(super) fn paragraph_ui_test(
    s: &Rc<ParagraphPopover>,
    menu: &gtk::MenuButton,
    app: &Application,
) {
    s.header.pin.set_active(true);
    menu.popup();
    let ui = s.ui.upgrade().unwrap();
    let click = |name: &str| {
        find_widget(&s.popover.upgrade().unwrap().child().unwrap(), name)
            .unwrap_or_else(|| panic!("missing {name}"))
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked()
    };
    assert!(
        find_widget(&s.main, "paragraph-list")
            .unwrap()
            .is::<gtk::Button>()
    );
    click("paragraph-alignment");
    click("paragraph-align-center");
    assert_eq!(selection_paragraph(&ui)["align"], "center");
    s.home();
    click("paragraph-more-indents");
    click("paragraph-indent");
    s.home();
    assert!(selection_paragraph(&ui)["indentLeft"].as_f64().unwrap() > 0.0);
    click("paragraph-list");
    click("paragraph-more-bullets");
    assert_eq!(s.stack.visible_child_name().as_deref(), Some("detail"));
    click("paragraph-list-•");
    assert!(selection_paragraph(&ui)["numbering"].is_object());
    click("paragraph-back");
    assert_eq!(s.header.label.text(), wordcraft_gnome::t!("List"));
    click("paragraph-back");
    click("paragraph-more-spacing");
    click("paragraph-line-1.5");
    assert_eq!(selection_paragraph(&ui)["lineSpacing"]["value"], 1.5);
    click("paragraph-back");
    click("paragraph-more-borders");
    test_choose_color(&s.popover.upgrade().unwrap(), "paragraph-shading-chooser", "FFF2CC");
    assert!(selection_paragraph(&ui)["shading"].is_array());
    click("paragraph-back");
    click("paragraph-more-borders");
    click("paragraph-border-outside");
    assert!(selection_paragraph(&ui)["borders"].is_object());
    click("paragraph-back");
    click("paragraph-more-options");
    assert!(find_widget(&s.detail, "paragraph-sort-asc").is_some());
    click("paragraph-back");
    assert_eq!(s.stack.visible_child_name().as_deref(), Some("main"));
    assert_eq!(app.windows().len(), 1);
    assert!(ui.current.borrow().as_ref().unwrap().session.can_undo());
    eprintln!("PARAGRAPH_POPOVER_OK");
    app.quit();
}
