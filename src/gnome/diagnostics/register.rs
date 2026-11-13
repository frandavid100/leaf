use super::*;

/// Existing controls borrowed only while opt-in diagnostic callbacks are installed.
pub(in crate::gnome) struct DiagnosticControls<'a> {
    pub(in crate::gnome) app_menu: &'a gtk::MenuButton,
    pub(in crate::gnome) style_menu: &'a gtk::MenuButton,
    pub(in crate::gnome) align_menu: &'a gtk::MenuButton,
    pub(in crate::gnome) text_menu: &'a gtk::MenuButton,
    pub(in crate::gnome) table_menu: &'a gtk::MenuButton,
    pub(in crate::gnome) menu_button: &'a gtk::MenuButton,
    pub(in crate::gnome) picture_menu: &'a gtk::MenuButton,
    pub(in crate::gnome) style_popover: &'a Rc<StylePopover>,
    pub(in crate::gnome) paragraph_popover: &'a Rc<ParagraphPopover>,
    pub(in crate::gnome) text_popover: &'a gtk::Popover,
    pub(in crate::gnome) picture_panel: &'a Rc<PicturePopover>,
    pub(in crate::gnome) outline_panel: &'a Rc<OutlinePanel>,
    pub(in crate::gnome) titles_toggle: &'a gtk::ToggleButton,
    pub(in crate::gnome) search_panel: &'a Rc<SearchPanel>,
}

// One captured owner set and callback type per diagnostic signature. Preserve
// each registration's original ordering, delay and first-tab condition.
struct DiagnosticRunner<'a> {
    ui: &'a Rc<Ui>,
    window: &'a ApplicationWindow,
    app: &'a Application,
}
impl DiagnosticRunner<'_> {
    fn ui(&self, millis: u64, run: fn(&Rc<Ui>, &Application)) {
        let (ui, app) = (self.ui.clone(), self.app.clone());
        capture_ready::after(std::time::Duration::from_millis(millis), move || {
            run(&ui, &app)
        });
    }
    fn window(&self, millis: u64, run: fn(&Rc<Ui>, &ApplicationWindow, &Application)) {
        let (ui, window, app) = (self.ui.clone(), self.window.clone(), self.app.clone());
        capture_ready::after(std::time::Duration::from_millis(millis), move || {
            run(&ui, &window, &app)
        });
    }
}
fn command_line() -> &'static [String] {
    static ARGS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    ARGS.get_or_init(|| std::env::args().collect())
}
fn requested(flag: &str) -> bool {
    command_line().iter().any(|arg| arg == flag)
}

