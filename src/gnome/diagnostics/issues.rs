use super::*;
use serde_json::json;

fn click(root: &impl IsA<gtk::Widget>, name: &str) {
    find_widget(root, name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
}

/// Exercise the real controls against synthetic documents, both native and installed.
pub(super) fn issue_sweep_ui_test(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    host: &Rc<Workspace>,
    app: &Application,
    document_menu: &gtk::MenuButton,
    main_menu: &gtk::MenuButton,
    paragraph_menu: &gtk::MenuButton,
    text_menu: &gtk::MenuButton,
) {
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "Alpha beta\nSecond paragraph",
        )),
        "Issues",
    );
    let props = show_document_properties(ui, window);
    let sizes = find_widget(&props.window, "paper-size")
        .unwrap()
        .downcast::<adw::ComboRow>()
        .unwrap();
    let orientation = find_widget(&props.window, "paper-orientation")
        .unwrap()
        .downcast::<adw::ComboRow>()
        .unwrap();
    for preset in 1..=5 {
        sizes.set_selected(preset);
        assert!(!props.width.is_sensitive() && !props.height.is_sensitive());
    }
    let dimensions = (props.width.value(), props.height.value());
    orientation.set_selected(1);
    assert!((props.width.value() - dimensions.1).abs() < 0.01);
    assert!(!props.width.is_sensitive());
    sizes.set_selected(0);
    assert!(props.width.is_sensitive() && props.height.is_sensitive());
    props.window.close();
    eprintln!("ISSUE_PAPER_PRESETS_OK");

    {
        let mut current = ui.current.borrow_mut();
        let s = &mut current.as_mut().unwrap().session;
        selection::select_range(s, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
    }
    refresh_selection(ui);
    let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let context = show_document_context(ui, window, 140, 160, DocumentContext::Text);
    context_activate_row(&context, wordcraft_gnome::t!("Link"));
    assert!(
        ui.link_dialog.borrow().is_some(),
        "context row opens link editor"
    );
    let link_window = ui.link_dialog.borrow().as_ref().unwrap().clone();
    link_window.close();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, original);
    let link = show_link(ui).unwrap();
    assert!(link.destination.text().is_empty());
    link.destination.set_text("https://example.org/leaf");
    link.apply.emit_clicked();
    let edited = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let link = show_link(ui).unwrap();
    assert_eq!(link.destination.text(), "https://example.org/leaf");
    link.destination.set_text("https://example.org/cancelled");
    link.window.close();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, edited);
    let stale = show_link(ui).unwrap();
    {
        let mut current = ui.current.borrow_mut();
        selection::set_caret(
            &mut current.as_mut().unwrap().session,
            &Pos::body(1, 0),
            false,
        )
        .unwrap();
    }
    stale.destination.set_text("https://example.org/stale");
    stale.apply.emit_clicked();
    assert!(stale.window.is_visible());
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, edited);
    stale.window.close();
    eprintln!("ISSUE_LINK_DIALOG_OK");

    let selection = ui.current.borrow().as_ref().unwrap().session.sel.clone();
    gio::prelude::ActionGroupExt::activate_action(
        window,
        "editing-mode",
        Some(&"reviewing".to_variant()),
    );
    assert!(
        ui.current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .doc
            .settings
            .track_changes
    );
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel, selection);
    apply_user_edit(ui, Edit::Insert("Tracked ".into()));
    let changes = ui
        .current
        .borrow_mut()
        .as_mut()
        .unwrap()
        .session
        .run("review.changes", &json!({}))
        .unwrap();
    gio::prelude::ActionGroupExt::activate_action(
        window,
        "editing-mode",
        Some(&"editing".to_variant()),
    );
    assert!(
        !ui.current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .doc
            .settings
            .track_changes
    );
    assert_eq!(
        ui.current
            .borrow_mut()
            .as_mut()
            .unwrap()
            .session
            .run("review.changes", &json!({}))
            .unwrap(),
        changes
    );
    assert!(
        changes.to_string().contains("Tracked"),
        "reviewing edits are tracked"
    );
    let first_page = host.tabs.selected_page().unwrap();
    blank_tab(app, host);
    let second_ui = selected_ui(host).unwrap();
    assert!(
        !second_ui
            .current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .doc
            .settings
            .track_changes
    );
    host.tabs.set_selected_page(&first_page);
    refresh_main_actions(host);
    eprintln!("ISSUE_REVIEWING_MODE_OK");

    let pop = document_menu.popover().unwrap();
    assert!(find_widget(&pop, "zoom-controls").is_some());
    assert!(find_widget(&ui.scroll, "zoom-controls").is_none());
    click(&pop, "zoom-reset");
    assert_eq!(ui.current.borrow().as_ref().unwrap().zoom, 1.0);
    assert!(!ui.current.borrow().as_ref().unwrap().fit_to_width);
    click(&pop, "zoom-in");
    assert!(ui.current.borrow().as_ref().unwrap().zoom > 1.0);
    click(&pop, "zoom-reset");
    click(&pop, "zoom-fit-width");
    assert!(ui.current.borrow().as_ref().unwrap().fit_to_width);
    eprintln!("ISSUE_ZOOM_MENU_OK percentage={}", ui.zoom_label.text());

    let theme = main_menu.popover().unwrap();
    for (mode, scheme) in [
        (1, adw::ColorScheme::ForceLight),
        (0, adw::ColorScheme::Default),
        (2, adw::ColorScheme::ForceDark),
    ] {
        find_widget(&theme, &format!("theme-{mode}"))
            .unwrap()
            .downcast::<gtk::ToggleButton>()
            .unwrap()
            .set_active(true);
        assert_eq!(wordcraft_gnome::preferences::theme(), mode);
        assert_eq!(adw::StyleManager::default().color_scheme(), scheme);
    }
    eprintln!("ISSUE_THEME_MENU_OK");

    // An existing author's messages and document metadata must survive identity changes.
    {
        let mut current = ui.current.borrow_mut();
        let s = &mut current.as_mut().unwrap().session;
        s.run("file.setAuthor", &json!({"name":"Previous author"}))
            .unwrap();
        wordcraft_gnome::annotations::add_comment(s, "Existing message").unwrap();
    }
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    show_preferences(window);
    let prefs = gtk::Window::list_toplevels()
        .into_iter()
        .find_map(|w| w.downcast::<adw::PreferencesWindow>().ok())
        .unwrap();
    let name = find_widget(&prefs, "preferences-user-name")
        .unwrap()
        .downcast::<adw::EntryRow>()
        .unwrap();
    assert_ne!(
        wordcraft_gnome::preferences::system_user_name(),
        "WordCraft User"
    );
    name.set_text("Leaf QA");
    assert_eq!(wordcraft_gnome::preferences::user_name(), "Leaf QA");
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.author,
        "Leaf QA"
    );
    assert_eq!(
        second_ui.current.borrow().as_ref().unwrap().session.author,
        "Leaf QA"
    );
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
    name.set_text("");
    assert_eq!(wordcraft_gnome::preferences::user_name(), "Leaf QA");
    prefs.close();
    {
        let mut current = ui.current.borrow_mut();
        wordcraft_gnome::annotations::add_comment(
            &mut current.as_mut().unwrap().session,
            "New message",
        )
        .unwrap();
    }
    assert!(
        ui.current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .doc
            .comments
            .values()
            .any(|c| c.author == "Leaf QA")
    );
    eprintln!(
        "ISSUE_USER_IDENTITY_OK system={}",
        wordcraft_gnome::preferences::system_user_name()
    );

    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text("First\nSecond")),
        "Format",
    );
    {
        let mut current = ui.current.borrow_mut();
        let s = &mut current.as_mut().unwrap().session;
        s.run("para.spacing", &json!({"before":5})).unwrap();
        selection::set_caret(s, &Pos::body(1, 0), false).unwrap();
        s.run("para.spacing", &json!({"before":17})).unwrap();
        selection::set_caret(s, &Pos::body(0, 0), false).unwrap();
    }
    paragraph_menu.popup();
    let paragraph = ui.paragraph.borrow().as_ref().unwrap().clone();
    paragraph.header.pin.set_active(true);
    paragraph.page("spacing");
    ui.document_box.grab_focus();
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    {
        let mut current = ui.current.borrow_mut();
        selection::set_caret(
            &mut current.as_mut().unwrap().session,
            &Pos::body(1, 0),
            false,
        )
        .unwrap();
    }
    refresh_selection(ui);
    assert_eq!(
        paragraph.stack.visible_child_name().as_deref(),
        Some("detail")
    );
    let spacing = find_widget(&paragraph.detail, "paragraph-spaceBefore")
        .unwrap()
        .downcast::<gtk::SpinButton>()
        .unwrap();
    assert_eq!(spacing.value(), 17.0);
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.doc,
        before,
        "sync never formats text"
    );
    let depth = ui.current.borrow().as_ref().unwrap().session.undo_depth();
    spacing.set_value(20.0);
    spacing.set_value(21.0);
    assert_eq!(selection_paragraph(ui)["spaceBefore"], 21.0);
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.undo_depth(),
        depth + 1
    );
    apply_user_edit(ui, Edit::Undo);
    assert_eq!(selection_paragraph(ui)["spaceBefore"], 17.0);
    paragraph.header.pin.set_active(false);
    paragraph.popover.upgrade().unwrap().popdown();
    eprintln!("ISSUE_PINNED_PARAGRAPH_IMMEDIATE_OK");
    {
        let mut current = ui.current.borrow_mut();
        let s = &mut current.as_mut().unwrap().session;
        selection::set_caret(s, &Pos::body(0, 0), false).unwrap();
        s.run("para.heading1", &json!({})).unwrap();
        selection::set_caret(s, &Pos::body(1, 0), false).unwrap();
    }
    let style = ui.style.borrow().as_ref().unwrap().clone();
    style.popover.upgrade().unwrap().popup();
    style.header.pin.set_active(true);
    ui.document_box.grab_focus();
    {
        let mut current = ui.current.borrow_mut();
        selection::set_caret(
            &mut current.as_mut().unwrap().session,
            &Pos::body(0, 0),
            false,
        )
        .unwrap();
    }
    let depth = ui.current.borrow().as_ref().unwrap().session.undo_depth();
    refresh_selection(ui);
    assert!(
        find_widget(&style.list, "style-apply-Heading1")
            .unwrap()
            .parent()
            .unwrap()
            .has_css_class("current-style")
    );
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.undo_depth(),
        depth
    );
    style.header.pin.set_active(false);
    style.popover.upgrade().unwrap().popdown();
    eprintln!("ISSUE_PINNED_STYLE_OK");

    {
        let mut current = ui.current.borrow_mut();
        selection::select_all(&mut current.as_mut().unwrap().session).unwrap();
    }
    refresh_selection(ui);
    text_menu.popup();
    let text_popover = text_menu.popover().unwrap();
    assert!(find_widget(&text_popover, "text-apply-size").is_none());
    assert_eq!(
        ui.font_input.parent().unwrap().parent(),
        ui.size_input.parent().unwrap().parent()
    );
    ui.font_input.set_text("serif");
    ui.font_input.emit_activate();
    assert_eq!(selection_character(ui)["font"], "serif");
    ui.size_input.set_value(13.25);
    assert_eq!(selection_character(ui)["size"], 13.25);
    let choices = find_widget(&text_popover, "text-size-choices")
        .unwrap()
        .downcast::<gtk::MenuButton>()
        .unwrap();
    choices.popup();
    assert!(find_widget(&choices.popover().unwrap(), "size-choice-13.25").is_some());
    choices.popdown();
    let fonts = find_widget(&text_popover, "text-font-choices")
        .unwrap()
        .downcast::<gtk::MenuButton>()
        .unwrap();
    fonts.popup();
    let preview = find_widget(&fonts.popover().unwrap(), "font-preview")
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap();
    assert!(preview.attributes().unwrap().to_string().contains("13.25"));
    fonts.popdown();

    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    ui.size_input.set_text("13.");
    ui.size_input.emit_activate();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
    let u = ui.clone();
    let a = app.clone();
    ui.size_input.set_text("13.75");
    glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
        assert_eq!(
            selection_character(&u)["size"],
            13.75,
            "complete typed value applies without Enter"
        );
        assert_eq!(u.size_input.text(), "13.75");
        eprintln!("ISSUE_INLINE_FONT_SIZE_OK");
        eprintln!("ISSUE_SWEEP_UI_OK");
        a.quit();
    });
}

