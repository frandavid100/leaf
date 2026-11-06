use super::*;

pub(super) fn proofing_ui_test(ui: &Rc<Ui>, parent: &ApplicationWindow, app: &Application) {
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "El qqqgtkwordzz examen.\nEl qqqotherzz texto.",
        )),
        "Proofing",
    );
    {
        let mut current = ui.current.borrow_mut();
        wordcraft_gnome::proofing_defaults::apply(
            &mut current.as_mut().unwrap().session,
            "es-ES",
            true,
        )
        .unwrap();
    }
    after_content_change(ui);
    show_text_language(ui, parent);
    let language = format_button(ui, "text-menu").unwrap();
    assert!(find_widget(&language.popover().unwrap(), "language-choice-off").is_some());
    assert!(find_widget(&language.popover().unwrap(), "language-choice-default").is_some());
    assert!(find_widget(&language.popover().unwrap(), "install-dictionaries").is_some());
    if let Some(root) = std::env::var_os("LEAF_DICTIONARY_TEST_DIR") {
        let root = PathBuf::from(root);
        let aff = root.join("it_IT.aff");
        let dic = root.join("it_IT.dic");
        std::fs::write(&aff, "SET UTF-8\n").unwrap();
        std::fs::write(&dic, "2\nprova\nfoglio\n").unwrap();
        wordcraft_gnome::proofing::install_dictionary(&[aff, dic]).unwrap();
        assert!(wordcraft_engine::proof::it::has_dictionary());
        assert!(wordcraft_engine::proof::it::is_correct("foglio"));
        assert!(!wordcraft_engine::proof::it::is_correct("qqqgtkwordzz"));
        eprintln!("DICTIONARY_INSTALL_OK");
    }
    language.popdown();
    let prefs = show_preferences(parent);
    let live = find_widget(&prefs, "preferences-live-proofing")
        .unwrap()
        .downcast::<gtk::Switch>()
        .unwrap();
    live.set_active(false);
    assert!(!wordcraft_gnome::preferences::check_spelling());
    assert!(
        ui.current.borrow().as_ref().unwrap().session.view.proofing,
        "preferences do not disable existing document review"
    );
    prefs.close();
    assert!(find_widget(&ui.navigation_pages, "preferences-live-proofing").is_none());
    let before = ui.current.borrow().as_ref().unwrap().session.sel.clone();
    show_spelling(ui, parent);
    assert_eq!(
        ui.navigation_pages.visible_child_name().as_deref(),
        Some("review")
    );
    let weak = Rc::downgrade(ui);
    let app = app.clone();
    let parent = parent.clone();
    let attempts = Cell::new(0);
    let phase = Cell::new(0);
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        let ui = weak.upgrade().unwrap();
        attempts.set(attempts.get() + 1);
        assert!(attempts.get() < 300, "proofing sidebar did not finish");
        let s = ui.spelling.borrow().as_ref().unwrap().clone();
        if s.source.borrow().is_some() || s.debounce.borrow().is_some() {
            return glib::ControlFlow::Continue;
        }
        match phase.get() {
            0 => {
                assert_eq!(
                    ui.current.borrow().as_ref().unwrap().session.sel,
                    before,
                    "scanning must not navigate"
                );
                assert!(s.issues.borrow().len() >= 2);
                assert!(!s.status.is_visible(), "no introductory text above issues");
                let filter = find_widget(&ui.sidebar_filters, "review-filter").unwrap();
                assert!(
                    filter.is_mapped(),
                    "Review filter remains in the visible header"
                );
                assert!(find_widget(&ui.navigation_pages, "proofing-panel").is_none());
                let rev = ui.current.borrow().as_ref().unwrap().session.rev();
                let dirty = ui.dirty.get();
                let index = s
                    .issues
                    .borrow()
                    .iter()
                    .position(|i| i.text == "qqqgtkwordzz")
                    .unwrap();
                find_widget(&ui.navigation_pages, &format!("proofing-go-{index}"))
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap()
                    .emit_clicked();
                assert_eq!(s.item().unwrap().text, "qqqgtkwordzz");
                assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(), rev);
                assert_eq!(ui.dirty.get(), dirty);
                assert!(!s.status.is_visible());
                {
                    let mut v = ui.current.borrow_mut();
                    assert_eq!(
                        selection::copy_text(&mut v.as_mut().unwrap().session).unwrap(),
                        "qqqgtkwordzz"
                    );
                }
                let menu = find_widget(&ui.navigation_pages, &format!("proofing-menu-{index}"))
                    .unwrap()
                    .downcast::<gtk::MenuButton>()
                    .unwrap();
                assert!(
                    find_widget(&ui.navigation_pages, &format!("proofing-apply-{index}")).is_some()
                );
                assert!(
                    find_widget(&ui.navigation_pages, &format!("proofing-ignore-{index}"))
                        .is_some()
                );
                menu.popup();
                let pop = menu
                    .popover()
                    .unwrap()
                    .downcast::<gtk::PopoverMenu>()
                    .unwrap();
                assert!(
                    context_model_row(&pop, wordcraft_gnome::t!("Add to Dictionary")).is_some()
                );
                for label in [
                    wordcraft_gnome::t!("Cut"),
                    wordcraft_gnome::t!("Copy"),
                    wordcraft_gnome::t!("Text"),
                ] {
                    assert!(context_model_row(&pop, label).is_none());
                }
                assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(), rev);
                pop.popdown();
                eprintln!("PROOFING_ENTRY_MENU_OK");
                let context = show_document_context(&ui, &parent, 120, 180, DocumentContext::Text);
                assert!(
                    context_model_row(&context, wordcraft_gnome::t!("Add to Dictionary")).is_some()
                );
                context.popdown();
                apply_user_edit(&ui, Edit::Insert("profesor".into()));
                // New-document preferences do not disable this document.
                assert!(find_widget(&ui.navigation_pages, "proofing-refresh").is_none());
                // The visible list refreshes automatically after this edit.
                assert!(
                    ui.current
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .session
                        .doc
                        .plain_text(wordcraft_doc::StoryRef::Body)
                        .starts_with("El profesor examen.")
                );
                phase.set(1);
            }
            1 => {
                let index = s
                    .issues
                    .borrow()
                    .iter()
                    .position(|i| i.text == "qqqotherzz")
                    .unwrap();
                find_widget(&ui.navigation_pages, &format!("proofing-go-{index}"))
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap()
                    .emit_clicked();
                let old = s.item().unwrap();
                apply_user_edit(&ui, Edit::Insert("otro".into()));
                assert!(s.issues.borrow().is_empty());
                assert!(
                    s.debounce.borrow().is_some(),
                    "the visible list refreshes independently of new-document preferences"
                );
                {
                    let mut v = ui.current.borrow_mut();
                    let session = &mut v.as_mut().unwrap().session;
                    let doc = session.doc.clone();
                    assert!(proofing::replace(session, &old, "stale").is_err());
                    assert_eq!(session.doc, doc);
                }
                let prefs = show_preferences(&parent);
                let live = find_widget(&prefs, "preferences-live-proofing")
                    .unwrap()
                    .downcast::<gtk::Switch>()
                    .unwrap();
                live.set_active(true);
                prefs.close();
                assert!(wordcraft_gnome::preferences::check_spelling());
                apply_user_edit(&ui, Edit::Insert(" qqqbrandnewzz".into()));
                phase.set(2);
            }
            2 => {
                assert_eq!(s.identity.get(), document_identity(&ui));
                assert!(s.issues.borrow().iter().any(|i| i.text == "qqqbrandnewzz"));
                // Replace the document while a scan is in flight. Its old result must be discarded.
                s.scan();
                let mut session =
                    Session::new(wordcraft_doc::Document::from_text("El qqqfinalzz texto."));
                proofing::language(&mut session, "es-ES", false, true).unwrap();
                opened(&ui, session, "Replacement proofing");
                s.scan();
                phase.set(3);
            }
            _ => {
                assert_eq!(s.identity.get(), document_identity(&ui));
                assert!(s.issues.borrow().iter().any(|i| i.text == "qqqfinalzz"));
                assert!(!s.issues.borrow().iter().any(|i| i.text == "qqqbrandnewzz"));
                if std::env::args().any(|a| a == "--proofing-demo") {
                    ui.sidebar_button.set_active(true);
                    let index = s
                        .issues
                        .borrow()
                        .iter()
                        .position(|i| i.text == "qqqfinalzz")
                        .unwrap();
                    s.list
                        .row_at_index(index as i32)
                        .unwrap()
                        .emit_by_name::<()>("activate", &[]);
                    let weak = Rc::downgrade(&ui);
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(600),
                        move || {
                            let ui = weak.upgrade().unwrap();
                            assert!(
                                ui.sidebar.shows_sidebar(),
                                "viewport refits must preserve the proofing panel"
                            );
                            eprintln!("PROOFING_DEMO_READY");
                        },
                    );
                } else {
                    s.stop();
                    eprintln!("PROOFING_UI_OK sidebar=true preferences=true live=true stale=true");
                    app.quit();
                }
                return glib::ControlFlow::Break;
            }
        }
        glib::ControlFlow::Continue
    });
}
