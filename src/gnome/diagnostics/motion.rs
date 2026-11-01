use super::*;

pub(super) fn install_motion_check(ui: &Rc<Ui>, app: &Application) {
    if std::env::args().any(|a| a == "--motion-ui-test") {
        assert!(
            ui.sidebar.shows_sidebar(),
            "sidebar must be open before mapping"
        );
        let changes = Rc::new(Cell::new(0));
        let count = changes.clone();
        ui.sidebar
            .connect_show_sidebar_notify(move |_| count.set(count.get() + 1));
        let test_ui = ui.clone();
        let test_app = app.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1500), move || {
            assert_eq!(changes.get(), 0, "startup must not toggle the sidebar");
            assert!(test_ui.sidebar.shows_sidebar());
            let settings = test_ui.document_box.settings();
            let old = settings.property::<bool>("gtk-enable-animations");
            settings.set_property("gtk-enable-animations", true);
            {
                let mut state = test_ui.current.borrow_mut();
                state
                    .as_mut()
                    .unwrap()
                    .session
                    .run("caret.docStart", &serde_json::json!({}))
                    .unwrap();
            }
            test_ui.caret_motion.borrow_mut().retarget(None, 0, false);
            refresh_selection(&test_ui);
            let revision = test_ui.current.borrow().as_ref().unwrap().session.rev();
            selected_command(&test_ui, "right", false);
            assert!(test_ui.caret_motion.borrow().is_moving());
            assert_ne!(
                test_ui.caret_motion.borrow().position(),
                test_ui.selection_data.borrow().caret.as_ref()
            );
            let next = test_ui.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
                assert!(
                    !next.caret_motion.borrow().is_moving(),
                    "frame clock must finish the motion"
                );
                assert_eq!(
                    next.caret_motion.borrow().position(),
                    next.selection_data.borrow().caret.as_ref()
                );
                settings.set_property("gtk-enable-animations", false);
                selected_command(&next, "right", false);
                assert!(!next.caret_motion.borrow().is_moving());
                assert_eq!(
                    next.caret_motion.borrow().position(),
                    next.selection_data.borrow().caret.as_ref()
                );
                assert_eq!(
                    revision,
                    next.current.borrow().as_ref().unwrap().session.rev()
                );
                settings.set_property("gtk-enable-animations", old);
                eprintln!("MOTION_UI_OK");
                test_app.quit();
            });
        });
    }
}
