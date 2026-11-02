//! Selection dragging and cancellable edge autoscroll.
use super::*;

pub(super) fn stop_document_drag(ui: &Rc<Ui>) {
    if let Some(id) = ui.drag_scroll.borrow_mut().take() {
        id.remove();
    }
    ui.drag_point.set(None);
    ui.drag_anchor.borrow_mut().take();
    ui.word_drag.borrow_mut().take();
    ui.paragraph_drag.borrow_mut().take();
}
pub(super) fn update_document_drag(ui: &Rc<Ui>, x: f64, y: f64) {
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
            let paragraph = ui
                .paragraph_drag
                .borrow()
                .as_ref()
                .and_then(|r| selection::paragraph_drag_range(&mut d.session, r, &focus));
            let (a, b) = paragraph
                .or_else(|| {
                    ui.word_drag
                        .borrow()
                        .as_ref()
                        .and_then(|r| selection::word_drag_range(&d.session, r, &focus))
                })
                .unwrap_or((anchor, focus));
            selection::select_range(&mut d.session, &a, &b)
        })
    };
    if let Some(Err(e)) = result {
        ui.toasts.add_toast(Toast::new(&e));
    }
    refresh_selection(ui);
}
pub(super) fn schedule_drag_scroll(ui: &Rc<Ui>) {
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
