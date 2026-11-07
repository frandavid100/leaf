//! Native control and viewport regressions for #9 and #156.
use super::*;
use serde_json::json;

async fn settle(ms: u64) {
    glib::timeout_future(std::time::Duration::from_millis(ms)).await;
}
fn caret_view(ui: &Ui) -> (usize, f64, f64, f64) {
    let mut current = ui.current.borrow_mut();
    let d = current.as_mut().unwrap();
    let c = d
        .session
        .layout()
        .caret_on(&d.session.sel.focus, d.session.page_hint)
        .unwrap();
    let frame = d.page_frames[c.page]
        .compute_bounds(&ui.document_box)
        .unwrap();
    let y = frame.y() as f64 + c.top as f64 * d.page_zoom(c.page) as f64;
    let a = ui.scroll.vadjustment();
    (c.page, y, a.value(), a.page_size())
}
fn assert_visible(ui: &Ui, label: &str) {
    let (page, y, scroll, height) = caret_view(ui);
    eprintln!("ISSUE156_VIEW {label} page={page} caret_y={y} scroll={scroll} height={height}");
    assert!(
        y >= scroll - 1. && y <= scroll + height + 1.,
        "caret/viewport diverged: {label} page={page} caret={y} scroll={scroll} height={height}"
    );
}
async fn key(ui: &Rc<Ui>, window: &ApplicationWindow, stage: usize, value: &str, click: bool) {
    let (x, y) = if click {
        let mut current = ui.current.borrow_mut();
        let d = current.as_mut().unwrap();
        let c = d
            .session
            .layout()
            .caret_on(&d.session.sel.focus, d.session.page_hint)
            .unwrap();
        let frame = d.page_frames[c.page].compute_bounds(window).unwrap();
        let z = d.page_zoom(c.page);
        let scale = window.scale_factor() as f32;
        let (tx, ty) = window.surface_transform();
        (
            ((frame.x() + (c.x + 2.) * z + tx as f32) * scale) as i32,
            ((frame.y() + (c.top + c.height / 2.) * z + ty as f32) * scale) as i32,
        )
    } else {
        (-1, -1)
    };
    eprintln!("PENDING_KEY {stage} {value} {x} {y}");
    let dir = std::env::var("LEAF_PENDING_ACK").unwrap();
    for _ in 0..1000 {
        if Path::new(&dir).join(stage.to_string()).exists() {
            settle(250).await;
            return;
        }
        settle(10).await;
    }
    panic!("keyboard driver did not acknowledge stage {stage}");
}
fn later_position(ui: &Ui, fraction: f32) -> (usize, Pos) {
    use wordcraft_engine::layout::Placed;
    let mut current = ui.current.borrow_mut();
    let s = &mut current.as_mut().unwrap().session;
    let layout = s.layout();
    let page = ((layout.pages.len() - 1) as f32 * fraction) as usize;
    let p = &layout.pages[page];
    let (path, para, l0, l1) = p
        .items
        .iter()
        .find_map(|it| {
            if let Placed::Lines {
                story: wordcraft_doc::StoryRef::Body,
                path,
                para,
                l0,
                l1,
                ..
            } = it
            {
                Some((path, para, *l0, *l1))
            } else {
                None
            }
        })
        .unwrap();
    let line = &para.lines[(l0 + l1) / 2];
    let mut off = (line.start + line.stop) / 2;
    let model = s.doc.para(wordcraft_doc::StoryRef::Body, path).unwrap();
    while off > line.start && !model.text.is_char_boundary(off) {
        off -= 1;
    }
    (
        page,
        Pos {
            story: wordcraft_doc::StoryRef::Body,
            path: path.clone(),
            off,
        },
    )
}
pub(super) fn pending_end_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let (ui, window, app) = (ui.clone(), window.clone(), app.clone());
    glib::spawn_future_local(async move {
        window.set_default_size(1100, 850);
        ui.document_box
            .settings()
            .set_property("gtk-enable-animations", false);
        let text = (0..160)
            .map(|i| {
                format!(
                    "Paragraph {i}: {}\n",
                    "ordinary text for wrapped visual lines and later pages. ".repeat(8)
                )
            })
            .collect::<String>();
        opened(
            &ui,
            Session::new(wordcraft_doc::Document::from_text(&text)),
            "Pending End checks",
        );
        window.set_title(Some("Leaf Pending Issues"));
        settle(300).await;
        let mut stage = 0;
        for zoom in [0.5, 1., 1.5] {
            zoom_to(&ui, zoom);
            settle(250).await;
            for (value, home, extend) in [
                ("End", false, false),
                ("Home", true, false),
                ("shift+End", false, true),
                ("shift+Home", true, true),
            ] {
                let (page, from) = later_position(&ui, 0.7);
                let expected = {
                    let mut current = ui.current.borrow_mut();
                    let d = current.as_mut().unwrap();
                    selection::set_caret(&mut d.session, &from, false).unwrap();
                    d.session.page_hint = page;
                    let (a, b) = d.session.layout().line_bounds(&from, page).unwrap();
                    if home { a } else { b }
                };
                refresh_selection(&ui);
                caret_ensure_visible(&ui);
                ui.document_box.grab_focus();
                settle(150).await;
                let (doc, depth, dirty) = {
                    let c = ui.current.borrow();
                    let s = &c.as_ref().unwrap().session;
                    (s.doc.clone(), s.undo_depth(), s.dirty)
                };
                key(&ui, &window, stage, value, value == "End").await;
                stage += 1;
                {
                    let c = ui.current.borrow();
                    let s = &c.as_ref().unwrap().session;
                    assert_eq!(s.sel.focus, expected, "{value} zoom={zoom}");
                    if extend {
                        assert_eq!(s.sel.anchor, from);
                    }
                    assert_eq!(s.doc, doc);
                    assert_eq!(s.undo_depth(), depth);
                    assert_eq!(s.dirty, dirty);
                }
                assert_visible(&ui, &format!("{value}:zoom={zoom}"));
            }
        }
        // Queue a zoom allocation, then a newer navigation request before its idle.
        // A stale view restoration must not override the newer caret reveal.
        let (_, old) = later_position(&ui, 0.1);
        {
            let mut c = ui.current.borrow_mut();
            selection::set_caret(&mut c.as_mut().unwrap().session, &old, false).unwrap();
        }
        refresh_selection(&ui);
        caret_ensure_visible(&ui);
        settle(150).await;
        let (_, target) = later_position(&ui, 0.8);
        zoom_to(&ui, 0.85);
        {
            let mut c = ui.current.borrow_mut();
            selection::set_caret(&mut c.as_mut().unwrap().session, &target, false).unwrap();
        }
        selected_command(&ui, "end", false);
        settle(500).await;
        assert_visible(&ui, "newer-navigation-after-queued-zoom");
        eprintln!("PENDING_END_UI_OK keyboard=true shift=true viewport=true zoom_race=true");
        app.quit();
    });
}
pub(super) fn pending_spacing_ui_test(ui: &Rc<Ui>, app: &Application) {
    let (ui, app) = (ui.clone(), app.clone());
    glib::spawn_future_local(async move {
        let mut baseline = None;
        for previous_after in [0.0f64, 5.5] {
            let mut session = Session::new(wordcraft_doc::Document::from_text(
                "Preceding paragraph.\nFollowing paragraph.",
            ));
            session
                .run("para.spacing", &json!({"before":0.,"after":previous_after}))
                .unwrap();
            selection::set_caret(&mut session, &Pos::body(1, 0), false).unwrap();
            session
                .run("para.spacing", &json!({"before":0.,"after":0.}))
                .unwrap();
            opened(&ui, session, "Pending spacing checks");
            settle(150).await;
            let panel = ui.paragraph.borrow().as_ref().unwrap().clone();
            panel.header.pin.set_active(true);
            let menu = format_button(&ui, "paragraph-menu").unwrap();
            menu.popup();
            panel.page("spacing");
            settle(150).await;
            let spin = find_widget(&panel.detail, "paragraph-spaceBefore")
                .unwrap()
                .downcast::<gtk::SpinButton>()
                .unwrap();
            for (index, value) in [0., 1., 3., 5., 5.5, 6., 24., 6., 5.5, 5., 3., 1., 0.]
                .into_iter()
                .enumerate()
            {
                spin.grab_focus();
                spin.set_text(&if value == 5.5 {
                    "5,5".into()
                } else {
                    format!("{value}")
                });
                settle(450).await;
                let state = selection_paragraph(&ui);
                let actual = state["spaceBefore"].as_f64().unwrap();
                assert!(
                    (actual - value).abs() < 0.001,
                    "typed Before did not commit: wanted={value}, actual={actual}"
                );
                let delta = {
                    let mut c = ui.current.borrow_mut();
                    let s = &mut c.as_mut().unwrap().session;
                    let l = s.layout();
                    l.caret_on(&Pos::body(1, 0), 0).unwrap().top
                        - l.caret_on(&Pos::body(0, 0), 0).unwrap().top
                };
                let base = *baseline.get_or_insert(delta);
                assert!(
                    (delta as f64 - base as f64 - previous_after.max(value)).abs() < 0.05,
                    "spacing geometry does not match adjacent values"
                );
                eprintln!(
                    "ISSUE9_CONTROL after={previous_after} before={value} committed={actual} delta={delta}"
                );
                if index == 3 {
                    spin.spin(gtk::SpinType::StepForward, 1.);
                    settle(100).await;
                    assert_eq!(selection_paragraph(&ui)["spaceBefore"], json!(5.5));
                }
            }
            menu.popover().unwrap().popdown();
        }
        eprintln!(
            "PENDING_SPACING_UI_OK typing=true commas=true steppers=true below_threshold=true plateau=true"
        );
        app.quit();
    });
}
