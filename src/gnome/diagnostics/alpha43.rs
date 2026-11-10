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
    let dir = std::env::var("LEAF43_CAPTURE_DIR")
        .ok()
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).unwrap();
    texture
        .save_to_png(dir.join(format!("{name}.png")))
        .unwrap();
}
pub(super) fn test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_default_size(1280, 850);
        let theme = gtk::IconTheme::for_display(&ui.document_box.display());
        for group in ["text", "paragraph", "elements"] {
            for tool in wordcraft_gnome::toolbar::tools(group) {
                assert!(
                    theme.has_icon(tool.icon),
                    "missing toolbar icon {} for {}",
                    tool.icon,
                    tool.id
                );
            }
        }
        settle().await;
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::new()),
            wordcraft_gnome::t!("New Document"),
        );
        templates::offer(&ui);
        settle().await;
        let gallery = ui.template_gallery.borrow().clone();
        assert!(
            gallery.is_some(),
            "Start gallery must exist for this reproduction"
        );
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
        text.reapply_language();
        format_execute(&ui, "leaf.proofText", json!({"choice":"es-ES"})).unwrap();
        text.reapply_language();
        settle().await;
        assert_eq!(
            wordcraft_gnome::proofing_defaults::choice(
                &ui.current.borrow().as_ref().unwrap().session
            ),
            "es-ES"
        );
        let palette = text.palette.borrow().as_ref().unwrap().clone();
        palette.reapply_color(true);
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
        paragraph.page("shading");
        settle().await;
        test_choose_color(&para_pop, "paragraph-shading-chooser", "1C71D8");
        settle().await;
        assert_eq!(
            selection_paragraph(&ui)["shading"],
            json!([28, 113, 216]),
            "shared property chooser survives its builder returning"
        );
        paragraph.page("borders");
        settle().await;
        click(&para_pop, "paragraph-border-bottom");
        settle().await;
        paragraph.page("border-color");
        settle().await;
        test_choose_color(&para_pop, "paragraph-border-chooser", "1C71D8");
        settle().await;
        assert_eq!(
            selection_paragraph(&ui)["borders"]["bottom"]["color"],
            json!([28, 113, 216])
        );
        paragraph.page("bullets");
        settle().await;
        click(&para_pop, "paragraph-list-•");
        settle().await;
        let list_before = state(&ui);
        click(&para_pop, "paragraph-list-•");
        settle().await;
        assert_eq!(
            state(&ui),
            list_before,
            "choosing the same list design does not toggle it off"
        );
        click(&para_pop, "paragraph-list-none");
        settle().await;
        paragraph.page("numbering");
        settle().await;
        click(&para_pop, "paragraph-list-numbered");
        settle().await;
        let start = find_widget(&para_pop, "paragraph-list-start")
            .unwrap()
            .downcast::<gtk::SpinButton>()
            .unwrap();
        start.set_value(7.);
        click(&para_pop, "paragraph-list-set-start");
        settle().await;
        {
            let current = ui.current.borrow();
            let session = &current.as_ref().unwrap().session;
            let p = session.doc.para_at(&session.sel.focus).unwrap();
            let n = p.props.numbering.unwrap();
            assert!(
                session
                    .doc
                    .numbering
                    .num(n.num)
                    .unwrap()
                    .start_overrides
                    .iter()
                    .any(|(level, start)| *level == n.level && *start == 7)
            );
        }
        click(&para_pop, "paragraph-list-custom");
        settle().await;
        let custom = window.visible_dialog().unwrap();
        assert_eq!(
            find_widget(&custom, "list-custom-template")
                .unwrap()
                .downcast::<gtk::Entry>()
                .unwrap()
                .text(),
            "%1."
        );
        assert!(
            alpha42_checks::response(custom.upcast_ref(), wordcraft_gnome::t!("Close")) || {
                custom.close();
                true
            }
        );
        settle().await;
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
        document_tools_fixture(&ui);
        refresh_selection(&ui);
        let elements = ui.elements.borrow().as_ref().unwrap().clone();
        format_bar::open_group(&ui, "elements", Some("image-size"));
        settle().await;
        elements.header.pin.set_active(true);
        let picture = ui.picture.borrow().as_ref().unwrap().clone();
        assert!(
            picture.width.is_mapped(),
            "independent size tool leases the real controls"
        );
        picture.width.set_value(4.0);
        settle().await;
        let (_, w, h) = pictures::selected(&ui.current.borrow().as_ref().unwrap().session).unwrap();
        assert!((w as f64 - 4. * 72. / 2.54).abs() < 0.02 && (w / h - 2.).abs() < 0.01);
        assert!(elements.pop.is_visible());
        elements.prepare("image-wrap");
        settle().await;
        click(&elements.pop, "picture-placement-left");
        settle().await;
        assert_eq!(
            pictures::layout(&ui.current.borrow().as_ref().unwrap().session)
                .unwrap()
                .wrap,
            wordcraft_doc::para::Wrap::Square
        );
        assert!(
            find_widget(&elements.pop, "picture-placement-left")
                .unwrap()
                .is_mapped()
        );
        elements.prepare("image");
        settle().await;
        assert!(
            picture.width.is_mapped(),
            "general Image page restores controls leased by individual tools"
        );
        elements.header.pin.set_active(false);
        elements.pop.popdown();
        settle().await;
        image_options::alt_text(&ui);
        settle().await;
        let dialog = window
            .visible_dialog()
            .expect("native alternative-text editor");
        let input = find_widget(&dialog, "image-alt-text")
            .unwrap()
            .downcast::<gtk::TextView>()
            .unwrap();
        input.buffer().set_text("A blue rectangle");
        find_widget(&dialog, "image-decorative")
            .unwrap()
            .downcast::<gtk::CheckButton>()
            .unwrap()
            .set_active(true);
        click(&dialog, "image-alt-save");
        settle().await;
        let image_pos = pictures::selected(&ui.current.borrow().as_ref().unwrap().session)
            .unwrap()
            .0;
        {
            let current = ui.current.borrow();
            let object = current
                .as_ref()
                .unwrap()
                .session
                .doc
                .para_at(&image_pos)
                .unwrap()
                .object_at(image_pos.off)
                .unwrap();
            assert_eq!(object.alt_text(), "A blue rectangle");
            assert!(object.is_decorative());
        }
        image_options::rotate(&ui, "right");
        settle().await;
        assert!(
            (pictures::layout(&ui.current.borrow().as_ref().unwrap().session)
                .unwrap()
                .rot
                - 90.)
                .abs()
                < 0.01
        );
        image_options::crop(&ui);
        settle().await;
        let dialog = window.visible_dialog().expect("native crop editor");
        let before = state(&ui);
        find_widget(&dialog, "image-crop-left")
            .unwrap()
            .downcast::<gtk::SpinButton>()
            .unwrap()
            .set_value(10.);
        settle().await;
        assert_eq!(state(&ui), before, "crop draft is read-only");
        assert!(alpha42_checks::response(
            dialog.upcast_ref(),
            wordcraft_gnome::t!("Cancel")
        ));
        settle().await;
        assert_eq!(state(&ui), before);
        image_options::crop(&ui);
        settle().await;
        let dialog = window.visible_dialog().unwrap();
        find_widget(&dialog, "image-crop-left")
            .unwrap()
            .downcast::<gtk::SpinButton>()
            .unwrap()
            .set_value(10.);
        find_widget(&dialog, "image-crop-right")
            .unwrap()
            .downcast::<gtk::SpinButton>()
            .unwrap()
            .set_value(15.);
        click(&dialog, "image-crop-apply");
        settle().await;
        {
            let current = ui.current.borrow();
            let s = &current.as_ref().unwrap().session;
            let p = pictures::selected(s).unwrap().0;
            let wordcraft_doc::para::InlineObject::Image { crop, .. } =
                s.doc.para_at(&p).unwrap().object_at(p.off).unwrap()
            else {
                panic!("image")
            };
            assert!((crop[0] - 0.1).abs() < 0.001 && (crop[2] - 0.15).abs() < 0.001);
            assert_eq!(s.undo_depth(), before.1 + 1);
        }
        let mut table_session = Session::new(wordcraft_doc::Document::new());
        table_session
            .run("insert.table", &json!({"rows":2,"cols":2}))
            .unwrap();
        table_session
            .run(
                "table.properties",
                &json!({"rowHeight":22.,"rowHeightRule":"exact"}),
            )
            .unwrap();
        table_session.run("table.selectTable", &json!({})).unwrap();
        opened(&ui, table_session, "Table property controls");
        refresh_selection(&ui);
        settle().await;
        let before = state(&ui);
        let properties = show_table_properties(&ui, &window);
        settle().await;
        assert_eq!(
            state(&ui),
            before,
            "opening current table properties is read-only"
        );
        let column = find_widget(&properties, "tool-control-table.columnWidth-width")
            .unwrap()
            .downcast::<gtk::SpinButton>()
            .unwrap();
        assert!(column.is_mapped());
        column.set_value(2.);
        settle().await;
        let snapshot = table_properties::state(&ui).unwrap();
        assert!((snapshot["columnWidth"].as_f64().unwrap() - 2. * 72. / 2.54).abs() < 0.01);
        let height = find_widget(&properties, "tool-control-table.properties-rowHeight")
            .unwrap()
            .downcast::<gtk::SpinButton>()
            .unwrap();
        assert!(
            !height.is_sensitive(),
            "mixed rules require an explicit choice"
        );
        let rule = find_widget(&properties, "tool-control-table.properties-rowHeightRule")
            .unwrap()
            .downcast::<adw::ComboRow>()
            .unwrap();
        let before_rule = state(&ui);
        rule.set_selected(3);
        settle().await;
        assert_eq!(
            state(&ui),
            before_rule,
            "choosing a rule for automatic rows awaits a height"
        );
        assert!(height.is_sensitive());
        height.set_value(1.);
        settle().await;
        assert_eq!(
            table_properties::state(&ui).unwrap()["row"]["heightRule"],
            "exact"
        );
        let alignment = find_widget(&properties, "tool-control-table.properties-valign")
            .unwrap()
            .downcast::<adw::ComboRow>()
            .unwrap();
        let page = alignment
            .ancestor(adw::PreferencesPage::static_type())
            .and_downcast::<adw::PreferencesPage>()
            .unwrap();
        properties.set_visible_page(&page);
        settle().await;
        assert!(alignment.is_mapped());
        alignment.set_selected(2);
        settle().await;
        assert_eq!(
            table_properties::state(&ui).unwrap()["cell"]["valign"],
            "center"
        );
        properties.close();
        settle().await;
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("AAA\nBBB\nCCC")),
            "Cumulative Repeat",
        );
        {
            let mut c = ui.current.borrow_mut();
            selection::select_range(
                &mut c.as_mut().unwrap().session,
                &Pos::body(0, 0),
                &Pos::body(0, 3),
            )
            .unwrap();
        }
        format_execute(&ui, "format.bold", json!({"value":true})).unwrap();
        format_execute(&ui, "format.italic", json!({"value":true})).unwrap();
        {
            let mut c = ui.current.borrow_mut();
            selection::select_range(
                &mut c.as_mut().unwrap().session,
                &Pos::body(1, 0),
                &Pos::body(1, 3),
            )
            .unwrap();
        }
        refresh_selection(&ui);
        ui.redo_button.emit_clicked();
        settle().await;
        assert_eq!(selection_character(&ui)["italic"], true);
        assert_ne!(
            selection_character(&ui)["bold"],
            true,
            "Repeat body replays only Italic"
        );
        apply_user_edit(&ui, Edit::Undo);
        apply_user_edit(&ui, Edit::Redo);
        {
            let mut c = ui.current.borrow_mut();
            selection::select_range(
                &mut c.as_mut().unwrap().session,
                &Pos::body(2, 0),
                &Pos::body(2, 3),
            )
            .unwrap();
        }
        refresh_selection(&ui);
        let more = find_widget(&window, "redo-history")
            .unwrap()
            .downcast::<gtk::MenuButton>()
            .unwrap();
        let before = state(&ui);
        more.popup();
        settle().await;
        let pop = more
            .popover()
            .unwrap()
            .downcast::<gtk::PopoverMenu>()
            .unwrap();
        let model = pop.menu_model().unwrap();
        assert_eq!(
            model.n_items(),
            2,
            "menu contains Italic and older Bold boundary"
        );
        let target = model.item_attribute_value(1, "target", None).unwrap();
        assert_eq!(target.get::<(u64, u64, u64, u32)>().unwrap().3, 2);
        assert_eq!(state(&ui), before, "opening Repeat history is read-only");
        pop.activate_action("history.repeat", Some(&target))
            .unwrap();
        settle().await;
        assert_eq!(selection_character(&ui)["bold"], true);
        assert_eq!(selection_character(&ui)["italic"], true);
        assert_eq!(
            state(&ui).1,
            before.1 + 1,
            "cumulative Repeat is one Undo step"
        );
        let repeated = state(&ui);
        pop.activate_action("history.repeat", Some(&target))
            .unwrap();
        assert_eq!(
            state(&ui),
            repeated,
            "stale Repeat menu target cannot apply twice"
        );
        more.popdown();
        apply_user_edit(&ui, Edit::Undo);
        assert_eq!(state(&ui).0, before.0);
        apply_user_edit(&ui, Edit::Redo);
        assert_eq!(state(&ui), repeated);
        eprintln!("ALPHA43_UI_OK");
        app.quit();
    });
}
