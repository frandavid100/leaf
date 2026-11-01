use super::*;

pub(super) fn install_pointer_and_view_input(
    ui: &Rc<Ui>,
    compact_app_menu: &gtk::MenuButton,
    compact_search: &gtk::Button,
) {
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
                ui.pointer_moved.set(false);
                ui.word_drag.borrow_mut().take();
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
                if count == 1
                    && gesture
                        .current_event_state()
                        .contains(gdk::ModifierType::CONTROL_MASK)
                {
                    if let Some(url) = link_at_canvas_position(&ui, &pos) {
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
                move |_, _, _| {
                    stop_document_drag(&ui);
                    refresh_selection(&ui);
                }
            ));
        }
        let weak = Rc::downgrade(&ui);
        drag.connect_cancel(move |_, _| {
            if let Some(ui) = weak.upgrade() {
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

pub(super) fn install_keyboard_input(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    host: &Rc<Workspace>,
    app: &Application,
    open_button: &gtk::Button,
) {
    {
        let ui = ui.clone();
        let open_button = open_button.clone();
        let save_parent = window.clone();
        let key_host = host.clone();
        let key_app = app.clone();
        let controller = gtk::EventControllerKey::new();
        // GTK supplies dead-key composition, accents and input methods.
        // Committed text enters WordCraft; ordinary shortcuts remain in
        // the key controller instead of being converted into characters.
        let input = gtk::IMMulticontext::new();
        input.set_client_widget(Some(&ui.document_box));
        // Attach the canvas IM only while the canvas owns keyboard focus.
        {
            let ui = ui.clone();
            input.connect_commit(glib::clone!(
                #[weak]
                ui,
                move |_, text| {
                    if ui.document_box.has_focus() && !text.is_empty() {
                        apply_user_edit(&ui, Edit::Insert(text.to_owned()));
                    }
                }
            ));
        }
        {
            let input = input.clone();
            let controller = controller.downgrade();
            ui.document_box.connect_notify_local(
                Some("has-focus"),
                glib::clone!(
                    #[weak]
                    input,
                    move |widget, _| {
                        if let Some(controller) = controller.upgrade() {
                            if widget.has_focus() {
                                controller.set_im_context(Some(&input));
                                input.focus_in();
                            } else {
                                input.focus_out();
                                controller.set_im_context(None::<&gtk::IMContext>);
                            }
                        }
                    }
                ),
            );
        }
        let key_target = ui.sidebar.clone();
        controller.connect_key_pressed(glib::clone!(
            #[weak]
            ui,
            #[weak]
            key_host,
            #[weak]
            key_app,
            #[weak]
            save_parent,
            #[weak]
            open_button,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, modifiers| {
                let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
                let control = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
                if ui.document_box.has_focus() {
                    if control && shift && key == gdk::Key::F5 {
                        show_annotation_dialog(&ui, AnnotationEdit::Bookmark);
                        return glib::Propagation::Stop;
                    }
                    if control
                        && modifiers.contains(gdk::ModifierType::ALT_MASK)
                        && matches!(key, gdk::Key::m | gdk::Key::M)
                    {
                        show_annotation_dialog(&ui, AnnotationEdit::NewComment);
                        return glib::Propagation::Stop;
                    }
                    if key == gdk::Key::Escape
                        && ui
                            .current
                            .borrow()
                            .as_ref()
                            .is_some_and(|d| d.session.in_header_footer())
                    {
                        header_change(&ui, header_footer::close);
                        return glib::Propagation::Stop;
                    }
                    if key == gdk::Key::Menu || (shift && key == gdk::Key::F10) {
                        let y = ui.scroll.vadjustment().value() as i32 + 40;
                        show_table_context_at(&ui, &save_parent, 60, y);
                        return glib::Propagation::Stop;
                    }
                    if (key == gdk::Key::Tab || key == gdk::Key::ISO_Left_Tab)
                        && !control
                        && !modifiers.contains(gdk::ModifierType::ALT_MASK)
                    {
                        let inside = ui
                            .current
                            .borrow()
                            .as_ref()
                            .is_some_and(|d| wordcraft_gnome::tables::in_table(&d.session));
                        if inside {
                            table_navigate(&ui, shift || key == gdk::Key::ISO_Left_Tab, control);
                            return glib::Propagation::Stop;
                        }
                    }
                    // Commands which change the document are resolved by
                    // WordCraft, including replacement of selected text.
                    if !control && !modifiers.contains(gdk::ModifierType::ALT_MASK) {
                        let change = match key {
                            gdk::Key::Return | gdk::Key::KP_Enter => Some(if shift {
                                Edit::LineBreak
                            } else {
                                Edit::Paragraph
                            }),
                            gdk::Key::BackSpace => Some(Edit::Backspace),
                            gdk::Key::Delete | gdk::Key::KP_Delete => Some(Edit::Delete),
                            _ => None,
                        };
                        if let Some(edit) = change {
                            apply_user_edit(&ui, edit);
                            return glib::Propagation::Stop;
                        }
                    }
                    let movement = match key {
                        gdk::Key::Left => Some(if control { "wordLeft" } else { "left" }),
                        gdk::Key::Right => Some(if control { "wordRight" } else { "right" }),
                        gdk::Key::Up => Some("up"),
                        gdk::Key::Down => Some("down"),
                        gdk::Key::Home => Some(if control { "docStart" } else { "home" }),
                        gdk::Key::End => Some(if control { "docEnd" } else { "end" }),
                        _ => None,
                    };
                    if let Some(action) = movement {
                        selected_command(&ui, action, shift);
                        return glib::Propagation::Stop;
                    }
                }
                if key == gdk::Key::F9 {
                    ui.sidebar_button.set_active(!ui.sidebar_button.is_active());
                    return glib::Propagation::Stop;
                }
                if !modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                    return glib::Propagation::Proceed;
                }
                if key == gdk::Key::n || key == gdk::Key::N {
                    if shift {
                        make_window(
                            &key_app, false, false, false, false, false, false, false, None,
                        );
                    } else {
                        blank_tab(&key_app, &key_host);
                    }
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::t || key == gdk::Key::T {
                    blank_tab(&key_app, &key_host);
                    return glib::Propagation::Stop;
                }
                if shift && (key == gdk::Key::o || key == gdk::Key::O) {
                    key_host.overview.set_open(true);
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::w || key == gdk::Key::W {
                    if let Some(page) = key_host.tabs.selected_page() {
                        key_host.tabs.close_page(&page);
                    }
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::f
                    || key == gdk::Key::F
                    || key == gdk::Key::h
                    || key == gdk::Key::H
                {
                    show_search(&ui, &save_parent, key == gdk::Key::h || key == gdk::Key::H);
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::s || key == gdk::Key::S {
                    save_document(&ui, &save_parent, shift, Box::new(|| {}));
                    return glib::Propagation::Stop;
                }
                if key == gdk::Key::o || key == gdk::Key::O {
                    open_button.emit_clicked();
                    return glib::Propagation::Stop;
                }
                if ui.document_box.has_focus() {
                    let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);
                    let format_shortcut = if alt {
                        match key {
                            gdk::Key::_1 | gdk::Key::KP_1 => Some(Format::Heading1),
                            gdk::Key::_2 | gdk::Key::KP_2 => Some(Format::Heading2),
                            gdk::Key::_3 | gdk::Key::KP_3 => Some(Format::Heading3),
                            _ => None,
                        }
                    } else if shift {
                        match key {
                            gdk::Key::n | gdk::Key::N => Some(Format::Normal),
                            gdk::Key::m | gdk::Key::M => Some(Format::Outdent),
                            gdk::Key::l | gdk::Key::L => Some(Format::Bullets),
                            _ => None,
                        }
                    } else {
                        match key {
                            gdk::Key::b | gdk::Key::B => Some(Format::Bold),
                            gdk::Key::i | gdk::Key::I => Some(Format::Italic),
                            gdk::Key::u | gdk::Key::U => Some(Format::Underline),
                            gdk::Key::l | gdk::Key::L => Some(Format::AlignLeft),
                            gdk::Key::e | gdk::Key::E => Some(Format::AlignCenter),
                            gdk::Key::r | gdk::Key::R => Some(Format::AlignRight),
                            gdk::Key::j | gdk::Key::J => Some(Format::Justify),
                            gdk::Key::m | gdk::Key::M => Some(Format::Indent),
                            _ => None,
                        }
                    };
                    if let Some(action) = format_shortcut {
                        apply_user_format(&ui, action);
                        return glib::Propagation::Stop;
                    }
                    if key == gdk::Key::z || key == gdk::Key::Z {
                        apply_user_edit(&ui, if shift { Edit::Redo } else { Edit::Undo });
                        return glib::Propagation::Stop;
                    }
                    if key == gdk::Key::y || key == gdk::Key::Y {
                        apply_user_edit(&ui, Edit::Redo);
                        return glib::Propagation::Stop;
                    }
                    if key == gdk::Key::v || key == gdk::Key::V {
                        context_paste(&ui);
                        return glib::Propagation::Stop;
                    }
                    if key == gdk::Key::x || key == gdk::Key::X {
                        let has_selected = ui
                            .current
                            .borrow()
                            .as_ref()
                            .is_some_and(|doc| !doc.session.sel.is_collapsed());
                        if has_selected {
                            if context_copy(&ui) {
                                apply_user_edit(&ui, Edit::Delete);
                            }
                        }
                        return glib::Propagation::Stop;
                    }
                    if key == gdk::Key::c || key == gdk::Key::C {
                        copy_selection(&ui);
                        return glib::Propagation::Stop;
                    }
                    if key == gdk::Key::a || key == gdk::Key::A {
                        selected_command(&ui, "select-all", false);
                        return glib::Propagation::Stop;
                    }
                }
                match key {
                    gdk::Key::plus | gdk::Key::equal | gdk::Key::KP_Add => zoom_step(&ui, true),
                    gdk::Key::minus | gdk::Key::KP_Subtract => zoom_step(&ui, false),
                    gdk::Key::_0 | gdk::Key::KP_0 => fit_width(&ui),
                    _ => return glib::Propagation::Proceed,
                }
                glib::Propagation::Stop
            }
        ));
        key_target.add_controller(controller);
    }
    {
        let ui = ui.clone();
        let scroll = ui.scroll.clone();
        let controller = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        controller.connect_scroll(glib::clone!(
            #[weak]
            ui,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |ctrl, _, dy| {
                if ctrl
                    .current_event_state()
                    .contains(gdk::ModifierType::CONTROL_MASK)
                {
                    zoom_step(&ui, dy < 0.0);
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        ));
        scroll.add_controller(controller);
    }
}

pub(super) fn stop_document_drag(ui: &Rc<Ui>) {
    if let Some(id) = ui.drag_scroll.borrow_mut().take() {
        id.remove();
    }
    ui.drag_point.set(None);
    ui.drag_anchor.borrow_mut().take();
    ui.word_drag.borrow_mut().take();
}
fn update_document_drag(ui: &Rc<Ui>, x: f64, y: f64) {
    let Some(anchor) = ui.drag_anchor.borrow().clone() else {
        return;
    };
    let Some(focus) = pos_at_canvas(ui, x, y) else {
        return;
    };
    if anchor.story != focus.story {
        return;
    }
    let result = {
        let mut v = ui.current.borrow_mut();
        v.as_mut().map(|d| {
            let (a, b) = ui
                .word_drag
                .borrow()
                .as_ref()
                .and_then(|r| selection::word_drag_range(&d.session, r, &focus))
                .unwrap_or((anchor, focus));
            selection::select_range(&mut d.session, &a, &b)
        })
    };
    if let Some(Err(e)) = result {
        ui.toasts.add_toast(Toast::new(&e));
    }
    refresh_selection(ui);
}
fn schedule_drag_scroll(ui: &Rc<Ui>) {
    if ui.drag_scroll.borrow().is_some() {
        return;
    }
    let identity = document_identity(ui);
    let weak = Rc::downgrade(ui);
    let id = glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let Some(ui) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        if ui.closed.get()
            || document_identity(&ui) != identity
            || ui.drag_anchor.borrow().is_none()
        {
            ui.drag_scroll.borrow_mut().take();
            return glib::ControlFlow::Break;
        }
        let Some((x, y)) = ui.drag_point.get() else {
            ui.drag_scroll.borrow_mut().take();
            return glib::ControlFlow::Break;
        };
        let velocity = |p: f64, extent: f64| {
            if p < 28.0 {
                (p - 28.0).max(-24.0)
            } else if p > extent - 28.0 {
                (p - extent + 28.0).min(24.0)
            } else {
                0.0
            }
        };
        let mut moved = false;
        for (adj, p, extent) in [
            (ui.scroll.hadjustment(), x, ui.scroll.width() as f64),
            (ui.scroll.vadjustment(), y, ui.scroll.height() as f64),
        ] {
            let value = (adj.value() + velocity(p, extent)).clamp(
                adj.lower(),
                (adj.upper() - adj.page_size()).max(adj.lower()),
            );
            if (value - adj.value()).abs() > 0.01 {
                adj.set_value(value);
                moved = true;
            }
        }
        if !moved {
            ui.drag_scroll.borrow_mut().take();
            return glib::ControlFlow::Break;
        }
        if let Some(r) = ui.document_box.compute_bounds(&ui.scroll) {
            update_document_drag(&ui, x - r.x() as f64, y - r.y() as f64);
        }
        glib::ControlFlow::Continue
    });
    ui.drag_scroll.replace(Some(id));
}
