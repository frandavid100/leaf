//! One close decision per window, with revision-bound saves and deferred discard.
use super::*;

struct CloseGroup {
    host: std::rc::Weak<Workspace>,
    docs: Vec<Rc<Ui>>,
    identities: RefCell<Vec<Option<(u64, u64)>>>,
}
impl CloseGroup {
    fn valid(&self) -> Option<Rc<Workspace>> {
        let host = self.host.upgrade()?;
        let live = host.docs.borrow();
        let identities = self.identities.borrow();
        (live.len() == self.docs.len()
            && live
                .iter()
                .zip(&self.docs)
                .all(|((_, a), b)| Rc::ptr_eq(a, b))
            && self
                .docs
                .iter()
                .zip(identities.iter())
                .all(|(ui, id)| !ui.closed.get() && document_identity(ui) == *id))
        .then(|| host.clone())
    }
    fn cancel(&self) {
        if let Some(host) = self.host.upgrade() {
            host.close_pending.set(false);
        }
    }
    fn changed(&self) {
        self.cancel();
        if let Some(ui) = self.docs.iter().find(|ui| !ui.closed.get()) {
            ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                "The document has changed; open this dialog again."
            )));
        }
    }
    fn finish(&self) {
        let Some(host) = self.valid() else {
            self.changed();
            return;
        };
        // Discard only after every requested save succeeded and the full scope
        // is still current. A cancelled/failed Save As never reaches this point.
        for ui in &self.docs {
            if ui.dirty.get() {
                if let Some(doc) = ui.current.borrow_mut().as_mut() {
                    doc.session.discard_recovery_copy();
                    doc.session.finish_recovery();
                }
            }
            release_document(ui);
        }
        host.allow_close.set(true);
        let window = host.window.clone();
        glib::idle_add_local_once(move || window.close());
    }
    fn save(self: &Rc<Self>, selected: Rc<Vec<bool>>, from: usize) {
        let Some(host) = self.valid() else {
            self.changed();
            return;
        };
        let Some(index) =
            (from..self.docs.len()).find(|i| selected[*i] && self.docs[*i].dirty.get())
        else {
            self.finish();
            return;
        };
        let group = self.clone();
        save_document_result(
            &self.docs[index],
            &host.window,
            false,
            Box::new(move |saved| {
                if !saved {
                    group.cancel();
                    return;
                }
                group.identities.borrow_mut()[index] = document_identity(&group.docs[index]);
                group.save(selected, index + 1);
            }),
        );
    }
}

pub(super) fn request(host: &Rc<Workspace>) {
    if host.close_pending.replace(true) {
        return;
    }
    let docs: Vec<_> = host
        .docs
        .borrow()
        .iter()
        .map(|(_, ui)| ui.clone())
        .collect();
    let group = Rc::new(CloseGroup {
        host: Rc::downgrade(host),
        identities: RefCell::new(docs.iter().map(|ui| document_identity(ui)).collect()),
        docs,
    });
    if !group.docs.iter().any(|ui| ui.dirty.get()) {
        group.finish();
        return;
    }
    if group.docs.len() == 1 {
        // Preserve the familiar single-document close prompt.
        host.close_pending.set(false);
        let weak = Rc::downgrade(host);
        ask_discard(&group.docs[0], &host.window, move || {
            if let Some(host) = weak.upgrade() {
                for (_, ui) in host.docs.borrow().iter() {
                    release_document(ui);
                }
                host.allow_close.set(true);
                host.window.close();
            }
        });
        return;
    }
    let dialog = adw::AlertDialog::new(
        Some(wordcraft_gnome::t!("Save Changes?")),
        Some(wordcraft_gnome::t!(
            "Select the documents to save. Changes in unchecked documents will be discarded."
        )),
    );
    // This native preference appeared in libadwaita 1.6. Use it when the
    // installed runtime provides it without raising Leaf's 1.5 API baseline.
    if dialog.find_property("prefer-wide-layout").is_some() {
        dialog.set_property("prefer-wide-layout", true);
    } else {
        dialog.set_content_width(620);
    }
    dialog.add_response("cancel", wordcraft_gnome::t!("Cancel"));
    dialog.add_response("discard", wordcraft_gnome::t!("Discard All"));
    dialog.add_response("save", wordcraft_gnome::t!("Save"));
    dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("save"));
    dialog.set_close_response("cancel");
    let rows = adw::PreferencesGroup::new();
    let mut choices = Vec::new();
    for (index, ui) in group
        .docs
        .iter()
        .enumerate()
        .filter(|(_, ui)| ui.dirty.get())
    {
        let row = adw::ActionRow::new();
        row.set_use_markup(false);
        let name = ui.document_name.borrow().clone();
        let untitled = ui.save_path.borrow().is_none() && ui.source_path.borrow().is_none();
        let duplicates = group
            .docs
            .iter()
            .filter(|doc| {
                doc.dirty.get()
                    && doc.document_name.borrow().as_str() == name
                    && doc.save_path.borrow().is_none()
                    && doc.source_path.borrow().is_none()
            })
            .count();
        let title = if untitled && duplicates > 1 {
            let number = group.docs[..=index]
                .iter()
                .filter(|doc| {
                    doc.dirty.get()
                        && doc.document_name.borrow().as_str() == name
                        && doc.save_path.borrow().is_none()
                        && doc.source_path.borrow().is_none()
                })
                .count();
            wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("{name} ({number})"),
                &[("name", name), ("number", number.to_string())],
            )
        } else {
            name
        };
        row.set_title(&title);
        row.set_title_lines(2);
        row.set_subtitle_lines(2);
        let path = ui
            .save_path
            .borrow()
            .clone()
            .or_else(|| ui.source_path.borrow().clone());
        row.set_subtitle(
            &path
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| wordcraft_gnome::t!("Not saved to a file").into()),
        );
        let check = gtk::CheckButton::new();
        check.set_active(true);
        check.set_valign(gtk::Align::Center);
        accessible_name(&check, &title);
        row.add_prefix(&check);
        row.set_activatable_widget(Some(&check));
        rows.add(&row);
        choices.push((index, check));
    }
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_propagate_natural_height(true);
    scroll.set_max_content_height(320);
    scroll.set_child(Some(&rows));
    dialog.set_extra_child(Some(&scroll));
    for (_, check) in &choices {
        let others: Vec<_> = choices.iter().map(|(_, c)| c.downgrade()).collect();
        let weak = dialog.downgrade();
        check.connect_toggled(move |_| {
            if let Some(dialog) = weak.upgrade() {
                dialog.set_response_enabled(
                    "save",
                    others
                        .iter()
                        .any(|c| c.upgrade().is_some_and(|c| c.is_active())),
                );
            }
        });
    }
    dialog.connect_response(None, move |_, response| match response {
        "discard" => group.finish(),
        "save" => {
            let mut selected = vec![false; group.docs.len()];
            for (index, check) in &choices {
                selected[*index] = check.is_active();
            }
            group.save(Rc::new(selected), 0);
        }
        _ => group.cancel(),
    });
    dialog.present(Some(&host.window));
}
