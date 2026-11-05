use super::*;
use serde_json::json;

pub(super) fn alpha28_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    window.set_default_size(1250, 900);
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        &(0..100)
            .map(|i| format!("Paragraph {i}. Section layout and reference checks.\n"))
            .collect::<String>(),
    ));
    s.run("para.style", &json!({"style":"Heading1"})).unwrap();
    selection::set_caret(&mut s, &Pos::body(55, 0), false).unwrap();
    s.run("layout.break", &json!({"kind":"nextPage"})).unwrap();
    let mut setup = PageSetup::current(&s);
    std::mem::swap(&mut setup.width, &mut setup.height);
    document_properties::apply(&mut s, setup, false).unwrap();
    opened(ui, s, "Sections.docx");
    zoom_to(ui, 0.5);
    let (u, a) = (ui.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
        {
            let c = u.current.borrow();
            let d = c.as_ref().unwrap();
            assert!(d.arrangement.rows[0].count >= 2);
        }
        jump_to(&u, 1);
        let (u, a) = (u.clone(), a.clone());
        glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            refresh_pages(&u);
            {
                let c = u.current.borrow();
                let d = c.as_ref().unwrap();
                assert_eq!(d.active, 1);
                assert!(d.thumb_buttons[1].has_css_class("selected-thumb"));
                assert!(!d.thumb_buttons[0].has_css_class("selected-thumb"));
            }
            fit_width(&u);
            let (u, a) = (u.clone(), a.clone());
            glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
                {
                    let c = u.current.borrow();
                    let d = c.as_ref().unwrap();
                    let last = d.sizes.len() - 1;
                    assert!(
                        (d.sizes[0].0 * d.page_zoom(0) - d.sizes[last].0 * d.page_zoom(last)).abs()
                            < 2.
                    );
                    assert!((d.page_zoom(0) - d.page_zoom(last)).abs() > 0.1);
                    for thumb in &d.thumbs {
                        assert!(thumb.width() <= 169);
                    }
                }
                let host = u.host.upgrade().unwrap();
                host.window
                    .lookup_action("insert-footnote")
                    .unwrap()
                    .activate(None);
                {
                    let mut c = u.current.borrow_mut();
                    let d = c.as_mut().unwrap();
                    assert!(matches!(
                        d.session.sel.focus.story,
                        wordcraft_doc::StoryRef::Part(_)
                    ));
                    d.session
                        .run("text.insert", &json!({"text":"Native note editing"}))
                        .unwrap();
                }
                after_content_change(&u);
                host.window
                    .lookup_action("footnote-reference")
                    .unwrap()
                    .activate(None);
                host.window
                    .lookup_action("edit-footnote")
                    .unwrap()
                    .activate(None);
                assert!(matches!(
                    u.current.borrow().as_ref().unwrap().session.sel.focus.story,
                    wordcraft_doc::StoryRef::Part(_)
                ));
                eprintln!("ALPHA28_UI_OK sections, fit, thumbnails, active page, note actions");
                host.window
                    .lookup_action("footnote-reference")
                    .unwrap()
                    .activate(None);
                for (action, title) in [
                    ("insert-toc", wordcraft_gnome::t!("Table of Contents")),
                    (
                        "insert-cross-reference",
                        wordcraft_gnome::t!("Cross-reference"),
                    ),
                    ("insert-caption", wordcraft_gnome::t!("Caption")),
                    ("footnote-options", wordcraft_gnome::t!("Footnote Options")),
                ] {
                    host.window.lookup_action(action).unwrap().activate(None);
                    let window = gtk::Window::list_toplevels()
                        .into_iter()
                        .filter_map(|w| w.downcast::<gtk::Window>().ok())
                        .find(|w| w.is_visible() && w.title().as_deref() == Some(title))
                        .unwrap();
                    find_widget(&window, "reference-apply")
                        .unwrap()
                        .downcast::<gtk::Button>()
                        .unwrap()
                        .emit_clicked();
                    assert!(
                        !window.is_visible(),
                        "reference dialog must commit without an error"
                    );
                }
                host.window
                    .lookup_action("update-references")
                    .unwrap()
                    .activate(None);
                eprintln!("ALPHA28_REFERENCE_DIALOGS_OK");
                alpha28_image_probe(&u, &a, 0);
            });
        });
    });
}

