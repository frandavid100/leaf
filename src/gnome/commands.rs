use super::*;

pub(super) fn after_content_change(ui: &Rc<Ui>) {
    remember_repeat(ui);
    refresh_after_content_change(ui);
    schedule_derived_refresh(ui);
}
pub(super) fn after_history_change(ui: &Rc<Ui>) {
    if let Some(source) = ui.derived_source.borrow_mut().take() {
        source.remove();
    }
    {
        let mut current = ui.current.borrow_mut();
        if let Some(d) = current.as_mut() {
            if let Err(e) = wordcraft_gnome::references::refresh_derived(&mut d.session) {
                ui.toasts.add_toast(Toast::new(&e));
            }
        }
    }
    refresh_content(ui, true);
}
pub(super) fn refresh_after_content_change(ui: &Rc<Ui>) {
    refresh_content(ui, false);
}
fn refresh_content(ui: &Rc<Ui>, history: bool) {
    templates::sync_after_edit(ui);
    image_drop::clear_preview(ui);
    let _perf = perf_scope("leaf.after_content_change");
    // A formatting entry owns its edit even when pagination rebuilds the canvas.
    let was_canvas_focused = ui.document_box.has_focus();
    // Deleting/restoring a floating object exposes an unrelated anchor caret.
    // Keep the reading position through that transition, including Undo/Redo.
    let floating_deleted_or_restored = ui.current.borrow().as_ref().is_some_and(|d| {
        let deletion = d
            .session
            .last_command
            .as_ref()
            .is_some_and(|(id, _)| matches!(id.as_str(), "text.delete" | "text.backspace"));
        if !history && !deletion {
            return false;
        }
        let floating_now = pictures::layout(&d.session)
            .is_some_and(|f| f.wrap != wordcraft_doc::para::Wrap::Inline);
        if !ui.selected_floating_image.get() && !floating_now {
            return false;
        }
        let (pos, count) = selection_float_state(&d.session);
        ui.selection_floating_objects
            .borrow()
            .as_ref()
            .is_some_and(|(old, prior)| {
                old.story == pos.story && old.path == pos.path && *prior != count
            })
    });
    let reading = floating_deleted_or_restored
        .then(|| capture_reading_anchor(ui))
        .flatten();
    let picture_anchor = {
        let geometry = ui.selection_data.borrow();
        let current = ui.current.borrow();
        current.as_ref().and_then(|d| {
            pictures::selected(&d.session)?;
            let old = geometry
                .highlights
                .first()
                .filter(|_| geometry.caret.is_none() && geometry.highlights.len() == 1)?;
            let frame = d
                .page_frames
                .get(old.page)?
                .compute_bounds(&ui.document_box)?;
            Some((
                old.page,
                frame.y() as f64 + old.y * d.page_zoom(old.page) as f64 - document_scroll_y(ui),
            ))
        })
    };
    let identity = document_identity(ui);

    let (rebuild, old_zoom, fit_width_mode, old_scroll) = {
        let mut state = ui.current.borrow_mut();
        let Some(doc) = state.as_mut() else { return };
        let layout = {
            let _p = perf_scope("boundary.layout_after_edit");
            doc.session.layout()
        };
        let sizes: Vec<(f32, f32)> = layout.pages.iter().map(|p| (p.w, p.h)).collect();
        let sections_changed = !layout
            .pages
            .iter()
            .map(|p| p.section)
            .eq(doc.sections.iter().copied());
        let needs_rebuild = sizes != doc.sizes || sections_changed;
        if !needs_rebuild {
            for index in doc.painted_pages.drain() {
                doc.pages[index].set_paintable(None::<&gdk::Paintable>);
            }
        }
        (
            needs_rebuild,
            doc.zoom,
            doc.fit_to_width,
            (
                ui.scroll.hadjustment().value(),
                ui.scroll.vadjustment().value(),
            ),
        )
    };
    if rebuild {
        let prior = ui.current.borrow_mut().take();
        if let Some(previous) = prior {
            let name = ui.document_name.borrow().clone();
            let saved_path = ui.save_path.borrow().clone();
            let source_path = ui.source_path.borrow().clone();
            opened(ui, previous.session, &name);
            ui.save_path.replace(saved_path);
            ui.source_path.replace(source_path);
            if !fit_width_mode {
                if let Some(doc) = ui.current.borrow_mut().as_mut() {
                    doc.fit_to_width = false;
                }
                zoom_to(ui, old_zoom);
            }
            let weak = Rc::downgrade(ui);
            glib::idle_add_local_once(move || {
                let Some(ui) = weak
                    .upgrade()
                    .filter(|u| !u.closed.get() && document_identity(u) == identity)
                else {
                    return;
                };
                if fit_width_mode {
                    fit_width(&ui);
                }
                let vadj = ui.scroll.vadjustment();
                vadj.set_value(old_scroll.1.min((vadj.upper() - vadj.page_size()).max(0.0)));
                let hadj = ui.scroll.hadjustment();
                hadj.set_value(old_scroll.0.min((hadj.upper() - hadj.page_size()).max(0.)));
                refresh_pages(&ui);
                refresh_thumbnails(&ui);
                refresh_selection(&ui);
                restore_picture_anchor(&ui, picture_anchor);
                if reading.is_some() {
                    restore_reading_anchor(&ui, reading);
                } else if was_canvas_focused || ui.document_box.has_focus() {
                    caret_ensure_visible(&ui);
                    ui.document_box.grab_focus();
                }
            });
        }
    } else {
        refresh_pages(ui);
        schedule_thumbnail_refresh(ui);
        refresh_selection(ui);
        let weak = Rc::downgrade(ui);
        glib::idle_add_local_once(move || {
            if let Some(ui) = weak
                .upgrade()
                .filter(|u| !u.closed.get() && document_identity(u) == identity)
            {
                restore_picture_anchor(&ui, picture_anchor);
                if reading.is_some() {
                    restore_reading_anchor(&ui, reading);
                } else if was_canvas_focused || ui.document_box.has_focus() {
                    caret_ensure_visible(&ui);
                }
            }
        });
    }
    if let Some(s) = ui.search.borrow().as_ref() {
        s.schedule();
    }
    if let Some(s) = ui.outline.borrow().as_ref() {
        s.schedule();
    }
    if let Some(s) = ui.annotations.borrow().as_ref() {
        s.schedule();
    }
    if let Some(s) = ui.spelling.borrow().as_ref() {
        s.invalidate();
    }
    ui.dirty.set(true);

    sync_tab(ui);
}

