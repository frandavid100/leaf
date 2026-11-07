//! Native file drops use GDK's portal-aware FileList, never raw URI parsing.
use super::*;
pub(super) fn install(ui: &Rc<Ui>) {
    let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    let weak = Rc::downgrade(ui);
    target.connect_drop(move |_, value, x, y| {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return false;
        };
        let Ok(files) = value.get::<gdk::FileList>() else {
            return false;
        };
        let paths = files
            .files()
            .iter()
            .filter_map(|file| file.path())
            .collect::<Vec<_>>();
        if paths.len() != 1 {
            return false;
        }
        let Some(path) = paths.into_iter().next() else {
            return false;
        };
        if !path.extension().and_then(|v| v.to_str()).is_some_and(|v| {
            [
                "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "wmf", "emf",
            ]
            .iter()
            .any(|ext| v.eq_ignore_ascii_case(ext))
        }) {
            return false;
        }
        let Some(pos) = pos_at_canvas(&ui, x, y) else {
            return false;
        };
        let changed = tools_change(&ui, |s| {
            wordcraft_gnome::operations::atomic(s, wordcraft_gnome::t!("Insert Image"), |s| {
                selection::set_caret(s, &pos, false)?;
                wordcraft_gnome::pictures::insert(s, &path)
            })
        });
        ui.document_box.grab_focus();
        caret_ensure_visible(&ui);
        changed
    });
    ui.document_box.add_controller(target);
}
