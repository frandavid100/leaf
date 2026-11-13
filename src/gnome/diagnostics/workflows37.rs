//! Native geometry, modal input and first-save regression checks for alpha.37.
use super::*;

thread_local! { static STARTED: Cell<bool> = const { Cell::new(false) }; }

async fn settle() {
    glib::timeout_future(std::time::Duration::from_millis(400)).await;
}
async fn native_frames(widget: &impl IsA<gtk::Widget>, minimum: u32) {
    let ticks = Rc::new(Cell::new(0));
    let count = ticks.clone();
    widget.add_tick_callback(move |_, _| {
        count.set(count.get() + 1);
        if count.get() >= minimum {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
    for _ in 0..500 {
        if ticks.get() >= minimum {
            glib::timeout_future_with_priority(
                glib::Priority::DEFAULT_IDLE,
                std::time::Duration::ZERO,
            )
            .await;
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(10)).await;
    }
    panic!(
        "native allocation frames did not finish: {:?}",
        widget.as_ref().widget_name()
    );
}
async fn presented(dialog: &impl IsA<adw::Dialog>) {
    native_frames(dialog.as_ref(), 4).await;
    assert!(
        dialog.as_ref().is_mapped() && dialog.as_ref().root().is_some(),
        "dialog did not become interactive: {:?}",
        dialog.as_ref().title()
    );
}
async fn thumbnail_revealed(ui: &Ui) {
    // update_status queues a native idle reveal. Wait for that operation and
    // its two allocation corrections, not for a particular adjustment value.
    for _ in 0..500 {
        if !ui.thumbnail_reveal_pending.get() {
            native_frames(&ui.thumbnails_box, 3).await;
            return;
        }
        glib::timeout_future_with_priority(
            glib::Priority::DEFAULT_IDLE,
            std::time::Duration::from_millis(10),
        )
        .await;
    }
    panic!("active thumbnail reveal did not run");
}
async fn closed(dialog: &impl IsA<adw::Dialog>) {
    for _ in 0..500 {
        if dialog.as_ref().root().is_none() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(10)).await;
    }
    panic!(
        "dialog did not finish closing: {:?}",
        dialog.as_ref().title()
    );
}
async fn restored(ui: &Ui) {
    for _ in 0..500 {
        if ui.hidden_navigation_scroll.borrow().is_none() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(10)).await;
    }
    panic!("navigation restoration did not finish its allocation lifecycle");
}
fn unchanged(ui: &Ui, before: &(wordcraft_doc::Document, wordcraft_engine::Selection, usize)) {
    let current = ui.current.borrow();
    let s = &current.as_ref().unwrap().session;
    assert_eq!(s.doc, before.0);
    assert_eq!(s.sel, before.1);
    assert_eq!(s.undo_depth(), before.2);
}
async fn input(stage: u32, widget: Option<&gtk::Widget>, window: &ApplicationWindow) {
    let (x, y) = widget
        .map(|w| {
            let r = w.compute_bounds(window).expect("mapped native control");
            let (tx, ty) = window.surface_transform();
            let scale = window.scale_factor() as f64;
            (
                ((r.x() as f64 + r.width() as f64 / 2. + tx) * scale).round() as i32,
                ((r.y() as f64 + r.height() as f64 / 2. + ty) * scale).round() as i32,
            )
        })
        .unwrap_or((0, 0));
    eprintln!("WORKFLOWS37_POINT {stage} {x} {y}");
    let ack = PathBuf::from(std::env::var_os("LEAF29_ACK_DIR").expect("native input driver"))
        .join(format!("workflows37-{stage}"));
    for _ in 0..1200 {
        if ack.exists() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(10)).await;
    }
    panic!("input driver did not acknowledge stage {stage}");
}
fn offset(widget: &impl IsA<gtk::Widget>, window: &ApplicationWindow) -> f32 {
    widget.compute_bounds(window).unwrap().y()
}

