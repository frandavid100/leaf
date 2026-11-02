use super::*;

/// Existing controls borrowed only while opt-in diagnostic callbacks are installed.
pub(super) struct DiagnosticControls<'a> {
    pub(super) app_menu: &'a gtk::MenuButton,
    pub(super) style_menu: &'a gtk::MenuButton,
    pub(super) align_menu: &'a gtk::MenuButton,
    pub(super) text_menu: &'a gtk::MenuButton,
    pub(super) table_menu: &'a gtk::MenuButton,
    pub(super) menu_button: &'a gtk::MenuButton,
    pub(super) picture_menu: &'a gtk::MenuButton,
    pub(super) style_popover: &'a Rc<StylePopover>,
    pub(super) paragraph_popover: &'a Rc<ParagraphPopover>,
    pub(super) text_popover: &'a gtk::Popover,
    pub(super) picture_panel: &'a Rc<PicturePopover>,
    pub(super) outline_panel: &'a Rc<OutlinePanel>,
    pub(super) titles_toggle: &'a gtk::ToggleButton,
    pub(super) search_panel: &'a Rc<SearchPanel>,
}

pub(super) fn install_diagnostics(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    host: &Rc<Workspace>,
    app: &Application,
    controls: DiagnosticControls<'_>,
) {
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
    if std::env::args().any(|a| a == "--heading-top-test") {
        let (u, s, a) = (ui.clone(), outline_panel.clone(), app.clone());
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            heading_top_ui_test(&u, &s, &a)
        });
    }
    if std::env::args().any(|a| a == "--uri-launch-test") {
        let u = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1000), move || {
            launch_document_link(&u, "https://example.org/leaf-launch-test")
        });
    }
    if std::env::args().any(|a| a == "--interaction-pointer-test") {
        let (u, w, a, m) = (ui.clone(), window.clone(), app.clone(), text_menu.clone());
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            interaction_pointer_test(&u, &w, &a, &m)
        });
    }
    if host.tabs.n_pages() == 1 && std::env::args().any(|a| a == "--issue-sweep-ui-test") {
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
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            issue_sweep_ui_test(&u, &w, &h, &a, &m, &main, &pm, &tm)
        });
    }
    if std::env::args().any(|a| a == "--comment-pointer-test") {
        let (u, w, a) = (ui.clone(), window.clone(), app.clone());
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            comment_pointer_ui_test(&u, &w, &a)
        });
    }
    if std::env::args().any(|a| a == "--save-current-file-test") {
        let (u, w, a) = (ui.clone(), window.clone(), app.clone());
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            save_current_file_ui_test(&u, &w, &a)
        });
    }
    if std::env::args().any(|a| {
        matches!(
            a.as_str(),
            "--annotations-ui-test" | "--annotations-demo" | "--bookmarks-demo"
        )
    }) {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1000), move || {
            annotations_ui_test(&u, &w, &a);
        });
    }
    if std::env::args().any(|a| a == "--localization-ui-test") {
        let u = ui.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(700), move || {
            localization_ui_test(&u, &a)
        });
    }
    if std::env::args().any(|a| a == "--adaptive-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1400), move || {
            adaptive_ui_test(&u, &w, &app)
        });
    }
    if let Some(label) = std::env::args().find_map(|a| match a.as_str() {
        "--context-style-demo" => Some(wordcraft_gnome::t!("Style")),
        "--context-paragraph-demo" => Some(wordcraft_gnome::t!("Paragraph")),
        "--context-text-demo" => Some(wordcraft_gnome::t!("Text")),
        _ => None,
    }) {
        let u = ui.clone();
        let w = window.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1400), move || {
            let p = show_document_context(&u, &w, 140, 160, DocumentContext::Text);
            context_activate_row(&p, label);
        });
    }

    if std::env::args().any(|a| a == "--context-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            context_ui_test(&u, &w, &a)
        });
    }
    if std::env::args().any(|a| a == "--table-keyboard-test") {
        let u = ui.clone();
        let app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            opened(
                &u,
                Session::new(wordcraft_doc::Document::new()),
                "Prueba de tablas",
            );
            table_command(&u, "insert.table", serde_json::json!({"rows":1,"cols":2}));
            eprintln!("TABLE_KEYBOARD_READY");
            glib::timeout_add_local_once(std::time::Duration::from_secs(6), move || {
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
    if std::env::args().any(|a| {
        a == "--table-menu-demo" || a == "--table-properties-demo" || a == "--table-insert-demo"
    }) {
        let u = ui.clone();
        let w = window.clone();
        let m = table_menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            if std::env::args().any(|a| a == "--table-insert-demo") {
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
            if std::env::args().any(|a| a == "--table-properties-demo") {
                show_table_properties(&u, &w);
            } else {
                m.popup();
            }
        });
    }
    if std::env::args().any(|a| a == "--split-states-demo") {
        let m = align_menu.clone();
        let s = paragraph_popover.clone();
        window.set_title(Some("Leaf Split Button Regression"));
        let events = gtk::EventControllerLegacy::new();
        events.set_propagation_phase(gtk::PropagationPhase::Capture);
        events.set_propagation_limit(gtk::PropagationLimit::None);
        events.connect_event(|_, event| {
            if matches!(
                event.event_type(),
                gdk::EventType::ButtonPress | gdk::EventType::ButtonRelease
            ) {
                eprintln!(
                    "SPLIT_RAW_EVENT {:?} {:?}",
                    event.event_type(),
                    event.position()
                );
            }
            glib::Propagation::Proceed
        });
        window.add_controller(events);
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            m.popup();
            glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
                use gdk::prelude::PopupExt;
                let pop = s.popover.upgrade().unwrap();
                let body = find_widget(&s.main, "paragraph-indent")
                    .unwrap()
                    .compute_bounds(&pop)
                    .unwrap();
                let arrow_widget = find_widget(&s.main, "paragraph-more-indents").unwrap();
                let arrow = arrow_widget.compute_bounds(&pop).unwrap();
                let events = gtk::EventControllerLegacy::new();
                events.set_propagation_phase(gtk::PropagationPhase::Capture);
                events.set_propagation_limit(gtk::PropagationLimit::None);
                events.connect_event(|_, event| {
                    if matches!(
                        event.event_type(),
                        gdk::EventType::ButtonPress | gdk::EventType::ButtonRelease
                    ) {
                        eprintln!("SPLIT_BUTTON_EVENT {:?}", event.event_type());
                    }
                    glib::Propagation::Proceed
                });
                arrow_widget.add_controller(events);
                eprintln!(
                    "SPLIT_TARGET sensitive={} mapped={} target={} picked={:?}",
                    arrow_widget.is_sensitive(),
                    arrow_widget.is_mapped(),
                    arrow_widget.can_target(),
                    pop.pick(
                        arrow.x() as f64 + arrow.width() as f64 / 2.0,
                        arrow.y() as f64 + arrow.height() as f64 / 2.0,
                        gtk::PickFlags::DEFAULT
                    )
                    .map(|w| (w.type_().name(), w.widget_name()))
                );
                let surface = pop.surface().unwrap().downcast::<gdk::Popup>().unwrap();
                let (tx, ty) = pop.surface_transform();
                let x = surface.position_x() as f64 - tx;
                let y = surface.position_y() as f64 - ty;
                let trace = gtk::GestureClick::new();
                trace.set_propagation_phase(gtk::PropagationPhase::Capture);
                let weak = pop.downgrade();
                trace.connect_pressed(move |_, _, x, y| {
                    if let Some(pop) = weak.upgrade() {
                        eprintln!(
                            "SPLIT_CLICK {x} {y} target={:?}",
                            pop.pick(x, y, gtk::PickFlags::DEFAULT)
                                .map(|w| w.widget_name())
                        );
                    }
                });
                pop.add_controller(trace);
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
        if std::env::args().any(|a| a == flag) {
            let u = ui.clone();
            let w = window.clone();
            let a = app.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
                if proof {
                    proofing_ui_test(&u, &w, &a);
                } else {
                    page_options_ui_test(&u, &w, &a);
                }
            });
        }
    }
    if std::env::args().any(|a| a == "--headers-ui-test" || a == "--headers-pointer-test") {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
            header_footer_ui_test(
                &u,
                &w,
                &a,
                std::env::args().any(|x| x == "--headers-pointer-test"),
            )
        });
    }
    if std::env::args().any(|a| {
        a == "--picture-demo" || a == "--document-properties-demo" || a == "--document-tools-test"
    }) {
        let u = ui.clone();
        let p = picture_panel.clone();
        let m = picture_menu.clone();
        let w = window.clone();
        let app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            document_tools_fixture(&u);
            if std::env::args().any(|a| a == "--document-tools-test") {
                document_tools_ui_test(&u, &p, &m, &w, &app);
            } else if std::env::args().any(|a| a == "--document-properties-demo") {
                show_document_properties(&u, &w);
            } else {
                m.popup();
            }
        });
    }
    if std::env::args().any(|a| a == "--columns-demo") {
        let u = ui.clone();
        let m = align_menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            m.popup();
            if let Some(s) = u.paragraph.borrow().as_ref() {
                s.page("columns");
            }
        });
    }
    if std::env::args().any(|a| a == "--print-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
            print_ui_test(&u, &w, &a)
        });
    }
    if std::env::args().any(|a| a == "--host-cache-ui-test") {
        let u = ui.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
            host_cache_ui_test(&u, &a)
        });
    }
    if std::env::args().any(|a| a == "--performance-ui-test") {
        let u = ui.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
            performance_ui_test(&u, &a)
        });
    }
    if std::env::args().any(|a| a == "--a11y-ui-test") {
        let u = ui.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
            accessible_document_ui_test(&u, &a)
        });
    }
    if std::env::args().any(|a| a == "--keyboard-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let m = app_menu.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
            keyboard_ui_test(&u, &w, &m, &a)
        });
    }
    if std::env::args().any(|a| a == "--hidpi-ui-test") {
        let u = ui.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1300), move || {
            hidpi_ui_test(&u, &a)
        });
    }
    if std::env::args().any(|a| a == "--columns-ui-test") {
        let u = ui.clone();
        let w = window.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            columns_ui_test(&u, &w, &a)
        });
    }
    if std::env::args().any(|a| a == "--paragraph-demo" || a == "--paragraph-spacing-demo") {
        let m = align_menu.clone();
        let s = paragraph_popover.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            m.popup();
            if std::env::args().any(|a| a == "--paragraph-spacing-demo") {
                s.page("spacing");
            }
        });
    }
    if std::env::args().any(|a| a == "--paragraph-ui-test") {
        let s = paragraph_popover.clone();
        let m = align_menu.clone();
        let app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            paragraph_ui_test(&s, &m, &app)
        });
    }
    if std::env::args().any(|a| a == "--search-ui-test") {
        let s = search_panel.clone();
        let ui = ui.clone();
        let app = app.clone();
        let host = host.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            search_ui_test(&ui, &s, &app, &host)
        });
    }
    if std::env::args().any(|a| a == "--view-pointer-test") {
        let u = ui.clone();
        let m = style_menu.clone();
        let w = window.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1300), move || {
            view_pointer_test(&u, &m, &w, &a)
        });
    }
    if std::env::args().any(|a| a == "--spreads-demo") {
        window.set_default_size(1800, 950);
        let u = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1300), move || {
            if let Some(d) = u.current.borrow_mut().as_mut() {
                d.fit_to_width = false;
            }
            zoom_to(&u, 1.0);
        });
    }
    if std::env::args().any(|a| a == "--outline-demo") {
        let b = titles_toggle.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            b.set_active(true)
        });
    }
    if std::env::args().any(|a| a == "--pin-ui-test") {
        let u = ui.clone();
        let sm = style_menu.clone();
        let sp = style_popover.clone();
        let pm = align_menu.clone();
        let pp = paragraph_popover.clone();
        let tm = text_menu.clone();
        let tp = text_popover.clone();
        let tb = table_menu.clone();
        let app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            pin_ui_test(&u, &sm, &sp, &pm, &pp, &tm, &tp, &tb, &app)
        });
    }
    if std::env::args().any(|a| a == "--chrome-ui-test") {
        let u = ui.clone();
        let m = menu_button.clone();
        let app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            chrome_ui_test(&u, &m, &app)
        });
    }
    if std::env::args().any(|a| a == "--document-menu-demo") {
        let m = menu_button.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || m.popup());
    }
    if std::env::args().any(|a| a == "--main-menu-demo") {
        let m = app_menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || m.popup());
    }
    if std::env::args().any(|a| a == "--overview-demo") && host.tabs.n_pages() == 1 {
        let h = host.clone();
        let app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            blank_tab(&app, &h);
            h.overview.set_open(true);
        });
    }
    if std::env::args().any(|a| a == "--nav-ui-test") && host.tabs.n_pages() == 1 {
        let u = ui.clone();
        let h = host.clone();
        let app = app.clone();
        let s = outline_panel.clone();
        let b = titles_toggle.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || {
            navigation_ui_test(&u, &s, &b, &h, &app)
        });
    }
    if std::env::args().any(|a| a == "--advanced-text-demo") {
        let w = window.clone();
        let u = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            show_text_options(&u, &w)
        });
    }
    if std::env::args().any(|a| a == "--advanced-paragraph-demo") {
        let w = window.clone();
        let u = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            show_paragraph_options(&u, &w)
        });
    }
    if std::env::args().any(|a| a == "--styles-ui-test") {
        let s = style_popover.clone();
        let m = style_menu.clone();
        let a = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            style_ui_test(&s, &m, &a)
        });
    }
    if std::env::args()
        .any(|a| a == "--styles-demo" || a == "--style-details-demo" || a == "--style-new-demo")
    {
        let menu = style_menu.clone();
        let s = style_popover.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            menu.popup();
            if std::env::args().any(|a| a == "--pin-demo") {
                popover_pin(&menu.popover().unwrap()).set_active(true);
                style_click(&s, "style-apply-Title");
            }
            if std::env::args().any(|a| a == "--style-details-demo") {
                s.show_detail("Heading1");
            }
            if std::env::args().any(|a| a == "--style-new-demo") {
                s.show_form(None);
            }
        });
    }
    if std::env::args().any(|a| a == "--preferences-demo") {
        let w = window.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            show_preferences(&w);
        });
    }
    if std::env::args().any(|a| a == "--search-demo") {
        let s = search_panel.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || {
            s.show(true);
            s.query.set_text("the");
            s.replacement.set_text("THE");
            s.refresh();
        });
    }
}
