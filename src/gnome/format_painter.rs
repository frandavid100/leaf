//! Engine-owned format painter; callbacks never retain a document.
use super::*;
pub(super) fn start(ui: &Rc<Ui>, sticky: bool) {
    tools_change(ui, |s| {
        s.painter = None;
        s.run("edit.formatPainter", &serde_json::json!({"sticky":sticky}))
            .map(|_| ())
            .map_err(|e| e.to_string())
    });
    ui.document_box.set_cursor_from_name(Some("crosshair"));
    ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
        "Select a destination to apply formatting. Escape cancels."
    )));
}
pub(super) fn cancel(ui: &Rc<Ui>) -> bool {
    let active = ui
        .current
        .borrow_mut()
        .as_mut()
        .is_some_and(|d| d.session.painter.take().is_some());
    if active {
        ui.document_box.set_cursor_from_name(Some("text"));
    }
    active
}
pub(super) fn apply(ui: &Rc<Ui>) {
    if !ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| d.session.painter.is_some())
    {
        return;
    }
    tools_change(ui, |s| {
        s.run("edit.pasteFormat", &serde_json::json!({}))
            .map(|_| ())
            .map_err(|e| e.to_string())
    });
    if !ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| d.session.painter.is_some())
    {
        ui.document_box.set_cursor_from_name(Some("text"));
    }
}
