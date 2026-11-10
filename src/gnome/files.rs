use super::*;

pub(super) fn ask_discard(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    action: impl FnOnce() + 'static,
) {
    if !ui.dirty.get() {
        action();
        return;
    }
    let dialog = adw::AlertDialog::new(
        Some(wordcraft_gnome::t!("Save Changes?")),
        Some(wordcraft_gnome::t!(
            "If you continue without saving, your changes will be lost."
        )),
    );
    dialog.add_response("cancel", wordcraft_gnome::t!("Cancel"));
    dialog.add_response("discard", "Descartar");
    dialog.add_response("save", wordcraft_gnome::t!("Save"));
    dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
    dialog.set_response_appearance("save", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("save"));
    dialog.set_close_response("cancel");
    let once = Rc::new(RefCell::new(Some(action)));
    let ui = ui.clone();
    let parent = parent.clone();
    dialog.connect_response(
        None,
        glib::clone!(
            #[weak]
            ui,
            #[weak]
            parent,
            move |_, response| {
                if response == "discard" {
                    if let Some(doc) = ui.current.borrow_mut().as_mut() {
                        doc.session.discard_recovery_copy();
                        doc.session.finish_recovery();
                    }
                    if let Some(action) = once.borrow_mut().take() {
                        action();
                    }
                } else if response == "save" {
                    let once = once.clone();
                    save_document(
                        &ui,
                        &parent,
                        false,
                        Box::new(move || {
                            if let Some(action) = once.borrow_mut().take() {
                                action();
                            }
                        }),
                    );
                } else {
                    once.borrow_mut().take();
                    cancel_tab_close(&ui);
                }
            }
        ),
    );
    dialog.present(Some(&parent));
}

pub(super) fn save_to(ui: &Rc<Ui>, path: &Path) -> bool {
    let (result, derived_changed) = {
        let mut state = ui.current.borrow_mut();
        let Some(doc) = state.as_mut() else {
            return false;
        };
        let revision = doc.session.rev();
        let result = wordcraft_gnome::saving::save(&mut doc.session, path);
        (result, doc.session.rev() != revision)
    };
    match result {
        Ok(()) => {
            ui.save_path.replace(Some(path.to_path_buf()));
            if derived_changed {
                refresh_after_content_change(ui);
            }
            ui.dirty.set(false);
            ui.document_name.replace(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );
            if let Some(doc) = ui.current.borrow_mut().as_mut() {
                doc.session.finish_recovery();
            }
            sync_tab(ui);
            ui.toasts
                .add_toast(Toast::new(wordcraft_gnome::t!("Document Saved")));
            true
        }
        Err(e) => {
            ui.toasts
                .add_toast(Toast::new(&wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Could Not Save: {e}"),
                    &[("e", format!("{}", e))],
                )));
            false
        }
    }
}

pub(super) fn save_document(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    save_as: bool,
    done: Box<dyn FnOnce()>,
) {
    save_document_result(
        ui,
        parent,
        save_as,
        Box::new(move |saved| {
            if saved {
                done();
            }
        }),
    );
}

pub(super) fn save_document_result(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    save_as: bool,
    done: Box<dyn FnOnce(bool)>,
) {
    if ui.current.borrow().is_none() {
        cancel_tab_close(ui);
        done(false);
        return;
    }
    let existing = ui
        .save_path
        .borrow()
        .clone()
        .or_else(|| ui.source_path.borrow().clone());
    let needs_docx = ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| wordcraft_gnome::saving::requires_docx(&d.session));
    let incompatible = needs_docx
        && existing
            .as_ref()
            .is_some_and(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("odt")));
    if !save_as && !incompatible {
        if let Some(path) = existing {
            if save_to(ui, &path) {
                done(true);
            } else {
                cancel_tab_close(ui);
                done(false);
            }
            return;
        }
    }
    let source = existing.or_else(|| ui.source_path.borrow().clone());
    let dialog = save_chooser(ui, source.as_deref(), needs_docx);
    let identity = document_identity(ui);
    let ui = ui.clone();
    dialog.save(
        Some(parent),
        None::<&gio::Cancellable>,
        glib::clone!(
            #[weak]
            ui,
            move |result| {
                if ui.closed.get() {
                    done(false);
                    return;
                }
                if document_identity(&ui) != identity {
                    cancel_tab_close(&ui);
                    ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                        "The document has changed; open this dialog again."
                    )));
                    done(false);
                    return;
                }
                match result {
                    Ok(file) => {
                        let Some(path) = file.path() else {
                            ui.toasts
                                .add_toast(Toast::new(wordcraft_gnome::t!("Choose a Local File")));
                            cancel_tab_close(&ui);
                            done(false);
                            return;
                        };
                        // Require an extension so GTK's overwrite confirmation covers the actual target.
                        if path.extension().is_none() {
                            ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                                "Include .docx or .odt in the file name"
                            )));
                            cancel_tab_close(&ui);
                            done(false);
                            return;
                        }
                        if save_to(&ui, &path) {
                            done(true);
                        } else {
                            cancel_tab_close(&ui);
                            done(false);
                        }
                    }
                    Err(e)
                        if e.matches(gtk::DialogError::Dismissed)
                            || e.matches(gtk::DialogError::Cancelled) =>
                    {
                        cancel_tab_close(&ui);
                        done(false);
                    }
                    Err(e) => {
                        ui.toasts
                            .add_toast(Toast::new(&wordcraft_gnome::i18n::interpolate(
                                wordcraft_gnome::t!("Could Not Open Save Dialog: {e}"),
                                &[("e", format!("{}", e))],
                            )));
                        cancel_tab_close(&ui);
                        done(false);
                    }
                }
            }
        ),
    );
}

/// Native file chooser configuration is shared with the integration probe.
pub(super) fn save_chooser(ui: &Ui, source: Option<&Path>, needs_docx: bool) -> gtk::FileDialog {
    let dialog = gtk::FileDialog::new();
    dialog.set_title(if source.is_some() {
        wordcraft_gnome::t!("Save a Copy — DOCX or ODT")
    } else {
        wordcraft_gnome::t!("Save Document — DOCX or ODT")
    });
    dialog.set_modal(true);
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(wordcraft_gnome::t!("Documents (.docx, .odt)")));
    filter.add_pattern("*.docx");
    filter.add_pattern("*.odt");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));
    let name = wordcraft_gnome::save_name::suggestion(
        &ui.current.borrow().as_ref().unwrap().session,
        source,
        needs_docx,
    );
    dialog.set_initial_name(Some(&name));
    if let Some(folder) = source.and_then(|p| p.parent()) {
        dialog.set_initial_folder(Some(&gio::File::for_path(folder)));
    }
    dialog
}
