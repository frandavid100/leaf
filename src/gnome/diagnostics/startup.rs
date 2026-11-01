use super::*;

pub(super) fn install_menu_demos(text_menu: &gtk::MenuButton, zoom_menu: &gtk::MenuButton) {
    if std::env::args().any(|a| a == "--text-menu-demo") {
        let menu = text_menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || menu.popup());
    }
    if std::env::args().any(|a| a == "--zoom-menu-demo") {
        let menu = zoom_menu.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(1100), move || menu.popup());
    }
}

pub(super) fn install_content_demos(ui: &Rc<Ui>, formatting_demo: bool, editing_demo: bool, selection_demo: bool) {
    if formatting_demo {
        let ui = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(950), move || {
            {
                let mut borrowed = ui.current.borrow_mut();
                if let Some(doc) = borrowed.as_mut() {
                    let _ = doc.session.run(
                        "select.text",
                        &serde_json::json!({
                            "text": "Why share a studio?"
                        }),
                    );
                }
            }
            apply_user_format(&ui, Format::Heading1);
            apply_user_format(&ui, Format::AlignCenter);
        });
    }
    if editing_demo {
        let ui = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(900), move || {
            {
                let mut value = ui.current.borrow_mut();
                if let Some(doc) = value.as_mut() {
                    let _ = doc.session.run(
                        "select.text",
                        &serde_json::json!({
                            "text": "The Open Studio Handbook"
                        }),
                    );
                }
            }
            apply_user_edit(
                &ui,
                Edit::Insert("Documento editable — fase 3B".to_string()),
            );
        });
    }

    if selection_demo {
        let ui = ui.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(650), move || {
            {
                let mut value = ui.current.borrow_mut();
                if let Some(doc) = value.as_mut() {
                    let _ = doc.session.run(
                        "select.text",
                        &serde_json::json!({
                            "text": "The Open Studio Handbook"
                        }),
                    );
                }
            }
            refresh_selection(&ui);
        });
    }
}
