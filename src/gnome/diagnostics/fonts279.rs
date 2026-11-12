//! A local font installed from one tab must refresh a different tab's provider.
use super::*;
use serde_json::json;
use wordcraft_gnome::font_install as fonts;

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
    let current = ui.current.borrow();
    let session = &current.as_ref().unwrap().session;
    (
        session.doc.clone(),
        session.sel.clone(),
        session.rev(),
        session.undo_depth(),
        ui.dirty.get(),
    )
}

fn button(dialog: &adw::Dialog, name: &str) -> gtk::Button {
    find_widget(dialog, name)
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
}

pub(super) fn test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_default_size(1100, 850);
        let family = "Leaf279 QA";
        assert!(
            !fonts::available(family),
            "requires a clean QA font profile"
        );
        let ack = PathBuf::from(std::env::var("LEAF279_ACK_DIR").unwrap());
        let mode = std::env::var("LEAF279_MODE").unwrap_or_else(|_| "local".into());
        assert!(mode == "local" || mode == "dismiss");
        let source = ack.join("background.docx");
        let text = (0..35)
            .map(|n| format!("Paragraph {n}: {}", "Background font reading. ".repeat(20)))
            .collect::<Vec<_>>()
            .join("\n");
        let mut sample = Session::new(wordcraft_doc::Document::from_text(&text));
        selection::select_all(&mut sample).unwrap();
        sample.run("format.font", &json!({"name":family})).unwrap();
        selection::set_caret(&mut sample, &Pos::body(0, 0), false).unwrap();
        wordcraft_gnome::saving::save(&mut sample, &source).unwrap();
        let source_bytes = std::fs::read(&source).unwrap();
        opened(
            &ui,
            open_document(&source).unwrap(),
            "Background font document",
        );
        settle().await;
        assert!(ui.current.borrow().as_ref().unwrap().sizes.len() > 3);
        scroll_to_page(&ui, 2);
        settle().await;
        ui.scroll
            .vadjustment()
            .set_value(ui.scroll.vadjustment().value() + 75.);
        settle().await;
        let anchor = capture_reading_anchor(&ui).unwrap();
        assert!(anchor.page >= 2);
        let before = state(&ui);
        let image_before = wordcraft_gnome::render_page(
            &mut ui.current.borrow_mut().as_mut().unwrap().session,
            0,
            1.,
        )
        .unwrap()
        .rgba;
        let host = ui.host.upgrade().unwrap();
        blank_tab(&app, &host);
        let foreground = selected_ui(&host).unwrap();
        assert!(!Rc::ptr_eq(&foreground, &ui));
        opened(
            &foreground,
            Session::new(wordcraft_doc::Document::from_text(
                "This tab does not use the local font.",
            )),
            "Font installation",
        );
        settle().await;
        assert!(
            wordcraft_gnome::document_fonts::inventory(
                &foreground.current.borrow().as_ref().unwrap().session
            )
            .iter()
            .all(|f| f.name != family)
        );
        let foreground_before = state(&foreground);
        font_install::show(&foreground);
        settle().await;
        let dialog = foreground
            .font_install_dialog
            .borrow()
            .as_ref()
            .unwrap()
            .clone();
        button(&dialog, "font-install-local").emit_clicked();
        std::fs::write(ack.join("ready-local"), b"choose real local file").unwrap();
        for _ in 0..600 {
            if ack.join("viewer-done").exists() {
                break;
            }
            settle().await;
        }
        assert!(
            ack.join("viewer-done").exists(),
            "local chooser/viewer driver did not finish"
        );
        button(&dialog, "font-install-check").emit_clicked();
        settle().await;
        assert_eq!(before, state(&ui), "background authored state changed");
        assert_eq!(
            foreground_before,
            state(&foreground),
            "foreground authored state changed"
        );
        let image_after = wordcraft_gnome::render_page(
            &mut ui.current.borrow_mut().as_mut().unwrap().session,
            0,
            1.,
        )
        .unwrap()
        .rgba;
        if mode == "local" {
            assert!(
                fonts::available(family),
                "installed family used only by the background tab was not refreshed"
            );
            assert!(
                fonts::missing(&mut ui.current.borrow_mut().as_mut().unwrap().session)
                    .iter()
                    .all(|f| f.font != family)
            );
            assert!(
                image_before != image_after,
                "actual rendering did not refresh"
            );
            assert!(
                !wordcraft_fonts::FontDb::global()
                    .user_fonts()
                    .iter()
                    .any(|f| f.families().iter().any(|n| n == family)),
                "private font installation is forbidden"
            );
            for extension in ["docx", "odt"] {
                let mut saved =
                    Session::new(ui.current.borrow().as_ref().unwrap().session.doc.clone());
                let path = ack.join(format!("roundtrip.{extension}"));
                wordcraft_gnome::saving::save(&mut saved, &path).unwrap();
                let reopened = open_document(&path).unwrap();
                assert!(
                    wordcraft_gnome::document_fonts::inventory(&reopened)
                        .iter()
                        .any(|f| f.name == family && !f.uses.is_empty())
                );
            }
        } else {
            assert!(
                !fonts::available(family),
                "opening the viewer is not installation"
            );
            assert!(
                image_before == image_after,
                "dismissing the viewer changed rendering"
            );
        }
        dialog.close();
        settle().await;
        let page = ui.tab_page.borrow().as_ref().unwrap().clone();
        host.tabs.set_selected_page(&page);
        settle().await;
        settle().await;
        let restored = capture_reading_anchor(&ui).unwrap();
        assert_eq!(
            restored.page, anchor.page,
            "background reading page changed"
        );
        assert!(
            (restored.y - anchor.y).abs() < 0.03,
            "background reading offset changed"
        );
        if mode == "local" {
            ui.document_box.grab_focus();
            apply_user_edit(&ui, Edit::Insert("New text with installed font ".into()));
            settle().await;
            assert!(
                fonts::missing(&mut ui.current.borrow_mut().as_mut().unwrap().session)
                    .iter()
                    .all(|f| f.font != family)
            );
            apply_user_edit(&ui, Edit::Undo);
            settle().await;
            assert_eq!(before.0, state(&ui).0);
        }
        assert_eq!(
            source_bytes,
            std::fs::read(&source).unwrap(),
            "original file was modified"
        );
        eprintln!("FONT279_ACCEPTANCE_OK mode={mode}");
        app.quit();
    });
}
