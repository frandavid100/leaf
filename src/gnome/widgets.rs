//! Shared GTK tree helpers; no document or panel state.
use super::*;

pub(super) fn clear_box(parent: &gtk::Box) {
    while let Some(child) = parent.first_child() {
        parent.remove(&child);
    }
}

/// Find a named native widget in a contained panel or diagnostic fixture.
pub(super) fn find_widget(root: &impl IsA<gtk::Widget>, name: &str) -> Option<gtk::Widget> {
    let root = root.as_ref();
    if root.widget_name() == name {
        return Some(root.clone());
    }
    let mut child = root.first_child();
    while let Some(w) = child {
        if let Some(found) = find_widget(&w, name) {
            return Some(found);
        }
        child = w.next_sibling();
    }
    None
}
