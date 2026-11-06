use super::*;
use serde_json::json;

struct Sweep {
    ui: Rc<Ui>,
    window: ApplicationWindow,
    app: Application,
    selection: wordcraft_engine::Selection,
    depth: Cell<usize>,
    margins: Cell<[f32; 4]>,
    margin_zoom: Cell<f32>,
    scroll_before: Cell<f64>,
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
        margins: Cell::new([0.; 4]),
        margin_zoom: Cell::new(1.),
        scroll_before: Cell::new(0.),
    });
    glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || stage(t, 0));
}
fn margins(session: &Session) -> [f32; 4] {
    let props = session.doc.sections()[0].1;
    [
        props.margin_left,
        props.margin_right,
        props.margin_top,
        props.margin_bottom,
    ]
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
                    point(
                        &r.area,
                        &t.window,
                        x + 2. * wordcraft_gnome::ruler::HALF_CM * z,
                        19.,
                    ),
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
            let sheet = &doc.page_frames[viewport_page(u, doc)];
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
            t.margin_zoom.set(z);
            (
                if n == 19 { "cancel-drag" } else { "drag" },
                vec![
                    point(&r.area, &t.window, x, 19.),
                    point(
                        &r.area,
                        &t.window,
                        x + 2. * wordcraft_gnome::ruler::HALF_CM * z,
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
        26 => ("hover", vec![(10, 20)]),
        29 => {
            u.current.borrow_mut().as_mut().unwrap().fit_to_width = false;
            ("hover", vec![(10, 20)])
        }
        30..=41 if (n - 30) % 3 == 2 => ("undo", vec![]),
        30..=41 => {
            let side = usize::from((n - 30) / 3);
            let mut current = u.current.borrow_mut();
            let doc = current.as_mut().unwrap();
            let body = doc.session.layout().pages[0].body;
            let z = doc.zoom;
            let (x, y, dx, dy) = match side {
                0 => (body.x * z, 11., 12., 0.),
                1 => ((body.x + body.w) * z, 11., -12., 0.),
                2 => (11., body.y * z, 0., 12.),
                _ => (11., (body.y + body.h) * z, 0., -12.),
            };
            let sheet = &doc.page_frames[0];
            let start = point(sheet, &t.window, x, y);
            if (n - 30) % 3 == 0 {
                t.margins.set(margins(&doc.session));
                t.depth.set(doc.session.undo_depth());
                ("hover", vec![start])
            } else {
                t.margin_zoom.set(z);
                (
                    "margin-drag",
                    vec![start, point(sheet, &t.window, x + dx, y + dy)],
                )
            }
        }
        42 | 43 => {
            let current = u.current.borrow();
            let sheet = &current.as_ref().unwrap().page_frames[0];
            let target = point(sheet, &t.window, sheet.width() as f32 / 2., 11.);
            if n == 42 {
                ("hover", vec![target])
            } else {
                t.scroll_before.set(u.scroll.vadjustment().value());
                ("wheel", vec![target])
            }
        }
        44 => {
            selected(u, 0);
            {
                let mut c = u.current.borrow_mut();
                let s = &mut c.as_mut().unwrap().session;
                s.run(
                    "para.indents",
                    &json!({"left":19.25,"right":27.75,"firstLine":3.125}),
                )
                .unwrap();
                s.run(
                    "para.tabs",
                    &json!({"set":[{"pos":61.25,"align":"right","leader":"dot"}]}),
                )
                .unwrap();
            }
            after_content_change(u);
            zoom_to(u, 0.85);
            u.scroll.vadjustment().set_value(0.);
            ("hover", vec![(10, 20)])
        }
        45 | 48 | 51 | 54 | 57 => {
            let strip = find_widget(&t.window, "ruler-activation-strip").unwrap();
            (
                "hover",
                vec![point(&strip, &t.window, strip.width() as f32 / 2., 2.)],
            )
        }
        46 | 49 | 52 | 55 | 58 => {
            use wordcraft_gnome::ruler::{self, HALF_CM, Marker};
            let r = u.ruler.borrow().as_ref().unwrap().clone();
            let mut c = u.current.borrow_mut();
            let d = c.as_mut().unwrap();
            let state = ruler::state(&mut d.session).unwrap();
            t.depth.set(d.session.undo_depth());
            let frame = d.page_frames[state.page].compute_bounds(&r.area).unwrap();
            let (from, to, y) = match n {
                46 | 55 => (state.distance(Marker::First), state.left + HALF_CM * 2., 5.),
                49 => (
                    state.distance(Marker::Right),
                    state.width - HALF_CM * 3.,
                    19.,
                ),
                52 => (state.tabs[0].pos, HALF_CM * 6., 18.),
                _ => (HALF_CM * 8. + 3.5, HALF_CM * 10., 18.),
            };
            let start = point(&r.area, &t.window, frame.x() + state.x(from) * d.zoom, y);
            let end = point(&r.area, &t.window, frame.x() + state.x(to) * d.zoom, y);
            (
                if n == 55 { "cancel-drag" } else { "drag" },
                vec![start, end],
            )
        }
        47 | 50 | 53 | 59 => ("undo", vec![]),
        56 => ("hover", vec![(10, 20)]),
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
        let dir = std::path::Path::new(&dir);
        let preview = dir.join(format!("preview-{n}"));
        let verified = dir.join(format!("verified-{n}"));
        if role == "margin-drag" && preview.exists() && !verified.exists() {
            let current = t.ui.current.borrow();
            let doc = current.as_ref().unwrap();
            assert_eq!(margins(&doc.session), t.margins.get());
            assert_eq!(doc.session.undo_depth(), t.depth.get());
            let ruler = t.ui.ruler.borrow();
            let delta = ruler.as_ref().unwrap().margin_preview_delta().unwrap();
            let side = usize::from((n - 30) / 3);
            let before = t.margins.get()[side];
            let expected = wordcraft_gnome::ruler::snap_drag(
                before + 12. / t.margin_zoom.get(),
                None,
                0.,
                1500.,
            );
            assert!(
                (before + delta - expected).abs() < 0.01,
                "margin preview must show the snapped value"
            );
            std::fs::write(verified, []).unwrap();
        }
        if !dir.join(n.to_string()).exists() {
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
    if (30..=41).contains(&n) && (n - 30) % 3 != 0 {
        let current = u.current.borrow();
        let doc = current.as_ref().unwrap();
        let changed = (n - 30) % 3 == 1;
        let mut expected = t.margins.get();
        if changed {
            let side = usize::from((n - 30) / 3);
            expected[side] = wordcraft_gnome::ruler::snap_drag(
                expected[side] + 12. / t.margin_zoom.get(),
                None,
                0.,
                1500.,
            );
        }
        let actual = margins(&doc.session);
        assert!(
            actual
                .iter()
                .zip(expected)
                .all(|(a, b)| (a - b).abs() < 1.6),
            "margin stage {n}: {actual:?}, expected {expected:?}"
        );
        assert_eq!(
            doc.session.undo_depth(),
            t.depth.get() + usize::from(changed)
        );
    }
    if n == 42 {
        assert!(u.ruler.borrow().as_ref().unwrap().reveal.reveals_child());
    }
    if n == 43 {
        assert!(u.scroll.vadjustment().value() > t.scroll_before.get());
    }
    if n == 17 {
        u.scroll.vadjustment().set_value(0.);
    }
    if matches!(n, 19 | 22 | 23) {
        let mut c = u.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        let state = wordcraft_gnome::ruler::state(s).unwrap();
        let expected = if n == 22 { 4. } else { 2. } * wordcraft_gnome::ruler::HALF_CM;
        // One physical pointer pixel maps to more than one document point
        // below 100%; pointer markers now snap to half-centimetre increments.
        let tolerance = if n == 22 {
            let zoom = t.margin_zoom.get();
            0.5 / (t.window.scale_factor() as f32 * zoom) + 0.26
        } else {
            0.6
        };
        assert!(
            (state.left - expected).abs() < tolerance,
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
        let visible = viewport_page(u, d);
        assert!(visible > 0, "the scroll probe must reach another page");
        assert_eq!(d.active, 0, "scrolling must preserve the caret page");
        let strip = find_widget(&t.window, "vertical-ruler-activation-strip").unwrap();
        assert!(
            strip.is_visible() && strip.height() > 0,
            "vertical scale follows the visible page, not the offscreen caret"
        );
        let frame = d.page_frames[visible].compute_bounds(&t.window).unwrap();
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
        let expected = 2. * wordcraft_gnome::ruler::HALF_CM + if n == 10 { 1. } else { 0. };
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
    if matches!(n, 46 | 47 | 49 | 50 | 52 | 53 | 55 | 58 | 59) {
        use wordcraft_gnome::ruler::{self, HALF_CM};
        let mut c = u.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        let state = ruler::state(s).unwrap();
        let changed = matches!(n, 46 | 49 | 52 | 58);
        assert_eq!(s.undo_depth(), t.depth.get() + usize::from(changed));
        assert!(
            (state.left - 19.25).abs() < 0.001,
            "an untouched imported-style value must stay exact"
        );
        assert!(
            (state.first - if n == 46 { HALF_CM * 2. } else { 3.125 }).abs() < 0.001,
            "first-line origin/grid stage {n}: {}",
            state.first
        );
        assert!(
            (state.right - if n == 49 { HALF_CM * 3. } else { 27.75 }).abs() < 0.001,
            "opposite-edge origin/grid stage {n}: {}",
            state.right
        );
        assert!(
            (state.tabs[0].pos - if n == 52 { HALF_CM * 6. } else { 61.25 }).abs() < 0.001,
            "tab grid stage {n}: {:?}",
            state.tabs
        );
        assert_eq!(state.tabs.len(), if n == 58 { 2 } else { 1 });
        if n == 58 {
            assert!((state.tabs[1].pos - HALF_CM * 10.).abs() < 0.001);
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