/// The same test is run on a normal path and a real Flatpak document-portal grant.
pub(super) fn save_current_file_ui_test(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    app: &Application,
) {
    let path = ui.source_path.borrow().clone().expect("opened source file");
    assert!(ui.save_path.borrow().is_none());
    apply_user_edit(ui, Edit::Insert("Leaf CtrlS saved ".into()));
    gio::prelude::ActionGroupExt::activate_action(window, "save", None);
    let reopened = open_document(&path).unwrap();
    assert!(
        reopened
            .doc
            .plain_text(wordcraft_doc::StoryRef::Body)
            .contains("Leaf CtrlS saved")
    );
    assert!(!ui.dirty.get());
    assert_eq!(ui.save_path.borrow().as_ref(), Some(&path));
    eprintln!("SAVE_CURRENT_FILE_UI_OK");
    app.quit();
}

fn anchor_coordinates(ui: &Rc<Ui>, window: &ApplicationWindow) -> (i32, i32, i32) {
    let mut current = ui.current.borrow_mut();
    let d = current.as_mut().unwrap();
    let p = d.session.doc.para_at(&Pos::body(0, 0)).unwrap();
    let (start, end) = (
        p.text.find("beta").unwrap() + 1,
        p.text.find("gamma").unwrap() + 3,
    );
    let c = d
        .session
        .layout()
        .caret_on(&Pos::body(0, start), 0)
        .unwrap();
    let e = d.session.layout().caret_on(&Pos::body(0, end), 0).unwrap();
    let r = d.page_frames[0].compute_bounds(window).unwrap();
    (
        (r.x() + c.x * d.zoom) as i32,
        (r.y() + (c.top + c.height / 2.0) * d.zoom) as i32,
        (r.x() + e.x * d.zoom) as i32,
    )
}