/// Synthetic public showcase, captured from the real application window.
pub(super) fn alpha28_gallery(ui: &Rc<Ui>, window: &ApplicationWindow) {
    window.set_default_size(1080, 800);
    let variant = std::env::var("LEAF_GALLERY").unwrap_or_default();
    let mut s = sample_session();
    selection::set_caret(&mut s, &Pos::body(4, 5), false).unwrap();
    wordcraft_gnome::references::insert_footnote(&mut s).unwrap();
    s.run(
        "text.insert",
        &json!({"text":"Membership supports shared equipment, maintenance and open workshops."}),
    )
    .unwrap();
    wordcraft_gnome::references::return_to_reference(&mut s).unwrap();
    let mut comment = None;
    if variant == "review" {
        s.run("file.setAuthor", &json!({"name":"Alex Morgan"}))
            .unwrap();
        selection::select_range(&mut s, &Pos::body(4, 0), &Pos::body(4, 28)).unwrap();
        wordcraft_gnome::annotations::add_comment(
            &mut s,
            "Could we add an example from the ceramics workshop?",
        )
        .unwrap();
        let id = wordcraft_gnome::annotations::comments(&s)[0].id;
        s.run("file.setAuthor", &json!({"name":"Sam Taylor"}))
            .unwrap();
        wordcraft_gnome::annotations::reply(
            &mut s,
            id,
            "Yes, the shared kiln is a good example. I will add it before Thursday.",
        )
        .unwrap();
        comment = Some(id);
    }
    opened(ui, s, "The Open Studio Handbook.docx");
    if let Some(d) = ui.current.borrow_mut().as_mut() {
        d.session.view.proofing = false;
    }
    refresh_document_appearance(ui);
    let u = ui.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(700), move || {
        if variant == "main" {
            zoom_to(&u, 0.80);
        } else {
            fit_width(&u);
        }
        let u = u.clone();
        glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
            if variant == "format" {
                if let Some(pop) = u
                    .paragraph
                    .borrow()
                    .as_ref()
                    .and_then(|p| p.popover.upgrade())
                {
                    pop.popup();
                }
            } else if variant == "notes" {
                if let Some(d) = u.current.borrow_mut().as_mut() {
                    wordcraft_gnome::references::open_footnote(&mut d.session, true).unwrap();
                }
                refresh_selection(&u);
                caret_ensure_visible(&u);
            } else if let Some(id) = comment {
                if let Some(panel) = u.annotations.borrow().as_ref() {
                    panel.reveal(vec![id]);
                }
            }
            eprintln!("ALPHA28_GALLERY_READY");
        });
    });
}

fn alpha28_image_probe(ui: &Rc<Ui>, app: &Application, phase: u8) {
    if phase == 0 {
        let path = std::env::temp_dir().join("leaf-alpha28-picture.png");
        std::fs::write(
            &path,
            wordcraft_engine::cmd::insert::base64_decode(include_str!(
                "../../../tests/fixtures/picture.b64"
            ))
            .unwrap(),
        )
        .unwrap();
        let mut s = Session::new(wordcraft_doc::Document::from_text(
            &(0..25)
                .map(|_| "Text should flow around this picture and remain editable. ")
                .collect::<String>(),
        ));
        selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
        pictures::insert(&mut s, &path).unwrap();
        pictures::placement(&mut s, Some("left"), 6.).unwrap();
        opened(ui, s, "Picture placement.docx");
        zoom_to(ui, 0.85);
    }
    let (u, a) = (ui.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        let (pos, rect, depth, ax, ay, bx, by) = {
            let mut c = u.current.borrow_mut();
            let d = c.as_mut().unwrap();
            let (pos, _, _) = pictures::selected(&d.session).unwrap();
            let hit = d.session.layout().object(&pos, 0).unwrap();
            let frame = d.page_frames[hit.page]
                .compute_bounds(&u.host.upgrade().unwrap().window)
                .unwrap();
            let z = d.page_zoom(hit.page);
            let scale = u.document_box.scale_factor() as f32;
            let (tx, ty) = u.host.upgrade().unwrap().window.surface_transform();
            let (x, y) = if phase == 0 {
                (hit.rect.x + hit.rect.w / 2., hit.rect.y + hit.rect.h / 2.)
            } else {
                (hit.rect.right() - 1., hit.rect.bottom() - 1.)
            };
            let ax = ((frame.x() + x * z + tx as f32) * scale).round() as i32;
            let ay = ((frame.y() + y * z + ty as f32) * scale).round() as i32;
            (
                pos,
                (hit.rect.x, hit.rect.y, hit.rect.w, hit.rect.h),
                d.session.undo_depth(),
                ax,
                ay,
                ax + (40. * scale) as i32,
                ay + (if phase == 0 { 25. * scale } else { 20. * scale }) as i32,
            )
        };
        eprintln!("ALPHA28_POINTER {phase} {ax} {ay} {bx} {by}");
        let dir = std::env::var("LEAF28_ACK_DIR").unwrap();
        let attempts = Cell::new(0);
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            attempts.set(attempts.get() + 1);
            assert!(attempts.get() < 200, "image pointer stalled");
            if !Path::new(&dir).join(phase.to_string()).exists() {
                return glib::ControlFlow::Continue;
            }
            {
                let mut c = u.current.borrow_mut();
                let d = c.as_mut().unwrap();
                let (current, _, _) = pictures::selected(&d.session).unwrap();
                let hit = d.session.layout().object(&current, 0).unwrap();
                if phase == 0 {
                    assert!((hit.rect.x - rect.0).abs() > 10.);
                } else {
                    assert!(hit.rect.w > rect.2 + 10.);
                }
                assert_eq!(d.session.undo_depth(), depth + 1);
                d.session.run("edit.undo", &json!({})).unwrap();
                let hit = d.session.layout().object(&pos, 0).unwrap();
                assert!((hit.rect.x - rect.0).abs() < 0.2 && (hit.rect.w - rect.2).abs() < 0.2);
                d.session.run("edit.redo", &json!({})).unwrap();
            }
            if phase == 0 {
                alpha28_image_probe(&u, &a, 1);
            } else {
                eprintln!("ALPHA28_POINTER_OK move, resize, undo, redo");
                a.quit();
            }
            glib::ControlFlow::Break
        });
    });
}
