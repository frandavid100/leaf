use super::*;

struct Followup {
    ui: Rc<Ui>,
    window: ApplicationWindow,
    app: Application,
    menu: gtk::MenuButton,
    mode: String,
    family: String,
    body: String,
    initial_pages: usize,
}
fn followup_point(w: &impl IsA<gtk::Widget>, root: &impl IsA<gtk::Window>) -> (i32, i32) {
    followup_point_at(w, root, 0.5)
}
fn followup_point_at(
    w: &impl IsA<gtk::Widget>,
    root: &impl IsA<gtk::Window>,
    fraction: f64,
) -> (i32, i32) {
    let root = root.as_ref();
    let scale = root.scale_factor() as f64;
    let (x, y) = if let Some(pop) = w.native().and_then(|n| n.downcast::<gtk::Popover>().ok()) {
        use gdk::prelude::PopupExt;
        let r = w.compute_bounds(&pop).expect("mapped popup control");
        let (tx, ty) = pop.surface_transform();
        let (mut x, mut y) = (
            r.x() as f64 + r.width() as f64 * fraction - tx,
            r.y() as f64 + r.height() as f64 / 2. - ty,
        );
        let mut surface = pop.surface();
        while let Some(p) = surface.and_then(|s| s.downcast::<gdk::Popup>().ok()) {
            x += p.position_x() as f64;
            y += p.position_y() as f64;
            surface = PopupExt::parent(&p);
        }
        (x, y)
    } else {
        let r = w.compute_bounds(root).expect("mapped window control");
        let (tx, ty) = root.surface_transform();
        (
            r.x() as f64 + r.width() as f64 * fraction + tx,
            r.y() as f64 + r.height() as f64 / 2. + ty,
        )
    };
    ((x * scale).round() as i32, (y * scale).round() as i32)
}
fn followup_caret(t: &Followup, pos: Pos) -> (i32, i32) {
    let mut v = t.ui.current.borrow_mut();
    let d = v.as_mut().unwrap();
    let c = d.session.layout().caret_on(&pos, 0).unwrap();
    let r = d.page_frames[c.page].compute_bounds(&t.window).unwrap();
    let (tx, ty) = t.window.surface_transform();
    let scale = t.window.scale_factor() as f64;
    (
        ((r.x() as f64 + (c.x as f64 + 0.2) * d.zoom as f64 + tx) * scale).round() as i32,
        ((r.y() as f64 + (c.top + c.height / 2.) as f64 * d.zoom as f64 + ty) * scale).round()
            as i32,
    )
}
fn followup_emit(stage: u8, role: &str, points: &[(i32, i32)]) {
    let values = points
        .iter()
        .map(|(x, y)| format!("{x} {y}"))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("ISSUE19_READY {stage} {role} {values}");
}
fn followup_wait(t: Rc<Followup>, stage: u8) {
    let attempts = Cell::new(0);
    let directory =
        std::env::var("LEAF19_ACK_DIR").expect("pointer driver supplies acknowledgements");
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        attempts.set(attempts.get() + 1);
        assert!(
            attempts.get() < 200,
            "pointer driver stalled at {} stage {stage}",
            t.mode
        );
        if !std::path::Path::new(&directory)
            .join(stage.to_string())
            .exists()
        {
            return glib::ControlFlow::Continue;
        }
        // Drain native input/layout sources before checking the driver's ACK.
        // A busy repagination can leave X events queued behind this timeout.
        let t = t.clone();
        glib::idle_add_local_once(move || {
            followup_check(&t, stage);
            glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                followup_stage(t, stage + 1)
            });
        });
        glib::ControlFlow::Break
    });
}
pub(super) fn followup_pointer_test(
    ui: &Rc<Ui>,
    w: &ApplicationWindow,
    app: &Application,
    menu: &gtk::MenuButton,
    mode: String,
) {
    if mode == "font" {
        let weak = Rc::downgrade(ui);
        ui.size_input.connect_changed(move |field| {
            if let Some(ui)=weak.upgrade() {
                let focus=ui.host.upgrade().and_then(|h|gtk::prelude::GtkWindowExt::focus(&h.window));
                eprintln!("ISSUE19_SIZE_INPUT text={:?} cursor={} selected={:?} focused={} syncing={} numeric={} root_focus={:?}",field.text(),field.position(),field.selection_bounds(),control_has_focus(field),ui.format_syncing.get(),field.is_numeric(),focus.map(|w|(w.type_().name().to_string(),w.widget_name().to_string())));
            }
        });
    }
    let text = if mode == "paragraph" {
        (0..150).map(|n|format!("Paragraph {n}. Words to select as a complete paragraph, including its final boundary.")).collect::<Vec<_>>().join("\n")
    } else if mode == "link" {
        "before linked words spanning a visual line in the document with a second portion of the link continuing here after the wrap, followed by plain text".into()
    } else {
        "alpha beta gamma delta".into()
    };
    let text = if mode == "font" && std::env::var("LEAF19_REFLOW").is_ok() {
        format!("alpha beta gamma delta\n{}",(0..80).map(|n|format!("Reflow paragraph {n}: ordinary body text that grows and shrinks while a font field is edited.")).collect::<Vec<_>>().join("\n"))
    } else {
        text
    };
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(&text)),
        "Followup regression",
    );
    let family = ui
        .document_box
        .pango_context()
        .list_families()
        .iter()
        .map(|f| f.name().to_string())
        .find(|f| f == "DejaVu Sans" || f == "Liberation Sans" || f == "Cantarell")
        .unwrap_or("sans".into());
    if mode == "font" {
        let mut v = ui.current.borrow_mut();
        let s = &mut v.as_mut().unwrap().session;
        if std::env::var("LEAF19_REFLOW").is_ok() {
            selection::select_all(s).unwrap();
        } else if std::env::var("LEAF19_RANGE").as_deref() != Ok("caret") {
            selection::select_range(s, &Pos::body(0, 6), &Pos::body(0, 10)).unwrap();
        } else {
            selection::set_caret(s, &Pos::body(0, 6), false).unwrap();
        }
    }
    if mode == "link" {
        let mut v = ui.current.borrow_mut();
        let s = &mut v.as_mut().unwrap().session;
        selection::select_range(s, &Pos::body(0, 7), &Pos::body(0, 128)).unwrap();
        wordcraft_gnome::links::apply(s, "https://example.org/original").unwrap();
        selection::set_caret(s, &Pos::body(0, 20), false).unwrap();
    }
    refresh_selection(ui);
    ui.document_box.grab_focus();
    w.set_title(Some("Leaf Followup Regression"));
    let t = Rc::new(Followup {
        ui: ui.clone(),
        window: w.clone(),
        app: app.clone(),
        menu: menu.clone(),
        initial_pages: ui.current.borrow().as_ref().unwrap().sizes.len(),
        body: text,
        mode,
        family,
    });
    eprintln!("ISSUE19_FAMILY {}", t.family);
    if std::env::var("LEAF19_NARROW").is_ok() {
        w.set_default_size(440, 800);
        ui.sidebar.set_show_sidebar(false);
        ui.sidebar_button.set_active(false);
    }
    let next = t.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(700), move || {
        followup_stage(next, 0)
    });
}
fn followup_stage(t: Rc<Followup>, stage: u8) {
    let u = &t.ui;
    t.window.set_title(Some("Leaf Followup Regression"));
    match t.mode.as_str() {
        "font" => match stage {
            0 => followup_emit(stage, "open", &[followup_point(&t.menu, &t.window)]),
            1 | 5 => {
                if stage == 5 {
                    u.font_input.clipboard().set_text(&(t.family.clone() + "x"));
                }
                followup_emit(
                    stage,
                    if stage == 1 { "font" } else { "paste" },
                    &[followup_point(&u.font_input, &t.window)],
                );
            }
            2 | 6 => {
                followup_emit(
                    stage,
                    if stage == 2 { "size" } else { "comma" },
                    &[followup_point_at(&u.size_input, &t.window, 0.1)],
                );
            }
            3 | 7 => {
                let menu = find_widget(&t.menu.popover().unwrap(), "text-font-choices").unwrap();
                followup_emit(stage, "choices", &[followup_point(&menu, &t.window)]);
            }
            4 => followup_emit(
                stage,
                "pin",
                &[followup_point(
                    &popover_pin(&t.menu.popover().unwrap()),
                    &t.window,
                )],
            ),
            8 => {
                followup_emit(stage, "return", &[followup_caret(&t, Pos::body(0, 22))]);
            }
            9 => {
                eprintln!(
                    "ISSUE19_FONT_OK real_open=true installed_family=true slow_size=true paste=true comma=true pinned=true width=true canvas=true"
                );
                t.app.quit();
                return;
            }
            _ => unreachable!(),
        },
        "paragraph" => match stage {
            0 => followup_emit(
                stage,
                "triple",
                &[
                    followup_caret(&t, Pos::body(1, 15)),
                    followup_caret(&t, Pos::body(3, 20)),
                ],
            ),
            1 => followup_emit(
                stage,
                "triple",
                &[
                    followup_caret(&t, Pos::body(2, 15)),
                    followup_caret(&t, Pos::body(0, 20)),
                ],
            ),
            2 => {
                let r = u.scroll.compute_bounds(&t.window).unwrap();
                let scale = t.window.scale_factor() as f64;
                followup_emit(
                    stage,
                    "autoscroll",
                    &[
                        followup_caret(&t, Pos::body(1, 15)),
                        (
                            ((r.x() + r.width() / 2.) as f64 * scale) as i32,
                            ((r.y() + r.height() - 8.) as f64 * scale) as i32,
                        ),
                    ],
                );
            }
            3 => {
                eprintln!(
                    "ISSUE19_PARAGRAPH_OK forward=true reverse=true autoscroll=true cross_page=true history_unchanged=true"
                );
                t.app.quit();
                return;
            }
            _ => unreachable!(),
        },
        "link" => match stage {
            0 | 4 => {
                u.document_box.grab_focus();
                followup_emit(stage, "shortcut", &[]);
            }
            1 => {
                let w = u
                    .link_dialog
                    .borrow()
                    .clone()
                    .expect("Ctrl+K opened link editor");
                let dest = find_widget(&w, "link-destination").unwrap();
                let apply = find_widget(&w, "link-apply").unwrap();
                assert_eq!(
                    dest.clone().downcast::<gtk::Entry>().unwrap().text(),
                    "https://example.org/original"
                );
                followup_emit(
                    stage,
                    "edit",
                    &[followup_point(&dest, &w), followup_point(&apply, &w)],
                );
            }
            2 => {
                u.document_box.grab_focus();
                followup_emit(stage, "context", &[followup_caret(&t, Pos::body(0, 90))]);
            }
            3 => {
                let w = u
                    .link_dialog
                    .borrow()
                    .clone()
                    .expect("right click selected Link");
                assert_eq!(
                    find_widget(&w, "link-destination")
                        .unwrap()
                        .downcast::<gtk::Entry>()
                        .unwrap()
                        .text(),
                    "https://example.org/edited?a=1&b=<value>"
                );
                let cancel = find_widget(&w, "link-cancel").unwrap();
                followup_emit(stage, "cancel", &[followup_point(&cancel, &w)]);
            }
            5 => {
                let w = u
                    .link_dialog
                    .borrow()
                    .clone()
                    .expect("Ctrl+K insertion editor");
                let text = find_widget(&w, "link-display-text").unwrap();
                let dest = find_widget(&w, "link-destination").unwrap();
                let apply = find_widget(&w, "link-apply").unwrap();
                followup_emit(
                    stage,
                    "insert",
                    &[
                        followup_point(&text, &w),
                        followup_point(&dest, &w),
                        followup_point(&apply, &w),
                    ],
                );
            }
            6 => {
                followup_emit(stage, "hover", &[followup_caret(&t, Pos::body(0, 20))]);
            }
            7 => {
                eprintln!(
                    "ISSUE19_LINK_OK shortcut=true whole_range=true context=true cancel=true insert=true hover=true"
                );
                t.app.quit();
                return;
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
    followup_wait(t, stage);
}
fn followup_font_labels(root: &impl IsA<gtk::Widget>, out: &mut Vec<gtk::Label>) {
    let w = root.as_ref();
    if w.widget_name() == "font-preview" {
        if let Some(label) = w.downcast_ref::<gtk::Label>() {
            out.push(label.clone());
        }
    }
    let mut child = w.first_child();
    while let Some(c) = child {
        followup_font_labels(&c, out);
        child = c.next_sibling();
    }
}
fn followup_check(t: &Followup, stage: u8) {
    let u = &t.ui;
    match t.mode.as_str() {
        "font" => {
            if stage == 1 || stage == 5 {
                assert!(
                    u.link_dialog.borrow().is_none(),
                    "Ctrl+K inside a native field must stay in that field"
                );
                assert_eq!(u.font_input.text(), t.family);
                assert_eq!(selection_character(u)["font"], t.family);
            }
            if stage == 2 {
                assert_eq!(selection_character(u)["size"], 17.5);
                if std::env::var("LEAF19_REFLOW").is_ok() {
                    assert_ne!(
                        u.current.borrow().as_ref().unwrap().sizes.len(),
                        t.initial_pages,
                        "test must actually rebuild the page set while typing"
                    );
                    assert!(
                        control_has_focus(&u.size_input),
                        "typed size keeps focus across a page-set rebuild"
                    );
                }
            }
            if stage == 6 {
                assert_eq!(
                    selection_character(u)["size"],
                    if std::env::var("LEAF19_LARGE").is_ok() {
                        64.5
                    } else {
                        18.75
                    }
                );
            }
            if stage == 3 || stage == 7 {
                let menu = find_widget(&t.menu.popover().unwrap(), "text-font-choices")
                    .unwrap()
                    .downcast::<gtk::MenuButton>()
                    .unwrap();
                let pop = menu.popover().unwrap();
                assert!(pop.is_visible());
                let mut previews = Vec::new();
                followup_font_labels(&pop, &mut previews);
                assert_eq!(
                    previews.iter().filter(|l| l.text() == t.family).count(),
                    1,
                    "one visible name per family"
                );
                let preview = previews.into_iter().find(|l| l.text() == t.family).unwrap();
                eprintln!(
                    "ISSUE19_FONT_GEOMETRY popup={} preview={} ellipsized={}",
                    pop.width(),
                    preview.width(),
                    preview.layout().is_ellipsized()
                );
                assert!(pop.width() >= 260, "font popup must have useful width");
                assert!(
                    preview.width() >= 200,
                    "font labels may not collapse into ellipses"
                );
            }
            if stage == 4 {
                assert!(popover_pin(&t.menu.popover().unwrap()).is_active());
            }
            let text = u
                .current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .doc
                .plain_text(wordcraft_doc::StoryRef::Body);
            if stage < 8 {
                assert_eq!(text, t.body);
            } else {
                assert_eq!(
                    text,
                    t.body
                        .replacen("alpha beta gamma delta", "alpha beta gamma deltaQ", 1)
                );
            }
        }
        "paragraph" => {
            let mut v = u.current.borrow_mut();
            let s = &mut v.as_mut().unwrap().session;
            let (a, b) = s.sel.ordered();
            assert_eq!(a.off, 0);
            assert_eq!(b.off, 0);
            if stage == 0 {
                assert_eq!(
                    (a.path.clone(), b.path.clone()),
                    (Pos::body(1, 0).path, Pos::body(4, 0).path)
                );
            }
            if stage == 1 {
                assert_eq!(
                    (a.path.clone(), b.path.clone()),
                    (Pos::body(0, 0).path, Pos::body(3, 0).path)
                );
            }
            if stage == 2 {
                assert!(
                    s.layout().caret_on(&b, 0).unwrap().page
                        > s.layout().caret_on(&a, 0).unwrap().page,
                    "autoscroll must cross a real page boundary"
                );
            }
            assert!(!s.dirty);
            assert_eq!(s.undo_depth(), 0);
        }
        "link" => match stage {
            1 => {
                let v = u.current.borrow();
                let s = &v.as_ref().unwrap().session;
                for off in [7, 20, 90, 127] {
                    assert_eq!(
                        wordcraft_gnome::links::at(s, &Pos::body(0, off)).as_deref(),
                        Some("https://example.org/edited?a=1&b=<value>")
                    );
                }
                assert!(u.link_dialog.borrow().is_none());
            }
            3 => {
                assert!(u.link_dialog.borrow().is_none());
                let mut v = u.current.borrow_mut();
                let s = &mut v.as_mut().unwrap().session;
                assert_eq!(s.undo_depth(), 2, "cancel may not edit");
                selection::set_caret(s, &Pos::body(0, 0), false).unwrap();
            }
            5 => {
                let mut v = u.current.borrow_mut();
                let s = &mut v.as_mut().unwrap().session;
                assert_eq!(s.undo_depth(), 3);
                assert!(
                    s.doc
                        .plain_text(wordcraft_doc::StoryRef::Body)
                        .starts_with("New linkbefore")
                );
                s.run("edit.undo", &serde_json::json!({})).unwrap();
                assert!(
                    s.doc
                        .plain_text(wordcraft_doc::StoryRef::Body)
                        .starts_with("before")
                );
                selection::set_caret(s, &Pos::body(0, 0), false).unwrap();
                drop(v);
                refresh_selection(u);
            }
            6 => {
                assert_eq!(
                    u.current.borrow().as_ref().unwrap().session.sel.focus,
                    Pos::body(0, 0),
                    "hover must not move caret"
                );
            }
            _ => {}
        },
        _ => unreachable!(),
    }
}
