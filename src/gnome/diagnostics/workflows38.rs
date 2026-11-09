//! Start-view combinations and real-input dismissal, using actual recovery records.
use super::*;
thread_local! { static STARTED: Cell<bool> = const { Cell::new(false) }; }
async fn settle() {
    glib::timeout_future(std::time::Duration::from_millis(400)).await;
}
async fn until(mut ready: impl FnMut() -> bool) {
    for _ in 0..500 {
        if ready() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(10)).await;
    }
    panic!("start view did not finish its native/worker lifecycle");
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
fn button(ui: &Ui) -> Option<gtk::Button> {
    find_widget(&ui.template_slot, "start-recovery-document").and_downcast()
}
fn make(app: &Application) -> (Rc<Workspace>, Rc<Ui>) {
    let window = make_window(app, false, false, false, false, false, false, false, None);
    let host = WORKSPACES.with(|v| {
        v.borrow()
            .iter()
            .find(|h| h.window == window)
            .unwrap()
            .clone()
    });
    let ui = selected_ui(&host).unwrap();
    (host, ui)
}
fn background(window: &ApplicationWindow, widget: &gtk::Widget) -> [u8; 4] {
    let paintable = gtk::WidgetPaintable::new(Some(window));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, window.width() as f64, window.height() as f64);
    let node = snapshot.to_node().unwrap();
    let image = window.renderer().unwrap().render_texture(&node, None);
    let bounds = widget.compute_bounds(window).unwrap();
    let x = (bounds.x() + bounds.width() * 0.25) as usize;
    let y = (bounds.y() + bounds.height() * 0.5) as usize;
    let stride = image.width() as usize * 4;
    let mut pixels = vec![0; stride * image.height() as usize];
    image.download(&mut pixels, stride);
    pixels[y * stride + x * 4..y * stride + x * 4 + 4]
        .try_into()
        .unwrap()
}
async fn selected_modes(ui: &Rc<Ui>, window: &ApplicationWindow) {
    let navigation = &ui.navigation_button;
    let review = find_widget(window, "sidebar-review")
        .unwrap()
        .downcast::<gtk::ToggleButton>()
        .unwrap();
    let manager = adw::StyleManager::default();
    let previous = manager.color_scheme();
    let identity = document_identity(ui);
    for theme in [adw::ColorScheme::ForceLight, adw::ColorScheme::ForceDark] {
        manager.set_color_scheme(theme);
        for (active, inactive) in [(navigation, &review), (&review, navigation)] {
            active.emit_clicked();
            ui.document_box.grab_focus();
            settle().await;
            assert!(active.is_active() && !inactive.is_active());
            assert!(active.state_flags().contains(gtk::StateFlags::CHECKED));
            assert_ne!(
                background(window, active.upcast_ref()),
                background(window, inactive.upcast_ref()),
                "active mode lacks persistent background"
            );
            inactive.set_state_flags(gtk::StateFlags::PRELIGHT, false);
            settle().await;
            assert!(active.is_active() && !inactive.is_active());
            inactive.unset_state_flags(gtk::StateFlags::PRELIGHT);
            active.grab_focus();
            assert!(active.has_focus());
        }
    }
    navigation.set_active(true);
    manager.set_color_scheme(previous);
    assert_eq!(document_identity(ui), identity);
    eprintln!(
        "WORKFLOWS39_MODES_OK persistent=true checked=true light-dark=true hover=true focus=true"
    );
}
async fn check(ui: &Rc<Ui>, has_templates: bool, has_recovery: bool) {
    until(|| {
        ui.template_gallery
            .borrow()
            .as_ref()
            .is_some_and(|g| g.inventory_ready())
    })
    .await;
    if has_templates {
        until(|| find_widget(&ui.template_slot, "personal-template").is_some()).await;
    }
    if has_recovery {
        until(|| {
            find_widget(&ui.template_slot, "recovery-preview").is_some_and(|slot| {
                slot.first_child()
                    .is_some_and(|child| child.is::<gtk::Picture>())
            })
        })
        .await;
    }
    settle().await;
    for name in ["template-preview", "recovery-preview"] {
        if let Some(slot) = find_widget(&ui.template_slot, name) {
            let picture = slot
                .first_child()
                .unwrap()
                .downcast::<gtk::Picture>()
                .unwrap();
            let paintable = picture.paintable().unwrap();
            let ratio = paintable.intrinsic_width() as f64 / paintable.intrinsic_height() as f64;
            let bounds = picture.compute_bounds(&slot).unwrap();
            assert!(bounds.width() <= 105. && bounds.height() <= 149.);
            assert!(
                (bounds.width() as f64 / bounds.height() as f64 - ratio).abs() < 0.015,
                "label stretched first-page paper: {bounds:?}, ratio={ratio}"
            );
        }
    }
    let templates = find_widget(&ui.template_slot, "start-templates").unwrap();
    let recoveries = find_widget(&ui.template_slot, "start-recovery").unwrap();
    let has_folder = wordcraft_gnome::templates::folder().is_some_and(|p| p.is_dir());
    assert_eq!(templates.is_visible(), has_templates || has_folder);
    assert_eq!(recoveries.is_visible(), has_recovery);
    assert_eq!(button(ui).is_some(), has_recovery);
    for (group, present) in [
        (&templates, has_templates || has_folder),
        (&recoveries, has_recovery),
    ] {
        if !present {
            assert!(!group.is_mapped());
            assert_eq!(group.measure(gtk::Orientation::Vertical, -1).0, 0);
        }
    }
    assert!(find_widget(&ui.template_slot, "start-blank-heading").is_none());
    assert_eq!(document_content_top(ui), 0.);
    let choices = has_folder || has_templates || has_recovery;
    assert_eq!(
        ui.sidebar_pages.visible_child_name().as_deref(),
        Some(if choices { "start" } else { "navigation" })
    );
    assert_eq!(ui.sidebar_selector.is_visible(), !choices);
    assert_eq!(
        ui.sidebar_title.text(),
        if choices {
            wordcraft_gnome::t!("Start")
        } else {
            wordcraft_gnome::t!("Navigation")
        }
    );
    let mut previous = -1.;
    for group in [&recoveries, &templates] {
        if group.is_visible() {
            let r = group.compute_bounds(&ui.template_slot).unwrap();
            assert!(r.y() >= previous);
            previous = r.y() + r.height();
        }
    }
    assert!(ui.template_eligible.get());
    assert!(text(ui).is_empty());
    assert!(!ui.dirty.get());
}
pub(super) fn write_pending(root: PathBuf) {
    let before = records(&root).len();
    let mut s = Session::new(wordcraft_doc::Document::new());
    s.enable_recovery(root.clone(), std::sync::Arc::new(|| {}))
        .unwrap();
    s.path = Some(PathBuf::from("Pending notes.docx"));
    s.run(
        "text.insert",
        &serde_json::json!({"text":"Pending recovery ñ"}),
    )
    .unwrap();
    let end = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        s.recovery_tick();
        if s.recovery_ready() && !s.recovery_busy() && records(&root).len() > before {
            break;
        }
        assert!(std::time::Instant::now() < end);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
async fn seed(root: PathBuf) {
    gio::spawn_blocking(move || write_pending(root))
        .await
        .unwrap();
}
pub(super) fn prepare_startup() {
    let root = wordcraft_gnome::preferences::recovery_root();
    assert!(
        wordcraft_gnome::recovery::inventory(&root)
            .unwrap()
            .documents
            .is_empty(),
        "requires isolated test profile"
    );
    let folder = root.parent().unwrap().join("startup38-templates");
    std::fs::create_dir_all(&folder).unwrap();
    wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
    let mut s = Session::new(wordcraft_doc::Document::from_text("Startup template"));
    wordcraft_gnome::saving::save(&mut s, &folder.join("Startup.docx")).unwrap();
    write_pending(root);
    println!("WORKFLOWS38_STARTUP_PREPARED");
}
pub(super) fn startup(ui: &Rc<Ui>, app: &Application) {
    let (ui, app) = (ui.clone(), app.clone());
    glib::spawn_future_local(async move {
        check(&ui, true, true).await;
        eprintln!("WORKFLOWS38_STARTUP_VISIBLE");
        let capture =
            PathBuf::from(std::env::var_os("LEAF29_ACK_DIR").unwrap()).join("workflows38-startup");
        until(|| capture.exists()).await;

        let root = wordcraft_gnome::preferences::recovery_root();
        let before = records(&root);
        assert!(
            ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .plain_text(wordcraft_doc::StoryRef::Body)
                .is_empty()
        );
        assert_eq!(records(&root), before);
        let root2 = root.clone();
        gio::spawn_blocking(move || {
            let records = wordcraft_gnome::recovery::inventory(&root2)
                .unwrap()
                .documents;
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].name, "Pending notes.docx");
            let mut s = wordcraft_gnome::recovery::restore(&root2, &records[0].id).unwrap();
            assert_eq!(
                s.doc.plain_text(wordcraft_doc::StoryRef::Body),
                "Pending recovery ñ"
            );
            s.discard_recovery_copy();
            s.finish_recovery();
        })
        .await
        .unwrap();
        let folder = wordcraft_gnome::preferences::template_folder().unwrap();
        wordcraft_gnome::preferences::reset_template_folder().unwrap();
        std::fs::remove_dir_all(folder).unwrap();
        eprintln!(
            "WORKFLOWS38_STARTUP_OK process-restart=true headings=true recovery-preserved=true editable-blank=true"
        );
        app.quit();
    });
}
fn records(root: &Path) -> Vec<Vec<u8>> {
    let mut paths = std::fs::read_dir(root)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|p| std::fs::read_dir(p.path()).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|p| p.path().extension().is_some_and(|e| e == "json"))
        .map(|p| p.path())
        .collect::<Vec<_>>();
    paths.sort();
    paths.iter().map(|p| std::fs::read(p).unwrap()).collect()
}
pub(super) fn test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    if STARTED.with(|s| s.replace(true)) {
        return;
    }
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_title(Some("Leaf Workflows38"));
        window.set_default_size(1100, 850);
        selected_modes(&ui, &window).await;
        let host = ui.host.upgrade().unwrap();
        let root = wordcraft_gnome::preferences::recovery_root();
        let folder = root.parent().unwrap().join(format!(
            "leaf38-templates-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&folder).unwrap();
        let previous_folder = wordcraft_gnome::preferences::template_folder();
        wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        templates::offer(&ui);
        check(&ui, false, false).await;
        let template = folder.join("Sample template with a deliberately long name.docx");
        let mut source = Session::new(wordcraft_doc::Document::from_text("Original template ñ"));
        let mut source_bytes = Vec::new();
        for (t, r) in [(false, false), (true, false), (false, true), (true, true)] {
            if t {
                wordcraft_gnome::saving::save(&mut source, &template).unwrap();
                source_bytes = std::fs::read(&template).unwrap();
            } else if template.exists() {
                std::fs::remove_file(&template).unwrap();
            }
            if r && records(&root).is_empty() {
                seed(root.clone()).await;
            }
            blank_tab(&app, &host);
            let tab = selected_ui(&host).unwrap();
            let page = host.tabs.selected_page().unwrap();
            check(&tab, t, r).await;
            let (other, view) = make(&app);
            check(&view, t, r).await;
            close_workspace(&other);
            host.tabs.close_page(&page);
            eprintln!(
                "WORKFLOWS38_COMBINATION_OK templates={t} recovery={r} new-tab=true new-window=true"
            );
        }
        blank_tab(&app, &host);
        let target = selected_ui(&host).unwrap();
        check(&target, true, true).await;
        let before = records(&root);
        assert!(!before.is_empty());
        // Click, then edit before the restore future can deliver: no late overwrite.
        button(&target).unwrap().emit_clicked();
        target
            .input_context
            .emit_by_name::<()>("commit", &[&"Keep my edit"]);
        settle().await;
        assert_eq!(text(&target), "Keep my edit");
        assert_eq!(records(&root), before);
        apply_user_edit(&target, Edit::Undo);
        settle().await;
        assert!(!target.template_slot.is_visible());
        target.dirty.set(false);
        let page = host.tabs.selected_page().unwrap();
        host.tabs.close_page(&page);
        // A template choice fills the intended blank and leaves its original untouched.
        blank_tab(&app, &host);
        let copied = selected_ui(&host).unwrap();
        check(&copied, true, true).await;
        find_widget(&copied.template_slot, "personal-template")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        until(|| text(&copied) == "Original template ñ").await;
        assert!(!copied.template_eligible.get());
        assert_eq!(std::fs::read(&template).unwrap(), source_bytes);
        let page = host.tabs.selected_page().unwrap();
        host.tabs.close_page(&page);
        // A record held elsewhere disappears after a failed stale activation;
        // releasing that result makes the same retained copy recoverable again.
        blank_tab(&app, &host);
        let failed = selected_ui(&host).unwrap();
        check(&failed, true, true).await;
        let root2 = root.clone();
        let held = gio::spawn_blocking(move || {
            let id = wordcraft_gnome::recovery::inventory(&root2)
                .unwrap()
                .documents[0]
                .id
                .clone();
            wordcraft_gnome::recovery::restore(&root2, &id).unwrap()
        })
        .await
        .unwrap();
        let identity = document_identity(&failed);
        button(&failed).unwrap().emit_clicked();
        until(|| button(&failed).is_none()).await;
        assert_eq!(document_identity(&failed), identity);
        assert!(text(&failed).is_empty());
        assert!(!failed.dirty.get());
        assert_eq!(records(&root), before);
        drop(held);
        failed
            .template_gallery
            .borrow()
            .as_ref()
            .unwrap()
            .refresh_recovery();
        check(&failed, true, true).await;
        let page = host.tabs.selected_page().unwrap();
        host.tabs.close_page(&page);
        // The actual pending record opens as an unsaved dirty document.
        blank_tab(&app, &host);
        let recovered = selected_ui(&host).unwrap();
        check(&recovered, true, true).await;
        let recovered_page = host.tabs.selected_page().unwrap();
        blank_tab(&app, &host);
        let animated = selected_ui(&host).unwrap();
        let browsing_page = host.tabs.selected_page().unwrap();
        check(&animated, true, true).await;
        let untouched = document_identity(&animated);
        host.tabs.set_selected_page(&recovered_page);
        button(&recovered).unwrap().emit_clicked();
        host.tabs.set_selected_page(&browsing_page);
        animated.document_box.grab_focus();
        until(|| text(&recovered) == "Pending recovery ñ").await;
        assert!(recovered.dirty.get());
        assert!(recovered.source_path.borrow().is_none());
        assert_eq!(document_identity(&animated), untouched);
        assert!(animated.document_box.has_focus());
        assert_eq!(records(&root), before);
        check(&animated, true, false).await;
        assert_eq!(
            window.lookup_action("recover"),
            None,
            "obsolete recovery menu action removed"
        );
        let a = animated.scroll.vadjustment();
        a.set_value(0.);
        settle().await;
        animated.document_box.grab_focus();
        let samples = Rc::new(RefCell::new(Vec::<(f64, f64, f64)>::new()));
        let values = samples.clone();
        let weak = Rc::downgrade(&animated);
        let clock = animated.template_revealer.frame_clock().unwrap();
        let capture = clock.connect_after_paint(move |_| {
            let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                return;
            };
            let page = ui.current.borrow().as_ref().unwrap().page_frames[0]
                .compute_bounds(&ui.scroll)
                .unwrap();
            values.borrow_mut().push((
                if ui.sidebar_pages.is_transition_running() {
                    1.
                } else {
                    0.
                },
                page.y() as f64,
                ui.scroll.vadjustment().value(),
            ));
        });
        let ack =
            PathBuf::from(std::env::var_os("LEAF29_ACK_DIR").unwrap()).join("workflows38-input");
        eprintln!("WORKFLOWS38_INPUT_READY");
        until(|| ack.exists()).await;
        until(|| !animated.template_slot.is_visible()).await;
        clock.disconnect(capture);
        assert_eq!(text(&animated), "Animated typing Paste ñ");
        let heights = samples.borrow();
        eprintln!(
            "WORKFLOWS40_SIDEBAR_TRANSITION samples={} values={:?}",
            heights.len(),
            &*heights
        );
        assert!(
            heights.iter().filter(|p| p.0 > 0.).count() >= 2,
            "sidebar must crossfade through native frames"
        );
        assert!(
            heights.windows(2).all(|p| (p[1].1 - p[0].1).abs() < 1.5),
            "sidebar transitions must not move the page"
        );
        assert_eq!(document_content_top(&animated), 0.);
        assert_eq!(records(&root), before);
        drop(heights);
        // IME stays responsive through collapse, with exactly-once input.
        blank_tab(&app, &host);
        let ime = selected_ui(&host).unwrap();
        check(&ime, true, false).await;
        ime.input_context.emit_by_name::<()>("commit", &[&"á"]);
        ime.input_context
            .emit_by_name::<()>("commit", &[&" rápido"]);
        settle().await;
        assert_eq!(text(&ime), "á rápido");
        assert!(!ime.template_slot.is_visible());
        // Reduced motion is immediate; a closed/switching tab cannot revive rows.
        let settings = window.settings();
        let animations = settings.is_gtk_enable_animations();
        settings.set_gtk_enable_animations(false);
        blank_tab(&app, &host);
        let reduced = selected_ui(&host).unwrap();
        check(&reduced, true, false).await;
        reduced
            .input_context
            .emit_by_name::<()>("commit", &[&"Reduced"]);
        assert!(!reduced.template_slot.is_visible());
        assert_eq!(text(&reduced), "Reduced");
        settings.set_gtk_enable_animations(animations);
        let (closing, view) = make(&app);
        check(&view, true, false).await;
        view.input_context
            .emit_by_name::<()>("commit", &[&"Closing"]);
        view.closing_confirmed.set(true);
        close_workspace(&closing);
        settle().await;
        assert!(view.closed.get());
        assert_eq!(records(&root), before);
        assert_eq!(std::fs::read(&template).unwrap(), source_bytes);
        {
            let mut current = recovered.current.borrow_mut();
            let session = &mut current.as_mut().unwrap().session;
            session.discard_recovery_copy();
            session.finish_recovery();
            session.dirty = false;
        }
        recovered.dirty.set(false);
        for (_, view) in host.docs.borrow().iter() {
            if let Some(doc) = view.current.borrow_mut().as_mut() {
                doc.session.discard_recovery_copy();
                doc.session.finish_recovery();
                doc.session.dirty = false;
            }
            view.dirty.set(false);
        }

        if let Some(folder) = previous_folder {
            wordcraft_gnome::preferences::set_template_folder(&folder).unwrap();
        } else {
            wordcraft_gnome::preferences::reset_template_folder().unwrap();
        }
        std::fs::remove_dir_all(folder).unwrap();
        eprintln!(
            "WORKFLOWS38_UI_OK combinations=true recovery=true retained-copies=true template=true stale=true motion=true input=true ime=true reduced-motion=true close=true"
        );
        app.quit();
    });
}
