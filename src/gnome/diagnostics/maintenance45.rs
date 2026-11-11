//! Real native controls plus document/history invariants for alpha.45.
use super::*;
use serde_json::json;
async fn settle() {
    glib::timeout_future(std::time::Duration::from_millis(500)).await;
}
fn state(
    ui: &Ui,
) -> (
    wordcraft_doc::Document,
    wordcraft_engine::Selection,
    u64,
    usize,
    bool,
) {
    let value = ui.current.borrow();
    let s = &value.as_ref().unwrap().session;
    (
        s.doc.clone(),
        s.sel.clone(),
        s.rev(),
        s.undo_depth(),
        ui.dirty.get(),
    )
}
fn same_reading(ui: &Ui, expected: viewer_layout::ReadingAnchor) {
    let anchor = capture_reading_anchor(ui).unwrap();
    assert_eq!(anchor.page, expected.page, "reading page changed");
    assert!(
        (anchor.y - expected.y).abs() < 0.03,
        "within-page reading anchor changed {anchor:?} vs {expected:?}"
    );
}
fn document_end_visible(ui: &Ui) {
    let caret = ui
        .selection_data
        .borrow()
        .caret
        .clone()
        .expect("text caret");
    let value = ui.current.borrow();
    let d = value.as_ref().unwrap();
    assert_eq!(
        caret.page,
        d.sizes.len() - 1,
        "navigation must reach the document end"
    );
    let frame = d.page_frames[caret.page]
        .compute_bounds(&ui.scroll)
        .unwrap();
    let y = frame.y() as f64 + caret.y * d.page_zoom(caret.page) as f64;
    assert!(
        y >= 0. && y <= ui.scroll.height() as f64,
        "newer navigation must reveal its caret: y={y}, viewport={}",
        ui.scroll.height()
    );
}
pub(super) fn zoom(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_default_size(1050, 780);
        let text = (0..60)
            .map(|i| format!("Paragraph {i}: {}", "Reading anchor body text. ".repeat(35)))
            .collect::<Vec<_>>()
            .join("\n");
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text(&text)),
            "Zoom 278",
        );
        settle().await;
        assert!(ui.current.borrow().as_ref().unwrap().sizes.len() > 8);
        let before = state(&ui);
        scroll_to_page(&ui, 4);
        settle().await;
        ui.scroll
            .vadjustment()
            .set_value(ui.scroll.vadjustment().value() + 160.);
        let anchor = capture_reading_anchor(&ui).unwrap();
        assert!(anchor.page >= 4);
        for zoom in [1.5, 0.67, 1.25] {
            zoom_to(&ui, zoom);
            settle().await;
            same_reading(&ui, anchor);
        }
        fit_width(&ui);
        settle().await;
        same_reading(&ui, anchor);
        for width in [780, 1450, 950] {
            window.set_default_size(width, 780);
            settle().await;
            settle().await;
            same_reading(&ui, anchor);
        }
        zoom_to(&ui, 0.5);
        zoom_to(&ui, 1.75);
        settle().await;
        same_reading(&ui, anchor);
        assert_eq!(
            before,
            state(&ui),
            "zoom/resizing must not change the caret, document or history"
        );
        zoom_to(&ui, 1.);
        cancel_reading_restore(&ui);
        scroll_to_page(&ui, 2);
        settle().await;
        assert_eq!(
            viewport_page(&ui, ui.current.borrow().as_ref().unwrap()),
            2,
            "newer scroll lost"
        );
        zoom_to(&ui, 1.5);
        selected_command(&ui, "docEnd", false);
        settle().await;
        document_end_visible(&ui);
        eprintln!("ZOOM278_UI_OK");
        app.quit();
    });
}
pub(super) fn fonts(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        use wordcraft_gnome::font_install as fonts;
        let focused = std::env::var("LEAF279_STYLES").ok();
        if focused.is_none() {
            assert!(
                !fonts::available("Poppins"),
                "font test requires a clean Poppins-free host"
            );
        }
        let styles = focused
            .as_deref()
            .map(|s| s.split(',').collect::<Vec<_>>())
            .unwrap_or_else(|| fonts::STYLES.to_vec());
        let ack = PathBuf::from(
            std::env::var("LEAF279_ACK_DIR").expect("font viewer test driver required"),
        );
        std::fs::create_dir_all(&ack).unwrap();
        let path = ack.join("font279.docx");
        let mut s = Session::new(wordcraft_doc::Document::from_text(
            "Regular Poppins body\nBold Poppins body\nItalic Poppins body\nBold italic Poppins body",
        ));
        selection::select_all(&mut s).unwrap();
        s.run("format.font", &json!({"name":"Poppins"})).unwrap();
        for (p, bold, italic) in [(1, true, false), (2, false, true), (3, true, true)] {
            let end = s.doc.para_at(&Pos::body(p, 0)).unwrap().text.len();
            selection::select_range(&mut s, &Pos::body(p, 0), &Pos::body(p, end)).unwrap();
            s.run(
                "format.set",
                &json!({"props":{"bold":bold,"italic":italic}}),
            )
            .unwrap();
        }
        wordcraft_gnome::saving::save(&mut s, &path).unwrap();
        opened(&ui, open_document(&path).unwrap(), "Font 279");
        settle().await;
        let before = state(&ui);
        let alert = window
            .visible_dialog()
            .unwrap()
            .downcast::<adw::AlertDialog>()
            .expect("automatic missing-font notice");
        fn response_button(w: &gtk::Widget) -> Option<gtk::Button> {
            if let Some(b) = w.downcast_ref::<gtk::Button>() {
                if b.label().as_deref() == Some(wordcraft_gnome::t!("Review Fonts")) {
                    return Some(b.clone());
                }
            }
            let mut child = w.first_child();
            while let Some(w) = child {
                if let Some(b) = response_button(&w) {
                    return Some(b);
                }
                child = w.next_sibling();
            }
            None
        }
        response_button(alert.upcast_ref())
            .expect("Review Fonts response")
            .emit_clicked();
        settle().await;
        let dialog = ui.font_install_dialog.borrow().as_ref().unwrap().clone();
        let image_before = {
            let mut c = ui.current.borrow_mut();
            wordcraft_gnome::render_page(&mut c.as_mut().unwrap().session, 0, 1.)
                .unwrap()
                .rgba
        };
        for style in &styles {
            let button = find_widget(&dialog, &format!("font-download-{style}"))
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap();
            button.emit_clicked();
            for _ in 0..120 {
                if button.is_sensitive() {
                    break;
                }
                settle().await;
            }
            assert_eq!(
                button.label().as_deref(),
                Some(wordcraft_gnome::t!("Open Font File")),
                "download failed: {}",
                find_widget(&dialog, "font-install-status")
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap()
                    .text()
            );
            find_widget(&dialog, "font-install-check")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            settle().await;
            assert_eq!(before, state(&ui));
            assert!(
                fonts::missing(&mut ui.current.borrow_mut().as_mut().unwrap().session)
                    .iter()
                    .any(|f| f.font == "Poppins"),
                "opening a viewer must not claim installation"
            );
            std::fs::write(ack.join(format!("ready-{style}")), b"viewer requested").unwrap();
            for _ in 0..300 {
                if ack.join(format!("installed-{style}")).exists() {
                    break;
                }
                settle().await;
            }
            assert!(
                ack.join(format!("installed-{style}")).exists(),
                "system font viewer did not finish installing {style}"
            );
            find_widget(&dialog, "font-install-check")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            settle().await;
            assert_eq!(
                before,
                state(&ui),
                "installation changed authored content/selection/history/dirty state"
            );
            assert!(fonts::available("Poppins"));
        }
        assert!(
            fonts::missing(&mut ui.current.borrow_mut().as_mut().unwrap().session)
                .iter()
                .all(|f| f.font != "Poppins")
        );
        assert!(
            !wordcraft_fonts::FontDb::global()
                .user_fonts()
                .iter()
                .any(|f| f.families().iter().any(|n| n == "Poppins")),
            "font was installed privately instead of by the system viewer"
        );
        let image_after = {
            let mut c = ui.current.borrow_mut();
            wordcraft_gnome::render_page(&mut c.as_mut().unwrap().session, 0, 1.)
                .unwrap()
                .rgba
        };
        assert_ne!(
            image_before, image_after,
            "new font did not affect actual rendering"
        );
        dialog.close();
        settle().await;
        assert!(window.visible_dialog().is_none());
        eprintln!("FONT279_SYSTEM_INSTALL_OK styles={}", styles.join(","));
        app.quit();
    });
}

