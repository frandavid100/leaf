use super::*;

pub(in crate::gnome) fn columns_ui_test(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    app: &Application,
) {
    let (ui, parent, app) = (ui.clone(), parent.clone(), app.clone());
    glib::spawn_future_local(async move {
        async fn settle() {
            glib::timeout_future(std::time::Duration::from_millis(500)).await;
        }
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text(
                "Antes\nTexto para distribuir entre columnas\nDespués",
            )),
            "Prueba de columnas",
        );
        ui.current
            .borrow_mut()
            .as_mut()
            .unwrap()
            .session
            .run(
                "select.range",
                &serde_json::json!({"anchor":Pos::body(1,0),"focus":Pos::body(2,0)}),
            )
            .unwrap();
        refresh_selection(&ui);
        let (doc, depth, rev, text) = {
            let d = ui.current.borrow();
            let s = &d.as_ref().unwrap().session;
            (s.doc.clone(), s.undo_depth(), s.rev(), s.selected_text())
        };
        let dialog = show_columns(&ui, &parent);
        settle().await;
        assert_eq!(
            dialog.controls.scopes[dialog.controls.scope.selected() as usize],
            ColumnScope::Selection
        );
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(), rev);
        dialog.controls.count.set_value(2.0);
        settle().await;
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.undo_depth(),
            depth + 1
        );
        dialog.controls.gap.set_value(7.0);
        settle().await;
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.undo_depth(),
            depth + 2
        );
        dialog.controls.separator.set_active(true);
        settle().await;
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.undo_depth(),
            depth + 3
        );
        assert!(
            dialog.window.is_visible(),
            "confirmed field edits keep precise options open"
        );
        {
            let d = ui.current.borrow();
            let s = &d.as_ref().unwrap().session;
            assert_eq!(s.selected_text(), text);
            assert_eq!(
                s.doc
                    .sections()
                    .iter()
                    .map(|(_, p)| p.columns.count)
                    .collect::<Vec<_>>(),
                vec![1, 2, 1]
            );
        }
        let rev = ui.current.borrow().as_ref().unwrap().session.rev();
        dialog.controls.count.set_text("2.");
        settle().await;
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.rev(),
            rev,
            "incomplete numeric entry must not edit"
        );
        dialog.window.close();
        settle().await;
        for _ in 0..3 {
            apply_user_edit(&ui, Edit::Undo);
            settle().await;
        }
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, doc);
        for _ in 0..3 {
            apply_user_edit(&ui, Edit::Redo);
            settle().await;
        }
        {
            let mut current = ui.current.borrow_mut();
            let s = &mut current.as_mut().unwrap().session;
            let sections = s.doc.sections();
            let start = sections[0].0 + 1;
            let end = sections[1].0 + 1;
            s.run(
                "select.range",
                &serde_json::json!({"anchor":Pos::body(start,0),"focus":Pos::body(end,0)}),
            )
            .unwrap();
        }
        refresh_selection(&ui);
        let dialog = show_columns(&ui, &parent);
        settle().await;
        assert_eq!(
            dialog.controls.count.value_as_int(),
            2,
            "reopen shows actual selected section, excluding selection end boundary"
        );
        let rev = ui.current.borrow().as_ref().unwrap().session.rev();
        let selected_before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
        dialog.controls.scope.set_selected(0); // Current Section is the section at the focus end.
        settle().await;
        assert_eq!(dialog.controls.count.value_as_int(), 1);
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(), rev);
        dialog.controls.count.set_value(2.0); // Same value as the prior scope: must not be deduplicated.
        settle().await;
        assert!(
            ui.current.borrow().as_ref().unwrap().session.rev() > rev,
            "a value previously used in another scope must edit the current target"
        );
        dialog.window.close();
        settle().await;
        apply_user_edit(&ui, Edit::Undo);
        settle().await;
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.doc,
            selected_before
        );
        let dialog = show_columns(&ui, &parent);
        settle().await;
        let rev = ui.current.borrow().as_ref().unwrap().session.rev();
        dialog.controls.count.set_text("3"); // Incomplete user transaction must not survive a scope refresh.
        dialog.controls.scope.set_selected(1); // Whole Document: mixed section settings.
        settle().await;
        assert!(dialog.controls.mixed.get());
        assert!(
            !dialog.controls.count.has_css_class("error"),
            "programmatic mixed values must cancel older numeric input"
        );
        assert!(dialog.controls.separator.is_inconsistent());
        assert!(!dialog.controls.count.is_sensitive());
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.rev(),
            rev,
            "changing scope is read-only"
        );
        dialog.controls.scope.set_selected(2);
        settle().await;
        assert!(!dialog.controls.mixed.get());
        assert_eq!(dialog.controls.count.value_as_int(), 2);
        let preset = find_widget(&dialog.window, "columns-preset-left")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        preset.emit_clicked();
        settle().await;
        assert!(
            !dialog.window.is_mapped(),
            "preset did not commit: {}",
            find_widget(&dialog.window, "columns-error")
                .unwrap()
                .downcast::<gtk::Label>()
                .unwrap()
                .text()
        );
        let dialog = show_columns(&ui, &parent);
        settle().await;
        assert_eq!(
            dialog.controls.mode.selected(),
            1,
            "imported explicit widths match the narrow-left preset"
        );
        assert!(
            find_widget(&dialog.window, "columns-preset-left")
                .unwrap()
                .has_css_class("current-style")
        );
        dialog.window.close();
        settle().await;
        let paragraph = ui.paragraph.borrow().as_ref().unwrap().clone();
        let rev = ui.current.borrow().as_ref().unwrap().session.rev();
        paragraph.page("columns");
        assert!(find_widget(&paragraph.detail, "columns-count").is_some());
        paragraph.home();
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(), rev);
        for menu in ui.format_buttons.borrow().iter() {
            if let Some(name) = menu.icon_name() {
                assert!(
                    gtk::IconTheme::for_display(&gtk::prelude::WidgetExt::display(&parent))
                        .has_icon(&name)
                );
            }
        }
        eprintln!("COLUMNS_UI_OK");
        app.quit();
    });
}
