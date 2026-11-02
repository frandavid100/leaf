use super::*;
use serde_json::json;

struct Sweep {
    ui: Rc<Ui>,
    window: ApplicationWindow,
    app: Application,
    selection: wordcraft_engine::Selection,
    depth: Cell<usize>,
}
fn point(w: &impl IsA<gtk::Widget>, root: &ApplicationWindow, x: f32, y: f32) -> (i32, i32) {
    let bounds = w.compute_bounds(root).unwrap();
    let (tx, ty) = root.surface_transform();
    let scale = root.scale_factor() as f64;
    (
        (((bounds.x() + x) as f64 + tx) * scale).round() as i32,
        (((bounds.y() + y) as f64 + ty) * scale).round() as i32,
    )
}
fn centre(w: &impl IsA<gtk::Widget>, root: &ApplicationWindow) -> (i32, i32) {
    point(
        w,
        root,
        w.as_ref().width() as f32 / 2.,
        w.as_ref().height() as f32 / 2.,
    )
}
fn selected(ui: &Rc<Ui>, n: usize) {
    {
        let mut c = ui.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        let end = s.doc.para_at(&Pos::body(n, 0)).unwrap().len();
        selection::select_range(s, &Pos::body(n, 0), &Pos::body(n, end)).unwrap();
    }
    refresh_selection(ui);
}
pub(super) fn sweep20_pointer_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    window.set_title(Some("Leaf Sweep20 Regression"));
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            &(0..130)
                .map(|i| format!("Paragraph {i}: ruler, history and page tooltip.\n"))
                .collect::<String>(),
        )),
        "Sweep20",
    );
    selected(ui, 0);
    {
        let mut c = ui.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        wordcraft_gnome::formatting::size(s, 14.).unwrap();
    }
    after_content_change(ui);
    selected(ui, 1);
    ui.document_box.grab_focus();
    let selection = ui.current.borrow().as_ref().unwrap().session.sel.clone();
    let t = Rc::new(Sweep {
        ui: ui.clone(),
        window: window.clone(),
        app: app.clone(),
        selection,
        depth: Cell::new(0),
    });
    glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || stage(t, 0));
}
fn stage(t: Rc<Sweep>, n: u8) {
    let u = &t.ui;
    let (role, points) = match n {
        0 => ("click", vec![centre(&u.redo_button, &t.window)]),
        1 => ("click", vec![centre(&u.undo_button, &t.window)]),
        2 => ("click", vec![centre(&u.redo_button, &t.window)]),
        3 | 5 => {
            let name = if n == 3 {
                "undo-history"
            } else {
                "redo-history"
            };
            let w = find_widget(&t.window, name).unwrap();
            ("click", vec![centre(&w, &t.window)])
        }
        4 => ("second", vec![]),
        6 => ("cancel", vec![]),
        7 => {
            selected(u, 1);
            let strip = find_widget(&t.window, "ruler-activation-strip").unwrap();
            (
                "hover",
                vec![point(&strip, &t.window, strip.width() as f32 / 2., 2.)],
            )
        }
        8 => {
            let pin = find_widget(&t.window, "ruler-pin").unwrap();
            ("pin", vec![centre(&pin, &t.window)])
        }
        9 => {
            let r = u.ruler.borrow().as_ref().unwrap().clone();
            let (x, z) = {
                let mut c = u.current.borrow_mut();
                let d = c.as_mut().unwrap();
                let state = wordcraft_gnome::ruler::state(&mut d.session).unwrap();
                let b = d.page_frames[state.page].compute_bounds(&r.area).unwrap();
                (b.x() + state.x(state.left) * d.zoom, d.zoom)
            };
            t.depth
                .set(u.current.borrow().as_ref().unwrap().session.undo_depth());
            (
                "drag",
                vec![
                    point(&r.area, &t.window, x, 35.),
                    point(&r.area, &t.window, x + 36. * z, 35.),
                ],
            )
        }
        10 => ("right", vec![]),
        11 => ("undo", vec![]),
        12 => {
            // Alignment and visibility survive viewport scrolling and zoom, without edits.
            let s = u.current.borrow().as_ref().unwrap().session.doc.clone();
            zoom_to(u, 0.65);
            let a = u.scroll.vadjustment();
            a.set_value((a.upper() - a.page_size()) / 2.);
            update_status(u);
            assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, s);
            ("hover", vec![(10, 20)])
        }
        _ => {
            eprintln!("SWEEP20_POINTER_OK");
            t.app.quit();
            return;
        }
    };
    let values = points
        .iter()
        .map(|(x, y)| format!("{x} {y}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("SWEEP20_READY {n} {role} {values}");
    let attempts = Cell::new(0);
    let dir = std::env::var("LEAF20_ACK_DIR").unwrap();
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        attempts.set(attempts.get() + 1);
        assert!(attempts.get() < 200, "pointer stalled at {n}");
        if !std::path::Path::new(&dir).join(n.to_string()).exists() {
            return glib::ControlFlow::Continue;
        }
        let t = t.clone();
        glib::idle_add_local_once(move || {
            check(&t, n);
            glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                stage(t, n + 1)
            });
        });
        glib::ControlFlow::Break
    });
}
fn check(t: &Sweep, n: u8) {
    let u = &t.ui;
    if matches!(n, 0 | 2) {
        assert_eq!(selection_character(u)["size"], json!(14.));
    }
    if n == 1 {
        assert_ne!(selection_character(u)["size"], json!(14.));
        assert!(u.current.borrow().as_ref().unwrap().session.can_redo());
    }
    if n == 3 {
        let c = u.current.borrow();
        let s = &c.as_ref().unwrap().session;
        assert_eq!(s.undo_depth(), 2);
        assert_eq!(s.sel, t.selection);
    }
    if n == 4 {
        assert!(!u.current.borrow().as_ref().unwrap().session.can_undo());
        assert_ne!(selection_character(u)["size"], json!(14.));
    }
    if n == 5 {
        let w = find_widget(&t.window, "redo-history")
            .unwrap()
            .downcast::<gtk::MenuButton>()
            .unwrap();
        let p = w.popover().unwrap().downcast::<gtk::PopoverMenu>().unwrap();
        assert_eq!(p.menu_model().unwrap().n_items(), 2);
    }
    if (7..=12).contains(&n) {
        assert!(u.ruler.borrow().as_ref().unwrap().reveal.reveals_child());
    }
    if (9..=11).contains(&n) {
        let mut c = u.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        let state = wordcraft_gnome::ruler::state(s).unwrap();
        let expected = if n == 10 { 37. } else { 36. };
        assert!(
            (state.left - expected).abs() < 0.6,
            "indent={} expected={expected}",
            state.left
        );
        assert_eq!(s.sel, t.selection);
        if n == 9 {
            assert_eq!(s.undo_depth(), t.depth.get() + 1);
        }
    }
    let c = u.current.borrow();
    let d = c.as_ref().unwrap();
    let expected = wordcraft_gnome::i18n::interpolate(
        wordcraft_gnome::t!("Pages: {current} of {total}"),
        &[
            ("current", (d.active + 1).to_string()),
            ("total", d.sizes.len().to_string()),
        ],
    );
    assert_eq!(
        u.pages_button.tooltip_text().as_deref(),
        Some(expected.as_str())
    );
}
