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
        format_button(ui, "format-quick-alignment-options")
            .unwrap()
            .popup();
        settle().await;
        if close_first {
            paragraph.popover.upgrade().unwrap().popdown();
            settle().await;
        }
        format_bar::open_group(ui, "paragraph", None);
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
        assert!(
            paragraph.header.arrow.is_visible(),
            "More Options returns to shared menu"
        );
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
    blank_tab(
        &window
            .application()
            .unwrap()
            .downcast::<Application>()
            .unwrap(),
        &host,
    );
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
    shared_overflow(ui, window).await;
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
    eprintln!("ALPHA42_CLOSE_STAGE request");
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
    eprintln!("ALPHA42_CLOSE_STAGE cancel-complete");
    assert!(!host.close_pending.get() && !ui.closed.get() && !blank.closed.get());
    let root = std::env::temp_dir().join(format!("leaf-close42-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    ui.save_path
        .replace(Some(root.join("missing").join("failure.docx")));
    eprintln!("ALPHA42_CLOSE_STAGE request");
    close_workspace(&host);
    let dialog = window
        .visible_dialog()
        .unwrap()
        .downcast::<adw::AlertDialog>()
        .unwrap();
    dialog_presented_async(&dialog).await;
    assert!(response(dialog.upcast_ref(), wordcraft_gnome::t!("Save")));
    eprintln!("ALPHA42_CLOSE_STAGE save-response");
    settle().await;
    assert!(!host.close_pending.get() && ui.dirty.get() && blank.dirty.get());
    let saved = root.join("saved.docx");
    ui.save_path.replace(Some(saved.clone()));
    eprintln!("ALPHA42_CLOSE_STAGE request");
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
    eprintln!("ALPHA42_CLOSE_STAGE save-response");
    settle().await;
    assert!(saved.is_file() && ui.closed.get() && blank.closed.get());
    assert!(
        !window.is_visible(),
        "the tested workspace window actually closed"
    );
    std::fs::remove_dir_all(root).unwrap();
    eprintln!(
        "ALPHA42_CLOSE_OK combined=true cancel=true no-selection=true reentrant=true failure-keeps-work=true selected-save=true"
    );
}

async fn shared_overflow(ui: &Rc<Ui>, window: &ApplicationWindow) {
    use wordcraft_gnome::{preferences as prefs, toolbar::Placement};
    let saved: Vec<_> = ["text", "paragraph", "elements"]
        .into_iter()
        .map(|group| (group, prefs::toolbar_placements(group)))
        .collect();
    for (group, order) in &saved {
        let order: Vec<_> = order
            .iter()
            .map(|(id, _)| {
                (
                    id.clone(),
                    if *group == "text"
                        && [
                            "bold",
                            "italic",
                            "underline",
                            "strike",
                            "superscript",
                            "font",
                            "ink",
                            "highlight",
                            "language",
                        ]
                        .contains(&id.as_str())
                    {
                        Placement::Pinned
                    } else if *group == "text" {
                        Placement::More
                    } else {
                        Placement::Adaptive
                    },
                )
            })
            .collect();
        prefs::set_toolbar_placements(group, &order).unwrap();
    }
    ui.quick_tools.borrow().as_ref().unwrap().layout();
    window.set_default_size(420, 850);
    settle().await;
    let menu = format_button(ui, "format-overflow-menu").unwrap();
    let scroll = find_widget(&ui.format_toolbar, "format-pinned-scroll")
        .unwrap()
        .downcast::<gtk::ScrolledWindow>()
        .unwrap();
    let fixed = menu.compute_bounds(window).unwrap();
    eprintln!(
        "OVERFLOW248_LAYOUT window={} sidebar={:?} toolbar={:?} scroll={:?} header={:?} fixed={fixed:?}",
        window.width(),
        ui.sidebar.measure(gtk::Orientation::Horizontal, -1),
        ui.format_toolbar.measure(gtk::Orientation::Horizontal, -1),
        scroll.measure(gtk::Orientation::Horizontal, -1),
        find_widget(window, "document-primary-header")
            .unwrap()
            .measure(gtk::Orientation::Horizontal, -1)
    );
    assert!(fixed.x() >= 0. && fixed.x() + fixed.width() <= window.width() as f32);
    assert!(scroll.hadjustment().upper() > scroll.hadjustment().page_size());
    let last = find_widget(&ui.format_toolbar, "format-quick-language-options").unwrap();
    assert!(last.is_mapped());
    last.grab_focus();
    settle().await;
    let target = last.compute_bounds(&scroll).unwrap();
    assert!(
        target.x() >= -1. && target.x() + target.width() <= scroll.width() as f32 + 1.,
        "keyboard focus reveals pinned tool"
    );
    let after = menu.compute_bounds(window).unwrap();
    assert!((after.x() - fixed.x()).abs() < 1. && menu.is_mapped());
    ui.quick_tools.borrow().as_ref().unwrap().root();
    settle().await;
    for group in ["text", "paragraph", "elements"] {
        assert!(
            find_widget(&ui.format_toolbar, &format!("overflow-section-{group}"))
                .unwrap()
                .is_mapped()
        );
        assert!(
            find_widget(&ui.format_toolbar, &format!("overflow-more-{group}"))
                .unwrap()
                .is_mapped()
        );
    }
    assert!(find_widget(&ui.format_toolbar, "overflow-tool-text-language").is_none());
    assert!(find_widget(&ui.format_toolbar, "overflow-tool-elements-image").is_some());
    let root = find_widget(&ui.format_toolbar, "overflow-section-text")
        .unwrap()
        .parent()
        .unwrap();
    let mut names = Vec::new();
    let mut separators = 0;
    let mut child = root.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        if widget.is::<gtk::Separator>() {
            separators += 1;
            assert!(
                names
                    .last()
                    .is_some_and(|n: &String| n.starts_with("overflow-more-"))
            );
        }
        names.push(widget.widget_name().to_string());
    }
    assert_eq!(separators, 3);
    assert_eq!(names.last().unwrap(), "overflow-customize-toolbar");
    menu.popdown();
    window.set_default_size(1280, 850);
    settle().await;
    assert!(
        find_widget(&ui.format_toolbar, "format-quick-alignment-options")
            .unwrap()
            .is_mapped()
    );
    let mut modes = prefs::toolbar_placements("elements");
    modes.iter_mut().find(|(id, _)| id == "table").unwrap().1 = Placement::More;
    prefs::set_toolbar_placements("elements", &modes).unwrap();
    ui.quick_tools.borrow().as_ref().unwrap().layout();
    ui.quick_tools.borrow().as_ref().unwrap().root();
    settle().await;
    for group in ["text", "paragraph", "elements"] {
        assert!(
            find_widget(&ui.format_toolbar, &format!("overflow-more-{group}"))
                .unwrap()
                .is_mapped()
        );
    }
    find_widget(&ui.format_toolbar, "overflow-more-elements")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    settle().await;
    let elements = ui.elements.borrow().as_ref().unwrap().clone();
    find_widget(&elements.pop, "format-row-elements-table")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    settle().await;
    assert!(elements.header.arrow.is_visible());
    elements.header.button.emit_clicked();
    settle().await;
    assert_eq!(elements.stack.visible_child_name().as_deref(), Some("home"));
    elements.header.button.emit_clicked();
    settle().await;
    assert!(
        find_widget(&ui.format_toolbar, "overflow-section-text")
            .unwrap()
            .is_mapped()
    );
    menu.popdown();
    for width in [420, 1280] {
        window.set_default_size(width, 850);
        settle().await;
        ui.quick_tools.borrow().as_ref().unwrap().root();
        settle().await;
        let customize = find_widget(&ui.format_toolbar, "overflow-customize-toolbar")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        assert!(customize.is_mapped());
        customize.emit_clicked();
        settle().await;
        let dialog = window
            .visible_dialog()
            .unwrap()
            .downcast::<adw::PreferencesDialog>()
            .unwrap();
        assert_eq!(dialog.visible_page_name().as_deref(), Some("toolbar"));
        for name in [
            "toolbar-placement-text-bold",
            "toolbar-move-text-bold-1",
            "toolbar-preferences-paragraph",
            "toolbar-preferences-elements",
        ] {
            assert!(find_widget(&dialog, name).is_some());
        }
        dialog.close();
        settle().await;
    }
    for (group, order) in &saved {
        prefs::set_toolbar_placements(group, order).unwrap();
    }
    ui.quick_tools.borrow().as_ref().unwrap().layout();
    settle().await;
    eprintln!(
        "OVERFLOW248_OK fixed-menu=true pinned-scroll=true focus-reveal=true sections=true customize-wide-narrow=true restore=true"
    );
}
