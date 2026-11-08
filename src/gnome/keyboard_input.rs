//! Native keyboard shortcuts and scroll-controller dispatch.
use super::*;

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
        // Handle editor keys before ScrolledWindow's Home/End bindings consume
        // them. Focus guards below leave native entry navigation to GTK.
        controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        // GTK supplies dead-key composition, accents and input methods.
        // Committed text enters WordCraft; ordinary shortcuts remain in
        // the key controller instead of being converted into characters.
        // The tab owns the context even while it is detached from the controller.
        // Weak focus callbacks must not be its only owner between focus changes.
        let weak = Rc::downgrade(&ui);
        ui.document_box.connect_has_focus_notify(move |_| {
            if let Some(ui) = weak.upgrade() {
                restart_caret(&ui);
                if let Some(d) = ui.current.borrow().as_ref() {
                    for area in &d.selection_areas {
                        area.queue_draw();
                    }
                }
            }
        });
        let weak = Rc::downgrade(&ui);
        window.connect_is_active_notify(move |_| {
            if let Some(ui) = weak.upgrade() {
                restart_caret(&ui);
                if let Some(d) = ui.current.borrow().as_ref() {
                    for area in &d.selection_areas {
                        area.queue_draw();
                    }
                }
            }
        });
        let input = ui.input_context.clone();
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
                // Adaptive dialogs share this root window. Their entries and
                // shortcuts own input until libadwaita closes the modal surface.
                if save_parent.visible_dialog().is_some() {
                    return glib::Propagation::Proceed;
                }
                if key == gdk::Key::Escape
                    && ui.document_box.has_focus()
                    && format_painter::cancel(&ui)
                {
                    return glib::Propagation::Stop;
                }
                let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
                let control = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
                if key == gdk::Key::Escape
                    && (picture_drag_cancel(&ui) || table_pointer_cancel(&ui))
                {
                    return glib::Propagation::Stop;
                }
                if ui.document_box.has_focus() {
                    restart_caret(&ui);
                    if modifiers.contains(gdk::ModifierType::ALT_MASK) && key == gdk::Key::equal {
                        if let Some(host) = ui.host.upgrade() {
                            show_equation(&ui, &host.window);
                        }
                        return glib::Propagation::Stop;
                    }
                    if modifiers.contains(gdk::ModifierType::ALT_MASK) && key == gdk::Key::Return {
                        if let Some(h) = ui.host.upgrade() {
                            h.window
                                .lookup_action("follow-note")
                                .unwrap()
                                .activate(None);
                        }
                        return glib::Propagation::Stop;
                    }
                    if control
                        && !shift
                        && !modifiers.contains(gdk::ModifierType::ALT_MASK)
                        && matches!(key, gdk::Key::k | gdk::Key::K)
                    {
                        show_link(&ui);
                        return glib::Propagation::Stop;
                    }
                    if control && shift && key == gdk::Key::F5 {
                        show_annotation_dialog(&ui, AnnotationEdit::Bookmark);
                        return glib::Propagation::Stop;
                    }
                    if control
                        && modifiers.contains(gdk::ModifierType::ALT_MASK)
                        && matches!(key, gdk::Key::m | gdk::Key::M)
                    {
                        compose_comment(&ui, None);
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
                        let y = document_scroll_y(&ui).max(0.) as i32 + 40;
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
                        if shift {
                            clipboard::paste_plain(&ui)
                        } else {
                            context_paste(&ui)
                        };
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
                        if shift {
                            format_painter::start(&ui, false);
                        } else {
                            copy_selection(&ui);
                        }
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
