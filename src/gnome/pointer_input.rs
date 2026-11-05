//! Document pointer selection, input methods, zoom and viewport signals.
use super::*;

fn document_pointer_surface(ui: &Ui, controller: &impl IsA<gtk::EventController>) -> bool {
    let Some(surface) = controller
        .as_ref()
        .current_event()
        .and_then(|e| e.surface())
    else {
        return false;
    };
    ui.document_box.native().and_then(|n| n.surface()).as_ref() == Some(&surface)
}

pub(super) fn install_pointer_and_view_input(
    ui: &Rc<Ui>,
    compact_app_menu: &gtk::MenuButton,
    compact_search: &gtk::Button,
) {
    install_table_hover(ui);
    install_link_tooltip(ui);
    let hover = gtk::EventControllerMotion::new();
    let weak = Rc::downgrade(ui);
    let owns_cursor = Cell::new(false);
    hover.connect_motion(move |_, x, y| {
        if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
            if ui.picture_drag.borrow().is_none() {
                if let Some(cursor) = picture_hover(&ui, x, y) {
                    ui.document_box.set_cursor_from_name(Some(cursor));
                    owns_cursor.set(true);
                } else if owns_cursor.replace(false) {
                    ui.document_box.set_cursor_from_name(None);
                }
            }
        }
    });
    ui.document_box.add_controller(hover);
    // The centered page stack can be narrower than its viewport. Its surrounding
    // background belongs to the scroller, so page-stack gestures never see it.
    let outside = gtk::GestureClick::new();
    outside.set_button(gdk::BUTTON_PRIMARY);
    outside.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = Rc::downgrade(ui);
    outside.connect_pressed(move |gesture, _, x, y| {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if !document_pointer_surface(&ui, gesture) {
            return;
        }
        let contains = |widget: &gtk::Widget| {
            widget.compute_bounds(&ui.scroll).is_some_and(|r| {
                x >= r.x() as f64
                    && x <= (r.x() + r.width()) as f64
                    && y >= r.y() as f64
                    && y <= (r.y() + r.height()) as f64
            })
        };
        if std::env::args().any(|a| a == "--headers-pointer-test") {
            eprintln!(
                "HEADER_VIEWPORT_CLICK x={x} y={y} document={} vertical={} horizontal={}",
                contains(ui.document_box.upcast_ref()),
                contains(&ui.scroll.vscrollbar()),
                contains(&ui.scroll.hscrollbar())
            );
        }
        if contains(ui.document_box.upcast_ref())
            || contains(&ui.scroll.vscrollbar())
            || contains(&ui.scroll.hscrollbar())
        {
            return;
        }
        let active = ui
            .current
            .borrow()
            .as_ref()
            .is_some_and(|d| d.session.in_header_footer());
        if active {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            header_change(&ui, header_footer::close);
        }
    });
    ui.scroll.add_controller(outside);
    // Click sets the caret, double-click selects a word and triple-click
    // selects a paragraph. Coordinates are relative to the document stack.
    {
        let click = gtk::GestureClick::new();
        click.set_button(gdk::BUTTON_PRIMARY);
        let document_box = ui.document_box.clone();
        let ui = ui.clone();
        click.connect_pressed(glib::clone!(
            #[weak]
            ui,
            move |gesture, count, x, y| {
                if !document_pointer_surface(&ui, gesture) {
                    gesture.set_state(gtk::EventSequenceState::Denied);
                    return;
                }
                if count == 1
                    && !gesture
                        .current_event_state()
                        .contains(gdk::ModifierType::SHIFT_MASK)
                    && follow_note_number_at(&ui, x, y)
                {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    return;
                }
                if count == 1
                    && picture_hover(&ui, x, y).is_some_and(|cursor| cursor.ends_with("resize"))
                {
                    ui.document_box.grab_focus();
                    return;
                }
                if table_pointer_press(&ui, count, x, y, gesture.current_event_state()) {
                    return;
                }
                restart_caret(&ui);
                ui.pointer_moved.set(false);
                ui.word_drag.borrow_mut().take();
                ui.paragraph_drag.borrow_mut().take();
                if leave_header_outside_at(&ui, x, y) == Some(false) {
                    ui.document_box.grab_focus();
                    refresh_selection(&ui);
                    return;
                }
                if count == 2 && enter_empty_header_at(&ui, x, y) {
                    return;
                }
                if count == 1
                    && !gesture
                        .current_event_state()
                        .contains(gdk::ModifierType::SHIFT_MASK)
                    && select_picture_at_canvas(&ui, x, y)
                {
                    ui.document_box.grab_focus();
                    refresh_selection(&ui);
                    return;
                }
                let Some(pos) = pos_at_canvas(&ui, x, y) else {
                    return;
                };
                if count == 2 {
                    let equation = {
                        let mut c = ui.current.borrow_mut();
                        let d = c.as_mut().unwrap();
                        let _ = selection::set_caret(&mut d.session, &pos, false);
                        wordcraft_gnome::equations::target(&d.session).is_some()
                    };
                    if equation {
                        if let Some(h) = ui.host.upgrade() {
                            show_equation(&ui, &h.window);
                        }
                        return;
                    }
                }
                if count == 2 {
                    let entered = {
                        let mut current = ui.current.borrow_mut();
                        current.as_mut().is_some_and(|d| {
                            selection::set_caret(&mut d.session, &pos, false).is_ok()
                                && wordcraft_gnome::references::open_footnote(&mut d.session, false)
                                    .is_ok()
                        })
                    };
                    if entered {
                        refresh_selection(&ui);
                        caret_ensure_visible(&ui);
                        return;
                    }
                }
                if count == 1
                    && gesture
                        .current_event_state()
                        .contains(gdk::ModifierType::CONTROL_MASK)
                {
                    if let Some(url) = document_link_at_point(&ui, x, y) {
                        launch_document_link(&ui, &url);
                        return;
                    }
                }
                ui.document_box.grab_focus();
                let shift = gesture
                    .current_event_state()
                    .contains(gdk::ModifierType::SHIFT_MASK);
                let result = {
                    let mut value = ui.current.borrow_mut();
                    let Some(doc) = value.as_mut() else { return };
                    let extend = shift && doc.session.sel.focus.story == pos.story;
                    selection::set_caret(&mut doc.session, &pos, extend).and_then(|_| {
                        let cmd = if count == 2 {
                            Some("select.word")
                        } else if count >= 3 {
                            Some("select.paragraph")
                        } else {
                            None
                        };
                        if count == 2 {
                            if let Some(range) = selection::word_range(&doc.session, &pos) {
                                selection::select_range(&mut doc.session, &range.0, &range.1)?;
                                ui.word_drag.replace(Some(range));
                                return Ok(());
                            }
                        }
                        if let Some(cmd) = cmd {
                            doc.session
                                .run(cmd, &serde_json::json!({}))
                                .map_err(|e| e.to_string())?;
                            if count >= 3 {
                                ui.paragraph_drag.replace(Some((
                                    doc.session.sel.anchor.clone(),
                                    doc.session.sel.focus.clone(),
                                )));
                            }
                        }
                        Ok(())
                    })
                };
                if let Err(e) = result {
                    ui.toasts.add_toast(Toast::new(&e));
                }
                refresh_selection(&ui);
            }
        ));
        let weak = Rc::downgrade(&ui);
        click.connect_released(move |gesture, count, x, y| {
            let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                return;
            };
            if !document_pointer_surface(&ui, gesture) {
                return;
            }
            if table_pointer_active(&ui) {
                return;
            }
            if count != 1
                || ui.pointer_moved.get()
                || gesture
                    .current_event_state()
                    .contains(gdk::ModifierType::SHIFT_MASK)
            {
                return;
            }
            let Some(pos) = pos_at_canvas(&ui, x, y) else {
                return;
            };
            let ids = ui
                .current
                .borrow()
                .as_ref()
                .map(|d| wordcraft_gnome::annotations::threads_at(&d.session, &pos))
                .unwrap_or_default();
            if !ids.is_empty() {
                if let Some(panel) = ui.annotations.borrow().as_ref() {
                    panel.reveal(ids);
                }
            }
        });
        document_box.add_controller(click);
    }
    // One drag gesture for the entire document, including page boundaries.
    {
        let drag = gtk::GestureDrag::new();
        drag.set_button(gdk::BUTTON_PRIMARY);
        {
            let ui = ui.clone();
            drag.connect_drag_begin(glib::clone!(
                #[weak]
                ui,
                move |gesture, x, y| {
                    if !document_pointer_surface(&ui, gesture) {
                        gesture.set_state(gtk::EventSequenceState::Denied);
                        return;
                    }
                    if table_pointer_active(&ui) {
                        return;
                    }
                    if picture_drag_begin(&ui, x, y) {
                        gesture.set_state(gtk::EventSequenceState::Claimed);
                        return;
                    }
                    *ui.drag_anchor.borrow_mut() = if gesture
                        .current_event_state()
                        .contains(gdk::ModifierType::SHIFT_MASK)
                    {
                        ui.current
                            .borrow()
                            .as_ref()
                            .map(|d| d.session.sel.anchor.clone())
                    } else {
                        pos_at_canvas(&ui, x, y)
                    };
                    ui.document_box.grab_focus();
                }
            ));
        }
        {
            let ui = ui.clone();
            drag.connect_drag_update(glib::clone!(
                #[weak]
                ui,
                move |gesture, dx, dy| {
                    if picture_drag_update(&ui, dx, dy) {
                        return;
                    }
                    if let Some((sx, sy)) = gesture.start_point() {
                        if table_pointer_update(
                            &ui,
                            sx + dx,
                            sy + dy,
                            gesture.current_event_state(),
                        ) {
                            return;
                        }
                    }
                    if !document_pointer_surface(&ui, gesture) || ui.drag_anchor.borrow().is_none()
                    {
                        return;
                    }
                    if dx.abs() <= 4.0 && dy.abs() <= 4.0 {
                        return;
                    }
                    ui.pointer_moved.set(true);
                    let Some((sx, sy)) = gesture.start_point() else {
                        return;
                    };
                    update_document_drag(&ui, sx + dx, sy + dy);
                    if let Some(r) = ui.document_box.compute_bounds(&ui.scroll) {
                        ui.drag_point
                            .set(Some((r.x() as f64 + sx + dx, r.y() as f64 + sy + dy)));
                        schedule_drag_scroll(&ui);
                    }
                }
            ));
        }
        {
            let ui = ui.clone();
            drag.connect_drag_end(glib::clone!(
                #[weak]
                ui,
                move |gesture, _, _| {
                    if picture_drag_finish(&ui) {
                        return;
                    }
                    if table_pointer_finish(&ui) {
                        return;
                    }
                    let document_drag =
                        document_pointer_surface(&ui, gesture) && ui.drag_anchor.borrow().is_some();
                    stop_document_drag(&ui);
                    if document_drag {
                        refresh_selection(&ui);
                    }
                }
            ));
        }
        let weak = Rc::downgrade(&ui);
        drag.connect_cancel(move |_, _| {
            if let Some(ui) = weak.upgrade() {
                picture_drag_cancel(&ui);
                table_pointer_cancel(&ui);
                stop_document_drag(&ui);
            }
        });
        ui.document_box.add_controller(drag);
    }

    {
        let weak = Rc::downgrade(&ui);
        ui.sidebar_button.connect_toggled(move |button| {
            if let Some(ui) = weak.upgrade() {
                ui.sidebar.set_show_sidebar(button.is_active());
            }
        });
    }
    {
        let ui = ui.clone();
        let split = ui.sidebar.clone();
        let compact_menu = compact_app_menu.clone();
        let compact_search = compact_search.clone();
        split.connect_show_sidebar_notify(glib::clone!(
            #[weak]
            ui,
            #[weak]
            compact_menu,
            #[weak]
            compact_search,
            move |split| {
                let visible = split.shows_sidebar();
                compact_menu.set_visible(!visible);
                compact_search.set_visible(!visible);
                if ui.sidebar_button.is_active() != visible {
                    ui.sidebar_button.set_active(visible);
                }
                if visible {
                    let weak = Rc::downgrade(&ui);
                    glib::idle_add_local_once(move || {
                        if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
                            refresh_thumbnails(&ui);
                            reveal_active_thumbnail(&ui);
                        }
                    });
                }
                // Ajustar al ancho sigue siendo un modo, no un zoom fijo. Tras
                // la animación del panel el ancho disponible puede cambiar.
                let weak = Rc::downgrade(&ui);
                glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
                    if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
                        refit_if_needed(&ui);
                    }
                });
            }
        ));
    }
    {
        let ui = ui.clone();
        ui.scroll.vadjustment().connect_value_changed(glib::clone!(
            #[weak]
            ui,
            move |_| refresh_pages(&ui)
        ));
    }
    {
        let weak = Rc::downgrade(&ui);
        ui.scroll.hadjustment().connect_changed(move |a| {
            if let Some(ui) = weak.upgrade() {
                let width = a.page_size().round() as i32;
                if width > 0 && ui.viewport_width.replace(width) != width {
                    if let Some(a) = ui.adaptive.borrow().as_ref() {
                        a.sync_width(width);
                    }
                    schedule_layout(&ui);
                }
            }
        });
    }
    {
        let ui = ui.clone();
        ui.thumbnails_scroll
            .vadjustment()
            .connect_value_changed(glib::clone!(
                #[weak]
                ui,
                move |_| refresh_thumbnails(&ui)
            ));
        let weak = Rc::downgrade(&ui);
        ui.thumbnails_scroll
            .vadjustment()
            .connect_changed(move |_| {
                let Some(ui) = weak.upgrade() else {
                    return;
                };
                if ui.thumbnail_bounds_pending.replace(true) {
                    return;
                }
                let weak = Rc::downgrade(&ui);
                glib::idle_add_local_once(move || {
                    if let Some(ui) = weak.upgrade() {
                        ui.thumbnail_bounds_pending.set(false);
                        if !ui.closed.get() {
                            refresh_thumbnails(&ui);
                        }
                    }
                });
            });
    }
    {
        let weak = Rc::downgrade(&ui);
        ui.zoom_out.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                zoom_step(&ui, false);
            }
        });
    }
    {
        let weak = Rc::downgrade(&ui);
        ui.zoom_in.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                zoom_step(&ui, true);
            }
        });
    }
}
