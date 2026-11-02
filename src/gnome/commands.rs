use super::*;

pub(super) fn after_content_change(ui: &Rc<Ui>) {
    let _perf = perf_scope("leaf.after_content_change");
    let (rebuild, old_zoom, fit_width_mode, old_scroll) = {
        let mut state = ui.current.borrow_mut();
        let Some(doc) = state.as_mut() else { return };
        let layout = {
            let _p = perf_scope("boundary.layout_after_edit");
            doc.session.layout()
        };
        let sizes: Vec<(f32, f32)> = layout.pages.iter().map(|p| (p.w, p.h)).collect();
        let needs_rebuild = sizes != doc.sizes;
        if !needs_rebuild {
            for index in doc.painted_pages.drain() {
                doc.pages[index].set_paintable(None::<&gdk::Paintable>);
            }
        }
        (
            needs_rebuild,
            doc.zoom,
            doc.fit_to_width,
            ui.scroll.vadjustment().value(),
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
                let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                    return;
                };
                if fit_width_mode {
                    fit_width(&ui);
                }
                let vadj = ui.scroll.vadjustment();
                vadj.set_value(old_scroll.min((vadj.upper() - vadj.page_size()).max(0.0)));
                refresh_pages(&ui);
                refresh_thumbnails(&ui);
                refresh_selection(&ui);
                caret_ensure_visible(&ui);
                ui.document_box.grab_focus();
            });
        }
    } else {
        refresh_pages(ui);
        schedule_thumbnail_refresh(ui);
        refresh_selection(ui);
        let weak = Rc::downgrade(ui);
        glib::idle_add_local_once(move || {
            if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
                caret_ensure_visible(&ui);
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
    let changed = {
        let mut value = ui.current.borrow_mut();
        value.as_mut().map(|doc| {
            let _p = perf_scope("boundary.edit");
            editing::apply(&mut doc.session, edit)
        })
    };
    match changed {
        Some(Ok(true)) => {
            after_content_change(ui);
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
        Some(Ok(false)) => refresh_selection(ui),
        Some(Err(err)) => ui.toasts.add_toast(Toast::new(&err)),
        None => {}
    }
    ui.document_box.grab_focus();
}
