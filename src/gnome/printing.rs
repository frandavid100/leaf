use super::*;

// GTK owns the native print/portal UI; WordCraft supplies the vector PDF.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PrintOutcome {
    Submitted,
    Dismissed,
    Failed,
}
#[derive(Default)]
pub(super) struct PrintState {
    pub(super) pending: RefCell<Option<gio::Cancellable>>,
    pub(super) outcome: Cell<Option<PrintOutcome>>,
}
impl PrintState {
    pub(super) fn cancel(&self) {
        if let Some(c) = self.pending.borrow().as_ref() {
            c.cancel();
        }
    }
}
pub(super) fn finish_print(ui: &Ui, outcome: PrintOutcome) {
    ui.printing.pending.borrow_mut().take();
    ui.printing.outcome.set(Some(outcome));
    if let Some(host) = ui.host.upgrade() {
        refresh_main_actions(&host);
    }
}
pub(super) fn print_document(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    if ui.closed.get() || ui.printing.pending.borrow().is_some() {
        return;
    }
    let Some(document) = ui.current.borrow().as_ref().map(|d| d.session.doc.clone()) else {
        return;
    };
    let cancel = gio::Cancellable::new();
    ui.printing.pending.replace(Some(cancel.clone()));
    ui.printing.outcome.set(None);
    if let Some(host) = ui.host.upgrade() {
        refresh_main_actions(&host);
    }
    ui.toasts
        .add_toast(Toast::new(wordcraft_gnome::t!("Preparing to Print…")));
    let title = format!("Imprimir — {}", ui.document_name.borrow());
    let weak = Rc::downgrade(ui);
    let parent = parent.downgrade();
    // Keep the main loop alive for cleanup if the last tab closes during export.
    let hold = parent
        .upgrade()
        .and_then(|p| p.application())
        .map(|app| app.hold());
    glib::MainContext::default().spawn_local(async move {
        let prepared =
            gio::spawn_blocking(move || wordcraft_gnome::printing::PrintFile::prepare(document))
                .await;
        let Some(ui) = weak.upgrade() else {
            return;
        };
        let Some(parent) = parent
            .upgrade()
            .filter(|_| !ui.closed.get() && !cancel.is_cancelled())
        else {
            finish_print(&ui, PrintOutcome::Dismissed);
            return;
        };
        let file = match prepared {
            Ok(Ok(file)) => file,
            result => {
                let message = match result {
                    Ok(Err(e)) => e,
                    _ => wordcraft_gnome::t!("Could Not Prepare Print PDF").into(),
                };
                ui.toasts.add_toast(Toast::new(&message));
                finish_print(&ui, PrintOutcome::Failed);
                return;
            }
        };
        let dialog = gtk::PrintDialog::new();
        dialog.set_title(&title);
        dialog.set_modal(true);
        let source = gio::File::for_path(file.path());
        let weak = Rc::downgrade(&ui);
        dialog.print_file(Some(&parent), None, &source, Some(&cancel), move |result| {
            // Retain the source until GTK/portal has consumed its descriptor.
            drop(file);
            drop(hold);
            if let Some(ui) = weak.upgrade() {
                let outcome = match result {
                    Ok(()) => {
                        if !ui.closed.get() {
                            ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                                "Document Sent to Printer"
                            )));
                        }
                        PrintOutcome::Submitted
                    }
                    Err(e)
                        if e.matches(gtk::DialogError::Cancelled)
                            || e.matches(gtk::DialogError::Dismissed)
                            || e.matches(gio::IOErrorEnum::Cancelled) =>
                    {
                        PrintOutcome::Dismissed
                    }
                    Err(e) => {
                        if !ui.closed.get() {
                            ui.toasts
                                .add_toast(Toast::new(&wordcraft_gnome::i18n::interpolate(
                                    wordcraft_gnome::t!("Could Not Print: {e}"),
                                    &[("e", format!("{}", e))],
                                )));
                        }
                        PrintOutcome::Failed
                    }
                };
                finish_print(&ui, outcome);
            }
        });
    });
}