pub(in crate::gnome) fn install_diagnostics(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    host: &Rc<Workspace>,
    app: &Application,
    controls: DiagnosticControls<'_>,
) {
    let runner = DiagnosticRunner { ui, window, app };
    if requested("--zoom278-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, maintenance45_checks::zoom);
    }
    if requested("--catalog259-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, maintenance45_checks::catalog);
    }
    if requested("--picture280-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, maintenance45_checks::picture_delete);
    }
    if requested("--font279-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, maintenance45_checks::fonts);
    }
    // Older formatting probes exercise the supported all-overflow configuration.
    // The toolbar41 probe independently covers defaults, direct and overflow targets,
    // preferences, reordering and resize. Keep the older interactions accessible.
    let legacy = command_line()
        .iter()
        .any(|a| a.ends_with("-test") || a == "--split-states-demo")
        && !command_line().iter().any(|a| {
            a.starts_with("--alpha43-")
                || a.starts_with("--workflows40-")
                || a.starts_with("--toolbar41-")
        });
    if legacy {
        for group in ["paragraph", "text", "elements"] {
            let order = wordcraft_gnome::preferences::toolbar(group)
                .into_iter()
                .map(|(id, _)| (id, false))
                .collect::<Vec<_>>();
            wordcraft_gnome::preferences::set_toolbar(group, &order).unwrap();
        }
        if let Some(tools) = ui.quick_tools.borrow().as_ref() {
            tools.layout();
        }
    }
    if requested("--toolbar41-reopen-test") && host.tabs.n_pages() == 1 {
        runner.ui(800, toolbar41_checks::reopen);
    }
    if requested("--alpha43-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, alpha43_checks::test);
    }
    if requested("--toolbar41-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, toolbar41_checks::test);
    }
    if command_line().iter().any(|arg| {
        arg.starts_with("--alpha43-")
            || arg == "--catalog259-ui-test"
            || arg == "--columns-ui-test"
            || arg == "--picture280-ui-test"
            || arg == "--zoom278-ui-test"
            || arg == "--font279-ui-test"
            || arg.starts_with("--toolbar41-")
            || arg.starts_with("--workflows36-")
            || arg.starts_with("--workflows37-")
            || arg.starts_with("--workflows38-")
            || arg.starts_with("--workflows40-")
    }) {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            previous(info);
            std::process::exit(1);
        }));
    }
    if requested("--workflows40-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, workflows40_checks::test);
    }
    if requested("--workflows40-folder-test") && host.tabs.n_pages() == 1 {
        runner.ui(800, workflows40_checks::folder);
    }
    if requested("--workflows38-startup-test") {
        runner.ui(800, workflows38_checks::startup);
    }
    if requested("--workflows38-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, workflows38_checks::test);
    }
    if requested("--workflows37-ui-test") && host.tabs.n_pages() == 1 {
        runner.window(800, workflows37_checks::test);
    }
    if requested("--workflows35-pointer-test") && host.tabs.n_pages() == 1 {
        let (u, w, a, outline) = (
            ui.clone(),
            window.clone(),
            app.clone(),
            controls.outline_panel.clone(),
        );
        capture_ready::after(std::time::Duration::from_millis(800), move || {
            workflows35_checks::pointer(&u, &w, &a, &outline)
        });
    }
    if requested("--workflows36-template-folder-test") && host.tabs.n_pages() == 1 {
        let (u, w, a) = (ui.clone(), window.clone(), app.clone());
        capture_ready::after(std::time::Duration::from_millis(800), move || {
            workflows36_checks::template_folder(&u, &w, &a)
        });
    }
    if requested("--workflows36-pointer-test") && host.tabs.n_pages() == 1 {
        let (u, w, a, s) = (
            ui.clone(),
            window.clone(),
            app.clone(),
            controls.search_panel.clone(),
        );
        capture_ready::after(std::time::Duration::from_millis(800), move || {
            workflows36_checks::pointer(&u, &w, &a, &s)
        });
    }
    if requested("--workflows36-ui-test") && host.tabs.n_pages() == 1 {
        let (u, w, a, s, o) = (
            ui.clone(),
            window.clone(),
            app.clone(),
            controls.search_panel.clone(),
            controls.outline_panel.clone(),
        );
        capture_ready::after(std::time::Duration::from_millis(800), move || {
            workflows36_checks::test(&u, &w, &a, &s, &o)
        });
    }
    if requested("--workflows35-ui-test") && host.tabs.n_pages() == 1 {
        let (u, w, a, s, style, outline) = (
            ui.clone(),
            window.clone(),
            app.clone(),
            controls.search_panel.clone(),
            controls.style_popover.clone(),
            controls.outline_panel.clone(),
        );
        capture_ready::after(std::time::Duration::from_millis(800), move || {
            workflows35_checks::test(&u, &w, &a, &s, &style, &outline)
        });
    }
    for (flag, mode) in [
        ("--dictionary84-ui-test", "folder"),
        ("--dictionary84-reopen-test", "reopen"),
        ("--dictionary84-native-test", "native"),
    ] {
        if command_line().iter().any(|a| a == flag) && host.tabs.n_pages() == 1 {
            let (u, a) = (ui.clone(), app.clone());
            capture_ready::after(std::time::Duration::from_millis(800), move || {
                dictionaries84_ui_test(&u, &a, mode)
            });
        }
    }
    for flag in ["--pending-end-ui-test", "--pending-spacing-ui-test"] {
        if command_line().iter().any(|arg| arg == flag) && host.tabs.n_pages() == 1 {
            let (u, w, a) = (ui.clone(), window.clone(), app.clone());
            capture_ready::after(std::time::Duration::from_millis(800), move || {
                if flag == "--pending-end-ui-test" {
                    pending_end_ui_test(&u, &w, &a)
                } else {
                    pending_spacing_ui_test(&u, &a)
                }
            });
        }
    }
    if requested("--captions-ui-test") && host.tabs.n_pages() == 1 {
        captions_ui_test(ui, window, app);
    }
    if requested("--format136-ui-test") && host.tabs.n_pages() == 1 {
        format136_ui_test(
            ui,
            window,
            app,
            controls.text_menu,
            controls.align_menu,
            controls.style_menu,
            controls.style_popover,
        );
    }
    if requested("--alpha31-ui-test") && host.tabs.n_pages() == 1 {
        alpha31_ui_test(ui, window, app);
    }
    if requested("--alpha30-ui-test") && host.tabs.n_pages() == 1 {
        alpha30_ui_test(ui, window, app);
    }
    if command_line()
        .iter()
        .any(|a| a == "--review29-ui-test" || a == "--review29-demo")
    {
        review29_ui_test(ui, window, app);
    }
    if requested("--alpha29-ui-test") {
        alpha29_ui_test(ui, window, app);
    }
    if requested("--alpha28-ui-test") {
        alpha28_ui_test(ui, window, app);
    }
    if requested("--alpha28-gallery") {
        alpha28_gallery(ui, window);
    }
    if requested("--sweep20-pointer-test") {
        runner.window(1200, sweep20_pointer_test);
    }
    let DiagnosticControls {
        app_menu,
        style_menu,
        align_menu,
        text_menu,
        table_menu,
        menu_button,
        picture_menu,
        style_popover,
        paragraph_popover,
        text_popover,
        picture_panel,
        outline_panel,
        titles_toggle,
        search_panel,
    } = controls;
    if let Some(mode) = command_line()
        .iter()
        .find_map(|a| a.strip_prefix("--followup19-").map(str::to_owned))
    {
        let (u, w, a, m) = (ui.clone(), window.clone(), app.clone(), text_menu.clone());
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            followup_pointer_test(&u, &w, &a, &m, mode)
        });
    }
    if requested("--text18-pointer-test") {
        runner.ui(1200, text18_pointer_test);
    }
    if requested("--sweep18-ui-test") {
        runner.ui(1200, sweep18_ui_test);
    }
    if let Some(case) = command_line()
        .iter()
        .find_map(|a| a.strip_prefix("--table18-").map(str::to_owned))
    {
        let (u, a) = (ui.clone(), app.clone());
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            table_pointer_ui_test(&u, &a, case)
        });
    }
    if requested("--heading-top-test") {
        let (u, s, a) = (ui.clone(), outline_panel.clone(), app.clone());
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            heading_top_ui_test(&u, &s, &a)
        });
    }
    if requested("--uri-launch-test") {
        let u = ui.clone();
        capture_ready::after(std::time::Duration::from_millis(1000), move || {
            launch_document_link(&u, "https://example.org/leaf-launch-test")
        });
    }
    if requested("--interaction-pointer-test") {
        let (u, w, a, m) = (ui.clone(), window.clone(), app.clone(), text_menu.clone());
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            interaction_pointer_test(&u, &w, &a, &m)
        });
    }
    if host.tabs.n_pages() == 1 && requested("--issue-sweep-ui-test") {
        let (u, w, h, a, m, main, pm, tm) = (
            ui.clone(),
            window.clone(),
            host.clone(),
            app.clone(),
            menu_button.clone(),
            app_menu.clone(),
            align_menu.clone(),
            text_menu.clone(),
        );
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            issue_sweep_ui_test(&u, &w, &h, &a, &m, &main, &pm, &tm)
        });
    }
    if requested("--comment-pointer-test") {
        runner.window(1200, comment_pointer_ui_test);
    }
    if requested("--save-current-file-test") {
        runner.window(1200, save_current_file_ui_test);
    }
    if command_line().iter().any(|a| {
        matches!(
            a.as_str(),
            "--annotations-ui-test" | "--annotations-demo" | "--bookmarks-demo"
        )
    }) {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1000), move || {
            annotations_ui_test(&u, &w, &a);
        });
    }
    if requested("--localization-ui-test") {
        runner.ui(700, localization_ui_test);
    }
    if requested("--adaptive-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let app = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1400), move || {
            adaptive_ui_test(&u, &w, &app)
        });
    }
    if let Some(label) = command_line().iter().find_map(|a| match a.as_str() {
        "--context-style-demo" => Some(wordcraft_gnome::t!("Style")),
        "--context-paragraph-demo" => Some(wordcraft_gnome::t!("Paragraph")),
        "--context-text-demo" => Some(wordcraft_gnome::t!("Text")),
        _ => None,
    }) {
        let u = ui.clone();
        let w = window.clone();
        capture_ready::after(std::time::Duration::from_millis(1400), move || {
            let p = show_document_context(&u, &w, 140, 160, DocumentContext::Text);
            context_activate_row(&p, label);
        });
    }

    if requested("--clipboard-ui-test") {
        let ui = ui.clone();
        let app = app.clone();
        capture_ready::after(std::time::Duration::from_millis(800), move || {
            clipboard_ui_test(&ui, &app)
        });
    }
    if requested("--context-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            context_ui_test(&u, &w, &a)
        });
    }
    if requested("--table-keyboard-test") {
        let u = ui.clone();
        let app = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            opened(
                &u,
                Session::new(wordcraft_doc::Document::new()),
                "Prueba de tablas",
            );
            table_command(&u, "insert.table", serde_json::json!({"rows":1,"cols":2}));
            eprintln!("TABLE_KEYBOARD_READY");
            capture_ready::after(std::time::Duration::from_secs(6), move || {
                {
                    let s = u.current.borrow();
                    let d = s.as_ref().unwrap();
                    let (tp, r, c) = d
                        .session
                        .sel
                        .focus
                        .path
                        .cell()
                        .expect("keyboard stayed in table");
                    let table = d.session.doc.table(d.session.sel.focus.story, &tp).unwrap();
                    assert_eq!(table.rows.len(), 2, "Tab at the end must add a row");
                    assert_eq!((r, c), (0, 1), "Shift+Tab must return to the previous cell");
                    assert!(
                        d.session
                            .doc
                            .plain_text(wordcraft_doc::StoryRef::Body)
                            .contains("GNOME"),
                        "keyboard input reaches the cell"
                    );
                }
                eprintln!("TABLE_KEYBOARD_OK");
                app.quit();
            });
        });
    }
    if command_line().iter().any(|a| {
        a == "--table-menu-demo" || a == "--table-properties-demo" || a == "--table-insert-demo"
    }) {
        let u = ui.clone();
        let w = window.clone();
        let m = table_menu.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            if requested("--table-insert-demo") {
                show_insert_table(&u, &w);
                return;
            }
            if let Some(d) = u.current.borrow_mut().as_mut() {
                d.session
                    .run("caret.docEnd", &serde_json::json!({}))
                    .unwrap();
                d.session
                    .run("insert.table", &serde_json::json!({"rows":3,"cols":3}))
                    .unwrap();
                d.session
                    .run(
                        "text.insert",
                        &serde_json::json!({"text":"Tabla de prueba"}),
                    )
                    .unwrap();
            }
            after_content_change(&u);
            caret_ensure_visible(&u);
            if requested("--table-properties-demo") {
                show_table_properties(&u, &w);
            } else {
                m.popup();
            }
        });
    }
    if requested("--split-states-demo") {
        let m = align_menu.clone();
        let s = paragraph_popover.clone();
        window.set_title(Some("Leaf Split Button Regression"));
        // Supply coordinates only: extra capture controllers change native routing.
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            s.execute("para.indents", serde_json::json!({"left":18.}));
            m.popup();
            capture_ready::after(std::time::Duration::from_millis(400), move || {
                use gdk::prelude::PopupExt;
                let pop = s.popover.upgrade().unwrap();
                let body = find_widget(&s.main, "paragraph-more-indents-apply")
                    .unwrap()
                    .compute_bounds(&pop)
                    .unwrap();
                let arrow = find_widget(&s.main, "paragraph-more-indents")
                    .unwrap()
                    .compute_bounds(&pop)
                    .unwrap();
                let surface = pop.surface().unwrap().downcast::<gdk::Popup>().unwrap();
                let (tx, ty) = pop.surface_transform();
                let x = surface.position_x() as f64 - tx;
                let y = surface.position_y() as f64 - ty;
                eprintln!(
                    "SPLIT_POINTER_READY {} {} {} {}",
                    (x + body.x() as f64 + body.width() as f64 / 2.0) as i32,
                    (y + body.y() as f64 + body.height() as f64 / 2.0) as i32,
                    (x + arrow.x() as f64 + arrow.width() as f64 / 2.0) as i32,
                    (y + arrow.y() as f64 + arrow.height() as f64 / 2.0) as i32
                );
            });
        });
    }
    for (flag, proof) in [
        ("--page-options-ui-test", false),
        ("--proofing-ui-test", true),
        ("--proofing-demo", true),
    ] {
        if command_line().iter().any(|a| a == flag) {
            let u = ui.clone();
            let w = window.clone();
            let a = app.clone();
            capture_ready::after(std::time::Duration::from_millis(900), move || {
                if proof {
                    proofing_ui_test(&u, &w, &a);
                } else {
                    page_options_ui_test(&u, &w, &a);
                }
            });
        }
    }
    if command_line()
        .iter()
        .any(|a| a == "--headers-ui-test" || a == "--headers-pointer-test")
    {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        capture_ready::after(std::time::Duration::from_millis(900), move || {
            header_footer_ui_test(&u, &w, &a, requested("--headers-pointer-test"))
        });
    }
    if command_line().iter().any(|a| {
        a == "--picture-demo" || a == "--document-properties-demo" || a == "--document-tools-test"
    }) {
        let u = ui.clone();
        let p = picture_panel.clone();
        let m = picture_menu.clone();
        let w = window.clone();
        let app = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            document_tools_fixture(&u);
            if requested("--document-tools-test") {
                document_tools_ui_test(&u, &p, &m, &w, &app);
            } else if requested("--document-properties-demo") {
                show_document_properties(&u, &w);
            } else {
                m.popup();
            }
        });
    }
    if requested("--columns-demo") {
        let u = ui.clone();
        let m = align_menu.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            m.popup();
            if let Some(s) = u.paragraph.borrow().as_ref() {
                s.page("columns");
            }
        });
    }
    if requested("--print-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        capture_ready::after(std::time::Duration::from_millis(900), move || {
            print_ui_test(&u, &w, &a)
        });
    }
    if requested("--host-cache-ui-test") {
        runner.ui(900, host_cache_ui_test);
    }
    if requested("--performance-ui-test") {
        runner.ui(900, performance_ui_test);
    }
    if requested("--a11y-ui-test") {
        runner.ui(900, accessible_document_ui_test);
    }
    if requested("--keyboard-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let m = app_menu.clone();
        let a = app.clone();
        capture_ready::after(std::time::Duration::from_millis(900), move || {
            keyboard_ui_test(&u, &w, &m, &a)
        });
    }
    if requested("--hidpi-ui-test") {
        runner.ui(1300, hidpi_ui_test);
    }
    if requested("--columns-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            columns_ui_test(&u, &w, &a)
        });
    }
    if command_line()
        .iter()
        .any(|a| a == "--paragraph-demo" || a == "--paragraph-spacing-demo")
    {
        let m = align_menu.clone();
        let s = paragraph_popover.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            m.popup();
            if requested("--paragraph-spacing-demo") {
                s.page("spacing");
            }
        });
    }
    if requested("--paragraph-ui-test") {
        let s = paragraph_popover.clone();
        let m = align_menu.clone();
        let app = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            paragraph_ui_test(&s, &m, &app)
        });
    }
    if requested("--search-ui-test") {
        let s = search_panel.clone();
        let ui = ui.clone();
        let app = app.clone();
        let host = host.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            search_ui_test(&ui, &s, &app, &host)
        });
    }
    if requested("--view-pointer-test") {
        let u = ui.clone();
        let m = style_menu.clone();
        let w = window.clone();
        let a = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1300), move || {
            view_pointer_test(&u, &m, &w, &a)
        });
    }
    if requested("--spreads-demo") {
        window.set_default_size(1800, 950);
        let u = ui.clone();
        capture_ready::after(std::time::Duration::from_millis(1300), move || {
            if let Some(d) = u.current.borrow_mut().as_mut() {
                d.fit_to_width = false;
            }
            zoom_to(&u, 1.0);
        });
    }
    if requested("--outline-demo") {
        let b = titles_toggle.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            b.set_active(true)
        });
    }
    if requested("--pin-ui-test") {
        let u = ui.clone();
        let sm = style_menu.clone();
        let sp = style_popover.clone();
        let pm = align_menu.clone();
        let pp = paragraph_popover.clone();
        let tm = text_menu.clone();
        let tp = text_popover.clone();
        let tb = table_menu.clone();
        let app = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            pin_ui_test(&u, &sm, &sp, &pm, &pp, &tm, &tp, &tb, &app)
        });
    }
    if requested("--chrome-ui-test") {
        let u = ui.clone();
        let m = menu_button.clone();
        let app = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            chrome_ui_test(&u, &m, &app)
        });
    }
    if requested("--document-menu-demo") {
        let m = menu_button.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || m.popup());
    }
    if requested("--main-menu-demo") {
        let m = app_menu.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || m.popup());
    }
    if requested("--overview-demo") && host.tabs.n_pages() == 1 {
        let h = host.clone();
        let app = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            blank_tab(&app, &h);
            h.overview.set_open(true);
        });
    }
    if requested("--nav-ui-test") && host.tabs.n_pages() == 1 {
        let u = ui.clone();
        let h = host.clone();
        let app = app.clone();
        let s = outline_panel.clone();
        let b = titles_toggle.clone();
        capture_ready::after(std::time::Duration::from_millis(1200), move || {
            navigation_ui_test(&u, &s, &b, &h, &app)
        });
    }
    if requested("--advanced-text-demo") {
        let w = window.clone();
        let u = ui.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            show_text_options(&u, &w)
        });
    }
    if requested("--advanced-paragraph-demo") {
        let w = window.clone();
        let u = ui.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            show_paragraph_options(&u, &w)
        });
    }
    if requested("--styles-ui-test") {
        let s = style_popover.clone();
        let m = style_menu.clone();
        let a = app.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            style_ui_test(&s, &m, &a)
        });
    }
    if command_line()
        .iter()
        .any(|a| a == "--styles-demo" || a == "--style-details-demo" || a == "--style-new-demo")
    {
        let menu = style_menu.clone();
        let s = style_popover.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            menu.popup();
            if requested("--pin-demo") {
                popover_pin(&menu.popover().unwrap()).set_active(true);
                style_click(&s, "style-apply-Title");
            }
            if requested("--style-details-demo") {
                s.show_detail("Heading1");
            }
            if requested("--style-new-demo") {
                s.show_form(None);
            }
        });
    }
    if requested("--preferences-demo") {
        let w = window.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            show_preferences(&w);
        });
    }
    if requested("--search-demo") {
        let s = search_panel.clone();
        capture_ready::after(std::time::Duration::from_millis(1100), move || {
            s.show(true);
            s.query.set_text("the");
            s.replacement.set_text("THE");
            s.refresh();
        });
    }
}
