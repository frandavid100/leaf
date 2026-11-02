use super::*;

fn pointer_pos(ui: &Rc<Ui>, w: &ApplicationWindow, off: usize) -> (i32, i32) {
    let mut v = ui.current.borrow_mut();
    let d = v.as_mut().unwrap();
    let c = d.session.layout().caret_on(&Pos::body(0, off), 0).unwrap();
    let r = d.page_frames[0].compute_bounds(w).unwrap();
    let (tx, ty) = w.surface_transform();
    eprintln!(
        "POINTER_TARGET off={off} caret={} frame={} zoom={} transform={tx},{ty}",
        c.x,
        r.x(),
        d.zoom
    );
    (
        (r.x() as f64 + (c.x as f64 + 0.2) * d.zoom as f64 - tx).round() as i32,
        (r.y() as f64 + (c.top + c.height / 2.0) as f64 * d.zoom as f64 - ty).round() as i32,
    )
}
pub(super) fn interaction_pointer_test(
    ui: &Rc<Ui>,
    w: &ApplicationWindow,
    app: &Application,
    text: &gtk::MenuButton,
) {
    let trace = gtk::GestureClick::new();
    trace.set_button(gdk::BUTTON_PRIMARY);
    trace.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = Rc::downgrade(ui);
    trace.connect_pressed(move |gesture, count, x, y| {
        if let Some(ui) = weak.upgrade() {
            eprintln!(
                "POINTER_HIT count={count} x={x} y={y} modifiers={:?} pos={:?}",
                gesture.current_event_state(),
                pos_at_canvas(&ui, x, y)
            );
        }
    });
    ui.document_box.add_controller(trace);
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text("alpha beta gamma delta")),
        "Pointer regression",
    );
    let t = Rc::new(InteractionTest {
        ui: ui.clone(),
        window: w.clone(),
        app: app.clone(),
        text: text.clone(),
    });
    let next = t.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
        interaction_stage(next, 0)
    });
}
struct InteractionTest {
    ui: Rc<Ui>,
    window: ApplicationWindow,
    app: Application,
    text: gtk::MenuButton,
}
fn interaction_stage(t: Rc<InteractionTest>, stage: u8) {
    let u = &t.ui;
    t.window.set_title(Some("Leaf Interaction Regression"));
    match stage {
        0 => {
            let (ax, ay) = pointer_pos(u, &t.window, 2);
            let (bx, by) = pointer_pos(u, &t.window, 14);
            eprintln!("SHIFT_POINTER_READY {ax} {ay} {bx} {by}");
        }
        1 => {
            let (x, y) = pointer_pos(u, &t.window, 8);
            eprintln!("WORD_POINTER_READY {x} {y}");
        }
        2 => {
            let (ax, ay) = pointer_pos(u, &t.window, 14);
            let (bx, by) = pointer_pos(u, &t.window, 2);
            eprintln!("WORD_DRAG_READY {ax} {ay} {bx} {by}");
        }
        3 => {
            {
                let mut v = u.current.borrow_mut();
                let s = &mut v.as_mut().unwrap().session;
                selection::select_range(s, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
                wordcraft_gnome::links::apply(s, "https://example.org/leaf-test").unwrap();
                selection::set_caret(s, &Pos::body(0, 21), false).unwrap();
            }
            after_content_change(u);
            t.window.set_title(Some("Leaf Interaction Regression"));
            let (x, y) = pointer_pos(u, &t.window, 2);
            eprintln!("LINK_POINTER_READY {x} {y}");
        }
        4 | 5 | 6 => {
            if stage == 4 {
                {
                    let mut v = u.current.borrow_mut();
                    selection::select_range(
                        &mut v.as_mut().unwrap().session,
                        &Pos::body(0, 6),
                        &Pos::body(0, 10),
                    )
                    .unwrap();
                }
                refresh_selection(u);
                t.text.popup();
            } else {
                popover_pin(&t.text.popover().unwrap()).set_active(true);
            }
            if stage == 5 {
                u.font_input.clipboard().set_text("monospacex");
            }
            if stage == 6 {
                u.size_input.grab_focus();
            } else {
                u.font_input.grab_focus();
            }
            let next = t.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
                use gdk::prelude::PopupExt;
                let pop = next.text.popover().unwrap();
                let r = if stage == 6 {
                    next.ui.size_input.compute_bounds(&pop)
                } else {
                    next.ui.font_input.compute_bounds(&pop)
                }
                .unwrap();
                let popup = pop.surface().unwrap().downcast::<gdk::Popup>().unwrap();
                let (tx, ty) = pop.surface_transform();
                eprintln!(
                    "FONT_INPUT_READY_{stage} {} {}",
                    (popup.position_x() as f64
                        + r.x() as f64
                        + if stage == 6 {
                            14.0
                        } else {
                            r.width() as f64 / 2.0
                        }
                        - tx) as i32,
                    (popup.position_y() as f64 + r.y() as f64 + r.height() as f64 / 2.0 - ty)
                        as i32
                );
            });
        }
        _ => unreachable!(),
    }
    glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
        let u = &t.ui;
        {
            let mut v = u.current.borrow_mut();
            let s = &mut v.as_mut().unwrap().session;
            let selected = selection::copy_text(s).unwrap();
            match stage {
                0 => assert_eq!(selected, "pha beta gam"),
                1 => assert_eq!(selected, "beta"),
                2 => assert_eq!(selected, "alpha beta gamma"),
                3 => assert_eq!(
                    s.sel.focus,
                    Pos::body(0, 21),
                    "opening a link retains caret"
                ),
                4 | 5 | 6 => {
                    assert_eq!(
                        s.doc.plain_text(wordcraft_doc::StoryRef::Body),
                        "alpha beta gamma delta",
                        "typing belongs in the entry"
                    );
                    assert_eq!(selected, "beta");
                    if stage == 6 {
                        assert_eq!(wordcraft_engine::cmd::format::state(s)["size"], 17.5);
                    } else {
                        assert_eq!(
                            wordcraft_engine::cmd::format::state(s)["font"],
                            if stage == 4 { "serif" } else { "monospace" }
                        );
                    }
                }
                _ => unreachable!(),
            }
            if stage < 3 {
                assert_eq!(s.undo_depth(), 0);
                assert!(!s.dirty);
            }
        }
        if stage == 6 {
            eprintln!(
                "INTERACTION_POINTER_OK shift=true words=true link=true font_keyboard=true pinned=true"
            );
            t.app.quit();
        } else {
            interaction_stage(t, stage + 1);
        }
    });
}
