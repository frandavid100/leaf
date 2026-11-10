//! Verify the shipped bottom bar and reversible sidebar through real controls.
use super::*;
thread_local! { static STARTED: Cell<bool> = const { Cell::new(false) }; }
async fn settle() {
    glib::timeout_future(std::time::Duration::from_millis(350)).await;
}
async fn until(mut ready: impl FnMut() -> bool) {
    for _ in 0..600 {
        if ready() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(15)).await;
    }
    panic!("alpha40 native operation did not complete");
}
fn children(root: &impl IsA<gtk::Widget>) -> Vec<gtk::Widget> {
    let mut result = Vec::new();
    let mut child = root.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        result.push(widget);
    }
    result
}
fn click(ui: &Ui, name: &str) {
    let widget = find_widget(&ui.format_toolbar, name)
        .unwrap_or_else(|| panic!("missing toolbar action {name}"));
    if let Ok(menu) = widget.clone().downcast::<gtk::MenuButton>() {
        menu.popup();
    } else {
        widget.downcast::<gtk::Button>().unwrap().emit_clicked();
    }
}
fn toggle(ui: &Ui, name: &str) -> gtk::Button {
    find_widget(&ui.format_toolbar, &format!("format-quick-{name}"))
        .unwrap()
        .downcast()
        .unwrap()
}
fn first_page_y(ui: &Ui) -> f32 {
    ui.current.borrow().as_ref().unwrap().page_frames[0]
        .compute_bounds(&ui.scroll)
        .unwrap()
        .y()
}
fn text(ui: &Ui) -> String {
    ui.current
        .borrow()
        .as_ref()
        .unwrap()
        .session
        .doc
        .plain_text(wordcraft_doc::StoryRef::Body)
}
pub(super) fn folder(ui: &Rc<Ui>, app: &Application) {
    if STARTED.with(|s| s.replace(true)) {
        return;
    }
    let (ui, app) = (ui.clone(), app.clone());
    glib::spawn_future_local(async move {
        let folder = PathBuf::from(std::env::var_os("LEAF40_FOLDER").expect("folder probe path"));
        let ack =
            PathBuf::from(std::env::var_os("LEAF29_ACK_DIR").unwrap()).join("folder40-response");
        assert!(folder.is_dir());
        wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        if std::env::var_os("FLATPAK_ID").is_some() {
            assert!(std::fs::File::create(folder.join("leaf40-must-not-write")).is_err());
        }
        let host = ui.host.upgrade().unwrap();
        blank_tab(&app, &host);
        let target = selected_ui(&host).unwrap();
        until(|| find_widget(&target.start_footer, "start-open-template-folder").is_some()).await;
        let identity = document_identity(&target);
        find_widget(&target.start_footer, "start-open-template-folder")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        until(|| ack.is_file()).await;
        settle().await;
        assert_eq!(document_identity(&target), identity);
        assert_eq!(wordcraft_gnome::templates::folder(), Some(folder));
        assert_eq!(target.sidebar_title.text(), wordcraft_gnome::t!("Start"));
        eprintln!(
            "WORKFLOWS40_FOLDER_UI_OK empty-folder=true native-launch=true source-unchanged=true"
        );
        app.quit();
    });
}
pub(super) fn test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    if STARTED.with(|s| s.replace(true)) {
        return;
    }
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        let host = ui.host.upgrade().unwrap();
        window.set_title(Some("Leaf Workflows40"));
        window.set_default_size(1280, 850);
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("Alpha Beta")),
            "Toolbar test",
        );
        settle().await;
        let adaptive = ui.adaptive.borrow().as_ref().unwrap().clone();
        assert!(ui.format_toolbar.is_mapped());
        assert_eq!(
            adaptive.formats.parent().as_ref(),
            Some(adaptive.bottom.upcast_ref())
        );
        assert_eq!(
            adaptive.history.parent().as_ref(),
            Some(adaptive.top_history.upcast_ref())
        );
        let groups = ["style", "text", "paragraph", "elements"];
        let mut previous = 0.;
        for name in groups {
            let group = find_widget(&ui.format_toolbar, &format!("format-{name}-group")).unwrap();
            let r = group.compute_bounds(&ui.format_toolbar).unwrap();
            assert!(r.x() >= previous);
            previous = r.x() + r.width();
            if name != "elements" {
                assert!(group.next_sibling().unwrap().is::<gtk::Separator>());
            }
        }
        ui.current
            .borrow_mut()
            .as_mut()
            .unwrap()
            .session
            .run("select.all", &serde_json::json!({"whole":true}))
            .unwrap();
        refresh_selection(&ui);
        let bold = toggle(&ui, "bold");
        bold.emit_clicked();
        assert_eq!(selection_character(&ui)["bold"], true);
        assert!(ui.document_box.has_focus());
        apply_user_edit(&ui, Edit::Undo);
        assert_eq!(selection_character(&ui)["bold"], false);
        {
            let mut current = ui.current.borrow_mut();
            let session = &mut current.as_mut().unwrap().session;
            selection::select_range(session, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
            session
                .run("format.bold", &serde_json::json!({"value":true}))
                .unwrap();
            selection::select_all(session).unwrap();
        }
        refresh_selection(&ui);
        assert!(selection_character(&ui)["bold"].is_null());
        assert!(
            find_widget(&ui.format_toolbar, "format-quick-bold-mixed")
                .unwrap()
                .is_visible()
        );
        bold.emit_clicked();
        assert_eq!(selection_character(&ui)["bold"], true);
        for name in ["italic", "underline"] {
            toggle(&ui, name).emit_clicked();
        }
        assert_eq!(selection_character(&ui)["italic"], true);
        assert_ne!(selection_character(&ui)["underline"], "none");
        click(&ui, "format-quick-alignment");
        assert!(
            find_widget(
                &ui.paragraph.borrow().as_ref().unwrap().detail,
                "paragraph-align-center"
            )
            .is_some()
        );
        let alignment = find_widget(
            &ui.paragraph.borrow().as_ref().unwrap().detail,
            "paragraph-align-center",
        )
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
        alignment.emit_clicked();
        assert_eq!(selection_paragraph(&ui)["align"], "center");
        click(&ui, "format-quick-list");
        let panel = ui.paragraph.borrow().as_ref().unwrap().clone();
        assert_eq!(panel.header.label.text(), wordcraft_gnome::t!("List"));
        assert!(!panel.header.arrow.is_visible());
        panel.page("bullets");
        assert!(panel.header.arrow.is_visible());
        panel.popover.upgrade().unwrap().popdown();
        let style_menu = format_button(&ui, "style-menu").unwrap();
        style_menu.popup();
        settle().await;
        let style = ui.style.borrow().as_ref().unwrap().clone();
        find_widget(&style.list, "style-apply-Heading1")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert!(format_button(&ui, "style-actions").is_none());
        format_execute(&ui, "format.bold", serde_json::json!({"value":false})).unwrap();
        let normal_before = ui
            .current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .doc
            .styles
            .resolve_char(Some("Normal"), &Default::default());
        style_menu.popup();
        style.show_detail("Heading1");
        find_widget(&style.detail, "style-update")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert!(
            !ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .styles
                .resolve_char(Some("Heading1"), &Default::default())
                .bold
        );
        assert_eq!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .styles
                .resolve_char(Some("Normal"), &Default::default()),
            normal_before
        );
        apply_user_edit(&ui, Edit::Undo);
        let state = (
            document_identity(&ui),
            ui.current.borrow().as_ref().unwrap().session.sel.clone(),
        );
        style_menu.popup();
        find_widget(&style.list, "style-manage-Heading1")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert_eq!(style.stack.visible_child_name().as_deref(), Some("detail"));
        assert_eq!(
            (
                document_identity(&ui),
                ui.current.borrow().as_ref().unwrap().session.sel.clone()
            ),
            state
        );
        // Let the actions-to-editor popup handoff finish before simulating
        // the next user gesture. Resize must start from a drawn, pinned panel.
        settle().await;
        assert!(
            style_menu.popover().unwrap().is_mapped(),
            "Edit Style must open its panel"
        );
        popover_pin(&style_menu.popover().unwrap()).set_active(true);
        settle().await;
        window.set_default_size(420, 800);
        settle().await;
        until(|| style_menu.popover().unwrap().is_mapped()).await;
        assert!(ui.format_toolbar.is_mapped());
        assert!(style_menu.popover().unwrap().is_mapped());
        assert!(!bold.is_mapped());
        assert!(format_button(&ui, "text-menu").unwrap().is_mapped());
        assert!(
            ui.format_toolbar.hadjustment().upper()
                <= ui.format_toolbar.hadjustment().page_size() + 1.
        );
        assert_eq!(
            (
                document_identity(&ui),
                ui.current.borrow().as_ref().unwrap().session.sel.clone()
            ),
            state
        );
        window.set_default_size(1280, 850);
        settle().await;
        until(|| style_menu.popover().unwrap().is_mapped()).await;
        assert!(style_menu.popover().unwrap().is_mapped());
        popover_pin(&style_menu.popover().unwrap()).set_active(false);
        style_menu.popdown();
        let elements = ui.elements.borrow().as_ref().unwrap().clone();
        for (name, origin) in [
            ("image", ui.picture.borrow().as_ref().unwrap().pop.clone()),
            ("table", ui.table_popover.borrow().as_ref().unwrap().clone()),
        ] {
            let before = (
                document_identity(&ui),
                ui.current.borrow().as_ref().unwrap().session.sel.clone(),
            );
            click(&ui, &format!("format-quick-{name}"));
            assert!(elements.body_for(&origin).is_some());
            assert!(elements.pop.is_visible());
            assert_eq!(
                (
                    document_identity(&ui),
                    ui.current.borrow().as_ref().unwrap().session.sel.clone()
                ),
                before
            );
            elements.menu.popdown();
            settle().await;
        }
        eprintln!(
            "WORKFLOWS40_TOOLBAR_OK groups=true mixed=true direct-actions=true styles=true elements=true overflow=true pinned-resize=true"
        );

        let folder = wordcraft_gnome::preferences::config_root().join("template40");
        std::fs::create_dir_all(&folder).unwrap();
        wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        blank_tab(&app, &host);
        let start = selected_ui(&host).unwrap();
        start.sidebar.set_show_sidebar(true);
        until(|| find_widget(&start.start_footer, "start-open-template-folder").is_some()).await;
        settle().await;
        assert_eq!(start.sidebar_title.text(), wordcraft_gnome::t!("Start"));
        assert!(!start.sidebar_selector.is_mapped());
        assert!(start.start_footer.is_mapped());
        assert!(start.sidebar_footer.height() > 0);
        let y = first_page_y(&start);
        for _ in 0..3 {
            start.document_box.grab_focus();
            apply_user_edit(&start, Edit::Insert("ñ".into()));
            settle().await;
            assert_eq!(
                start.sidebar_pages.visible_child_name().as_deref(),
                Some("navigation")
            );
            assert!(start.sidebar_selector.is_mapped());
            assert!((first_page_y(&start) - y).abs() < 1.5);
            apply_user_edit(&start, Edit::Undo);
            settle().await;
            assert_eq!(text(&start), "");
            assert_eq!(
                start.sidebar_pages.visible_child_name().as_deref(),
                Some("start")
            );
            assert!(!start.sidebar_selector.is_mapped());
            assert!(start.document_box.has_focus());
            assert!((first_page_y(&start) - y).abs() < 1.5);
            apply_user_edit(&start, Edit::Redo);
            apply_user_edit(&start, Edit::Undo);
        }
        settle().await;
        apply_user_edit(&start, Edit::Insert("x".into()));
        apply_user_edit(&start, Edit::Backspace);
        settle().await;
        assert_eq!(
            start.sidebar_pages.visible_child_name().as_deref(),
            Some("start")
        );
        apply_user_edit(&start, Edit::Insert("Delete all".into()));
        start
            .current
            .borrow_mut()
            .as_mut()
            .unwrap()
            .session
            .run("select.all", &serde_json::json!({"whole":true}))
            .unwrap();
        apply_user_edit(&start, Edit::Delete);
        settle().await;
        assert_eq!(
            start.sidebar_pages.visible_child_name().as_deref(),
            Some("start")
        );
        let png = wordcraft_engine::cmd::insert::base64_decode(include_str!(
            "../../../tests/fixtures/picture.b64"
        ))
        .unwrap();
        tools_change(&start, |s| pictures::insert_data(s, &png));
        settle().await;
        assert_eq!(
            start.sidebar_pages.visible_child_name().as_deref(),
            Some("navigation")
        );
        click(&start, "format-quick-image");
        assert!(
            start
                .elements
                .borrow()
                .as_ref()
                .unwrap()
                .body_for(&start.picture.borrow().as_ref().unwrap().pop)
                .is_some()
        );
        start.elements.borrow().as_ref().unwrap().menu.popdown();
        apply_user_edit(&start, Edit::Undo);
        settle().await;
        assert_eq!(
            start.sidebar_pages.visible_child_name().as_deref(),
            Some("start")
        );
        format_execute(
            &start,
            "insert.table",
            serde_json::json!({"rows":1,"cols":1}),
        )
        .unwrap();
        settle().await;
        assert_eq!(
            start.sidebar_pages.visible_child_name().as_deref(),
            Some("navigation")
        );
        click(&start, "format-quick-table");
        assert!(
            start
                .elements
                .borrow()
                .as_ref()
                .unwrap()
                .body_for(start.table_popover.borrow().as_ref().unwrap())
                .is_some()
        );
        start.elements.borrow().as_ref().unwrap().menu.popdown();
        apply_user_edit(&start, Edit::Undo);
        settle().await;
        assert_eq!(
            start.sidebar_pages.visible_child_name().as_deref(),
            Some("start")
        );
        eprintln!(
            "WORKFLOWS40_START_OK reversible=true undo-redo=true page-stable=true non-text=true empty-folder=true"
        );

        let recovery_root = wordcraft_gnome::preferences::recovery_root();
        gio::spawn_blocking(move || {
            workflows38_checks::write_pending(recovery_root.clone());
            std::thread::sleep(std::time::Duration::from_millis(1100));
            workflows38_checks::write_pending(recovery_root);
        })
        .await
        .unwrap();
        eprintln!("WORKFLOWS40_RECOVERY_SEEDED");
        start
            .template_gallery
            .borrow()
            .as_ref()
            .unwrap()
            .refresh_recovery();
        until(|| find_widget(&start.template_slot, "start-recovery-discard").is_some()).await;
        eprintln!("WORKFLOWS40_RECOVERY_LISTED");
        let template_bytes = wordcraft_engine::io::save_bytes(
            "sample.docx",
            &wordcraft_doc::Document::from_text("Newest personal template"),
        )
        .unwrap();
        for i in 0..6 {
            let path = folder.join(format!("Template {i}.docx"));
            std::fs::write(&path, &template_bytes).unwrap();
            std::fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(std::time::SystemTime::now() + std::time::Duration::from_secs(i))
                .unwrap();
        }
        templates::offer(&start);
        until(|| {
            find_widget(&start.template_slot, "personal-template-gallery")
                .is_some_and(|strip| children(&strip).len() == 6)
        })
        .await;
        until(|| find_widget(&start.template_slot, "start-recovery-discard").is_some()).await;
        settle().await;
        let recovery = find_widget(&start.template_slot, "start-recovery").unwrap();
        let templates_group = find_widget(&start.template_slot, "start-templates").unwrap();
        assert_eq!(recovery.next_sibling().as_ref(), Some(&templates_group));
        let rows = children(&recovery.first_child().unwrap());
        assert_eq!(rows.len(), 2);
        for row in &rows {
            let preview = find_widget(row, "recovery-preview")
                .unwrap()
                .compute_bounds(&window)
                .unwrap();
            let discard = find_widget(row, "start-recovery-discard")
                .unwrap()
                .compute_bounds(&window)
                .unwrap();
            assert!(
                discard.x() >= preview.x() + 4.0
                    && discard.y() >= preview.y() + 4.0
                    && discard.x() + discard.width() <= preview.x() + preview.width() - 4.0
                    && discard.y() + discard.height() <= preview.y() + preview.height() - 4.0,
                "discard bounds {discard:?} must be inset inside paper {preview:?}"
            );
        }

        let dates: Vec<_> = rows
            .iter()
            .map(|row| {
                let button = find_widget(row, "start-recovery-document")
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap();
                button
                    .child()
                    .unwrap()
                    .last_child()
                    .unwrap()
                    .last_child()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap()
                    .text()
                    .to_string()
            })
            .collect();
        let mut expected = gio::spawn_blocking(|| {
            wordcraft_gnome::recovery::inventory(&wordcraft_gnome::preferences::recovery_root())
                .unwrap()
                .documents
        })
        .await
        .unwrap();
        expected.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
        let expected: Vec<_> = expected
            .iter()
            .map(|d| {
                glib::DateTime::from_iso8601(&d.saved_at, None)
                    .unwrap()
                    .to_local()
                    .unwrap()
                    .format("%x %X")
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(dates, expected);
        let strip = find_widget(&start.template_slot, "personal-template-gallery").unwrap();
        let names: Vec<_> = children(&strip)
            .iter()
            .map(|row| {
                row.clone()
                    .downcast::<gtk::Button>()
                    .unwrap()
                    .child()
                    .unwrap()
                    .last_child()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap()
                    .text()
                    .to_string()
            })
            .collect();
        assert_eq!(
            names,
            (0..6)
                .rev()
                .map(|i| format!("Template {i}"))
                .collect::<Vec<_>>()
        );
        let footer_before = start.start_footer.compute_bounds(&window).unwrap();
        let scroller = start
            .sidebar_pages
            .child_by_name("start")
            .unwrap()
            .downcast::<gtk::ScrolledWindow>()
            .unwrap();
        let adj = scroller.vadjustment();
        assert!(adj.upper() > adj.page_size());
        adj.set_value(adj.upper() - adj.page_size());
        settle().await;
        assert_eq!(
            start.start_footer.compute_bounds(&window).unwrap(),
            footer_before
        );
        assert!(
            find_widget(&start.start_footer, "start-open-template-folder")
                .unwrap()
                .is_mapped()
        );
        adj.set_value(0.);
        settle().await;
        eprintln!(
            "WORKFLOWS40_START_ORDER_OK recoveries=2 templates=6 newest-first=true continuous=true fixed-footer=true scroll=true"
        );
        let before = gio::spawn_blocking(|| {
            wordcraft_gnome::recovery::inventory(&wordcraft_gnome::preferences::recovery_root())
                .unwrap()
                .documents
                .len()
        })
        .await
        .unwrap();
        find_widget(&start.template_slot, "start-recovery-discard")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        settle().await;
        let dialog = window
            .visible_dialog()
            .unwrap()
            .downcast::<adw::AlertDialog>()
            .unwrap();
        dialog.close();
        settle().await;
        let after = gio::spawn_blocking(|| {
            wordcraft_gnome::recovery::inventory(&wordcraft_gnome::preferences::recovery_root())
                .unwrap()
                .documents
                .len()
        })
        .await
        .unwrap();
        assert_eq!(before, after);
        eprintln!("WORKFLOWS40_RECOVERY_CANCEL_OK");
        find_widget(&start.template_slot, "start-recovery-discard")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        settle().await;
        let dialog = window
            .visible_dialog()
            .unwrap()
            .downcast::<adw::AlertDialog>()
            .unwrap();
        dialog.emit_by_name::<()>("response", &[&"discard"]);
        dialog.close();
        until(|| children(&recovery.first_child().unwrap()).len() == before - 1).await;
        assert!(
            find_widget(&start.template_slot, "start-recovery-discard").is_some(),
            "discarding one recovery preserves the other"
        );
        assert_eq!(start.sidebar_title.text(), wordcraft_gnome::t!("Start"));
        eprintln!(
            "WORKFLOWS40_UI_OK confirmation=true cancel-preserves=true discard=true start-footer-retained=true"
        );
        app.quit();
    });
}