fn fixed_modes(ui: &Ui, window: &ApplicationWindow) -> f32 {
    assert!(ui.sidebar_selector.is_mapped());
    assert!(
        ui.sidebar_selector
            .ancestor(gtk::ScrolledWindow::static_type())
            .is_none(),
        "mode tabs must be outside every sidebar scroll container"
    );
    let bar = ui.sidebar_selector.compute_bounds(window).unwrap();
    let sidebar = ui
        .sidebar
        .sidebar()
        .unwrap()
        .compute_bounds(window)
        .unwrap();
    assert!(
        (bar.y() + bar.height() - sidebar.y() - sidebar.height()).abs() < 1.1,
        "mode tabs must stay flush with the sidebar bottom: {bar:?} in {sidebar:?}"
    );
    for name in ["sidebar-navigation", "sidebar-review"] {
        let button = find_widget(&ui.sidebar_selector, name).unwrap();
        assert!(button.is_mapped() && button.is_sensitive());
        let r = button.compute_bounds(window).unwrap();
        assert!(r.y() >= bar.y() && r.y() + r.height() <= bar.y() + bar.height());
    }
    bar.y()
}

pub(in crate::gnome) fn test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    if STARTED.with(|s| s.replace(true)) {
        return;
    }
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_title(Some("Leaf Workflows37"));
        window.set_default_size(1100, 850);
        let host = ui.host.upgrade().unwrap();
        // Real gallery: asynchronous preview loading, shared vertical scrolling,
        // horizontal strip retained, first typing dismisses once, source unchanged.
        let directory =
            std::env::temp_dir().join(format!("leaf37-template-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let template = directory.join("Example.docx");
        let mut sample = Session::new(wordcraft_doc::Document::from_text("Original template"));
        wordcraft_gnome::saving::save(&mut sample, &template).unwrap();
        let bytes = std::fs::read(&template).unwrap();
        let previous_folder = wordcraft_gnome::preferences::template_folder();
        wordcraft_gnome::preferences::set_template_folder(&directory).unwrap();
        let original_page = host.tabs.selected_page().unwrap();
        blank_tab(&app, &host);
        let blank = selected_ui(&host).unwrap();
        let blank_page = host.tabs.selected_page().unwrap();
        for _ in 0..100 {
            if find_widget(&blank.template_slot, "personal-template").is_some() {
                break;
            }
            glib::timeout_future(std::time::Duration::from_millis(40)).await;
        }
        settle().await;
        assert!(blank.template_slot.is_visible());
        let gallery = find_widget(&blank.template_slot, "personal-template-gallery").unwrap();
        let page = blank.current.borrow().as_ref().unwrap().page_frames[0].clone();
        let (gallery_y, page_y) = (offset(&gallery, &window), offset(&page, &window));
        assert!(gallery.is_ancestor(&blank.sidebar_pages));
        assert!(!gallery.is_ancestor(&blank.scroll));
        let a = blank.scroll.vadjustment();
        let old_scroll = a.value();
        a.set_value(old_scroll + 100.);
        settle().await;
        let delta = a.value() - old_scroll;
        eprintln!(
            "WORKFLOWS37_GALLERY_GEOMETRY old_scroll={old_scroll} new_scroll={} gallery={gallery_y}/{} page={page_y}/{}",
            a.value(),
            offset(&gallery, &window),
            offset(&page, &window)
        );
        assert!(delta > 30.);
        assert!((offset(&gallery, &window) - gallery_y).abs() < 1.1);
        assert!((offset(&page, &window) - page_y + delta as f32).abs() < 1.1);
        assert!(blank.template_eligible.get());
        let browsing_position = a.value();
        let zoom = blank.current.borrow().as_ref().unwrap().zoom;
        zoom_to(&blank, zoom * 0.9);
        native_frames(&blank.document_box, 4).await;
        settle().await;
        assert!((a.value() - browsing_position).abs() < 1.1);
        assert!(
            blank.current.borrow().as_ref().unwrap().pages[0]
                .paintable()
                .is_some(),
            "refitting an eligible gallery must repaint the visible page without scrolling"
        );
        caret_ensure_visible(&blank);
        blank.document_box.grab_focus();
        apply_user_edit(&blank, Edit::Insert("First edit".into()));
        settle().await;
        assert!(!blank.template_slot.is_visible());
        assert_eq!(document_content_top(&blank), 0.);
        apply_user_edit(&blank, Edit::Undo);
        settle().await;
        assert!(blank.template_slot.is_visible());
        assert_eq!(std::fs::read(&template).unwrap(), bytes);
        if let Some(folder) = previous_folder {
            wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        } else {
            wordcraft_gnome::preferences::reset_template_folder().unwrap();
        }
        host.tabs.set_selected_page(&original_page);
        // Discard only this generated fixture after its Undo/source assertions.
        blank.dirty.set(false);
        host.tabs.close_page(&blank_page);
        settle().await;
        assert_eq!(host.tabs.n_pages(), 1);
        std::fs::remove_dir_all(directory).unwrap();
        eprintln!(
            "WORKFLOWS37_GALLERY_OK same-scroll=true refit-painted=true dismissed-once=true source-unchanged=true"
        );

        let content = (0..450)
            .map(|i| format!("Paragraph {i}: structure scrolling and continuity.\n"))
            .collect::<String>();
        let mut s = Session::new(wordcraft_doc::Document::from_text(&content));
        for i in (0..450).step_by(6) {
            selection::set_caret(&mut s, &Pos::body(i, 0), false).unwrap();
            s.run("para.style", &serde_json::json!({"style":"Heading1"}))
                .unwrap();
        }
        selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
        opened(&ui, s, "Scrolling controls");
        ui.representation.set_selected(0);
        ui.sidebar_button.set_active(true);
        settle().await;
        let readonly = {
            let c = ui.current.borrow();
            let s = &c.as_ref().unwrap().session;
            (s.doc.clone(), s.sel.clone(), s.undo_depth())
        };
        let main_scroll = ui.scroll.vadjustment().value();
        let pages = ui.thumbnails_scroll.vadjustment();
        let (choice_y, modes_y) = (
            offset(&ui.navigation_selector, &window),
            offset(&ui.sidebar_selector, &window),
        );
        pages.set_value(80.);
        settle().await;
        let delta = pages.value();
        assert!(delta > 30.);
        assert!((offset(&ui.navigation_selector, &window) - choice_y + delta as f32).abs() < 1.1);
        assert!((fixed_modes(&ui, &window) - modes_y).abs() < 1.1);
        pages.set_value((pages.upper() - pages.page_size()).max(0.));
        refresh_thumbnails(&ui);
        settle().await;
        assert!((fixed_modes(&ui, &window) - modes_y).abs() < 1.1);
        let last = ui.current.borrow().as_ref().unwrap().sizes.len() - 1;
        assert!(last > 4);
        assert!(
            ui.current.borrow().as_ref().unwrap().thumbs[last]
                .paintable()
                .is_some(),
            "manual sidebar scrolling rasterizes its last thumbnail independently of the document viewport"
        );
        assert!((ui.scroll.vadjustment().value() - main_scroll).abs() < 1.);
        unchanged(&ui, &readonly);
        pages.set_value(0.);
        jump_to(&ui, last);
        thumbnail_revealed(&ui).await;
        eprintln!(
            "WORKFLOWS37_ACTIVE_THUMB requested={last} active={} scroll={} upper={} page={} collapsed={} shown={} mapped={}",
            ui.current.borrow().as_ref().unwrap().active,
            pages.value(),
            pages.upper(),
            pages.page_size(),
            ui.sidebar.is_collapsed(),
            ui.sidebar.shows_sidebar(),
            ui.thumbnails_scroll.is_mapped()
        );
        let bounds = ui.current.borrow().as_ref().unwrap().thumb_buttons[last]
            .compute_bounds(&ui.thumbnails_scroll)
            .unwrap();
        assert!(
            bounds.y() >= -1.
                && bounds.y() + bounds.height() <= ui.thumbnails_scroll.height() as f32 + 1.,
            "active thumbnail must remain fully inside the actual viewport: {bounds:?}"
        );
        jump_to(&ui, 0);
        eprintln!(
            "WORKFLOWS37_THUMBNAILS_OK upper-selectors-scroll=true mode-tabs-fixed=true independent-raster=true actual-bounds=true readonly=true"
        );
        ui.representation.set_selected(1);
        ui.sidebar_button.set_active(true);
        settle().await;
        let outline = ui
            .navigation_pages
            .child_by_name("outline")
            .unwrap()
            .downcast::<gtk::ScrolledWindow>()
            .unwrap();
        let choice = find_widget(&outline, "navigation-representation-choice").unwrap();
        assert!(find_widget(&outline, "sidebar-navigation").is_none());
        let (choice_y, modes_y) = (offset(&choice, &window), fixed_modes(&ui, &window));
        let a = outline.vadjustment();
        a.set_value(210.);
        settle().await;
        let delta = a.value();
        assert!((offset(&choice, &window) - choice_y + delta as f32).abs() < 1.1);
        assert!((fixed_modes(&ui, &window) - modes_y).abs() < 1.1);
        for _ in 0..5 {
            ui.sidebar.set_show_sidebar(false);
            settle().await;
            ui.sidebar.set_show_sidebar(true);
            restored(&ui).await;
            settle().await;
            assert!(
                (a.value() - delta).abs() < 1.,
                "sidebar restoration must retain manual browsing"
            );
        }
        // A new input supersedes an in-flight restoration; it must not be
        // overwritten by a delayed timeout/frame callback.
        ui.sidebar.set_show_sidebar(false);
        settle().await;
        ui.sidebar.set_show_sidebar(true);
        let controllers = ui.navigation_pages.observe_controllers();
        let scroll_controller = (0..controllers.n_items())
            .find_map(|i| {
                controllers
                    .item(i)?
                    .downcast::<gtk::EventControllerScroll>()
                    .ok()
            })
            .unwrap();
        scroll_controller.emit_by_name::<bool>("scroll", &[&0.0f64, &1.0f64]);
        a.set_value(80.);
        settle().await;
        assert!((a.value() - 80.).abs() < 1.);
        eprintln!(
            "WORKFLOWS37_SIDEBAR_OK upper-selectors-scroll=true mode-tabs-fixed=true repeated-restore=true input-cancels=true"
        );

        let mut s = Session::new(wordcraft_doc::Document::from_text("Focus alpha beta."));
        s.doc.core.title = "First save 37".into();
        selection::select_range(&mut s, &Pos::body(0, 6), &Pos::body(0, 11)).unwrap();
        opened(&ui, s, "Untitled");
        ui.source_path.borrow_mut().take();
        ui.save_path.borrow_mut().take();
        let before = {
            let c = ui.current.borrow();
            let s = &c.as_ref().unwrap().session;
            (s.doc.clone(), s.sel.clone(), s.undo_depth())
        };
        window.set_title(Some("Leaf Workflows37"));
        for action in [
            "preferences",
            "shortcuts",
            "help",
            "about",
            "document-properties",
            "columns",
            "insert-page-number",
            "insert-table",
        ] {
            window.lookup_action(action).unwrap().activate(None);
            settle().await;
            let dialog = window
                .visible_dialog()
                .unwrap_or_else(|| panic!("missing integrated {action} dialog"));
            presented(&dialog).await;
            assert_eq!(
                dialog.root().as_ref(),
                Some(window.upcast_ref::<gtk::Root>())
            );
            window.set_default_size(380, 500);
            settle().await;
            assert!(
                dialog.width() <= window.width() && dialog.height() <= window.height(),
                "{action} exceeds narrow parent: {}x{} in {}x{}",
                dialog.width(),
                dialog.height(),
                window.width(),
                window.height()
            );
            dialog.close();
            closed(&dialog).await;
            assert!(dialog.root().is_none(), "{action} closes after adaptation");
            unchanged(&ui, &before);
            window.set_default_size(1100, 850);
            settle().await;
        }
        eprintln!("WORKFLOWS37_DIALOG_BOUNDS_OK integrated=8 narrow=true readonly=true");
        let other = make_window(&app, false, false, false, false, false, false, false, None);
        let other_prefs = show_preferences(&other);
        presented(&other_prefs).await;
        settle().await;
        assert_eq!(
            other_prefs.root().as_ref(),
            Some(other.upcast_ref::<gtk::Root>())
        );
        assert!(
            window.visible_dialog().is_none(),
            "dialogs belong to their own window"
        );
        other_prefs.close();
        closed(&other_prefs).await;
        other.destroy();
        settle().await;
        eprintln!("WORKFLOWS37_TWO_WINDOWS_OK");
        // A still-open panel from a replaced document must not mutate its replacement.
        show_insert_table(&ui, &window);
        let stale = window.visible_dialog().unwrap();
        presented(&stale).await;
        let mut replacement = Session::new(before.0.clone());
        selection::select_range(&mut replacement, &before.1.anchor, &before.1.focus).unwrap();
        opened(&ui, replacement, "Untitled");
        let grid = find_widget(&stale, "insert-table-grid")
            .unwrap()
            .downcast::<gtk::Grid>()
            .unwrap();
        grid.first_child()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        unchanged(&ui, &before);
        stale.close();
        closed(&stale).await;
        assert!(stale.root().is_none());
        eprintln!("WORKFLOWS37_STALE_PANEL_OK");
        let preferences_path = wordcraft_gnome::preferences::config_root().join("preferences.json");
        let saved_preferences = std::fs::read(&preferences_path).ok();
        ui.document_box.grab_focus();
        let prefs = show_preferences(&window);
        presented(&prefs).await;
        settle().await;
        assert_eq!(
            prefs.root().as_ref(),
            Some(window.upcast_ref::<gtk::Root>())
        );
        assert!(prefs.width() <= window.width() && prefs.height() <= window.height());
        let paintable = gtk::WidgetPaintable::new(Some(&window));
        let snapshot = gtk::Snapshot::new();
        paintable.snapshot(&snapshot, window.width() as f64, window.height() as f64);
        let node = snapshot
            .to_node()
            .expect("parent window includes the integrated dialog");
        let texture = window.renderer().unwrap().render_texture(&node, None);
        assert!(texture.width() > 0 && texture.height() > 0);
        // The shared diagnostic directory is writable inside the sandbox;
        // the host driver copies this evidence into its artifact directory.
        let capture = PathBuf::from(std::env::var_os("LEAF29_ACK_DIR").unwrap()).join(format!(
            "workflows37-dialog-{}-{}.png",
            std::env::var("LANGUAGE").unwrap_or_default(),
            window.scale_factor()
        ));
        texture.save_to_png(capture).unwrap();
        let name = find_widget(&prefs, "preferences-user-name").unwrap();
        name.grab_focus();
        input(0, Some(&name), &window).await;
        closed(&prefs).await;
        assert!(prefs.root().is_none());
        assert_eq!(
            host.tabs.n_pages(),
            1,
            "modal keys must not create/close tabs"
        );
        unchanged(&ui, &before);
        assert!(
            ui.document_box.has_focus(),
            "closing preferences restores editor focus"
        );
        // CI accounts can have no real-name fallback. Restore the original
        // settings bytes, including an unset author, instead of validating a
        // synthetic name through the user-facing nonempty-name setter.
        if let Some(bytes) = saved_preferences {
            std::fs::write(&preferences_path, bytes).unwrap();
        } else if preferences_path.exists() {
            std::fs::remove_file(&preferences_path).unwrap();
        }

        let link = show_link(&ui).unwrap();
        presented(&link.window).await;
        settle().await;
        let kind = find_widget(&link.window, "link-kind").unwrap();
        input(1, Some(&kind), &window).await;
        settle().await;
        assert_eq!(
            window.visible_dialog(),
            Some(link.window.clone()),
            "Escape closes the nested dropdown first"
        );
        unchanged(&ui, &before);
        input(2, None, &window).await;
        closed(&link.window).await;
        assert!(link.window.root().is_none());
        assert!(ui.document_box.has_focus());
        unchanged(&ui, &before);
        let link = show_link(&ui).unwrap();
        presented(&link.window).await;
        link.destination.set_text("https://example.org/37");
        settle().await;
        input(3, Some(link.apply.upcast_ref()), &window).await;
        closed(&link.window).await;
        assert!(link.window.root().is_none());
        assert!(ui.document_box.has_focus());
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.undo_depth(),
            before.2 + 1
        );
        input(4, None, &window).await;
        settle().await;
        unchanged(&ui, &before);
        let chooser = save_chooser(&ui, None, false);
        assert_eq!(
            chooser.initial_name().as_deref(),
            Some("First save 37.docx")
        );
        input(5, None, &window).await;
        settle().await;
        unchanged(&ui, &before);
        assert!(ui.save_path.borrow().is_none());
        // Regression #208: the shared mode bar stays reachable at any scroll
        // position; real pointer clicks switch long, search and empty views.
        let mut s = Session::new(wordcraft_doc::Document::from_text(&content));
        for i in 0..30 {
            selection::select_range(&mut s, &Pos::body(i, 0), &Pos::body(i, 9)).unwrap();
            wordcraft_gnome::annotations::add_comment(&mut s, "Check this passage.").unwrap();
        }
        selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
        opened(&ui, s, "Fixed sidebar modes");
        ui.representation.set_selected(0);
        ui.sidebar_button.set_active(true);
        settle().await;
        let readonly = {
            let c = ui.current.borrow();
            let s = &c.as_ref().unwrap().session;
            (s.doc.clone(), s.sel.clone(), s.undo_depth())
        };
        let bar_y = fixed_modes(&ui, &window);
        let pages = ui.thumbnails_scroll.vadjustment();
        let maximum = pages.upper() - pages.page_size();
        assert!(maximum > 100.);
        for value in [0., maximum / 2., maximum] {
            pages.set_value(value);
            settle().await;
            assert!((fixed_modes(&ui, &window) - bar_y).abs() < 1.1);
        }
        let review_button = find_widget(&ui.sidebar_selector, "sidebar-review").unwrap();
        let navigation_button = find_widget(&ui.sidebar_selector, "sidebar-navigation").unwrap();
        input(6, Some(&review_button), &window).await;
        settle().await;
        assert_eq!(
            ui.navigation_pages.visible_child_name().as_deref(),
            Some("review")
        );
        let review_scroll = ui
            .navigation_pages
            .child_by_name("review")
            .unwrap()
            .downcast::<gtk::ScrolledWindow>()
            .unwrap();
        let a = review_scroll.vadjustment();
        let maximum = a.upper() - a.page_size();
        assert!(maximum > 100.);
        for value in [0., maximum / 2., maximum] {
            a.set_value(value);
            settle().await;
            assert!((fixed_modes(&ui, &window) - bar_y).abs() < 1.1);
        }
        input(7, Some(&navigation_button), &window).await;
        settle().await;
        assert_eq!(
            ui.navigation_pages.visible_child_name().as_deref(),
            Some("thumbnails")
        );
        ui.search.borrow().as_ref().unwrap().show(false);
        settle().await;
        assert_eq!(
            ui.sidebar_pages.visible_child_name().as_deref(),
            Some("search")
        );
        assert!((fixed_modes(&ui, &window) - bar_y).abs() < 1.1);
        input(8, Some(&navigation_button), &window).await;
        settle().await;
        assert_eq!(
            ui.sidebar_pages.visible_child_name().as_deref(),
            Some("navigation")
        );
        assert!(ui.navigation_button.is_active());
        unchanged(&ui, &readonly);
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text("")),
            "Empty sidebar",
        );
        settle().await;
        assert!((fixed_modes(&ui, &window) - bar_y).abs() < 1.1);
        input(9, Some(&review_button), &window).await;
        settle().await;
        assert_eq!(
            ui.navigation_pages.visible_child_name().as_deref(),
            Some("review")
        );
        assert!((fixed_modes(&ui, &window) - bar_y).abs() < 1.1);
        ui.search.borrow().as_ref().unwrap().show(false);
        settle().await;
        input(10, Some(&review_button), &window).await;
        settle().await;
        assert_eq!(
            ui.sidebar_pages.visible_child_name().as_deref(),
            Some("navigation")
        );
        assert_eq!(
            ui.navigation_pages.visible_child_name().as_deref(),
            Some("review")
        );
        assert!((fixed_modes(&ui, &window) - bar_y).abs() < 1.1);
        eprintln!(
            "SIDEBAR208_OK fixed-bottom=true long-pages=true structure=true long-review=true search-both-modes=true empty=true native-clicks=true readonly=true"
        );
        eprintln!(
            "WORKFLOWS37_UI_OK dialogs-integrated=true nested-escape=true modal-shortcuts=true focus=true selection-history=true first-save=true"
        );
        app.quit();
    });
}