pub(super) fn picture_delete(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_default_size(1050, 780);
        for (wrap, adjacent) in [
            ("behindText", false),
            ("square", false),
            ("behindText", true),
            ("square", true),
        ] {
            let text = (0..60)
                .map(|i| {
                    format!(
                        "Paragraph {i}: {}",
                        "Floating picture body text. ".repeat(35)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let mut s = Session::new(wordcraft_doc::Document::from_text(&text));
            let data = wordcraft_engine::cmd::insert::base64_decode(include_str!(
                "../../../tests/fixtures/picture.b64"
            ))
            .unwrap();
            pictures::insert_data(&mut s, &data).unwrap();
            s.run("arrange.wrap", &json!({"wrap":wrap})).unwrap();
            if adjacent {
                selection::set_caret(
                    &mut s,
                    &Pos::body(0, wordcraft_doc::para::OBJ.len_utf8()),
                    false,
                )
                .unwrap();
                pictures::insert_data(&mut s, &data).unwrap();
                s.run("arrange.wrap", &json!({"wrap":wrap})).unwrap();
            }
            let before = s.doc.clone();
            let depth = s.undo_depth();
            opened(&ui, s, "Floating picture deletion");
            settle().await;
            // Selected drawing's anchor is at the beginning; the user is reading elsewhere.
            scroll_to_page(&ui, 4);
            settle().await;
            let anchor = capture_reading_anchor(&ui).unwrap();
            ui.document_box.grab_focus();
            apply_user_edit(&ui, Edit::Delete);
            settle().await;
            same_reading(&ui, anchor);
            assert_eq!(
                pictures::selected(&ui.current.borrow().as_ref().unwrap().session).is_some(),
                adjacent
            );
            assert_eq!(
                ui.current.borrow().as_ref().unwrap().session.undo_depth(),
                depth + 1
            );
            apply_user_edit(&ui, Edit::Undo);
            settle().await;
            same_reading(&ui, anchor);
            assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
            apply_user_edit(&ui, Edit::Redo);
            settle().await;
            same_reading(&ui, anchor);
            apply_user_edit(&ui, Edit::Undo);
            settle().await;
            let picture = ui.picture.borrow().as_ref().unwrap().clone();
            picture.execute(pictures::delete);
            settle().await;
            same_reading(&ui, anchor);
            apply_user_edit(&ui, Edit::Undo);
            settle().await;
            apply_user_edit(
                &ui,
                Edit::Insert("Typing replaces the selected object".into()),
            );
            settle().await;
            assert_eq!(
                capture_reading_anchor(&ui).unwrap().page,
                0,
                "typing must reveal its caret instead of retaining an unrelated reading position"
            );
            selected_command(&ui, "docEnd", false);
            settle().await;
            document_end_visible(&ui);
        }
        eprintln!("PICTURE280_UI_OK");
        app.quit();
    });
}

pub(super) fn catalog(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        use wordcraft_gnome::{
            preferences,
            toolbar::{self, Placement},
        };
        window.set_default_size(1200, 850);
        for group in ["paragraph", "elements"] {
            let mut order = toolbar::tools(group)
                .into_iter()
                .rev()
                .map(|t| (t.id.to_string(), Placement::Pinned))
                .collect::<Vec<_>>();
            for mode in [Placement::More, Placement::Adaptive, Placement::Pinned] {
                for (_, p) in &mut order {
                    *p = mode;
                }
                preferences::set_toolbar_placements(group, &order).unwrap();
                assert_eq!(preferences::toolbar_placements(group), order);
                ui.quick_tools.borrow().as_ref().unwrap().layout();
            }
        }
        async fn place(ui: &Rc<Ui>, group: &str, id: &str) {
            for name in ["paragraph", "elements"] {
                let order = wordcraft_gnome::preferences::toolbar_placements(name)
                    .into_iter()
                    .map(|(key, _)| {
                        let mode = if name == group && key == id {
                            Placement::Pinned
                        } else {
                            Placement::More
                        };
                        (key, mode)
                    })
                    .collect::<Vec<_>>();
                wordcraft_gnome::preferences::set_toolbar_placements(name, &order).unwrap();
            }
            ui.quick_tools.borrow().as_ref().unwrap().layout();
            settle().await;
            assert!(
                find_widget(&ui.format_toolbar, &format!("format-quick-{id}"))
                    .unwrap()
                    .is_mapped(),
                "toolbar launcher must be visible: {group}/{id}"
            );
        }
        for id in [
            "table-select-cell",
            "table-select-row",
            "table-select-column",
            "table-select-all",
            "table-insert-row-above",
            "table-insert-row-below",
            "table-insert-column-left",
            "table-insert-column-right",
            "table-delete-row",
            "table-delete-column",
            "table-delete-all",
            "table-split-cells",
            "table-split-table",
        ] {
            let mut s = Session::new(wordcraft_doc::Document::new());
            s.run("insert.table", &json!({"rows":3,"cols":3})).unwrap();
            if id == "table-split-table" {
                for _ in 0..3 {
                    wordcraft_gnome::tables::navigate(&mut s, false, false).unwrap();
                }
            }
            opened(&ui, s, id);
            settle().await;
            assert!(
                tool_actions::source_widget(&ui, id).is_some(),
                "missing live source for {id}"
            );
            place(&ui, "elements", id).await;
            let b = find_widget(&ui.format_toolbar, &format!("format-quick-{id}"))
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap();
            assert!(b.get_visible() && b.is_sensitive(), "not usable: {id}");
            let before = state(&ui);
            b.emit_clicked();
            settle().await;
            if id == "table-split-cells" {
                let dialog = window.visible_dialog().expect("Split Cells options");
                assert!(dialog.is_mapped());
                find_widget(&dialog, "table-split-apply")
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap()
                    .emit_clicked();
                settle().await;
                assert!(
                    state(&ui).2 > before.2,
                    "split cells must use the real command"
                );
                dialog.close();
                settle().await;
            } else if id.starts_with("table-select-") {
                assert_eq!(state(&ui).0, before.0);
                assert_eq!(state(&ui).3, before.3);
            } else {
                assert!(state(&ui).2 > before.2, "inert table launcher {id}");
                assert_eq!(state(&ui).3, before.3 + 1);
                apply_user_edit(&ui, Edit::Undo);
                settle().await;
                assert_eq!(state(&ui).0, before.0);
            }
        }
        for id in [
            "field-page",
            "field-pages",
            "field-author",
            "field-title",
            "field-filename",
            "field-date",
            "field-time",
            "break-textWrapping",
            "break-page",
            "break-column",
            "break-nextPage",
            "break-continuous",
            "break-evenPage",
            "break-oddPage",
        ] {
            opened(
                &ui,
                Session::new(wordcraft_doc::Document::from_text("Insertion target")),
                id,
            );
            settle().await;
            place(&ui, "elements", id).await;
            let before = state(&ui);
            find_widget(&ui.format_toolbar, &format!("format-quick-{id}"))
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            settle().await;
            assert!(state(&ui).2 > before.2, "inert insertion {id}");
            apply_user_edit(&ui, Edit::Undo);
            settle().await;
            assert_eq!(state(&ui).0, before.0);
        }
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("List paragraph")),
            "List tools",
        );
        settle().await;
        for (id, page) in [
            ("bullets", "bullets"),
            ("numbering", "numbering"),
            ("multilevel", "multilevel"),
        ] {
            place(&ui, "paragraph", id).await;
            let before = state(&ui);
            find_widget(&ui.format_toolbar, &format!("format-quick-{id}"))
                .unwrap()
                .downcast::<gtk::MenuButton>()
                .unwrap()
                .popup();
            settle().await;
            let p = ui.paragraph.borrow().as_ref().unwrap().clone();
            assert_eq!(p.stack.visible_child_name().as_deref(), Some("detail"));
            assert!(
                find_widget(&p.detail, &format!("paragraph-more-{page}"))
                    .unwrap()
                    .is_mapped()
            );
            assert_eq!(before, state(&ui), "opening a gallery is read-only");
            p.popover.upgrade().unwrap().popdown();
        }
        format_execute(&ui, "para.numbering", json!({"kind":"numbered"})).unwrap();
        settle().await;
        for (id, name, value) in [
            ("list-level", "paragraph-list-level", 2.),
            ("list-start", "paragraph-list-start", 7.),
        ] {
            place(&ui, "paragraph", id).await;
            find_widget(&ui.format_toolbar, &format!("format-quick-{id}"))
                .unwrap()
                .downcast::<gtk::MenuButton>()
                .unwrap()
                .popup();
            settle().await;
            let p = ui.paragraph.borrow().as_ref().unwrap().clone();
            let spin = find_widget(&p.detail, name)
                .unwrap()
                .downcast::<gtk::SpinButton>()
                .unwrap();
            assert!(spin.is_mapped());
            let before = state(&ui);
            spin.set_value(value);
            settle().await;
            assert!(state(&ui).2 > before.2, "inert list option {id}");
            p.popover.upgrade().unwrap().popdown();
        }
        place(&ui, "paragraph", "list-remove").await;
        let b = find_widget(&ui.format_toolbar, "format-quick-list-remove")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        b.emit_clicked();
        settle().await;
        assert!(!tool_actions::enabled(&ui, "paragraph", "list-remove"));
        assert!(tool_actions::enabled(&ui, "elements", "insert-new-table"));
        place(&ui, "elements", "insert-new-table").await;
        find_widget(&ui.format_toolbar, "format-quick-insert-new-table")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        settle().await;
        assert!(
            find_widget(&window.visible_dialog().unwrap(), "insert-table-grid")
                .unwrap()
                .is_mapped()
        );
        window.visible_dialog().unwrap().close();
        settle().await;
        eprintln!("CATALOG259_UI_OK");
        app.quit();
    });
}
