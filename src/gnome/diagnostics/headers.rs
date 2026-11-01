use super::*;

pub(super) fn header_footer_ui_test(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    app: &Application,
    pointer: bool,
) {
    let text = (0..150)
        .map(|i| format!("Paragraph {i}: test body text.\n"))
        .collect::<String>();
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(&text)),
        "Page stories",
    );
    let body_text = ui
        .current
        .borrow()
        .as_ref()
        .unwrap()
        .session
        .doc
        .plain_text(wordcraft_doc::StoryRef::Body);
    let back = {
        let mut v = ui.current.borrow_mut();
        let d = v.as_mut().unwrap();
        selection::set_caret(&mut d.session, &Pos::body(100, 5), false).unwrap();
        d.session.sel.clone()
    };
    header_change(ui, |s| header_footer::enter(s, 1, Region::Header));
    apply_user_edit(ui, Edit::Insert("Editable page header".into()));
    assert!(
        ui.current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .in_header_footer()
    );
    assert!(find_widget(&ui.sidebar, "close-header-footer").is_none());
    header_change(ui, header_footer::close);
    assert!(
        !ui.current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .in_header_footer()
    );
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel, back);
    let dialog = show_page_number(ui, parent);
    dialog.total.set_active(true);
    dialog.insert.emit_clicked();
    assert!(!dialog.window.is_visible());
    assert_eq!(
        ui.current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .sel
            .focus
            .story,
        wordcraft_doc::StoryRef::Body
    );
    apply_user_edit(ui, Edit::Undo);
    header_change(ui, |s| header_footer::enter(s, 1, Region::Footer));
    let dialog = show_page_number(ui, parent);
    dialog.total.set_active(true);
    dialog.insert.emit_clicked();
    assert_eq!(
        ui.current
            .borrow()
            .as_ref()
            .unwrap()
            .session
            .sel
            .focus
            .story,
        ui.current.borrow().as_ref().unwrap().session.doc.sections()[0]
            .1
            .footers
            .default
            .map(wordcraft_doc::StoryRef::Part)
            .unwrap()
    );
    header_change(ui, header_footer::close);
    header_change(ui, |s| header_footer::enter(s, 1, Region::Header));
    ui.dirty.set(false);
    ui.current.borrow_mut().as_mut().unwrap().session.dirty = false;
    let u = ui.clone();
    let w = parent.clone();
    let a = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
        let (expected, x, y) = {
            let mut value = u.current.borrow_mut();
            let d = value.as_mut().unwrap();
            let mut pos = d.session.doc.start_of(d.session.sel.focus.story);
            pos.off = 1;
            let c = d.session.layout().caret_on(&pos, 1).unwrap();
            let frame = d.page_frames[1].compute_bounds(&u.document_box).unwrap();
            let canvas = u.document_box.compute_bounds(&w).unwrap();
            let x = canvas.x() as f64 + frame.x() as f64 + c.x as f64 * d.zoom as f64;
            let y = canvas.y() as f64
                + frame.y() as f64
                + (c.top + c.height / 2.0) as f64 * d.zoom as f64;
            (pos, x, y)
        };
        if pointer {
            eprintln!("HEADER_POINTER_READY {} {}", x as i32, y as i32);
        } else {
            let mut v = u.current.borrow_mut();
            selection::set_caret(&mut v.as_mut().unwrap().session, &expected, false).unwrap();
        }
        glib::timeout_add_local_once(
            std::time::Duration::from_secs(if pointer { 2 } else { 0 }),
            move || {
                {
                    let value = u.current.borrow();
                    let d = value.as_ref().unwrap();
                    assert_eq!(d.session.sel.focus.story, expected.story);
                    assert_eq!(d.session.page_hint, 1);
                    assert!(d.session.sel.focus.off.abs_diff(expected.off) <= 1);
                    assert!(!d.session.dirty);
                }
                assert!(!u.dirty.get());
                if pointer {
                    let r = u.scroll.compute_bounds(&w).unwrap();
                    eprintln!(
                        "HEADER_RETURN_READY {} {}",
                        (r.x() + 2.0) as i32,
                        (r.y() + r.height() / 2.0) as i32
                    );
                } else {
                    assert_eq!(leave_header_outside_at(&u, -5.0, -5.0), Some(false));
                    refresh_selection(&u);
                }
                glib::timeout_add_local_once(
                    std::time::Duration::from_secs(if pointer { 2 } else { 0 }),
                    move || {
                        assert!(
                            !u.current
                                .borrow()
                                .as_ref()
                                .unwrap()
                                .session
                                .in_header_footer()
                        );
                        let v = u.current.borrow();
                        let d = v.as_ref().unwrap();
                        assert_eq!(d.session.sel, back);
                        assert_eq!(
                            d.session.doc.plain_text(wordcraft_doc::StoryRef::Body),
                            body_text
                        );
                        eprintln!("HEADER_FOOTER_UI_OK pointer={pointer}");
                        a.quit();
                    },
                );
            },
        );
    });
}
