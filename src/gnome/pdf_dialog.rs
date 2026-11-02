//! Native PDF destination picker; export remains separate from document saving.
use super::*;

pub(super) fn export_pdf_dialog(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let dialog = gtk::FileDialog::new();
    dialog.set_title(wordcraft_gnome::t!("Export to PDF"));
    dialog.set_modal(true);
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(wordcraft_gnome::t!("PDF Document (.pdf)")));
    filter.add_pattern("*.pdf");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));
    let name = ui.document_name.borrow().clone();
    let stem = Path::new(&name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    dialog.set_initial_name(Some(&format!("{stem}.pdf")));
    let identity = document_identity(ui);
    let ui = ui.clone();
    dialog.save(
        Some(parent),
        None::<&gio::Cancellable>,
        glib::clone!(
            #[weak]
            ui,
            move |result| match result {
                Ok(file) => {
                    if ui.closed.get() || document_identity(&ui) != identity {
                        ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                            "The document has changed; export it again"
                        )));
                        return;
                    }
                    let Some(path) = file.path() else {
                        ui.toasts
                            .add_toast(Toast::new(wordcraft_gnome::t!("Choose a Local File")));
                        return;
                    };
                    let doc = ui.current.borrow().as_ref().unwrap().session.doc.clone();
                    ui.toasts
                        .add_toast(Toast::new(wordcraft_gnome::t!("Exporting PDF…")));
                    let weak = Rc::downgrade(&ui);
                    glib::MainContext::default().spawn_local(async move {
                        let result = gio::spawn_blocking(move || {
                            pdf_export::export(&mut Session::new(doc), &path)
                        })
                        .await;
                        if let Some(ui) = weak.upgrade() {
                            if !ui.closed.get() {
                                let text = match result {
                                    Ok(Ok(())) => wordcraft_gnome::t!("PDF Exported").to_string(),
                                    Ok(Err(e)) => e,
                                    Err(_) => {
                                        wordcraft_gnome::t!("Could Not Export PDF").to_string()
                                    }
                                };
                                ui.toasts.add_toast(Toast::new(&text));
                            }
                        }
                    });
                }
                Err(e)
                    if e.matches(gtk::DialogError::Cancelled)
                        || e.matches(gtk::DialogError::Dismissed) => {}
                Err(e) => ui.toasts.add_toast(Toast::new(&e.to_string())),
            }
        ),
    );
}
