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
        assert!(find_widget(&ui.sidebar_pages, "search-whole-words").is_some());
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
        let mixed = gdk::ContentProvider::new_union(&[
            gdk::ContentProvider::for_value(&texture.to_value()),
            gdk::ContentProvider::for_bytes(
                "text/html",
                &glib::Bytes::from_static(b"<b>Rich text</b>"),
            ),
        ]);
        clipboard.set_content(Some(&mixed)).unwrap();
        context_paste(&ui);
        transfer(&ui).await;
        assert_eq!(text(&ui), "Rich text");
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
        apply_user_edit(&ui, Edit::Undo);
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
        find_widget(&row, "outline-heading-menu-0")
            .unwrap()
            .activate_action("heading.move-down", None)
            .unwrap();
        assert_eq!(text(&ui), "B\nbody B\nA\nbody A");
        apply_user_edit(&ui, Edit::Undo);
        assert!(text(&ui).starts_with("A"));
        let before = document_identity(&ui);
        let properties = show_document_properties(&ui, &window);
        wait_dialog_presented(&properties.window);
        assert!(
            find_widget(&properties.window, "properties-document-fonts")
                .unwrap()
                .is::<adw::PreferencesGroup>()
        );
        assert!(find_widget(&properties.window, "document-font-list").is_some());
        assert_eq!(document_identity(&ui), before);
        properties.window.close();
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
        blank_tab(&app, &host);
        let blank = selected_ui(&host).unwrap();
        for _ in 0..300 {
            if find_widget(&blank.template_slot, "personal-template").is_some() {
                break;
            }
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
        }
        find_widget(&blank.template_slot, "personal-template")
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        for _ in 0..300 {
            if text(&blank) == "Personal template" {
                break;
            }
            glib::timeout_future(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(host.tabs.n_pages(), count + 1);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(text(&blank), "Personal template");
        assert!(!blank.template_slot.is_visible());
        let restart = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("--workflows35-template-restart-test")
            .output()
            .unwrap();
        assert!(
            restart.status.success(),
            "{}",
            String::from_utf8_lossy(&restart.stderr)
        );
        assert!(
            String::from_utf8_lossy(&restart.stdout).contains("WORKFLOWS35_TEMPLATE_RESTART_OK")
        );
        eprintln!(
            "WORKFLOWS35_UI_OK search=true selection=true regex_error=true next_style=true plain=true painter=true image=true outline=true fonts=true template=true ownership=true"
        );
        app.quit();
    });
}

