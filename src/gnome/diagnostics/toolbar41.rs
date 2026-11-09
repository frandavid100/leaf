//! Acceptance checks for real controls in their direct and overflow locations.
use super::*;
async fn settle() {
    glib::timeout_future(std::time::Duration::from_millis(400)).await;
}
fn click(root: &impl IsA<gtk::Widget>, name: &str) {
    find_widget(root, name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
}
fn state(ui: &Ui) -> (wordcraft_doc::Document, usize) {
    let current = ui.current.borrow();
    let s = &current.as_ref().unwrap().session;
    (s.doc.clone(), s.undo_depth())
}
pub(super) fn test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_default_size(1280, 850);
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("Alpha Beta")),
            "Toolbar acceptance",
        );
        ui.current
            .borrow_mut()
            .as_mut()
            .unwrap()
            .session
            .run("select.all", &serde_json::json!({"whole":true}))
            .unwrap();
        refresh_selection(&ui);
        settle().await;
        let paragraph = ui.paragraph.borrow().as_ref().unwrap().clone();
        let text = ui.text_tools.borrow().as_ref().unwrap().clone();
        let elements = ui.elements.borrow().as_ref().unwrap().clone();
        let before = state(&ui);
        click(&ui.format_toolbar, "format-quick-alignment");
        settle().await;
        assert_eq!(state(&ui), before);
        assert!(!paragraph.header.arrow.is_visible());
        assert!(paragraph.popover.upgrade().unwrap().pointing_to().0);
        assert!(
            !find_widget(&paragraph.main, "format-row-paragraph-alignment")
                .unwrap()
                .get_visible()
        );
        paragraph.popover.upgrade().unwrap().popdown();
        click(&ui.format_toolbar, "format-quick-list");
        settle().await;
        assert!(!paragraph.header.arrow.is_visible());
        paragraph.page("bullets");
        assert!(paragraph.header.arrow.is_visible());
        paragraph.popover.upgrade().unwrap().popdown();
        click(&ui.format_toolbar, "format-quick-underline-options");
        settle().await;
        assert_eq!(state(&ui), before);
        assert!(!text.header.arrow.is_visible());
        text.popover.upgrade().unwrap().popdown();
        // Opening every eligible picker must be read-only and start at its own root.
        for name in ["image", "table"] {
            click(&ui.format_toolbar, &format!("format-quick-{name}"));
            settle().await;
            assert!(!elements.header.arrow.is_visible());
            assert_eq!(state(&ui), before);
            let insert_name = if name == "image" {
                "picture-insert"
            } else {
                "table-insert"
            };
            let origin = if name == "image" {
                ui.picture.borrow().as_ref().unwrap().pop.clone()
            } else {
                ui.table_popover.borrow().as_ref().unwrap().clone()
            };
            eprintln!(
                "TOOLBAR41_ELEMENT {name} title={} visible={} borrowed={} source_child={}",
                elements.header.label.text(),
                elements.pop.is_visible(),
                elements.body_for(&origin).is_some(),
                origin.child().is_some()
            );
            let insert = find_widget(&elements.pop, insert_name)
                .unwrap_or_else(|| panic!("missing insertion footer for {name}"));
            assert!(insert.is_mapped());
            // Only ancestors inside this popover count, not the toolbar scroller.
            let mut ancestor = insert.parent();
            while let Some(widget) = ancestor {
                if widget == elements.pop.clone().upcast::<gtk::Widget>() {
                    break;
                }
                assert!(
                    !widget.is::<gtk::ScrolledWindow>(),
                    "insertion must be outside the popup's scrolling options"
                );
                ancestor = widget.parent();
            }
            elements.pop.popdown();
            settle().await;
        }
        // Style names cannot move the adjacent controls, including a long custom name.
        let style_group = find_widget(&ui.format_toolbar, "format-style-group").unwrap();
        let normal_width = style_group.width();
        let next = find_widget(&ui.format_toolbar, "format-paragraph-group").unwrap();
        let normal_x = next.compute_bounds(&ui.format_toolbar).unwrap().x();
        for long in [false, true] {
            {
                let mut c = ui.current.borrow_mut();
                let s = &mut c.as_mut().unwrap().session;
                wordcraft_gnome::styles::edit(s,if long {wordcraft_gnome::styles::Edit::Create("An unusually long custom paragraph style name for the width regression".into())} else {wordcraft_gnome::styles::Edit::Apply("Heading1".into())}).unwrap();
            }
            after_content_change(&ui);
            settle().await;
            assert_eq!(style_group.width(), normal_width);
            assert!((next.compute_bounds(&ui.format_toolbar).unwrap().x() - normal_x).abs() < 1.0);
            apply_user_edit(&ui, Edit::Undo);
            settle().await;
        }
        assert_eq!(state(&ui), before);
        // All three native preference groups support keyboard reordering and switches.
        let dialog = show_preferences(&window);
        settle().await;
        for (group, id) in [
            ("paragraph", "list"),
            ("text", "italic"),
            ("elements", "table"),
        ] {
            let name = format!("toolbar-move-{group}-{id}--1");
            click(&dialog, &name);
            assert_eq!(wordcraft_gnome::preferences::toolbar(group)[0].0, id);
        }
        let switch = find_widget(&dialog, "toolbar-visible-elements-image")
            .unwrap()
            .downcast::<gtk::Switch>()
            .unwrap();
        switch.set_active(false);
        assert!(
            !find_widget(&ui.format_toolbar, "format-quick-image")
                .unwrap()
                .is_visible()
        );
        assert!(
            find_widget(&elements.pop, "format-row-elements-image")
                .unwrap()
                .get_visible()
        );
        switch.set_active(true);
        assert!(
            find_widget(&ui.format_toolbar, "format-quick-image")
                .unwrap()
                .is_visible()
        );
        let table = find_widget(&ui.format_toolbar, "format-quick-table").unwrap();
        let image = find_widget(&ui.format_toolbar, "format-quick-image").unwrap();
        settle().await;
        assert!(
            table.compute_bounds(&ui.format_toolbar).unwrap().x()
                < image.compute_bounds(&ui.format_toolbar).unwrap().x()
        );
        let ink_switch = find_widget(&dialog, "toolbar-visible-text-ink")
            .unwrap()
            .downcast::<gtk::Switch>()
            .unwrap();
        ink_switch.set_active(true);
        dialog.close();
        settle().await;
        let color_before = state(&ui);
        click(&ui.format_toolbar, "format-quick-ink");
        settle().await;
        assert!(!text.header.arrow.is_visible());
        assert_eq!(
            state(&ui),
            color_before,
            "opening a color picker cannot create an undo step"
        );
        assert!(find_widget(&ui.format_toolbar, "format-quick-ink-swatch").is_some());
        text.popover.upgrade().unwrap().popdown();
        settle().await;
        let saved = wordcraft_gnome::preferences::toolbar("elements");
        window.set_default_size(420, 850);
        settle().await;
        assert!(!table.is_mapped());
        assert!(
            find_widget(&elements.pop, "format-row-elements-table")
                .unwrap()
                .get_visible()
        );
        assert_eq!(wordcraft_gnome::preferences::toolbar("elements"), saved);
        format_button(&ui, "paragraph-menu").unwrap().popup();
        settle().await;
        click(&paragraph.main, "paragraph-alignment");
        assert!(paragraph.header.arrow.is_visible());
        paragraph.header.button.emit_clicked();
        assert_eq!(
            paragraph.stack.visible_child_name().as_deref(),
            Some("main")
        );
        paragraph.popover.upgrade().unwrap().popdown();
        window.set_default_size(1280, 850);
        settle().await;
        assert!(table.is_mapped() && image.is_mapped());
        assert_eq!(wordcraft_gnome::preferences::toolbar("elements"), saved);
        assert_eq!(state(&ui), before);
        // Leave every switch off: each command remains reachable once in overflow.
        for group in ["paragraph", "text", "elements"] {
            let off = wordcraft_gnome::preferences::toolbar(group)
                .into_iter()
                .map(|(id, _)| (id, false))
                .collect::<Vec<_>>();
            wordcraft_gnome::preferences::set_toolbar(group, &off).unwrap();
        }
        ui.quick_tools.borrow().as_ref().unwrap().layout();
        for group in ["paragraph", "text", "elements"] {
            for tool in wordcraft_gnome::toolbar::tools(group) {
                assert!(
                    !find_widget(&ui.format_toolbar, &format!("format-quick-{}", tool.id))
                        .unwrap()
                        .is_visible()
                );
            }
        }
        wordcraft_gnome::preferences::set_toolbar("elements", &saved).unwrap();
        ui.quick_tools.borrow().as_ref().unwrap().layout();
        // Start must use the same allocated width for long two-line template names.
        let folder = std::env::temp_dir().join(format!("leaf-start232-{}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let template = wordcraft_engine::io::save_bytes(
            "template.docx",
            &wordcraft_doc::Document::from_text("Template preview"),
        )
        .unwrap();
        for name in [
            "A two line personal template name",
            "A very long personal template name which must use two full lines before an ellipsis because it cannot possibly fit inside this sidebar",
        ] {
            std::fs::write(folder.join(format!("{name}.docx")), &template).unwrap();
        }
        wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        let host = ui.host.upgrade().unwrap();
        blank_tab(&app, &host);
        let blank = selected_ui(&host).unwrap();
        for _ in 0..500 {
            if find_widget(&blank.template_slot, "personal-template").is_some() {
                break;
            }
            glib::timeout_future(std::time::Duration::from_millis(10)).await;
        }
        settle().await;
        let sidebar_width = blank.sidebar.sidebar().unwrap().width();
        let page_x = blank.current.borrow().as_ref().unwrap().page_frames[0]
            .compute_bounds(&window)
            .unwrap()
            .x();
        let button = find_widget(&blank.template_slot, "personal-template")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        assert!(
            button.is_mapped(),
            "Start templates must be rendered, not merely present in a hidden widget tree"
        );
        assert_eq!(
            blank.sidebar_pages.visible_child_name().as_deref(),
            Some("start")
        );
        assert!(
            find_widget(&blank.template_slot, "start-recovery-document").is_none(),
            "live work must not be offered for recovery or discard"
        );
        let content = button.child().unwrap();
        let title = content
            .last_child()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap();
        assert_eq!(title.lines(), 2);
        assert_eq!(
            title.layout().line_count(),
            2,
            "the template name must actually use two lines"
        );
        let preview = content.first_child().unwrap();
        assert!(preview.width() <= 30 && preview.height() <= 42);
        apply_user_edit(&blank, Edit::Insert("x".into()));
        settle().await;
        assert_eq!(blank.sidebar.sidebar().unwrap().width(), sidebar_width);
        assert!(
            (blank.current.borrow().as_ref().unwrap().page_frames[0]
                .compute_bounds(&window)
                .unwrap()
                .x()
                - page_x)
                .abs()
                < 1.0
        );
        apply_user_edit(&blank, Edit::Backspace);
        settle().await;
        assert_eq!(blank.sidebar.sidebar().unwrap().width(), sidebar_width);
        assert!(
            (blank.current.borrow().as_ref().unwrap().page_frames[0]
                .compute_bounds(&window)
                .unwrap()
                .x()
                - page_x)
                .abs()
                < 1.0
        );
        std::fs::remove_dir_all(folder).unwrap();
        eprintln!(
            "TOOLBAR41_UI_OK direct=true overflow=true options-read-only=true insertion-fixed=true preferences=true keyboard-reorder=true saved-resize=true"
        );
        app.quit();
    });
}

pub(super) fn reopen(ui: &Rc<Ui>, app: &Application) {
    for (group, first) in [
        ("paragraph", "list"),
        ("text", "italic"),
        ("elements", "table"),
    ] {
        assert_eq!(wordcraft_gnome::preferences::toolbar(group)[0].0, first);
    }
    assert!(
        wordcraft_gnome::preferences::toolbar("paragraph")
            .iter()
            .all(|(_, on)| !on)
    );
    assert!(
        wordcraft_gnome::preferences::toolbar("text")
            .iter()
            .all(|(_, on)| !on)
    );
    let elements = wordcraft_gnome::preferences::toolbar("elements");
    assert!(elements.iter().find(|(id, _)| id == "image").unwrap().1);
    assert!(
        find_widget(&ui.format_toolbar, "format-quick-table")
            .unwrap()
            .is_visible()
    );
    eprintln!("TOOLBAR41_REOPEN_OK groups=3 order=true switches=true");
    app.quit();
}
