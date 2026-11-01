use super::*;

// Native GMenu/PopoverMenu rows: normal labels, right-hand accelerators,
// separators and disabled history actions, as in Files.
pub(super) fn selected_ui(host: &Workspace) -> Option<Rc<Ui>> {
    let page = host.tabs.selected_page()?;
    host.docs
        .borrow()
        .iter()
        .find(|(p, _)| *p == page)
        .map(|(_, u)| u.clone())
}
pub(super) fn refresh_main_actions(host: &Workspace) {
    let ui = selected_ui(host);
    let (undo, redo) = ui
        .as_ref()
        .and_then(|u| {
            u.current
                .borrow()
                .as_ref()
                .map(|d| (d.session.can_undo(), d.session.can_redo()))
        })
        .unwrap_or((false, false));
    for (name, on) in [
        ("undo", undo),
        ("redo", redo),
        ("tab-overview", host.tabs.n_pages() > 0),
        (
            "print",
            ui.as_ref().is_some_and(|u| {
                u.current.borrow().is_some() && u.printing.pending.borrow().is_none()
            }),
        ),
    ] {
        if let Some(a) = host
            .window
            .lookup_action(name)
            .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
        {
            a.set_enabled(on);
        }
    }
    if let Some(action) = host
        .window
        .lookup_action("editing-mode")
        .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
    {
        let reviewing = ui
            .as_ref()
            .and_then(|u| {
                u.current
                    .borrow()
                    .as_ref()
                    .map(|d| d.session.doc.settings.track_changes)
            })
            .unwrap_or(false);
        action.set_state(&(if reviewing { "reviewing" } else { "editing" }).to_variant());
    }
}
pub(super) fn window_action(
    host: &Rc<Workspace>,
    name: &str,
    callback: impl Fn(&Rc<Workspace>) + 'static,
) {
    let action = gio::SimpleAction::new(name, None);
    let weak = Rc::downgrade(host);
    action.connect_activate(move |_, _| {
        if let Some(h) = weak.upgrade() {
            callback(&h);
        }
    });
    host.window.add_action(&action);
}
pub(super) fn host_app(host: &Workspace) -> Option<Application> {
    host.window.application()?.downcast::<Application>().ok()
}
pub(super) fn setup_main_actions(host: &Rc<Workspace>, app: &Application) {
    let mode = gio::SimpleAction::new_stateful(
        "editing-mode",
        Some(glib::VariantTy::STRING),
        &"editing".to_variant(),
    );
    let weak = Rc::downgrade(host);
    mode.connect_activate(move |_, value| {
        let Some(host) = weak.upgrade() else { return };
        let Some(ui) = selected_ui(&host) else { return };
        let Some(value) = value
            .and_then(|v| v.str())
            .filter(|s| ["editing", "reviewing"].contains(s))
        else {
            return;
        };
        let result = {
            let mut current = ui.current.borrow_mut();
            let Some(d) = current.as_mut() else { return };
            if d.session.doc.settings.track_changes == (value == "reviewing") {
                return;
            }
            d.session
                .run(
                    "review.trackChanges",
                    &serde_json::json!({"value":value == "reviewing"}),
                )
                .map(|_| ())
        };
        match result {
            Ok(()) => {
                ui.dirty.set(true);
                sync_tab(&ui);
                let scroll = (
                    ui.scroll.hadjustment().value(),
                    ui.scroll.vadjustment().value(),
                );
                refresh_pages(&ui);
                let weak = Rc::downgrade(&ui);
                glib::idle_add_local_once(move || {
                    if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
                        ui.scroll.hadjustment().set_value(scroll.0);
                        ui.scroll.vadjustment().set_value(scroll.1);
                    }
                });
            }
            Err(e) => ui.toasts.add_toast(Toast::new(&e.to_string())),
        }
        refresh_main_actions(&host);
    });
    host.window.add_action(&mode);
    window_action(host, "new-tab", |h| {
        if let Some(app) = host_app(h) {
            blank_tab(&app, h);
        }
    });
    window_action(host, "new-window", |h| {
        if let Some(app) = host_app(h) {
            make_window(&app, false, false, false, false, false, false, false, None);
        }
    });
    window_action(host, "tab-overview", |h| h.overview.set_open(true));
    window_action(host, "open", |h| {
        if let Some(ui) = selected_ui(h) {
            ui.open_button.emit_clicked();
        }
    });
    window_action(host, "recover", |h| {
        if let (Some(ui), Some(app)) = (selected_ui(h), host_app(h)) {
            show_recovery(&ui, &h.window, &app, h);
        }
    });
    window_action(host, "undo", |h| {
        if let Some(ui) = selected_ui(h) {
            apply_user_edit(&ui, Edit::Undo);
            ui.document_box.grab_focus();
        }
    });
    window_action(host, "redo", |h| {
        if let Some(ui) = selected_ui(h) {
            apply_user_edit(&ui, Edit::Redo);
            ui.document_box.grab_focus();
        }
    });
    window_action(host, "save", |h| {
        if let Some(ui) = selected_ui(h) {
            save_document(&ui, &h.window, false, Box::new(|| {}));
        }
    });
    window_action(host, "save-as", |h| {
        if let Some(ui) = selected_ui(h) {
            save_document(&ui, &h.window, true, Box::new(|| {}));
        }
    });
    window_action(host, "insert-table", |h| {
        if let Some(ui) = selected_ui(h) {
            show_insert_table(&ui, &h.window);
        }
    });
    window_action(host, "insert-picture", |h| {
        if let Some(ui) = selected_ui(h) {
            choose_picture(&ui, &h.window, false);
        }
    });
    window_action(host, "insert-bookmark", |h| {
        if let Some(ui) = selected_ui(h) {
            show_annotation_dialog(&ui, AnnotationEdit::Bookmark);
        }
    });
    window_action(host, "insert-comment", |h| {
        if let Some(ui) = selected_ui(h) {
            show_annotation_dialog(&ui, AnnotationEdit::NewComment);
        }
    });
    window_action(host, "insert-header", |h| {
        if let Some(ui) = selected_ui(h) {
            enter_header(&ui, Region::Header);
        }
    });
    window_action(host, "insert-footer", |h| {
        if let Some(ui) = selected_ui(h) {
            enter_header(&ui, Region::Footer);
        }
    });
    window_action(host, "insert-page-number", |h| {
        if let Some(ui) = selected_ui(h) {
            show_page_number(&ui, &h.window);
        }
    });
    window_action(host, "check-spelling", |h| {
        if let Some(ui) = selected_ui(h) {
            show_spelling(&ui, &h.window);
        }
    });
    window_action(host, "text-language", |h| {
        if let Some(ui) = selected_ui(h) {
            show_text_language(&ui, &h.window);
        }
    });
    window_action(host, "columns", |h| {
        if let Some(ui) = selected_ui(h) {
            show_columns(&ui, &h.window);
        }
    });
    window_action(host, "document-properties", |h| {
        if let Some(ui) = selected_ui(h) {
            show_document_properties(&ui, &h.window);
        }
    });
    window_action(host, "print", |h| {
        if let Some(ui) = selected_ui(h) {
            print_document(&ui, &h.window);
        }
    });
    window_action(host, "export-pdf", |h| {
        if let Some(ui) = selected_ui(h) {
            export_pdf_dialog(&ui, &h.window);
        }
    });
    window_action(host, "preferences", |h| {
        show_preferences(&h.window);
    });
    window_action(host, "shortcuts", |h| show_keyboard_shortcuts(&h.window));
    window_action(host, "help", |h| show_editor_help(&h.window));
    window_action(host, "about", |h| {
        let w = adw::AboutWindow::new();
        w.set_transient_for(Some(&h.window));
        w.set_application_name("Leaf");
        w.set_application_icon(APP_ID);
        w.set_version(APP_VERSION);
        w.set_developer_name(wordcraft_gnome::t!("Leaf Project"));
        w.set_license_type(gtk::License::Gpl30);
        w.set_website("https://github.com/frandavid100/leaf");
        w.set_issue_url("https://github.com/frandavid100/leaf/issues");
        w.set_comments(wordcraft_gnome::t!(
            "Document editor for GNOME powered by WordCraft."
        ));
        w.add_link(
            wordcraft_gnome::t!("Help Translate Leaf"),
            "https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TRANSLATING.md",
        );
        w.present();
    });
    for (name, keys) in [
        ("new-tab", vec!["<Control>n", "<Control>t"]),
        ("new-window", vec!["<Control><Shift>n"]),
        ("tab-overview", vec!["<Control><Shift>o"]),
        ("open", vec!["<Control>o"]),
        ("preferences", vec!["<Control>comma"]),
        ("shortcuts", vec!["<Control>question"]),
        ("help", vec!["F1"]),
        ("print", vec!["<Control>p"]),
        ("check-spelling", vec!["F7"]),
    ] {
        app.set_accels_for_action(&format!("win.{name}"), &keys);
    }
    let weak = Rc::downgrade(host);
    host.tabs.connect_selected_page_notify(move |_| {
        if let Some(h) = weak.upgrade() {
            refresh_main_actions(&h);
        }
    });
}
pub(super) fn menu_item(menu: &gio::Menu, label: &str, action: &str, accel: Option<&str>) {
    let item = gio::MenuItem::new(Some(label), Some(&format!("win.{action}")));
    // Display history shortcuts without stealing Ctrl+Z from native text fields.
    if let Some(accel) = accel {
        item.set_attribute_value("accel", Some(&accel.to_variant()));
    }
    menu.append_item(&item);
}
pub(super) fn install_main_menu(button: &gtk::MenuButton) {
    let root = gio::Menu::new();
    let files = gio::Menu::new();
    menu_item(
        &files,
        wordcraft_gnome::t!("New Window"),
        "new-window",
        Some("<Control><Shift>n"),
    );
    menu_item(
        &files,
        wordcraft_gnome::t!("New Tab"),
        "new-tab",
        Some("<Control>n"),
    );
    menu_item(
        &files,
        wordcraft_gnome::t!("Show All Tabs"),
        "tab-overview",
        Some("<Control><Shift>o"),
    );
    menu_item(
        &files,
        wordcraft_gnome::t!("Open…"),
        "open",
        Some("<Control>o"),
    );
    menu_item(
        &files,
        wordcraft_gnome::t!("Recover Documents…"),
        "recover",
        None,
    );
    root.append_section(None, &files);
    let help = gio::Menu::new();
    menu_item(
        &help,
        wordcraft_gnome::t!("Preferences"),
        "preferences",
        Some("<Control>comma"),
    );
    menu_item(
        &help,
        wordcraft_gnome::t!("Keyboard Shortcuts"),
        "shortcuts",
        Some("<Control>question"),
    );
    menu_item(&help, wordcraft_gnome::t!("Help"), "help", Some("F1"));
    menu_item(&help, wordcraft_gnome::t!("About Leaf"), "about", None);
    root.append_section(None, &help);
    let theme = gio::MenuItem::new(None, None);
    theme.set_attribute_value("custom", Some(&"theme-selector".to_variant()));
    root.insert_item(0, &theme);
    button.set_menu_model(Some(&root));
    if let Some(pop) = button
        .popover()
        .and_then(|p| p.downcast::<gtk::PopoverMenu>().ok())
    {
        pop.add_child(&theme_selector(), "theme-selector");
    }
}
pub(super) fn install_document_menu(button: &gtk::MenuButton) {
    let root = gio::Menu::new();
    let history = gio::Menu::new();
    menu_item(
        &history,
        wordcraft_gnome::t!("Undo"),
        "undo",
        Some("<Control>z"),
    );
    menu_item(
        &history,
        wordcraft_gnome::t!("Redo"),
        "redo",
        Some("<Control><Shift>z"),
    );
    root.append_section(None, &history);
    let save = gio::Menu::new();
    menu_item(
        &save,
        wordcraft_gnome::t!("Save"),
        "save",
        Some("<Control>s"),
    );
    menu_item(
        &save,
        wordcraft_gnome::t!("Save As…"),
        "save-as",
        Some("<Control><Shift>s"),
    );
    root.append_section(None, &save);
    let add = gio::Menu::new();
    menu_item(&add, wordcraft_gnome::t!("Table…"), "insert-table", None);
    menu_item(&add, wordcraft_gnome::t!("Image…"), "insert-picture", None);
    menu_item(
        &add,
        wordcraft_gnome::t!("Bookmark…"),
        "insert-bookmark",
        Some("<Control><Shift>F5"),
    );
    menu_item(
        &add,
        wordcraft_gnome::t!("Comment…"),
        "insert-comment",
        Some("<Control><Alt>m"),
    );
    menu_item(&add, wordcraft_gnome::t!("Header"), "insert-header", None);
    menu_item(&add, wordcraft_gnome::t!("Footer"), "insert-footer", None);
    menu_item(
        &add,
        wordcraft_gnome::t!("Page Number…"),
        "insert-page-number",
        None,
    );
    root.append_submenu(Some(wordcraft_gnome::t!("Insert")), &add);
    let properties = gio::Menu::new();
    menu_item(
        &properties,
        wordcraft_gnome::t!("Document Properties…"),
        "document-properties",
        None,
    );
    menu_item(
        &properties,
        wordcraft_gnome::t!("Export to PDF…"),
        "export-pdf",
        None,
    );
    menu_item(
        &properties,
        wordcraft_gnome::t!("Print…"),
        "print",
        Some("<Control>p"),
    );
    root.append_section(None, &properties);
    let zoom = gio::MenuItem::new(None, None);
    zoom.set_attribute_value("custom", Some(&"zoom-controls".to_variant()));
    root.append_item(&zoom);
    let modes = gio::Menu::new();
    for (label, mode) in [
        (wordcraft_gnome::t!("Editing"), "editing"),
        (wordcraft_gnome::t!("Reviewing"), "reviewing"),
    ] {
        let item = gio::MenuItem::new(Some(label), None);
        item.set_action_and_target_value(Some("win.editing-mode"), Some(&mode.to_variant()));
        modes.append_item(&item);
    }
    root.append_section(None, &modes);
    button.set_menu_model(Some(&root));
}
pub(super) fn show_editor_help(parent: &ApplicationWindow) {
    let w = adw::Window::builder()
        .transient_for(parent)
        .title(wordcraft_gnome::t!("Leaf Help"))
        .default_width(520)
        .default_height(520)
        .build();
    let toolbar = ToolbarView::new();
    toolbar.add_top_bar(&HeaderBar::new());
    let text = wordcraft_gnome::t!(
        "DOCUMENTS\nNew Tab creates a blank document. The main menu opens DOCX/ODT documents, creates windows and shows all tabs. Save: Ctrl+S; Save As: Ctrl+Shift+S.\n\nNAVIGATION\nThe sidebar switches between Pages, Navigation, Comments, and Spelling and Grammar. Navigation groups bookmarks under their headings. Select a thumbnail, heading or bookmark to navigate. The magnifying glass opens search; Escape returns to the previous view.\n\nSEARCH AND REPLACE\nCtrl+F opens search. Its options menu offers case-sensitive search and replacement. The button beside the replacement field replaces all matches in one undo step. Each result also has its own replacement button.\n\nEDITING\nStyle, Paragraph and Text appear as icons on the right of the header, with Undo/Redo on the left. In narrow windows these controls move to the bottom toolbar. Right-click opens formatting pages inside the context menu. Insert in the document menu adds tables, images, bookmarks and comments. Bookmarks appear in Navigation; each comment thread shares one bubble in Comments. Spelling issues only select their text; corrections remain in the document context menu. Save comments as DOCX; ODT does not preserve them. Arrows open suboptions. Ctrl+Z undoes; Ctrl+Shift+Z redoes.\n\nAUTOSAVE\nPreferences can disable autosave, use a recovery file or save directly to the document. Recover Documents is in the main menu."
    );
    let l = gtk::Label::new(Some(text));
    l.set_wrap(true);
    l.set_selectable(true);
    l.set_xalign(0.0);
    l.set_margin_start(24);
    l.set_margin_end(24);
    l.set_margin_top(18);
    l.set_margin_bottom(24);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_child(Some(&l));
    toolbar.set_content(Some(&scroll));
    w.set_content(Some(&toolbar));
    w.present();
}
pub(super) fn show_keyboard_shortcuts(parent: &ApplicationWindow) {
    let groups = [
        (
            wordcraft_gnome::t!("Documents"),
            vec![
                (wordcraft_gnome::t!("New Tab"), "<Control>n"),
                (wordcraft_gnome::t!("New Window"), "<Control><Shift>n"),
                (wordcraft_gnome::t!("Show All Tabs"), "<Control><Shift>o"),
                (wordcraft_gnome::t!("Open"), "<Control>o"),
                (wordcraft_gnome::t!("Save"), "<Control>s"),
                (wordcraft_gnome::t!("Save As"), "<Control><Shift>s"),
                (wordcraft_gnome::t!("Close Tab"), "<Control>w"),
                (wordcraft_gnome::t!("Print"), "<Control>p"),
            ],
        ),
        (
            wordcraft_gnome::t!("Editing and Search"),
            vec![
                (wordcraft_gnome::t!("Undo"), "<Control>z"),
                (wordcraft_gnome::t!("Redo"), "<Control><Shift>z"),
                (wordcraft_gnome::t!("Search"), "<Control>f"),
                (wordcraft_gnome::t!("Replace"), "<Control>h"),
                (wordcraft_gnome::t!("Bold"), "<Control>b"),
                (wordcraft_gnome::t!("Italic"), "<Control>i"),
                (wordcraft_gnome::t!("Underline"), "<Control>u"),
            ],
        ),
        (
            wordcraft_gnome::t!("Interface"),
            vec![
                (wordcraft_gnome::t!("Show or Hide Sidebar"), "F9"),
                (wordcraft_gnome::t!("Preferences"), "<Control>comma"),
                (wordcraft_gnome::t!("Switch Pane"), "F6"),
                (wordcraft_gnome::t!("Main Menu"), "F10"),
                (wordcraft_gnome::t!("Help"), "F1"),
            ],
        ),
    ];
    let mut xml = String::from(
        "<interface><object class=\"GtkShortcutsWindow\" id=\"shortcuts\"><child><object class=\"GtkShortcutsSection\"><property name=\"section-name\">editor</property><property name=\"title\">Leaf</property>",
    );
    for (title, shortcuts) in groups {
        xml.push_str(&format!(
            "<child><object class=\"GtkShortcutsGroup\"><property name=\"title\">{}</property>",
            glib::markup_escape_text(title)
        ));
        for (title, key) in shortcuts {
            xml.push_str(&format!("<child><object class=\"GtkShortcutsShortcut\"><property name=\"title\">{}</property><property name=\"accelerator\">{}</property></object></child>",glib::markup_escape_text(title),glib::markup_escape_text(key)));
        }
        xml.push_str("</object></child>");
    }
    xml.push_str("</object></child></object></interface>");
    let builder = gtk::Builder::from_string(&xml);
    let w = builder.object::<gtk::ShortcutsWindow>("shortcuts").unwrap();
    w.set_transient_for(Some(parent));
    w.present();
}
