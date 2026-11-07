//! Native external clipboard formats, history and stale-target regression.
use super::*;

async fn wait_for_transfer(ui: &Ui) {
    for _ in 0..200 {
        if ui.clipboard_read.borrow().is_none() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(10)).await;
    }
    panic!("clipboard transfer did not complete");
}

pub(super) fn clipboard_ui_test(ui: &Rc<Ui>, app: &Application) {
    let ui = ui.clone();
    let app = app.clone();
    glib::spawn_future_local(async move {
        let clipboard = ui.document_box.clipboard();
        for (mime, bytes, expected, bold) in [
            (
                "text/rtf",
                b"{\\rtf1\\ansi\\b Rich}".as_slice(),
                "Rich",
                true,
            ),
            (
                "text/html",
                b"<p><b>Rich</b> text</p>".as_slice(),
                "Rich text",
                true,
            ),
            ("text/plain", b"Plain".as_slice(), "Plain", false),
        ] {
            opened(
                &ui,
                Session::new(wordcraft_doc::Document::new()),
                "Clipboard regression",
            );
            let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
            let depth = ui.current.borrow().as_ref().unwrap().session.undo_depth();
            let provider = if mime == "text/plain" {
                gdk::ContentProvider::for_value(&"Plain".to_value())
            } else {
                gdk::ContentProvider::for_bytes(mime, &glib::Bytes::from_owned(bytes.to_vec()))
            };
            clipboard.set_content(Some(&provider)).unwrap();
            let window = ui.host.upgrade().unwrap().window.clone();
            let menu = show_document_context(&ui, &window, 100, 150, DocumentContext::Text);
            context_activate_row(&menu, wordcraft_gnome::t!("Paste"));
            wait_for_transfer(&ui).await;
            {
                let current = ui.current.borrow();
                let session = &current.as_ref().unwrap().session;
                assert_eq!(
                    session.doc.plain_text(wordcraft_doc::StoryRef::Body).trim(),
                    expected,
                    "{mime}"
                );
                assert_eq!(session.undo_depth(), depth + 1, "one paste, one Undo");
                let paragraph = session.doc.body[0].as_para().unwrap();
                assert_eq!(
                    session
                        .doc
                        .styles
                        .resolve_char(paragraph.props.style.as_deref(), paragraph.props_of_char(0))
                        .bold,
                    bold,
                    "{mime}"
                );
            }
            apply_user_edit(&ui, Edit::Undo);
            assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, original);
        }
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::new()),
            "Stale clipboard",
        );
        clipboard
            .set_content(Some(&gdk::ContentProvider::for_bytes(
                "text/html",
                &glib::Bytes::from_static(b"<b>stale</b>"),
            )))
            .unwrap();
        context_paste(&ui);
        apply_user_edit(&ui, Edit::Insert("typed".into()));
        wait_for_transfer(&ui).await;
        assert_eq!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .plain_text(wordcraft_doc::StoryRef::Body),
            "typed"
        );
        context_paste(&ui);
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("Replacement")),
            "Replacement",
        );
        assert!(ui.clipboard_read.borrow().is_none());
        glib::timeout_future(std::time::Duration::from_millis(30)).await;
        assert_eq!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .plain_text(wordcraft_doc::StoryRef::Body),
            "Replacement"
        );
        eprintln!(
            "CLIPBOARD_FORMATS_UI_OK rtf=true html=true text=true undo=true stale=true cancellation=true"
        );
        app.quit();
    });
}
