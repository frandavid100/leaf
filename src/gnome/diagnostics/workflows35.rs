//! Exercise the shipped native controls on native GTK and both installed runtimes.
use super::*;
use serde_json::json;
async fn transfer(ui: &Ui) {
    for _ in 0..250 {
        if ui.clipboard_read.borrow().is_none() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(10)).await;
    }
    panic!("clipboard timed out")
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
pub(super) fn test(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    app: &Application,
    search: &Rc<SearchPanel>,
    style: &Rc<StylePopover>,
    outline: &Rc<OutlinePanel>,
) {
    let (ui, window, app, search, style, outline) = (
        ui.clone(),
        window.clone(),
        app.clone(),
        search.clone(),
        style.clone(),
        outline.clone(),
    );
    glib::spawn_future_local(async move {
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("cat catch CAT Cat")),
            "Workflows35",
        );
        search.show(true);
        search.whole_word.set_active(true);
        search.preserve_case.set_active(true);
        search.query.set_text("cat");
        search.replacement.set_text("dog");
        search.refresh();
        assert_eq!(search.hits.borrow().len(), 3);
        search.replace_all.emit_clicked();
        assert_eq!(text(&ui), "dog catch DOG Dog");
        apply_user_edit(&ui, Edit::Undo);
        assert_eq!(text(&ui), "cat catch CAT Cat");
        search.whole_word.set_active(false);
        search.preserve_case.set_active(false);
        {
            let mut current = ui.current.borrow_mut();
            selection::select_range(
                &mut current.as_mut().unwrap().session,
                &Pos::body(0, 10),
                &Pos::body(0, 13),
            )
            .unwrap();
        }
        search.show(true);
        search.scope.set_selected(4);
        search.query.set_text("cat");
        search.refresh();
        assert_eq!(search.hits.borrow().len(), 1);
        search.replacement.set_text("fox");
        search.replace_all.emit_clicked();
        assert_eq!(text(&ui), "cat catch fox Cat");
        search.scope.set_selected(0);
        search.regex.set_active(true);
        search.query.set_text("(");
        search.refresh();
        assert!(!search.replace_all.is_sensitive());
        search.regex.set_active(false);
        search.toggle.set_active(false);
        style.show_detail("Heading1");
        let next = find_widget(&style.detail, "style-next")
            .unwrap()
            .downcast::<gtk::DropDown>()
            .unwrap();
        next.set_selected(0);
        assert_eq!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .styles
                .get("Heading1")
                .unwrap()
                .next
                .as_deref(),
            Some("Normal")
        );
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::new()),
            "Plain text",
        );
        let clipboard = ui.document_box.clipboard();
        let provider = gdk::ContentProvider::new_union(&[
            gdk::ContentProvider::for_bytes(
                "text/html",
                &glib::Bytes::from_static(b"<b>Plain</b>"),
            ),
            gdk::ContentProvider::for_value(&"Plain".to_value()),
        ]);
        clipboard.set_content(Some(&provider)).unwrap();
        let menu = show_document_context(&ui, &window, 100, 150, DocumentContext::Text);
        context_activate_row(&menu, wordcraft_gnome::t!("Paste Plain Text"));
        transfer(&ui).await;
        assert_eq!(text(&ui), "Plain");
        assert!(
            !ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .caret_props()
                .bold
                .unwrap_or(false)
        );
        let mut s = Session::new(wordcraft_doc::Document::from_text("bold target"));
        selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 4)).unwrap();
        s.run("format.bold", &json!({"value":true})).unwrap();
        opened(&ui, s, "Painter");
        format_painter::start(&ui, false);
        assert!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .painter
                .is_some()
        );
        {
            let mut current = ui.current.borrow_mut();
            selection::select_range(
                &mut current.as_mut().unwrap().session,
                &Pos::body(0, 5),
                &Pos::body(0, 11),
            )
            .unwrap();
        }
        format_painter::apply(&ui);
        assert!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .painter
                .is_none()
        );
        assert!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .para_at(&Pos::body(0, 5))
                .unwrap()
                .props_of_char(5)
                .bold
                .unwrap_or(false)
        );
        format_painter::start(&ui, true);
        format_painter::apply(&ui);
        assert!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .painter
                .is_some()
        );
        assert!(format_painter::cancel(&ui));
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::new()),
            "External image",
        );
        let bytes = wordcraft_engine::cmd::insert::base64_decode(include_str!(
            "../../../tests/fixtures/picture.b64"
        ))
        .unwrap();
        let texture = gdk::Texture::from_bytes(&glib::Bytes::from(&bytes)).unwrap();
        clipboard.set_texture(&texture);
        context_paste(&ui);
        transfer(&ui).await;
        assert_eq!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .media
                .len(),
            1
        );
        apply_user_edit(&ui, Edit::Undo);
        assert!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .media
                .is_empty()
        );
        let mut s = Session::new(wordcraft_doc::Document::from_text("A\nbody A\nB\nbody B"));
        for i in [0, 2] {
            selection::set_caret(&mut s, &Pos::body(i, 0), false).unwrap();
            s.run("para.style", &json!({"style":"Heading1"})).unwrap();
        }
        opened(&ui, s, "Outline");
        outline.show();
        outline.refresh();
        let row = find_widget(&outline.list, "outline-heading-0").unwrap();
        find_widget(&row, "section-down")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert_eq!(text(&ui), "B\nbody B\nA\nbody A");
        apply_user_edit(&ui, Edit::Undo);
        assert!(text(&ui).starts_with("A"));
        let before = document_identity(&ui);
        document_fonts::show(&ui, &window);
        assert_eq!(document_identity(&ui), before);
        for window in app.windows() {
            if window.title().as_deref() == Some(wordcraft_gnome::t!("Document Fonts")) {
                window.close();
            }
        }
        // An actual template chooser action opens a new, dirty tab and keeps the source file.
        let folder = wordcraft_gnome::preferences::config_root().join("workflow35-templates");
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("Personal.docx");
        let mut source = Session::new(wordcraft_doc::Document::from_text("Personal template"));
        wordcraft_gnome::saving::save(&mut source, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        let host = ui.host.upgrade().unwrap();
        let count = host.tabs.n_pages();
        templates::show(&host);
        let dialog = app
            .windows()
            .into_iter()
            .find(|w| w.title().as_deref() == Some(wordcraft_gnome::t!("New from Template")))
            .unwrap();
        find_widget(&dialog, "personal-template")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert_eq!(host.tabs.n_pages(), count + 1);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(text(&selected_ui(&host).unwrap()), "Personal template");
        eprintln!(
            "WORKFLOWS35_UI_OK search=true selection=true regex_error=true next_style=true plain=true painter=true image=true outline=true fonts=true template=true"
        );
        app.quit();
    });
}
