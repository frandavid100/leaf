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
    let icons = gtk::IconTheme::for_display(&gdk::Display::default().unwrap());
    assert!(icons.has_icon("compass-symbolic") && icons.has_icon("comment-symbolic"));
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
            assert!(find_widget(&t.window, "ruler-pin").is_none());
            let r = u.ruler.borrow().as_ref().unwrap().clone();
            (
                "hover",
                vec![point(&r.area, &t.window, r.area.width() as f32 / 2., 12.)],
            )
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
                    point(&r.area, &t.window, x, 19.),
                    point(&r.area, &t.window, x + 36. * z, 19.),
                ],
            )
        }
        10 => ("right", vec![]),
        11 => ("undo", vec![]),
        12 => {
            // Alignment and visibility survive viewport scrolling and zoom, without edits.
            let s = u.current.borrow().as_ref().unwrap().session.doc.clone();
            zoom_to(u, 0.65);
            update_status(u);
            assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, s);
            ("hover", vec![(10, 20)])
        }
        13 => {
            // Zoom's page-restoration idle has settled before scrolling away.
            let a = u.scroll.vadjustment();
            a.set_value((a.upper() - a.page_size()) / 2.);
            ("hover", vec![(10, 20)])
        }
        14 => {
            let current = u.current.borrow();
            let doc = current.as_ref().unwrap();
            let sheet = &doc.page_frames[doc.active];
            let frame = sheet.compute_bounds(&t.window).unwrap();
            let viewport = u.scroll.compute_bounds(&t.window).unwrap();
            let top = frame.y().max(viewport.y());
            let bottom = (frame.y() + frame.height()).min(viewport.y() + viewport.height());
            assert!(
                bottom > top,
                "the scrolled sheet must intersect the viewport"
            );
            let y = (top + bottom) / 2. - frame.y();
            ("hover", vec![point(sheet, &t.window, -4., y)])
        }
        15 => ("hover", vec![(10, 20)]),
        16 => ("ruler-keyboard", vec![]),
        17 => ("cancel", vec![]),
        18 | 21 => {
            let current = u.current.borrow();
            let doc = current.as_ref().unwrap();
            let sheet = &doc.page_frames[0];
            let x = sheet.width() as f32 / 2.;
            ("hover", vec![point(sheet, &t.window, x, 4.)])
        }
        19 | 22 => {
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
                if n == 19 { "cancel-drag" } else { "drag" },
                vec![
                    point(&r.area, &t.window, x, 19.),
                    point(
                        &r.area,
                        &t.window,
                        x + 36. * z,
                        if n == 19 { 19. } else { 110. },
                    ),
                ],
            )
        }
        20 => ("hover", vec![(10, 20)]),
        23 => ("undo", vec![]),
        24 | 25 | 27 | 28 => {
            let state = u.current.borrow();
            let doc = state.as_ref().unwrap();
            let sheet = &doc.page_frames[doc.active];
            let x = sheet.width() as f32;
            let y = sheet.height() as f32;
            let offset = if matches!(n, 24 | 27) { -4. } else { 4. };
            let target = if n <= 25 {
                point(sheet, &t.window, x + offset, y / 2.)
            } else {
                point(sheet, &t.window, x / 2., y + offset)
            };
            ("hover", vec![target])
        }
        26 | 29 => ("hover", vec![(10, 20)]),
        _ => {
            let before = u.current.borrow().as_ref().unwrap().session.doc.clone();
            let settings = u.document_box.settings();
            let old = settings.property::<bool>("gtk-cursor-blink");
            settings.set_property("gtk-cursor-blink", false);
            {
                let mut c = u.current.borrow_mut();
                selection::set_caret(&mut c.as_mut().unwrap().session, &Pos::body(1, 0), false)
                    .unwrap();
            }
            refresh_selection(u);
            u.document_box.grab_focus();
            restart_caret(u);
            assert_eq!(caret_opacity(u, glib::monotonic_time()), 1.0);
            settings.set_property("gtk-cursor-blink", old);
            assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, before);
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
    if n == 0 {
        for body in [&u.undo_button, &u.redo_button] {
            let parent = body.parent().unwrap();
            assert!(parent.is::<adw::SplitButton>());
            let icon = body.child().unwrap().compute_bounds(body).unwrap();
            assert!((icon.x() + icon.width() / 2. - body.width() as f32 / 2.).abs() < 1.);
            assert!((icon.y() + icon.height() / 2. - body.height() as f32 / 2.).abs() < 1.);
        }
    }
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
    if (7..=11).contains(&n) || n == 16 {
        assert!(u.ruler.borrow().as_ref().unwrap().reveal.reveals_child());
    }
    if matches!(n, 7 | 8) {
        let r = u.ruler.borrow().as_ref().unwrap().clone();
        assert!(r.area.height() <= 24);
        let mut c = u.current.borrow_mut();
        let d = c.as_mut().unwrap();
        let state = wordcraft_gnome::ruler::state(&mut d.session).unwrap();
        let frame = d.page_frames[state.page].compute_bounds(&t.window).unwrap();
        let area = r.area.compute_bounds(&t.window).unwrap();
        assert!((area.x() - frame.x().max(area.x())).abs() < 2.);
        assert!((area.width() - frame.width()).abs() < 2.);
        eprintln!(
            "RULER_HORIZONTAL_GEOMETRY x={} width={} height={}",
            area.x(),
            area.width(),
            area.height()
        );
    }
    if matches!(n, 12 | 13 | 15 | 17 | 20 | 22) {
        let r = u.ruler.borrow().as_ref().unwrap().clone();
        assert!(
            !r.reveal.reveals_child(),
            "horizontal must hide after leaving/cancel"
        );
        assert!(!r.vertical_reveal.reveals_child());
    }
    if n == 17 {
        u.scroll.vadjustment().set_value(0.);
    }
    if matches!(n, 19 | 22 | 23) {
        let mut c = u.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        let state = wordcraft_gnome::ruler::state(s).unwrap();
        let expected = if n == 22 { 72. } else { 36. };
        assert!(
            (state.left - expected).abs() < 0.6,
            "cancel/outside drag: indent={} expected={expected}",
            state.left
        );
        assert_eq!(s.sel, t.selection);
        assert_eq!(s.undo_depth(), t.depth.get() + usize::from(n == 22));
    }
    if (24..=29).contains(&n) {
        let r = u.ruler.borrow().as_ref().unwrap().clone();
        assert_eq!(r.right.reveals_child(), matches!(n, 24 | 25));
        assert_eq!(r.bottom.reveals_child(), matches!(n, 27 | 28));
    }
    if n == 12 {
        let r = u.ruler.borrow().as_ref().unwrap().clone();
        for w in [
            r.reveal.upcast_ref::<gtk::Widget>(),
            r.vertical_reveal.upcast_ref(),
        ] {
            let b = w.compute_bounds(&t.window).unwrap();
            let hit = t.window.pick(
                (b.x() + b.width() / 2.) as f64,
                (b.y() + b.height() / 2.) as f64,
                gtk::PickFlags::DEFAULT,
            );
            eprintln!(
                "HIDDEN_RULER_PICK {:?}",
                hit.as_ref().map(|p| (p.type_().name(), p.widget_name()))
            );
            assert!(
                !hit.as_ref().is_some_and(|p| p == w || p.is_ancestor(w)),
                "hidden rulers must pass pointer input through"
            );
        }
    }
    if n == 13 {
        let r = u.ruler.borrow().as_ref().unwrap().clone();
        let c = u.current.borrow();
        let d = c.as_ref().unwrap();
        assert!(d.active > 0, "the scroll probe must reach another page");
        let strip = find_widget(&t.window, "vertical-ruler-activation-strip").unwrap();
        assert!(
            strip.is_visible() && strip.height() > 0,
            "vertical scale follows the visible page, not the offscreen caret"
        );
        let frame = d.page_frames[d.active].compute_bounds(&t.window).unwrap();
        let area = r.vertical_reveal.compute_bounds(&t.window).unwrap();
        assert!((area.x() - frame.x()).abs() < 2.);
    }
    if n == 14 {
        let r = u.ruler.borrow().as_ref().unwrap().clone();
        assert!(r.vertical_reveal.reveals_child());
        assert!(r.vertical.width() <= 24);
        assert!(!r.reveal.reveals_child(), "axes reveal independently");
        eprintln!(
            "RULER_VERTICAL_GEOMETRY width={} height={}",
            r.vertical.width(),
            r.vertical.height()
        );
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