pub(super) fn apply_user_edit(ui: &Rc<Ui>, edit: Edit) {
    let is_history = matches!(edit, Edit::Undo | Edit::Redo);
    let changed = {
        let mut value = ui.current.borrow_mut();
        value.as_mut().map(|doc| {
            let _p = perf_scope("boundary.edit");
            editing::apply(&mut doc.session, edit)
        })
    };
    match changed {
        Some(Ok(true)) => {
            if is_history {
                after_history_change(ui);
            } else {
                after_content_change(ui);
            }
        }
        Some(Ok(false)) | None => {}
        Some(Err(err)) => ui.toasts.add_toast(Toast::new(&err)),
    }
}

pub(super) fn apply_user_format(ui: &Rc<Ui>, action: Format) {
    apply_format_operation(ui, move |session| formatting::apply(session, action));
}

pub(super) fn apply_format_operation(
    ui: &Rc<Ui>,
    operation: impl FnOnce(&mut Session) -> Result<(), String>,
) {
    let result = {
        let mut state = ui.current.borrow_mut();
        state.as_mut().map(|doc| {
            let old_rev = doc.session.rev();
            operation(&mut doc.session).map(|_| doc.session.rev() != old_rev)
        })
    };
    match result {
        Some(Ok(true)) => {
            let first = !ui.dirty.get();
            after_content_change(ui);
            if first {
                ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                    "Formatting changed. Save to keep your changes."
                )));
            }
        }
        Some(Ok(false)) => {
            remember_repeat(ui);
            refresh_selection(ui);
        }
        Some(Err(err)) => ui.toasts.add_toast(Toast::new(&err)),
        None => {}
    }
    ui.document_box.grab_focus();
}

// Preserve the selected object's screen position through page/section rebuilds.
// This is view state only; the canonical object and selection stay in WordCraft.
fn restore_picture_anchor(ui: &Rc<Ui>, anchor: Option<(usize, f64)>) {
    let Some((old_page, offset)) = anchor else {
        return;
    };
    let top = {
        let mut current = ui.current.borrow_mut();
        current.as_mut().and_then(|d| {
            let (pos, _, _) = pictures::selected(&d.session)?;
            let hit = d.session.layout().object(&pos, d.session.page_hint)?;
            if hit.page != old_page {
                return None;
            }
            let frame = d
                .page_frames
                .get(hit.page)?
                .compute_bounds(&ui.document_box)?;
            Some(frame.y() as f64 + hit.rect.y as f64 * d.page_zoom(hit.page) as f64)
        })
    };
    if let Some(top) = top {
        let adjustment = ui.scroll.vadjustment();
        adjustment.set_value(
            (top + document_content_top(ui) - offset)
                .clamp(0., (adjustment.upper() - adjustment.page_size()).max(0.)),
        );
    }
}
