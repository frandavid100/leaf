use super::*;

pub(super) fn refresh_selection(ui: &Rc<Ui>) {
    let _perf = perf_scope("leaf.refresh_selection");
    let snapshot = {
        let Ok(mut value) = ui.current.try_borrow_mut() else {
            return;
        };
        let Some(doc) = value.as_mut() else { return };
        {
            let _p = perf_scope("boundary.selection_geometry");
            selection::geometry(&mut doc.session)
        }
    };
    let now = ui
        .document_box
        .frame_clock()
        .map(|c| c.frame_time())
        .unwrap_or(0);
    let animate = ui.document_box.is_mapped()
        && ui
            .document_box
            .settings()
            .property::<bool>("gtk-enable-animations")
        && snapshot.highlights.is_empty()
        && ui.drag_anchor.borrow().is_none();
    ui.caret_motion
        .borrow_mut()
        .retarget(snapshot.caret.clone(), now, animate);
    *ui.selection_data.borrow_mut() = snapshot;
    if ui.caret_motion.borrow().is_moving() && !ui.caret_tick_active.replace(true) {
        let weak = Rc::downgrade(ui);
        ui.document_box.add_tick_callback(move |widget, clock| {
            let Some(ui) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let enabled = widget.settings().property::<bool>("gtk-enable-animations");
            let moving = {
                let mut motion = ui.caret_motion.borrow_mut();
                if !enabled || ui.closed.get() {
                    motion.finish();
                }
                motion.advance(clock.frame_time());
                motion.is_moving()
            };
            if let Some(page) = ui.caret_motion.borrow().position().map(|r| r.page) {
                if let Some(area) = ui
                    .current
                    .borrow()
                    .as_ref()
                    .and_then(|d| d.selection_areas.get(page))
                {
                    perf_count("overlay_queue", 1);
                    area.queue_draw();
                }
            }
            if moving {
                glib::ControlFlow::Continue
            } else {
                ui.caret_tick_active.set(false);
                glib::ControlFlow::Break
            }
        });
    }
    refresh_overlay_layers(ui);
    refresh_format_controls(ui);
    if let Some(style) = ui.style.borrow().as_ref() {
        style.sync_selection();
    }
    if let Some(paragraph) = ui.paragraph.borrow().as_ref() {
        paragraph.sync_selection();
    }
    if let Some(text) = ui.text_tools.borrow().as_ref() {
        text.sync_selection();
    }
    if let Some(p) = ui.picture.borrow().as_ref() {
        p.refresh();
    }
    ui.document_box.sync(ui);
}

pub(super) fn refresh_overlay_layers(ui: &Rc<Ui>) {
    let selection = ui.selection_data.borrow();
    let search = ui.search_highlights.borrow();
    let motion = ui.caret_motion.borrow();
    let next: HashSet<usize> = selection
        .highlights
        .iter()
        .chain(search.iter())
        .map(|r| r.page)
        .chain(selection.caret.iter().map(|r| r.page))
        .chain(motion.position().map(|r| r.page))
        .collect();
    let Ok(mut current) = ui.current.try_borrow_mut() else {
        return;
    };
    let Some(doc) = current.as_mut() else {
        return;
    };
    let header = header_footer::active(&doc.session);
    let mut damaged = std::mem::replace(&mut doc.overlay_pages, next);
    damaged.extend(doc.overlay_pages.iter().copied());
    if header.is_some() || doc.overlay_header != header {
        damaged.extend(0..doc.selection_areas.len());
    }
    doc.overlay_header = header;
    for page in damaged {
        if let Some(area) = doc.selection_areas.get(page) {
            perf_count("overlay_queue", 1);
            area.queue_draw();
        }
    }
}

pub(super) fn refresh_format_controls(ui: &Rc<Ui>) {
    let _perf = perf_scope("leaf.refresh_format_controls");
    let state = selection_character(ui);
    if let Some(doc) = ui.current.borrow().as_ref() {
        ui.undo_button.set_sensitive(doc.session.can_undo());
        ui.redo_button.set_sensitive(doc.session.can_redo());
    }
    if let Some(host) = ui.host.upgrade() {
        refresh_main_actions(&host);
    }
    let effective = &state;
    ui.format_syncing.set(true);
    for (button, property) in [
        (&ui.format_bold, "bold"),
        (&ui.format_italic, "italic"),
        (&ui.format_underline, "underline"),
    ] {
        let mixed = effective.get(property).is_some_and(|v| v.is_null());
        let active = if property == "underline" {
            effective[property].as_str().is_some_and(|s| s != "none")
        } else {
            effective[property].as_bool().unwrap_or(false)
        };
        button.set_active(active);
        button.update_state(&[gtk::accessible::State::Pressed(if mixed {
            gtk::AccessibleTristate::Mixed
        } else if active {
            gtk::AccessibleTristate::True
        } else {
            gtk::AccessibleTristate::False
        })]);
        if mixed {
            button.add_css_class("mixed");
        } else {
            button.remove_css_class("mixed");
        }
    }
    let focus = ui
        .host
        .upgrade()
        .and_then(|h| gtk::prelude::GtkWindowExt::focus(&h.window));
    if !focus.as_ref().is_some_and(|w| {
        w == ui.font_input.upcast_ref::<gtk::Widget>() || w.is_ancestor(&ui.font_input)
    }) {
        if let Some(font) = state["font"].as_str() {
            if ui.font_input.text() != font {
                ui.font_input.set_text(font);
            }
            ui.font_input.remove_css_class("error");
        } else {
            ui.font_input.set_text("");
            ui.font_input
                .set_placeholder_text(Some(wordcraft_gnome::t!("Mixed")));
        }
    }
    if !focus.as_ref().is_some_and(|w| {
        w == ui.size_input.upcast_ref::<gtk::Widget>() || w.is_ancestor(&ui.size_input)
    }) {
        if let Some(size) = state["size"].as_f64() {
            ui.size_input.set_value(size);
        } else {
            ui.size_input.set_text("");
        }
    }
    ui.format_syncing.set(false);
}