pub(super) fn comment_pointer_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "Alpha beta gamma outside",
        )),
        "Comment anchors",
    );
    {
        let mut current = ui.current.borrow_mut();
        let s = &mut current.as_mut().unwrap().session;
        selection::select_range(s, &Pos::body(0, 0), &Pos::body(0, 10)).unwrap();
        wordcraft_gnome::annotations::add_comment(s, "First thread").unwrap();
        let p = s.doc.para_at(&Pos::body(0, 0)).unwrap();
        let (start, end) = (
            p.text.find("beta").unwrap(),
            p.text.find("outside").unwrap() - 1,
        );
        selection::select_range(s, &Pos::body(0, start), &Pos::body(0, end)).unwrap();
        wordcraft_gnome::annotations::add_comment(s, "Overlapping thread").unwrap();
        selection::set_caret(s, &Pos::body(0, 0), false).unwrap();
    }
    after_content_change(ui);
    ui.current.borrow_mut().as_mut().unwrap().session.dirty = false;
    ui.dirty.set(false);
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        let document = u.current.borrow().as_ref().unwrap().session.doc.clone();
        let rev = u.current.borrow().as_ref().unwrap().session.rev();
        let (x, y, _) = anchor_coordinates(&u, &w);
        eprintln!("COMMENT_ANCHOR_READY {x} {y}");
        glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
            let panel = u.annotations.borrow().as_ref().unwrap().clone();
            assert_eq!(panel.active.borrow().len(), 2, "overlaps reveal both roots");
            assert_eq!(
                u.navigation_pages.visible_child_name().as_deref(),
                Some("annotations")
            );
            assert_eq!(u.current.borrow().as_ref().unwrap().session.rev(), rev);
            assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, document);
            assert!(!u.dirty.get());
            let (x, y, end) = anchor_coordinates(&u, &w);
            eprintln!("COMMENT_DRAG_READY {x} {y} {end}");
            glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
                assert!(u.pointer_moved.get());
                assert!(
                    !u.current
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .session
                        .sel
                        .is_collapsed()
                );
                assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, document);
                assert_eq!(u.current.borrow().as_ref().unwrap().session.rev(), rev);
                opened(
                    &u,
                    Session::new(wordcraft_doc::Document::new()),
                    "Replacement",
                );
                panel.show();
                assert!(panel.active.borrow().is_empty());
                eprintln!("COMMENT_POINTER_UI_OK overlaps=2 drag=true stale=false");
                a.quit();
            });
        });
    });
}
