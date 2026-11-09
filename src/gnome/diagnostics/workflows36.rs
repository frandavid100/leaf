//! Verify revised UI through actual shipped controls and their allocations.
use super::*;
async fn settle() {
    glib::timeout_future(std::time::Duration::from_millis(400)).await;
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
fn row_offset(row: &gtk::Widget, scroll: &gtk::ScrolledWindow) -> f64 {
    row.compute_bounds(&row.parent().unwrap().parent().unwrap())
        .unwrap()
        .y() as f64
        - scroll.vadjustment().value()
}
pub(super) fn test(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    app: &Application,
    search: &Rc<SearchPanel>,
    outline: &Rc<OutlinePanel>,
) {
    let (ui, window, app, search, outline) = (
        ui.clone(),
        window.clone(),
        app.clone(),
        search.clone(),
        outline.clone(),
    );
    glib::spawn_future_local(async move {
        let host = ui.host.upgrade().unwrap();
        check_headers(&ui, &window);
        let height = ui.scroll.height();
        let first = host.tabs.selected_page().unwrap();
        assert!(
            !find_widget(&ui.sidebar, "document-header-secondary")
                .unwrap()
                .is_visible()
        );
        blank_tab(&app, &host);
        settle().await;
        let second = selected_ui(&host).unwrap();
        check_headers(&second, &window);
        assert!(
            find_widget(&second.sidebar, "document-header-secondary")
                .unwrap()
                .is_visible()
        );
        assert!(
            second.scroll.height() < height - 15,
            "tab strip must consume its own row"
        );
        let second_page = host.tabs.selected_page().unwrap();
        host.tabs.set_selected_page(&first);
        host.tabs.close_page(&second_page);
        settle().await;
        assert_eq!(host.tabs.n_pages(), 1);
        check_headers(&ui, &window);
        assert!(
            !find_widget(&ui.sidebar, "document-header-secondary")
                .unwrap()
                .is_visible()
        );
        assert!(
            (ui.scroll.height() - height).abs() <= 2,
            "one → two → one must reclaim the tab row"
        );
        eprintln!("WORKFLOWS36_TABS_OK");
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("a\tb\nc")),
            "Literal paste",
        );
        search.show(true);
        let choices = find_widget(&ui.sidebar_pages, "search-option-choices").unwrap();
        let mut labels = Vec::new();
        let mut child = choices.first_child();
        let mut last = None;
        while let Some(w) = child {
            if let Some(b) = w.downcast_ref::<gtk::CheckButton>() {
                labels.push(b.label().unwrap().to_string());
            }
            assert!(!w.is::<gtk::Expander>());
            last = Some(w.clone());
            child = w.next_sibling();
        }
        assert_eq!(
            labels,
            [
                wordcraft_gnome::t!("Replace"),
                wordcraft_gnome::t!("Case Sensitive"),
                wordcraft_gnome::t!("Whole Words"),
                wordcraft_gnome::t!("Preserve Capitalization"),
                wordcraft_gnome::t!("Regular Expressions")
            ]
        );
        assert_eq!(last.unwrap().widget_name(), "search-scope");
        assert!(search.regex.tooltip_text().is_some());
        assert!(search.scope.tooltip_text().is_some());
        let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
        for query in ["\t", "\n", "b\nc"] {
            search.query.set_text("");
            search.query.grab_focus();
            ui.document_box.clipboard().set_text(query);
            search
                .query
                .delegate()
                .unwrap()
                .emit_by_name::<()>("paste-clipboard", &[]);
            settle().await;
            assert_eq!(
                search.query.text(),
                query,
                "native paste must retain the entire query"
            );
            search.refresh();
            assert_eq!(search.hits.borrow().len(), 1);
            assert!(
                ui.current.borrow().as_ref().unwrap().session.doc == original,
                "hover/cancel must preserve the complete document"
            );
        }
        selection::select_range(
            &mut ui.current.borrow_mut().as_mut().unwrap().session,
            &Pos::body(0, 0),
            &Pos::body(0, 3),
        )
        .unwrap();
        search.query.set_text("a");
        search.scope.set_selected(4);
        search.refresh();
        assert_eq!(search.hits.borrow().len(), 1);
        search.replacement.grab_focus();
        settle().await;
        search.refresh();
        assert_eq!(search.hits.borrow().len(), 1);
        search.scope.set_selected(0);
        search.toggle.set_active(false);
        eprintln!("WORKFLOWS36_SEARCH_OK");
        let props = show_document_properties(&ui, &window);
        wait_dialog_presented(&props.window);
        let fonts = find_widget(&props.window, "properties-document-fonts").unwrap();
        assert!(fonts.is::<adw::PreferencesGroup>());
        assert!(
            fonts
                .ancestor(adw::PreferencesPage::static_type())
                .is_some()
        );
        props.window.set_content_width(380);
        props.window.set_content_height(560);
        settle().await;
        assert!(
            find_widget(&props.window, "document-font-list")
                .unwrap()
                .width()
                <= props.window.width()
        );
        assert!(host.window.lookup_action("document-fonts").is_none());
        assert!(
            ui.current.borrow().as_ref().unwrap().session.doc == original,
            "hover/cancel must preserve the complete document"
        );
        props.window.close();
        let mut font_doc = wordcraft_doc::Document::from_text(
            &(0..24)
                .map(|i| format!("Font example {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        let missing_name = "Leaf Missing Font With A Deliberately Long Family Name";
        for (i, b) in font_doc.body.iter_mut().enumerate() {
            if let wordcraft_doc::Block::Para(p) = std::sync::Arc::make_mut(b) {
                for run in &mut p.runs {
                    run.props.font = Some(if i == 0 {
                        missing_name.into()
                    } else {
                        format!("Leaf Missing Font {i}")
                    });
                }
            }
        }
        opened(&ui, Session::new(font_doc), "Many fonts");
        let before_fonts = ui.current.borrow().as_ref().unwrap().session.doc.clone();
        let props = show_document_properties(&ui, &window);
        wait_dialog_presented(&props.window);
        props.window.set_content_width(380);
        props.window.set_content_height(560);
        props.metadata[0].set_text("Pending title survives font replacement");
        settle().await;
        let row = find_widget(&props.window, &format!("document-font-{missing_name}")).unwrap();
        let fonts = find_widget(&props.window, "document-font-list").unwrap();
        assert!(fonts.width() <= props.window.width());
        find_widget(&row, "font-next-use")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert!(
            !ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .sel
                .is_collapsed()
        );
        find_widget(&row, "font-replace")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        settle().await;
        assert!(props.window.root().is_some());
        assert!(find_widget(&props.window, &format!("document-font-{missing_name}")).is_none());
        props.apply.emit_clicked();
        settle().await;
        assert_dialog_closed(&props.window);
        assert_eq!(
            wordcraft_gnome::document_properties::DocumentInfo::current(
                &ui.current.borrow().as_ref().unwrap().session
            )
            .title,
            "Pending title survives font replacement"
        );
        apply_user_edit(&ui, Edit::Undo);
        apply_user_edit(&ui, Edit::Undo);
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.doc,
            before_fonts
        );
        eprintln!("WORKFLOWS36_FONTS_OK");
        let mut doc = wordcraft_doc::Document::from_text(
            &(0..90)
                .map(|n| format!("Heading {n}"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        for (i, b) in doc.body.iter_mut().enumerate() {
            if let wordcraft_doc::Block::Para(p) = std::sync::Arc::make_mut(b) {
                p.props.outline_level = Some((i % 3) as u8);
                if i == 29 || i == 59 {
                    p.section = Some(Default::default());
                }
            }
        }
        opened(&ui, Session::new(doc), "Long structure");
        outline.show();
        outline.refresh();
        settle().await;
        let scroll = outline
            .list
            .ancestor(gtk::ScrolledWindow::static_type())
            .unwrap()
            .downcast::<gtk::ScrolledWindow>()
            .unwrap();
        let snapshot = ui.current.borrow().as_ref().unwrap().session.sel.clone();
        let revision = document_identity(&ui);
        let document_scroll = ui.scroll.vadjustment().value();
        for number in [3usize, 39, 84] {
            let name = format!("outline-heading-{number}");
            let row = find_widget(&outline.list, &name).unwrap();
            let y = row
                .compute_bounds(&row.parent().unwrap().parent().unwrap())
                .unwrap()
                .y() as f64;
            scroll.vadjustment().set_value(y - 75.);
            settle().await;
            for _ in 0..2 {
                let row = find_widget(&outline.list, &name).unwrap();
                let before = row_offset(&row, &scroll);
                let arrow = row
                    .first_child()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap();
                arrow.grab_focus();
                arrow.emit_clicked();
                outline.schedule();
                settle().await;
                let row = find_widget(&outline.list, &name).unwrap();
                let after = row_offset(&row, &scroll);
                let y = row
                    .compute_bounds(&row.parent().unwrap().parent().unwrap())
                    .unwrap()
                    .y() as f64;
                let adj = scroll.vadjustment();
                let expected = y
                    - (y - before).clamp(
                        adj.lower(),
                        (adj.upper() - adj.page_size()).max(adj.lower()),
                    );
                assert!(
                    (after - expected).abs() < 3.,
                    "branch {number} jumped: {before} → {after}, expected {expected}"
                );
                assert_eq!(document_identity(&ui), revision);
                assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel, snapshot);
                assert!((ui.scroll.vadjustment().value() - document_scroll).abs() < 1.);
            }
        }
        let mut sections = 0;
        let mut c = outline.list.first_child();
        while let Some(w) = c {
            if w.widget_name().starts_with("outline-section-") {
                let row = w.downcast_ref::<gtk::ListBoxRow>().unwrap();
                assert_eq!(row.header().is_some(), sections > 0);
                sections += 1;
            }
            c = w.next_sibling();
        }
        assert_eq!(sections, 3);
        scroll.vadjustment().set_value(0.);
        settle().await;
        let row = find_widget(&outline.list, "outline-heading-0").unwrap();
        assert!(find_widget(&row, "section-up").is_none());
        let menu = find_widget(&row, "outline-heading-menu-0")
            .unwrap()
            .downcast::<gtk::MenuButton>()
            .unwrap();
        let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
        menu.popup();
        settle().await;
        menu.popdown();
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
        eprintln!("WORKFLOWS36_OUTLINE_OK");
        {
            opened(
                &ui,
                Session::new(wordcraft_doc::Document::from_text("copy this text")),
                "Split clipboard",
            );
            selection::select_range(
                &mut ui.current.borrow_mut().as_mut().unwrap().session,
                &Pos::body(0, 0),
                &Pos::body(0, 4),
            )
            .unwrap();
            let pop = show_document_context(&ui, &window, 100, 150, DocumentContext::Text);
            settle().await;
            let copy = find_widget(&pop, "context-copy-split")
                .unwrap()
                .downcast::<adw::SplitButton>()
                .unwrap();
            assert!(copy.first_child().unwrap().is_sensitive());
            let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
            copy.popup();
            settle().await;
            let variants = copy
                .popover()
                .unwrap()
                .downcast::<gtk::PopoverMenu>()
                .unwrap();
            context_activate_row(&variants, wordcraft_gnome::t!("Back"));
            settle().await;
            assert!(!variants.is_visible());
            assert!(pop.is_visible());
            assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
            copy.emit_clicked();
            settle().await;
            assert_eq!(
                ui.document_box
                    .clipboard()
                    .read_text_future()
                    .await
                    .unwrap()
                    .unwrap(),
                "copy"
            );
            assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
        }
        settle().await;
        eprintln!("WORKFLOWS36_CONTEXT_OK");
        // A new blank stays editable; first committed input dismisses the gallery.
        let folder = wordcraft_gnome::preferences::config_root().join("template36");
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("Personal.docx");
        let mut sample = Session::new(wordcraft_doc::Document::from_text("Actual thumbnail"));
        wordcraft_gnome::saving::save(&mut sample, &path).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        blank_tab(&app, &host);
        let fresh = selected_ui(&host).unwrap();
        for _ in 0..300 {
            if find_widget(&fresh.template_slot, "personal-template").is_some() {
                break;
            }
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
        }
        assert!(fresh.template_slot.is_visible());
        let thumb = find_widget(&fresh.template_slot, "personal-template").unwrap();
        assert!(find_widget_type::<gtk::Picture>(&thumb).is_some());
        fresh.document_box.grab_focus();
        fresh.input_context.emit_by_name::<()>("commit", &[&"á"]);
        settle().await;
        assert_eq!(text(&fresh), "á");
        assert!(!fresh.template_slot.is_visible());
        assert!(fresh.template_eligible.get());
        apply_user_edit(&fresh, Edit::Undo);
        settle().await;
        assert!(fresh.template_slot.is_visible());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let prefs = show_preferences(&host.window);
        wait_dialog_presented(&prefs);
        assert!(find_widget(&prefs, "preferences-template-folder").is_some());
        find_widget(&prefs, "preferences-template-reset")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert!(wordcraft_gnome::preferences::template_folder().is_none());
        prefs.close();
        assert_dialog_closed(&prefs);
        wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        blank_tab(&app, &host);
        let rapid = selected_ui(&host).unwrap();
        let gallery = Rc::downgrade(rapid.template_gallery.borrow().as_ref().unwrap());
        rapid
            .input_context
            .emit_by_name::<()>("commit", &[&"before preview"]);
        settle().await;
        assert!(!rapid.template_slot.is_visible());
        assert!(gallery.upgrade().is_none());
        apply_user_edit(&rapid, Edit::Undo);
        settle().await;
        assert!(rapid.template_slot.is_visible());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        eprintln!(
            "WORKFLOWS36_UI_OK tabs=true search=true fonts=true outline=true context=true template=true"
        );
        app.quit();
    });
}
fn find_widget_type<T: IsA<gtk::Widget> + glib::types::StaticType>(
    root: &impl IsA<gtk::Widget>,
) -> Option<gtk::Widget> {
    let root = root.as_ref();
    if root.is::<T>() {
        return Some(root.clone());
    }
    let mut c = root.first_child();
    while let Some(w) = c {
        if let Some(w) = find_widget_type::<T>(&w) {
            return Some(w);
        }
        c = w.next_sibling();
    }
    None
}

fn point(widget: &impl IsA<gtk::Widget>, window: &ApplicationWindow) -> (i32, i32) {
    let b = widget.as_ref().compute_bounds(window).unwrap();
    let scale = window.scale_factor() as f32;
    (
        ((b.x() + b.width() / 2.) * scale) as i32,
        ((b.y() + b.height() / 2.) * scale) as i32,
    )
}
pub(super) fn pointer(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    app: &Application,
    search: &Rc<SearchPanel>,
) {
    let (ui, window, app, search) = (ui.clone(), window.clone(), app.clone(), search.clone());
    window.set_title(Some("Leaf Workflows36 Pointer"));
    glib::spawn_future_local(async move {
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("before\tneedle\nnext")),
            "Clipboard search",
        );
        search.show(false);
        search.query.grab_focus();
        settle().await;
        let (x, y) = point(&search.query, &window);
        eprintln!("WORKFLOWS36_POINT 0 query {x} {y} 0 0");
        for _ in 0..500 {
            if search.query.text() == "\tneedle\nnext" {
                break;
            }
            glib::timeout_future(std::time::Duration::from_millis(10)).await;
        }
        assert_eq!(search.query.text(), "\tneedle\nnext");
        search.refresh();
        assert_eq!(search.hits.borrow().len(), 1);
        assert_eq!(text(&ui), "before\tneedle\nnext");
        search.toggle.set_active(false);
        let body = (0..180)
            .map(|n| {
                format!("Paragraph {n}: an image can be inserted at this precise text boundary.")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let delayed = std::env::var_os("LEAF36_DELAYED_DROP").is_some();
        let stages = if delayed { 3 } else { 2 };
        for stage in 1..=stages {
            opened(
                &ui,
                Session::new(wordcraft_doc::Document::from_text(&body)),
                "Image preview",
            );
            ui.current.borrow_mut().as_mut().unwrap().fit_to_width = false;
            zoom_to(&ui, if stage == 1 { 0.5 } else { 1.25 });
            settle().await;
            let target = Pos::body(80, 10);
            let (page, cy) = {
                let mut d = ui.current.borrow_mut();
                let d = d.as_mut().unwrap();
                let c = d.session.layout().caret_on(&target, 0).unwrap();
                let b = d.page_frames[c.page]
                    .compute_bounds(&ui.document_box)
                    .unwrap();
                (
                    c.page,
                    b.y() as f64 + c.top as f64 * d.page_zoom(c.page) as f64,
                )
            };
            assert!(page > 0);
            if stage > 1 {
                ui.scroll
                    .vadjustment()
                    .set_value(cy - ui.scroll.vadjustment().page_size() * 0.35);
            }
            settle().await;
            let (x, y) = {
                let mut v = ui.current.borrow_mut();
                let d = v.as_mut().unwrap();
                let c = d.session.layout().caret_on(&target, page).unwrap();
                let b = d.page_frames[page].compute_bounds(&window).unwrap();
                let scale = window.scale_factor() as f32;
                (
                    ((b.x() + c.x * d.page_zoom(page)) * scale) as i32,
                    ((b.y() + (c.top + c.height / 2.) * d.page_zoom(page)) * scale) as i32,
                )
            };
            let scroll_bounds = ui.scroll.compute_bounds(&window).unwrap();
            let scale = window.scale_factor() as f32;
            let edge_y = ((scroll_bounds.y() + scroll_bounds.height() - 12.) * scale) as i32;
            let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
            let selection = ui.current.borrow().as_ref().unwrap().session.sel.clone();
            let depth = ui.current.borrow().as_ref().unwrap().session.undo_depth();
            let scroll_before = ui.scroll.vadjustment().value();
            let role = match stage {
                1 => "cancel",
                2 => "image",
                _ => "delayed",
            };
            eprintln!("WORKFLOWS36_POINT {stage} {role} {x} {y} {x} {edge_y}");
            let mut last = None;
            for _ in 0..700 {
                if let Some(p) = ui.image_drop.borrow().clone() {
                    last = Some(p);
                    break;
                }
                glib::timeout_future(std::time::Duration::from_millis(10)).await;
            }
            let mut last = last.expect("real drag must paint an insertion marker");
            assert_eq!(
                ui.current.borrow().as_ref().unwrap().session.sel,
                selection,
                "hover must not move the typing selection"
            );
            assert!(
                ui.current.borrow().as_ref().unwrap().session.doc == original,
                "hover/cancel must preserve the complete document"
            );
            let controllers = ui.document_box.observe_controllers();
            let target = (0..controllers.n_items())
                .find_map(|i| {
                    controllers
                        .item(i)
                        .and_then(|o| o.downcast::<gtk::DropTarget>().ok())
                })
                .unwrap();
            assert!(target.current_drop().is_some());
            let mut final_target = false;
            let mut crossed_gap = false;
            let mut resumed_after_gap = false;
            eprintln!("WORKFLOWS36_MARKER {stage}");
            for _ in 0..700 {
                let preview = ui.image_drop.borrow().clone();
                if let Some(p) = preview {
                    if stage == 2 && crossed_gap && !resumed_after_gap {
                        resumed_after_gap = true;
                        eprintln!("WORKFLOWS36_GAP_RESUMED {stage}");
                    }
                    if let Some(b) = ui.document_box.compute_bounds(&ui.scroll) {
                        let y = p.point.1 + b.y() as f64;
                        if stage == 2
                            && !final_target
                            && y > 40.
                            && y < ui.scroll.height() as f64 - 40.
                        {
                            final_target = true;
                            eprintln!("WORKFLOWS36_TARGET_READY {stage}");
                        }
                    }
                    last = p;
                } else if stage == 2 && ui.image_drag_active.get() {
                    crossed_gap = true;
                }
                // An absent marker can mean the pointer is between pages,
                // while the native drag is still active. Only native completion
                // ends this observation; the last displayed valid target is kept.
                if target.current_drop().is_none() {
                    break;
                }
                glib::timeout_future(std::time::Duration::from_millis(10)).await;
            }
            assert!(target.current_drop().is_none());
            assert!(ui.image_drop.borrow().is_none());
            if stage == 1 {
                {
                    let current = ui.current.borrow();
                    let d = current.as_ref().unwrap();
                    if d.session.doc != original {
                        eprintln!(
                            "WORKFLOWS36_CANCEL_STATE undo={} expected={} image={:?} rev={}",
                            d.session.undo_depth(),
                            depth,
                            pictures::selected(&d.session).map(|v| v.0),
                            d.session.rev()
                        );
                        report_difference(
                            &serde_json::to_value(&original).unwrap(),
                            &serde_json::to_value(&d.session.doc).unwrap(),
                            "document",
                            &mut 8,
                        );
                    }
                }
                assert!(
                    ui.current.borrow().as_ref().unwrap().session.doc == original,
                    "hover/cancel must preserve the complete document"
                );
                assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel, selection);
                assert_eq!(
                    ui.current.borrow().as_ref().unwrap().session.undo_depth(),
                    depth
                );
            } else if stage == 2 {
                assert!(
                    resumed_after_gap,
                    "edge scrolling must cross a gap and regain a valid marker"
                );
                for _ in 0..700 {
                    if ui.image_read.borrow().is_none() {
                        break;
                    }
                    glib::timeout_future(std::time::Duration::from_millis(10)).await;
                }
                let mut current = ui.current.borrow_mut();
                let d = current.as_mut().unwrap();
                let (actual, _, _) =
                    pictures::selected(&d.session).expect("drop must insert an image");
                assert_eq!(
                    actual, last.pos,
                    "committed target must equal the last displayed marker"
                );
                assert_eq!(d.session.undo_depth(), depth + 1);
                assert!(
                    ui.scroll.vadjustment().value() > scroll_before + 5.,
                    "edge hover must autoscroll"
                );
                d.session.run("edit.undo", &serde_json::json!({})).unwrap();
                assert_eq!(d.session.doc, original);
                d.session.run("edit.redo", &serde_json::json!({})).unwrap();
                assert!(pictures::selected(&d.session).is_some());
            } else {
                for _ in 0..300 {
                    if ui.image_read.borrow().is_some() {
                        break;
                    }
                    glib::timeout_future(std::time::Duration::from_millis(10)).await;
                }
                assert!(ui.image_read.borrow().is_some());
                ui.document_box.grab_focus();
                eprintln!("WORKFLOWS36_READ_PENDING {stage}");
                for _ in 0..700 {
                    if ui.image_read.borrow().is_none() {
                        break;
                    }
                    glib::timeout_future(std::time::Duration::from_millis(10)).await;
                }
                assert!(ui.image_read.borrow().is_none());
                assert!(
                    pictures::selected(&ui.current.borrow().as_ref().unwrap().session).is_none()
                );
                assert!(text(&ui).starts_with('X'));
                assert_eq!(
                    ui.current.borrow().as_ref().unwrap().session.undo_depth(),
                    depth + 1
                );
                apply_user_edit(&ui, Edit::Undo);
                assert!(
                    ui.current.borrow().as_ref().unwrap().session.doc == original,
                    "hover/cancel must preserve the complete document"
                );
            }
            eprintln!("WORKFLOWS36_STAGE_OK {stage}");
        }
        eprintln!(
            "WORKFLOWS36_POINTER_OK literal_clipboard=true marker_target=true cancel=true autoscroll=true stale={delayed}"
        );
        app.quit();
    });
}

/// Verify read-only XDG access in the sandbox and the native custom chooser.
pub(super) fn template_folder(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        wordcraft_gnome::preferences::reset_template_folder().unwrap();
        let prefs = show_preferences(&window);
        wait_dialog_presented(&prefs);
        let choose = find_widget(&prefs, "preferences-template-choose")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        let sandboxed = std::env::var_os("FLATPAK_ID").is_some();
        if sandboxed {
            assert!(
                !choose.is_sensitive(),
                "do not grant writable portal access for templates"
            );
            assert!(choose.tooltip_text().is_some());
            assert!(wordcraft_gnome::preferences::template_folder().is_none());
        } else {
            assert!(choose.is_sensitive());
            choose.grab_focus();
            choose.emit_clicked();
            eprintln!("WORKFLOWS36_TEMPLATE_CHOOSER_READY");
            for _ in 0..1500 {
                if wordcraft_gnome::preferences::template_folder().is_some() {
                    break;
                }
                glib::timeout_future(std::time::Duration::from_millis(20)).await;
            }
            assert!(wordcraft_gnome::preferences::template_folder().is_some());
        }
        let folder = wordcraft_gnome::templates::folder().expect("usable template folder");
        assert!(folder.join("Personal.docx").is_file());
        prefs.close();
        assert_dialog_closed(&prefs);
        let host = ui.host.upgrade().unwrap();
        let count = host.tabs.n_pages();
        blank_tab(&app, &host);
        let fresh = selected_ui(&host).unwrap();
        for _ in 0..300 {
            if find_widget(&fresh.template_slot, "personal-template").is_some() {
                break;
            }
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
        }
        find_widget(&fresh.template_slot, "personal-template")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        for _ in 0..300 {
            if text(&fresh) == "Read-only template" {
                break;
            }
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(text(&fresh), "Read-only template");
        assert_eq!(host.tabs.n_pages(), count + 1);
        assert!(
            fresh
                .current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .file_name
                .is_none()
        );
        let restart = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--workflows36-template-restart-test")
            .output()
            .unwrap();
        assert!(
            restart.status.success(),
            "{}",
            String::from_utf8_lossy(&restart.stderr)
        );
        eprintln!("{}", String::from_utf8_lossy(&restart.stdout));
        eprintln!(
            "WORKFLOWS36_TEMPLATE_FOLDER_OK thumbnail=true same_tab=true restart=true sandbox_read_only={sandboxed} custom_portal_disabled={sandboxed}"
        );
        app.quit();
    });
}

// Bounded failure evidence is useful across different installed runtimes.
fn report_difference(
    before: &serde_json::Value,
    after: &serde_json::Value,
    path: &str,
    remaining: &mut usize,
) {
    if before == after || *remaining == 0 {
        return;
    }
    match (before, after) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
            let keys = a
                .keys()
                .chain(b.keys())
                .collect::<std::collections::BTreeSet<_>>();
            for key in keys {
                report_difference(
                    &a.get(key).unwrap_or(&serde_json::Value::Null),
                    &b.get(key).unwrap_or(&serde_json::Value::Null),
                    &format!("{path}.{key}"),
                    remaining,
                );
            }
        }
        (serde_json::Value::Array(a), serde_json::Value::Array(b)) if a.len() == b.len() => {
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                report_difference(a, b, &format!("{path}[{i}]"), remaining);
            }
        }
        _ => {
            *remaining -= 1;
            eprintln!(
                "WORKFLOWS36_DELTA {path} before={} after={}",
                before.to_string().chars().take(240).collect::<String>(),
                after.to_string().chars().take(240).collect::<String>()
            );
        }
    }
}
