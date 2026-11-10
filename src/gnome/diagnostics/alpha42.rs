//! Behavioral checks on the shipped controls, reused by native and installed runs.
use super::*;
async fn settle() {
    glib::timeout_future(std::time::Duration::from_millis(450)).await;
}
fn checks(root: &gtk::Widget) -> Vec<gtk::CheckButton> {
    let mut result = Vec::new();
    if let Some(check) = root.downcast_ref::<gtk::CheckButton>() {
        result.push(check.clone());
    }
    let mut child = root.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        result.extend(checks(&widget));
    }
    result
}
pub(super) fn response(root: &gtk::Widget, label: &str) -> bool {
    if let Some(button) = root.downcast_ref::<gtk::Button>() {
        if button.label().as_deref() == Some(label) {
            button.emit_clicked();
            return true;
        }
    }
    let mut child = root.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        if response(&widget, label) {
            return true;
        }
    }
    false
}
pub(super) async fn controls(ui: &Rc<Ui>, window: &ApplicationWindow) {
    let text = find_widget(&ui.format_toolbar, "format-text-group").unwrap();
    let para = find_widget(&ui.format_toolbar, "format-paragraph-group").unwrap();
    assert!(
        text.compute_bounds(&ui.format_toolbar).unwrap().x()
            < para.compute_bounds(&ui.format_toolbar).unwrap().x()
    );
    let paragraph = ui.paragraph.borrow().as_ref().unwrap().clone();
    for close_first in [false, true] {
        format_button(ui, "format-quick-alignment").unwrap().popup();
        settle().await;
        if close_first {
            paragraph.popover.upgrade().unwrap().popdown();
            settle().await;
        }
        format_button(ui, "paragraph-menu").unwrap().popup();
        for _ in 0..15 {
            glib::timeout_future(std::time::Duration::from_millis(10)).await;
            assert!(
                !paragraph.stack.is_transition_running(),
                "independent entry must not animate Back"
            );
        }
        assert_eq!(
            paragraph.stack.visible_child_name().as_deref(),
            Some("main")
        );
        assert!(!paragraph.header.arrow.is_visible());
        paragraph.page("alignment");
        assert_ne!(
            paragraph.stack.transition_type(),
            gtk::StackTransitionType::None
        );
        paragraph.popover.upgrade().unwrap().popdown();
        settle().await;
    }
    let theme = gtk::IconTheme::for_display(&gdk::Display::default().unwrap());
    assert!(theme.has_icon(APP_ID) && theme.has_icon(&format!("{APP_ID}-symbolic")));
    for size in [32, 64] {
        let icon = theme.lookup_icon(
            APP_ID,
            &[],
            size,
            window.scale_factor(),
            gtk::TextDirection::None,
            gtk::IconLookupFlags::empty(),
        );
        assert!(icon.intrinsic_width() > 0 && icon.intrinsic_height() > 0);
    }
    let search = ui.search.borrow().as_ref().unwrap().clone();
    search.show(false);
    search.query.set_text("Alpha");
    settle().await;
    let options = find_widget(&ui.sidebar_pages, "search-options").unwrap();
    let field = search.query.compute_bounds(window).unwrap();
    let target = options.compute_bounds(window).unwrap();
    assert!(target.x() >= field.x() && target.x() + target.width() <= field.x() + field.width());
    assert!(
        gtk::IconTheme::for_display(&gdk::Display::default().unwrap())
            .has_icon("nautilus-search-filters-symbolic")
    );
    search.toggle.set_active(false);
    ui.sidebar_button.set_active(true);
    window.set_default_size(420, 850);
    settle().await;
    assert!(ui.sidebar.is_collapsed() && !ui.sidebar.shows_sidebar());
    ui.sidebar_button.set_active(true);
    settle().await;
    assert!(ui.sidebar.shows_sidebar());
    ui.sidebar_button.set_active(false);
    window.set_default_size(1280, 850);
    settle().await;
    assert!(
        ui.sidebar.shows_sidebar(),
        "temporary overlay dismissal preserves the wide preference"
    );
    ui.sidebar_button.set_active(false);
    window.set_default_size(420, 850);
    settle().await;
    window.set_default_size(1280, 850);
    settle().await;
    assert!(
        !ui.sidebar.shows_sidebar(),
        "a hidden wide sidebar stays hidden"
    );
    ui.sidebar_button.set_active(true);
    settle().await;
    // A second tab must not replace the first tab's adaptive breakpoint.
    let host = ui.host.upgrade().unwrap();
    let original = ui.tab_page.borrow().as_ref().unwrap().clone();
    blank_tab(&window.application().unwrap(), &host);
    let second = selected_ui(&host).unwrap();
    let second_page = second.tab_page.borrow().as_ref().unwrap().clone();
    settle().await;
    second.sidebar_button.set_active(false);
    for page in [&original, &second_page, &original] {
        host.tabs.set_selected_page(page);
        settle().await;
        window.set_default_size(420, 850);
        settle().await;
        let active = selected_ui(&host).unwrap();
        assert!(active.sidebar.is_collapsed() && !active.sidebar.shows_sidebar());
        window.set_default_size(1280, 850);
        settle().await;
        assert!(!active.sidebar.is_collapsed());
        assert_eq!(active.sidebar.shows_sidebar(), Rc::ptr_eq(&active, ui));
    }
    host.tabs.close_page(&second_page);
    settle().await;
    assert!(second.closed.get());
    let model = ui
        .document_menu
        .borrow()
        .as_ref()
        .unwrap()
        .upgrade()
        .unwrap()
        .downcast::<gtk::PopoverMenu>()
        .unwrap()
        .menu_model()
        .unwrap();
    let save = model.item_link(2, "section").unwrap();
    let actions: Vec<_> = (0..save.n_items())
        .map(|i| {
            save.item_attribute_value(i, "action", None)
                .unwrap()
                .get::<String>()
                .unwrap()
        })
        .collect();
    assert_eq!(
        actions,
        ["win.save", "win.save-as", "win.export-pdf", "win.print"]
    );
    let properties = model.item_link(3, "section").unwrap();
    assert_eq!(properties.n_items(), 1);
    eprintln!(
        "ALPHA42_CONTROLS_OK order=true entry-transitions=true search-field=true sidebar-preference=true tab-breakpoints=true document-menu=true"
    );
}
pub(super) async fn closing(
    ui: &Rc<Ui>,
    blank: &Rc<Ui>,
    window: &ApplicationWindow,
    app: &Application,
) {
    let _hold = app.hold();
    let host = ui.host.upgrade().unwrap();
    apply_user_edit(ui, Edit::Insert("close one".into()));
    apply_user_edit(blank, Edit::Insert("close two".into()));
    settle().await;
    close_workspace(&host);
    let dialog = window
        .visible_dialog()
        .unwrap()
        .downcast::<adw::AlertDialog>()
        .unwrap();
    dialog_presented_async(&dialog).await;
    let rows = checks(&dialog.extra_child().unwrap());
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|c| c.is_active()));
    for check in &rows {
        check.set_active(false);
    }
    assert!(!dialog.is_response_enabled("save"));
    close_workspace(&host);
    assert_eq!(window.visible_dialog().as_ref(), Some(dialog.upcast_ref()));
    dialog.close();
    dialog_closed_async(&dialog).await;
    assert!(!host.close_pending.get() && !ui.closed.get() && !blank.closed.get());
    let root = std::env::temp_dir().join(format!("leaf-close42-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    ui.save_path
        .replace(Some(root.join("missing").join("failure.docx")));
    close_workspace(&host);
    let dialog = window
        .visible_dialog()
        .unwrap()
        .downcast::<adw::AlertDialog>()
        .unwrap();
    dialog_presented_async(&dialog).await;
    assert!(response(dialog.upcast_ref(), wordcraft_gnome::t!("Save")));
    settle().await;
    assert!(!host.close_pending.get() && ui.dirty.get() && blank.dirty.get());
    let saved = root.join("saved.docx");
    ui.save_path.replace(Some(saved.clone()));
    close_workspace(&host);
    let dialog = window
        .visible_dialog()
        .unwrap()
        .downcast::<adw::AlertDialog>()
        .unwrap();
    dialog_presented_async(&dialog).await;
    let rows = checks(&dialog.extra_child().unwrap());
    rows[1].set_active(false);
    assert!(response(dialog.upcast_ref(), wordcraft_gnome::t!("Save")));
    settle().await;
    assert!(saved.is_file() && ui.closed.get() && blank.closed.get());
    std::fs::remove_dir_all(root).unwrap();
    eprintln!(
        "ALPHA42_CLOSE_OK combined=true cancel=true no-selection=true reentrant=true failure-keeps-work=true selected-save=true"
    );
}
