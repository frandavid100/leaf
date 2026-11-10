//! Exercise the actual widgets and engine state, including refresh and clipboard ownership.
use super::*;
use serde_json::json;
async fn settle() {
    glib::timeout_future(std::time::Duration::from_millis(450)).await;
}
fn click(root: &impl IsA<gtk::Widget>, name: &str) {
    let w = find_widget(root, name).unwrap_or_else(|| panic!("missing {name}"));
    assert!(w.is_sensitive(), "insensitive {name}");
    if let Some(b) = w.downcast_ref::<gtk::Button>() {
        b.emit_clicked();
    } else if let Some(m) = w.downcast_ref::<gtk::MenuButton>() {
        m.popup();
    } else {
        assert!(w.activate());
    }
}
fn state(ui: &Ui) -> (wordcraft_doc::Document, usize) {
    let c = ui.current.borrow();
    let s = &c.as_ref().unwrap().session;
    (s.doc.clone(), s.undo_depth())
}
async fn capture(window: &ApplicationWindow, widget: &impl IsA<gtk::Widget>, name: &str) {
    let widget = widget.as_ref();
    let p = gtk::WidgetPaintable::new(Some(widget));
    settle().await;
    let snapshot = gtk::Snapshot::new();
    let background = if adw::StyleManager::default().is_dark() {
        gdk::RGBA::new(0.18, 0.18, 0.18, 1.)
    } else {
        gdk::RGBA::WHITE
    };
    snapshot.append_color(
        &background,
        &gtk::graphene::Rect::new(0., 0., widget.width() as f32, widget.height() as f32),
    );
    p.snapshot(&snapshot, widget.width() as f64, widget.height() as f64);
    let node = snapshot.to_node().unwrap();
    let texture = window.renderer().unwrap().render_texture(&node, None);
    let dir = std::env::var("LEAF43_CAPTURE_DIR").unwrap_or_else(|_| ".".into());
    texture.save_to_png(format!("{dir}/{name}.png")).unwrap();
}
pub(super) fn test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_default_size(1280, 850);
        settle().await;
        let gallery = ui.template_gallery.borrow().clone();
        if let Some(first) = gallery {
            for _ in 0..3 {
                format_execute(&ui, "format.bold", json!({"value":true})).unwrap();
                settle().await;
                assert!(Rc::ptr_eq(
                    &first,
                    ui.template_gallery
                        .borrow()
                        .as_ref()
                        .expect("Start remains present after typing-format changes")
                ));
                format_execute(&ui, "format.bold", json!({"value":false})).unwrap();
            }
        }
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text(
                "Alpha Beta\nGamma Delta",
            )),
            "Alpha43 controls",
        );
        {
            let mut c = ui.current.borrow_mut();
            selection::select_range(
                &mut c.as_mut().unwrap().session,
                &Pos::body(0, 0),
                &Pos::body(0, 10),
            )
            .unwrap();
        }
        refresh_selection(&ui);
        settle().await;
        // Old placements retain identities; newly exposed tools start in More.
        for group in ["text", "paragraph", "elements"] {
            let mut order = wordcraft_gnome::preferences::toolbar_placements(group);
            for (_, mode) in &mut order {
                *mode = wordcraft_gnome::toolbar::Placement::More;
            }
            wordcraft_gnome::preferences::set_toolbar_placements(group, &order).unwrap();
        }
        ui.quick_tools.borrow().as_ref().unwrap().layout();
        let text = ui.text_tools.borrow().as_ref().unwrap().clone();
        let pop = text.popover.upgrade().unwrap();
        format_bar::open_group(&ui, "text", Some("underline"));
        settle().await;
        text.header.pin.set_active(true);
        for (name, expected) in [
            ("underline-choice-double", "double"),
            ("underline-choice-wave", "wave"),
            ("underline-choice-none", "none"),
        ] {
            click(&pop, name);
            settle().await;
            assert_eq!(selection_character(&ui)["underline"], expected);
            assert!(pop.is_visible());
            assert_eq!(
                text.header.label.text().as_str(),
                wordcraft_gnome::t!("Underline")
            );
        }
        for page in [
            "strike",
            "superscript",
            "ink",
            "highlight",
            "language",
            "font",
            "character-spacing",
            "enclose",
            "phonetic",
            "size",
        ] {
            text.page(page);
            settle().await;
            let before = state(&ui);
            assert!(pop.is_visible(), "{page} remains pinned");
            if page == "language" {
                let search = find_widget(&pop, "language-search")
                    .unwrap()
                    .downcast::<gtk::SearchEntry>()
                    .unwrap();
                search.set_text("Deutsch");
                settle().await;
                assert_eq!(state(&ui), before, "search never formats text");
                search.set_text("");
            }
            if page == "font" {
                ui.font_input.set_text("Sans");
                settle().await;
                assert_eq!(state(&ui), before, "font filtering never formats text");
            }
            capture(
                &window,
                &pop.child().unwrap(),
                &format!("alpha43-text-{page}"),
            )
            .await;
        }
        text.page("superscript");
        settle().await;
        click(&pop, "script-choice-subscript");
        settle().await;
        assert_eq!(selection_character(&ui)["vertAlign"], "subscript");
        click(&pop, "script-choice-baseline");
        settle().await;
        assert_eq!(selection_character(&ui)["vertAlign"], "baseline");
        text.page("ink");
        settle().await;
        test_choose_color(&pop, "text-ink-chooser", "1C71D8");
        settle().await;
        assert_eq!(
            selection_character(&ui)["color"],
            json!({"Rgb":[28,113,216]})
        );
        text.header.pin.set_active(false);
        pop.popdown();
        settle().await;
        let paragraph = ui.paragraph.borrow().as_ref().unwrap().clone();
        let para_pop = paragraph.popover.upgrade().unwrap();
        format_bar::open_group(&ui, "paragraph", Some("alignment"));
        settle().await;
        paragraph.header.pin.set_active(true);
        click(&para_pop, "paragraph-align-center");
        settle().await;
        assert_eq!(selection_paragraph(&ui)["align"], "center");
        {
            let mut c = ui.current.borrow_mut();
            selection::set_caret(&mut c.as_mut().unwrap().session, &Pos::body(1, 2), false)
                .unwrap();
        }
        refresh_selection(&ui);
        settle().await;
        let before = state(&ui);
        paragraph.page("alignment");
        settle().await;
        assert_eq!(state(&ui), before);
        paragraph.repeat_group("alignment");
        settle().await;
        assert_eq!(selection_paragraph(&ui)["align"], "center");
        for page in [
            "direction",
            "indents",
            "spacing",
            "tabs",
            "sort",
            "columns",
            "options",
            "borders",
            "shading",
        ] {
            paragraph.page(page);
            settle().await;
            assert!(para_pop.is_visible());
            capture(
                &window,
                &para_pop.child().unwrap(),
                &format!("alpha43-paragraph-{page}"),
            )
            .await;
        }
        paragraph.header.pin.set_active(false);
        para_pop.popdown();
        settle().await;
        {
            let mut c = ui.current.borrow_mut();
            let s = &mut c.as_mut().unwrap().session;
            selection::select_range(s, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
            s.run("format.bold", &json!({"value":true})).unwrap();
        }
        refresh_selection(&ui);
        assert!(context_copy(&ui));
        assert!(clipboard::can_paste_format(&ui));
        {
            let mut c = ui.current.borrow_mut();
            selection::select_range(
                &mut c.as_mut().unwrap().session,
                &Pos::body(1, 0),
                &Pos::body(1, 5),
            )
            .unwrap();
        }
        refresh_selection(&ui);
        let text_before = ui
            .current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .doc
            .plain_text(wordcraft_doc::StoryRef::Body);
        clipboard::paste_format(&ui);
        assert_eq!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .plain_text(wordcraft_doc::StoryRef::Body),
            text_before
        );
        assert_eq!(selection_character(&ui)["bold"], true);
        ui.document_box.clipboard().set_text("External copy");
        settle().await;
        assert!(
            !clipboard::can_paste_format(&ui),
            "provider loss invalidates appearance payload"
        );
        eprintln!("ALPHA43_UI_OK");
        app.quit();
    });
}