async fn pause() {
    glib::timeout_future(std::time::Duration::from_millis(350)).await;
}
async fn wait(mut predicate: impl FnMut() -> bool) {
    for _ in 0..1500 {
        if predicate() {
            pause().await;
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(20)).await;
    }
    panic!("workflow pointer did not complete")
}
fn point(widget: &impl IsA<gtk::Widget>, window: &ApplicationWindow) -> (i32, i32) {
    let r = widget.compute_bounds(window).unwrap();
    let (tx, ty) = window.surface_transform();
    let scale = window.scale_factor() as f64;
    (
        ((r.x() as f64 + r.width() as f64 / 2. + tx) * scale).round() as i32,
        ((r.y() as f64 + r.height() as f64 / 2. + ty) * scale).round() as i32,
    )
}
fn canvas_point(ui: &Ui, window: &ApplicationWindow, off: usize) -> (i32, i32) {
    let mut current = ui.current.borrow_mut();
    let d = current.as_mut().unwrap();
    let c = d.session.layout().caret_on(&Pos::body(0, off), 0).unwrap();
    let r = d.page_frames[0].compute_bounds(window).unwrap();
    let (tx, ty) = window.surface_transform();
    let scale = window.scale_factor() as f64;
    (
        ((r.x() as f64 + (c.x as f64 + 0.2) * d.zoom as f64 + tx) * scale).round() as i32,
        ((r.y() as f64 + (c.top + c.height / 2.) as f64 * d.zoom as f64 + ty) * scale).round()
            as i32,
    )
}
pub(super) fn pointer(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    app: &Application,
    outline: &Rc<OutlinePanel>,
) {
    window.set_title(Some("Leaf Workflows35 Pointer"));
    window.set_default_size(1100, 850);
    let (ui, window, app, outline) = (ui.clone(), window.clone(), app.clone(), outline.clone());
    glib::spawn_future_local(async move {
        for (stage, role) in ["file", "image", "plain"].into_iter().enumerate() {
            opened(
                &ui,
                Session::new(wordcraft_doc::Document::from_text("Before after")),
                "Drop target",
            );
            {
                let mut current = ui.current.borrow_mut();
                selection::set_caret(
                    &mut current.as_mut().unwrap().session,
                    &Pos::body(0, 7),
                    false,
                )
                .unwrap();
            }
            ui.document_box.grab_focus();
            pause().await;
            let (x, y) = canvas_point(&ui, &window, 7);
            let depth = ui.current.borrow().as_ref().unwrap().session.undo_depth();
            eprintln!("WORKFLOWS35_POINTER {stage} {role} {x} {y}");
            if role == "plain" {
                wait(|| text(&ui) == "Before Plainafter").await;
            } else {
                wait(|| {
                    ui.current
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .session
                        .doc
                        .media
                        .len()
                        == 1
                })
                .await;
                let current = ui.current.borrow();
                let s = &current.as_ref().unwrap().session;
                assert!(
                    s.doc
                        .para_at(&Pos::body(0, 7))
                        .unwrap()
                        .object_at(7)
                        .is_some()
                );
                let (_, w, h) = pictures::selected(s).unwrap();
                assert!((w / h - 2.).abs() < 0.02);
            }
            assert_eq!(
                ui.current.borrow().as_ref().unwrap().session.undo_depth(),
                depth + 1
            );
            apply_user_edit(&ui, Edit::Undo);
            assert_eq!(text(&ui), "Before after");
            apply_user_edit(&ui, Edit::Redo);
        }
        let mut s = Session::new(wordcraft_doc::Document::from_text("A\nbody A\nB\nbody B"));
        for i in [0, 2] {
            selection::set_caret(&mut s, &Pos::body(i, 0), false).unwrap();
            s.run("para.style", &json!({"style":"Heading1"})).unwrap();
        }
        opened(&ui, s, "Heading drag");
        outline.show();
        outline.refresh();
        pause().await;
        let source = find_widget(&outline.list, "outline-heading-1").unwrap();
        let target = find_widget(&outline.list, "outline-heading-0").unwrap();
        let (ax, ay) = point(&source, &window);
        let (bx, by) = point(&target, &window);
        let depth = ui.current.borrow().as_ref().unwrap().session.undo_depth();
        eprintln!("WORKFLOWS35_POINTER 3 heading {ax} {ay} {bx} {by}");
        wait(|| text(&ui) == "B\nbody B\nA\nbody A").await;
        assert_eq!(
            ui.current.borrow().as_ref().unwrap().session.undo_depth(),
            depth + 1
        );
        apply_user_edit(&ui, Edit::Undo);
        assert_eq!(text(&ui), "A\nbody A\nB\nbody B");
        eprintln!(
            "WORKFLOWS35_POINTER_OK external_file=true external_clipboard_image=true plain_shortcut=true heading_drag=true undo_redo=true"
        );
        app.quit();
    });
}
/// A separate native process exports a real GDK FileList. In installed checks,
/// the target must receive the host file through GTK's document portal transfer.
pub(super) fn drag_source() {
    let app = Application::builder()
        .application_id("io.github.frandavid100.leaf.DragSourceTest")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(|app| {
        let path = PathBuf::from(std::env::var_os("LEAF35_DROP_FILE").expect("drag fixture path"));
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Leaf Workflows35 Source")
            .default_width(240)
            .default_height(160)
            .build();
        let source = gtk::Box::new(gtk::Orientation::Vertical, 0);
        source.append(&gtk::Label::new(Some("Drag image")));
        window.set_content(Some(&source));
        let drag = gtk::DragSource::new();
        drag.set_actions(gdk::DragAction::COPY);
        drag.connect_prepare(move |_, _, _| {
            eprintln!("WORKFLOWS35_SOURCE_DRAG");
            let files = gdk::FileList::from_array(&[gio::File::for_path(&path)]);
            Some(gdk::ContentProvider::for_value(&files.to_value()))
        });
        drag.connect_drag_end(|_, drag, delete| {
            eprintln!(
                "WORKFLOWS35_SOURCE_END {:?} {delete}",
                drag.selected_action()
            )
        });
        drag.connect_drag_cancel(|_, _, why| {
            eprintln!("WORKFLOWS35_SOURCE_CANCEL {why:?}");
            false
        });
        let keys = gtk::EventControllerKey::new();
        let weak_drag = drag.downgrade();
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(drag) = weak_drag.upgrade() {
                    eprintln!("WORKFLOWS35_SOURCE_ESCAPE");
                    // Cancel the native drag too. GTK 4.14 DragSource::drag_cancel
                    // only ends its source bookkeeping and can leave Xdnd active.
                    if let Some(native) = drag.drag() {
                        native
                            .emit_by_name::<()>("cancel", &[&gdk::DragCancelReason::UserCancelled]);
                    }
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(keys);
        source.add_controller(drag);
        window.present();
        let weak = window.downgrade();
        glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
            if let Some(window) = weak.upgrade() {
                let (x, y) = point(&source, &window);
                eprintln!("WORKFLOWS35_SOURCE {x} {y}");
            }
        });
    });
    app.run_with_args(&["leaf-workflows35-source"]);
}

pub(super) fn template_restart() {
    let folder = wordcraft_gnome::templates::folder().unwrap();
    assert_eq!(
        Some(folder.clone()),
        wordcraft_gnome::preferences::template_folder()
    );
    let paths = wordcraft_gnome::templates::list(&folder).unwrap();
    assert_eq!(paths.len(), 1);
    let before = std::fs::read(&paths[0]).unwrap();
    let session = wordcraft_gnome::templates::open(&paths[0]).unwrap();
    assert!(session.dirty);
    assert!(session.file_name.is_none());
    assert_eq!(
        session.doc.plain_text(wordcraft_doc::StoryRef::Body),
        "Personal template"
    );
    assert_eq!(std::fs::read(&paths[0]).unwrap(), before);
    println!("WORKFLOWS35_TEMPLATE_RESTART_OK");
}