pub(super) fn pos_at_canvas(ui: &Rc<Ui>, x: f64, y: f64) -> Option<Pos> {
    let mut value = ui.current.borrow_mut();
    let doc = value.as_mut()?;
    let (page, rect, _) = doc
        .page_frames
        .iter()
        .enumerate()
        .filter_map(|(i, frame)| {
            let rect = frame.compute_bounds(&ui.document_box)?;
            let min_x = f64::from(rect.x());
            let min_y = f64::from(rect.y());
            let max_x = min_x + f64::from(rect.width());
            let max_y = min_y + f64::from(rect.height());
            let dx = (min_x - x).max(0.0).max(x - max_x);
            let dy = (min_y - y).max(0.0).max(y - max_y);
            Some((i, rect, (dx * dx + dy * dy).sqrt()))
        })
        .min_by(|a, b| a.2.total_cmp(&b.2))?;
    let z = f64::from(doc.zoom);
    let (w, h) = doc.sizes[page];
    let px = ((x - f64::from(rect.x())) / z).clamp(0.0, f64::from(w)) as f32;
    let py = ((y - f64::from(rect.y())) / z).clamp(0.0, f64::from(h)) as f32;
    header_footer::hit(&mut doc.session, page, px, py)
}

pub(super) fn select_picture_at_canvas(ui: &Rc<Ui>, x: f64, y: f64) -> bool {
    let mut value = ui.current.borrow_mut();
    let Some(doc) = value.as_mut() else {
        return false;
    };
    for (i, frame) in doc.page_frames.iter().enumerate() {
        let Some(r) = frame.compute_bounds(&ui.document_box) else {
            continue;
        };
        if x < r.x() as f64
            || y < r.y() as f64
            || x > (r.x() + r.width()) as f64
            || y > (r.y() + r.height()) as f64
        {
            continue;
        }
        let z = doc.zoom as f64;
        let px = ((x - r.x() as f64) / z) as f32;
        let py = ((y - r.y() as f64) / z) as f32;
        let layout = doc.session.layout();
        let story = layout
            .header_footer_at(i, py)
            .map(|(story, _)| story)
            .unwrap_or(wordcraft_doc::StoryRef::Body);
        let Some(hit) = layout.object_at_in(i, px, py, 0.0, story) else {
            return false;
        };
        doc.session.page_hint = i;
        return pictures::select(&mut doc.session, &hit.pos()).is_ok();
    }
    false
}

pub(super) fn caret_ensure_visible(ui: &Rc<Ui>) {
    let target = {
        let Ok(mut value) = ui.current.try_borrow_mut() else {
            return;
        };
        let Some(doc) = value.as_mut() else { return };
        let layout = doc.session.layout();
        layout
            .caret_on(&doc.session.sel.focus, doc.session.page_hint)
            .and_then(|c| {
                doc.page_frames
                    .get(c.page)
                    .and_then(|frame| frame.compute_bounds(&ui.document_box))
                    .map(|r| f64::from(r.y()) + f64::from(c.top * doc.zoom))
            })
    };
    if let Some(y) = target {
        let adjustment = ui.scroll.vadjustment();
        let top = adjustment.value();
        let bottom = top + adjustment.page_size();
        if y < top + 24.0 {
            adjustment.set_value((y - 32.0).max(0.0));
        } else if y > bottom - 32.0 {
            adjustment.set_value((y - adjustment.page_size() + 60.0).max(0.0));
        }
    }
}

pub(super) fn selected_command(ui: &Rc<Ui>, action: &str, extend: bool) {
    let result = {
        let mut value = ui.current.borrow_mut();
        value.as_mut().map(|doc| {
            if action == "select-all" {
                selection::select_all(&mut doc.session)
            } else {
                {
                    let _p = perf_scope("boundary.move_caret");
                    selection::move_caret(&mut doc.session, action, extend)
                }
            }
        })
    };
    if let Some(Err(e)) = result {
        ui.toasts.add_toast(Toast::new(&e));
    }
    refresh_selection(ui);
    caret_ensure_visible(ui);
}

pub(super) fn copy_selection(ui: &Rc<Ui>) {
    context_copy(ui);
}
