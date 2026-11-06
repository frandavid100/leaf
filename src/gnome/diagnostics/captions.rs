//! Actual GTK controls, keyboard editing and pointer hover for caption ownership.
use super::*;
use serde_json::json;
use wordcraft_gnome::captions::{self, Target};

fn check(root: &impl IsA<gtk::Widget>, name: &str) -> gtk::CheckButton {
    find_widget(root, name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .downcast()
        .unwrap()
}
fn request(window: &ApplicationWindow, widget: &impl IsA<gtk::Widget>, stage: u32, role: &str) {
    use gdk::prelude::PopupExt;
    let pop = widget.native().unwrap().downcast::<gtk::Popover>().unwrap();
    let r = widget.compute_bounds(&pop).unwrap();
    let (tx, ty) = pop.surface_transform();
    let (mut x, mut y) = (
        r.x() as f64 + r.width() as f64 / 2. - tx,
        r.y() as f64 + r.height() as f64 / 2. - ty,
    );
    let mut surface = pop.surface();
    while let Some(p) = surface.and_then(|s| s.downcast::<gdk::Popup>().ok()) {
        x += p.position_x() as f64;
        y += p.position_y() as f64;
        surface = PopupExt::parent(&p);
    }
    let scale = window.scale_factor() as f64;
    eprintln!(
        "CAPTION_INPUT {stage} {role} {} {}",
        (x * scale).round() as i32,
        (y * scale).round() as i32
    );
}
fn select_image(ui: &Rc<Ui>, n: usize) {
    let mut current = ui.current.borrow_mut();
    let s = &mut current.as_mut().unwrap().session;
    let entry = wordcraft_gnome::navigation::elements(&s.doc)
        .into_iter()
        .filter(|e| e.kind == 3)
        .nth(n)
        .unwrap();
    wordcraft_gnome::navigation::activate_element(s, &entry).unwrap();
}
pub(super) fn captions_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let compact = std::env::var("GDK_SCALE").as_deref() == Ok("2");
    window.set_default_size(if compact { 540 } else { 1250 }, 950);
    window.set_title(Some("Leaf Caption Regression"));
    if compact {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    }
    let mut s = Session::new(wordcraft_doc::Document::from_text("\n\nTail"));
    let file = std::env::temp_dir().join(format!("leaf-caption-ui-{}.png", std::process::id()));
    std::fs::write(
        &file,
        wordcraft_engine::cmd::insert::base64_decode(include_str!(
            "../../../tests/fixtures/picture.b64"
        ))
        .unwrap(),
    )
    .unwrap();
    for index in [0, 1] {
        selection::set_caret(&mut s, &Pos::body(index, 0), false).unwrap();
        pictures::insert(&mut s, &file).unwrap();
        pictures::resize(&mut s, 105., 65., false, true).unwrap();
    }
    std::fs::remove_file(file).unwrap();
    selection::set_caret(&mut s, &Pos::body(2, 0), false).unwrap();
    s.run("insert.table", &json!({"rows":2,"cols":2})).unwrap();
    opened(ui, s, "Captions and exact targets");
    ui.sidebar.set_show_sidebar(false);
    ui.sidebar_button.set_active(false);
    zoom_to(ui, if compact { 0.5 } else { 0.85 });
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    let phase = Cell::new(0u32);
    let ticks = Cell::new(0u32);
    let original = RefCell::new(None);
    let depth = Cell::new(0usize);
    let hover_id = RefCell::new(String::new());
    glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
        ticks.set(ticks.get() + 1);
        assert!(
            ticks.get() < 200,
            "caption test stalled phase={}",
            phase.get()
        );
        let elements = u.elements.borrow().as_ref().unwrap().clone();
        let ack = std::path::PathBuf::from(std::env::var("LEAF29_ACK_DIR").unwrap());
        let waiting = match phase.get() {
            5 => Some(0),
            7 => Some(1),
            8 => Some(2),
            9 => Some(3),
            11 => Some(4),
            13 => Some(5),
            17 => Some(6),
            _ => None,
        };
        if waiting.is_some_and(|n| !ack.join(format!("caption-{n}")).exists()) {
            return glib::ControlFlow::Continue;
        }
        eprintln!("CAPTION_PHASE {}", phase.get());
        w.set_title(Some("Leaf Caption Regression"));
        match phase.get() {
            0 => {
                // No-target controls stay visible, disabled, with insertion last.
                elements.show("image");
            }
            1 => unreachable!(),
            2 => {
                let button = check(&elements.pop, "picture-show-caption");
                assert!(button.is_visible() && !button.is_sensitive());
                assert!(
                    find_widget(&elements.pop, "picture-insert")
                        .unwrap()
                        .is_sensitive()
                );
                elements.pop.popdown();
                select_image(&u, 0);
                refresh_selection(&u);
                elements.show("image");
            }
            3 => unreachable!(),
            4 => {
                let button = check(&elements.pop, "picture-show-caption");
                assert!(button.is_sensitive() && !button.is_active());
                depth.set(u.current.borrow().as_ref().unwrap().session.undo_depth());
                request(&w, &button, 0, "type");
            }
            5 => {
                let c = u.current.borrow();
                let s = &c.as_ref().unwrap().session;
                assert!(
                    s.undo_depth() >= depth.get() + 2,
                    "toggle and native typing are separate edits"
                );
                let pairs = captions::all(&s.doc);
                assert_eq!(pairs.len(), 1);
                // XTest has delivered the keys, but slow HiDPI debug rendering
                // may still be processing the final native key event.
                if !captions::text(&s.doc, &pairs[0]).ends_with("Authored caption") {
                    return glib::ControlFlow::Continue;
                }
                drop(c);
                elements.show("image");
            }
            6 => {
                assert!(check(&elements.pop, "picture-show-caption").is_active());
                elements.header.pin.set_active(true);
                request(
                    &w,
                    &check(&elements.pop, "picture-show-caption"),
                    1,
                    "click",
                );
            }
            7 => {
                let pairs = captions::all(&u.current.borrow().as_ref().unwrap().session.doc);
                assert_eq!(pairs.len(), 1);
                assert!(!pairs[0].visible);
                assert!(
                    elements.pop.is_visible(),
                    "pinned properties survive toggle"
                );
                request(
                    &w,
                    &check(&elements.pop, "picture-show-caption"),
                    2,
                    "click",
                );
            }
            8 => {
                let c = u.current.borrow();
                let s = &c.as_ref().unwrap().session;
                let pair = captions::all(&s.doc).remove(0);
                assert!(pair.visible);
                assert!(captions::text(&s.doc, &pair).ends_with("Authored caption"));
                drop(c);
                tools_change(&u, |s| {
                    s.run("edit.undo", &json!({})).map_err(|e| e.to_string())?;
                    Ok(())
                });
                assert!(!check(&elements.pop, "picture-show-caption").is_active());
                tools_change(&u, |s| {
                    s.run("edit.redo", &json!({})).map_err(|e| e.to_string())?;
                    Ok(())
                });
                assert!(check(&elements.pop, "picture-show-caption").is_active());
                select_image(&u, 1);
                refresh_selection(&u);
                assert!(!check(&elements.pop, "picture-show-caption").is_active());
                request(
                    &w,
                    &check(&elements.pop, "picture-show-caption"),
                    3,
                    "click",
                );
            }
            9 => {
                assert_eq!(
                    captions::all(&u.current.borrow().as_ref().unwrap().session.doc).len(),
                    2
                );
                elements.header.pin.set_active(false);
                elements.pop.popdown();
                let mut c = u.current.borrow_mut();
                let s = &mut c.as_mut().unwrap().session;
                let table = wordcraft_gnome::navigation::elements(&s.doc)
                    .into_iter()
                    .find(|e| e.kind == 4)
                    .unwrap();
                wordcraft_gnome::navigation::activate_element(s, &table).unwrap();
                drop(c);
                refresh_selection(&u);
                elements.show("table");
            }
            10 => {
                let button = check(&elements.pop, "table-show-caption");
                assert!(button.is_sensitive() && !button.is_active());
                request(&w, &button, 4, "click");
            }
            11 => {
                assert_eq!(
                    captions::all(&u.current.borrow().as_ref().unwrap().session.doc).len(),
                    3
                );
                elements.pop.popdown();
                elements.show("table");
            }
            12 => {
                let (pos, original, depth) = {
                    let c = u.current.borrow();
                    let s = &c.as_ref().unwrap().session;
                    (s.sel.focus.clone(), s.doc.clone(), s.undo_depth())
                };
                table_command(&u, "table.selectTable", json!({}));
                {
                    let mut c = u.current.borrow_mut();
                    let s = &mut c.as_mut().unwrap().session;
                    assert!(
                        s.cell_range().is_some(),
                        "table actions resolve caption owner"
                    );
                    assert_eq!(s.doc, original);
                    assert_eq!(s.undo_depth(), depth);
                    selection::set_caret(s, &pos, false).unwrap();
                }
                refresh_selection(&u);
                let button = check(&elements.pop, "table-show-caption");
                assert!(
                    button.is_sensitive() && button.is_active(),
                    "caption caret retains table target"
                );
                request(&w, &button, 5, "click");
            }
            13 => {
                let c = u.current.borrow();
                let s = &c.as_ref().unwrap().session;
                assert!(
                    captions::all(&s.doc)
                        .iter()
                        .any(|p| matches!(p.owner, Target::Table { .. }) && !p.visible)
                );
                let bytes = wordcraft_engine::io::save_bytes("caption.docx", &s.doc).unwrap();
                let reopened =
                    Session::new(wordcraft_engine::io::open_bytes("caption.docx", &bytes).unwrap());
                drop(c);
                elements.pop.popdown();
                opened(&u, reopened, "Reopened caption ownership");
                select_image(&u, 0);
                refresh_selection(&u);
                let mut c = u.current.borrow_mut();
                let d = c.as_mut().unwrap();
                let pair = captions::all(&d.session.doc).remove(0);
                selection::set_caret(&mut d.session, &pair.pos, false).unwrap();
                hover_id.replace(pair.id);
                assert!(
                    !caption_cue_pages(&u, d).is_empty(),
                    "keyboard caret exposes ownership"
                );
                drop(c);
                refresh_selection(&u);
                u.outline.borrow().as_ref().unwrap().refresh();
                u.navigation_button.set_active(true);
                u.representation.set_selected(1);
            }
            14 => {
                // Navigation activates the actual table using its row, even
                // when a picture and table share their first text position.
                let outline = u.outline.borrow().as_ref().unwrap().clone();
                let c = u.current.borrow();
                let s = &c.as_ref().unwrap().session;
                let entries = wordcraft_gnome::navigation::elements(&s.doc);
                let index = entries.iter().position(|e| e.kind == 4).unwrap();
                let row = find_widget(&outline.list, &format!("outline-element-{index}"))
                    .unwrap()
                    .downcast::<gtk::ListBoxRow>()
                    .unwrap();
                drop(c);
                outline.list.emit_by_name::<()>("row-activated", &[&row]);
                assert!(
                    u.current
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .session
                        .cell_range()
                        .is_some()
                );
                u.sidebar.set_show_sidebar(false);
                u.sidebar_button.set_active(false);
            }
            16 => {
                let mut c = u.current.borrow_mut();
                let d = c.as_mut().unwrap();
                let end = d.session.doc.end_of(wordcraft_doc::StoryRef::Body);
                selection::set_caret(&mut d.session, &end, false).unwrap();
                let cue = captions::cues(&mut d.session).remove(0);
                let r = &cue.object[0];
                let frame = d.page_frames[r.page].compute_bounds(&w).unwrap();
                let z = d.page_zoom(r.page);
                let (tx, ty) = w.surface_transform();
                let scale = w.scale_factor() as f64;
                eprintln!(
                    "CAPTION_INPUT 6 hover {} {}",
                    ((frame.x() as f64 + (r.x + r.w / 2.) as f64 * z as f64 + tx) * scale).round()
                        as i32,
                    ((frame.y() as f64 + (r.y + r.h / 2.) as f64 * z as f64 + ty) * scale).round()
                        as i32
                );
                original.replace(Some(d.session.doc.clone()));
                depth.set(d.session.undo_depth());
                drop(c);
                refresh_selection(&u);
            }
            17 => {
                assert_eq!(*u.caption_hover.borrow(), Some(hover_id.borrow().clone()));
                let mut c = u.current.borrow_mut();
                let d = c.as_mut().unwrap();
                assert_eq!(d.session.doc, *original.borrow().as_ref().unwrap());
                assert_eq!(d.session.undo_depth(), depth.get());
                assert!(!caption_cue_pages(&u, d).is_empty());
                eprintln!(
                    "CAPTION_UI_OK native-toggle=true typing=true pin=true undo-redo=true image-switch=true table=true docx=true typed-navigation=true keyboard-cue=true pointer-hover=true readonly=true"
                );
                a.quit();
                return glib::ControlFlow::Break;
            }
            _ => unreachable!(),
        }
        // Startup/property transitions need a frame, no pointer acknowledgement.
        phase.set(match phase.get() {
            0 => 2,
            2 => 4,
            5 => 6,
            7 => 8,
            9 => 10,
            11 => 12,
            13 => 14,
            14 => 16,
            n => n + 1,
        });
        glib::ControlFlow::Continue
    });
}
