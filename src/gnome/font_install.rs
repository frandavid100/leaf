//! System font installation stays outside Leaf, through GTK's file portal.
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wordcraft_gnome::font_install as fonts;

fn current_id(ui: &Ui) -> Option<u64> {
    ui.current
        .borrow()
        .as_ref()
        .map(|d| d.session.document_id())
}
fn report(ui: &Ui) -> Vec<fonts::MissingFont> {
    ui.current
        .borrow_mut()
        .as_mut()
        .map(|d| fonts::missing(&mut d.session))
        .unwrap_or_default()
}
pub(super) fn opened(ui: &Rc<Ui>) {
    // Legacy synthetic UI probes have their own modal workflows. The dedicated
    // font279 probe exercises this automatic notice with normal product behavior.
    let args = std::env::args().collect::<Vec<_>>();
    if args.iter().any(|a| a.ends_with("-test")) && !args.iter().any(|a| a == "--font279-ui-test") {
        return;
    }
    let Some(id) = current_id(ui) else { return };
    if ui.font_notice_document.replace(Some(id)) == Some(id) {
        return;
    }
    let old = ui.font_install_dialog.borrow_mut().take();
    if let Some(dialog) = old {
        dialog.close();
    }
    let path = ui.source_path.borrow().is_some();
    if !path {
        return;
    }
    let weak = Rc::downgrade(ui);
    glib::idle_add_local_once(move || {
        let Some(ui) = weak
            .upgrade()
            .filter(|u| !u.closed.get() && current_id(u) == Some(id))
        else {
            return;
        };
        let missing = report(&ui);
        if missing.is_empty() {
            return;
        }
        let dialog = adw::AlertDialog::new(
            Some(wordcraft_gnome::t!("Some Fonts Are Unavailable")),
            Some(wordcraft_gnome::t!(
                "This document uses fonts that are not available. Substitutions can change its appearance and pagination. The original font names have been preserved."
            )),
        );
        dialog.add_response("continue", wordcraft_gnome::t!("Continue with Substitutes"));
        dialog.add_response("fonts", wordcraft_gnome::t!("Review Fonts"));
        dialog.set_default_response(Some("fonts"));
        dialog.set_close_response("continue");
        let list = gtk::Label::new(Some(
            &missing
                .iter()
                .map(|f| format!("{} → {}", f.font, f.shown_as.join(", ")))
                .collect::<Vec<_>>()
                .join("\n"),
        ));
        list.set_wrap(true);
        list.set_xalign(0.);
        let scroll = gtk::ScrolledWindow::builder()
            .child(&list)
            .max_content_height(220)
            .propagate_natural_height(true)
            .build();
        dialog.set_extra_child(Some(&scroll));
        let weak = Rc::downgrade(&ui);
        dialog.connect_response(None, move |_, response| {
            if response == "fonts" {
                if let Some(ui) = weak
                    .upgrade()
                    .filter(|u| !u.closed.get() && current_id(u) == Some(id))
                {
                    show(&ui);
                }
            }
        });
        dialog.present(Some(&ui.document_box));
    });
}

/// The worker owns no document/widgets. Closing the dialog cancels its bounded download.
fn download(
    button: &gtk::Button,
    status: &gtk::Label,
    dialog: &adw::Dialog,
    family: String,
    style: String,
    cancelled: Arc<AtomicBool>,
    path: Rc<RefCell<Option<PathBuf>>>,
) {
    button.set_sensitive(false);
    status.set_text(wordcraft_gnome::t!("Downloading font and license…"));
    let (tx, rx) = std::sync::mpsc::channel();
    // Keep files alive after handoff: the host viewer may read them after Leaf's callback.
    let folder = glib::user_cache_dir()
        .join("leaf/font-downloads")
        .join(fonts::CATALOG_REV)
        .join("Poppins")
        .join(format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
    let cancel = cancelled.clone();
    std::thread::spawn(move || {
        let _ = tx.send(fonts::download(&family, &style, &folder, &cancel));
    });
    let weak_button = button.downgrade();
    let weak_status = status.downgrade();
    let weak_dialog = dialog.downgrade();
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        let (Some(button), Some(status), Some(dialog)) = (
            weak_button.upgrade(),
            weak_status.upgrade(),
            weak_dialog.upgrade(),
        ) else {
            return glib::ControlFlow::Break;
        };
        if cancelled.load(Ordering::Acquire) {
            return glib::ControlFlow::Break;
        }
        match rx.try_recv() {
            Ok(Ok(file)) => {
                path.replace(Some(file.clone()));
                button.set_sensitive(true);
                button.set_label(wordcraft_gnome::t!("Open Font File"));
                status.set_text(wordcraft_gnome::t!(
                    "Downloaded. Install this style in the system font viewer, then check again."
                ));
                launch(&dialog, &file, &status);
                glib::ControlFlow::Break
            }
            Ok(Err(e)) => {
                button.set_sensitive(true);
                status.set_text(&format!("{} {e}", wordcraft_gnome::t!("Could not download this font. Check your connection and try again, or open a local font file.")));
                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(_) => {
                button.set_sensitive(true);
                status.set_text(wordcraft_gnome::t!("The font download failed."));
                glib::ControlFlow::Break
            }
        }
    });
}
fn launch(dialog: &adw::Dialog, path: &Path, status: &gtk::Label) {
    let Some(parent) = dialog.root().and_downcast::<gtk::Window>() else {
        return;
    };
    let launcher = gtk::FileLauncher::new(Some(&gio::File::for_path(path)));
    let cancel = gio::Cancellable::new();
    let pending = cancel.clone();
    dialog.connect_closed(move |_| pending.cancel());
    let status = status.downgrade();
    let dialog = dialog.downgrade();
    let file = path.to_owned();
    launcher.launch(Some(&parent), Some(&cancel), move |result| {
        let (Some(status), Some(dialog)) = (status.upgrade(), dialog.upgrade()) else { return; };
        if let Err(error) = result {
            if error.matches(gio::IOErrorEnum::Cancelled) { return; }
            status.set_text(wordcraft_gnome::t!("No font viewer could open this file. Save it and open it with your system's font installer."));
            let alert = adw::AlertDialog::new(Some(wordcraft_gnome::t!("Could Not Open the Font Viewer")), Some(&error.to_string()));
            alert.add_response("close", wordcraft_gnome::t!("Close")); alert.add_response("save", wordcraft_gnome::t!("Save Font File…"));
            let weak = dialog.downgrade();
            alert.connect_response(None, move |_, response| { if response == "save" { if let Some(dialog) = weak.upgrade() { save(&dialog, &file); } } });
            alert.present(Some(&dialog));
        }
    });
}
fn save(dialog: &adw::Dialog, source: &Path) {
    let Some(parent) = dialog.root().and_downcast::<gtk::Window>() else {
        return;
    };
    let chooser = gtk::FileDialog::new();
    chooser.set_title(wordcraft_gnome::t!("Save Font File"));
    chooser.set_initial_name(source.file_name().and_then(|s| s.to_str()));
    let source = source.to_owned();
    let weak = dialog.downgrade();
    let cancel = gio::Cancellable::new();
    let c = cancel.clone();
    dialog.connect_closed(move |_| c.cancel());
    chooser.save(Some(&parent), Some(&cancel), move |result| {
        let Ok(file) = result else {
            return;
        };
        let Some(dialog) = weak.upgrade().filter(|d| d.root().is_some()) else {
            return;
        };
        let result = file
            .path()
            .ok_or_else(|| wordcraft_gnome::t!("Choose a local file.").to_string())
            .and_then(|path| {
                std::fs::copy(&source, path)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            });
        if let Err(e) = result {
            let alert =
                adw::AlertDialog::new(Some(wordcraft_gnome::t!("Could Not Save")), Some(&e));
            alert.add_response("close", wordcraft_gnome::t!("Close"));
            alert.present(Some(&dialog));
        }
    });
}
fn local_file(dialog: &adw::Dialog, status: &gtk::Label) {
    let Some(parent) = dialog.root().and_downcast::<gtk::Window>() else {
        return;
    };
    let chooser = gtk::FileDialog::new();
    chooser.set_title(wordcraft_gnome::t!("Open Font File"));
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(wordcraft_gnome::t!("Font Files")));
    filter.add_pattern("*.ttf");
    filter.add_pattern("*.otf");
    chooser.set_default_filter(Some(&filter));
    let cancel = gio::Cancellable::new();
    let c = cancel.clone();
    dialog.connect_closed(move |_| c.cancel());
    let weak = dialog.downgrade();
    let status = status.downgrade();
    chooser.open(Some(&parent), Some(&cancel), move |result| {
        let (Ok(file), Some(dialog), Some(status)) = (result, weak.upgrade(), status.upgrade())
        else {
            return;
        };
        let Some(path) = file.path() else {
            return;
        };
        let valid = fonts::read_local(&path);
        match valid {
            Ok(_) => launch(&dialog, &path, &status),
            Err(e) => status.set_text(&e),
        }
    });
}
fn refresh(ui: &Rc<Ui>, status: &gtk::Label) {
    let names = ui
        .current
        .borrow()
        .as_ref()
        .map(|d| {
            wordcraft_gnome::document_fonts::inventory(&d.session)
                .into_iter()
                .filter(|f| !f.uses.is_empty())
                .map(|f| f.name)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    fonts::refresh_installed(&names);
    let documents = WORKSPACES.with(|hosts| {
        let mut documents = Vec::new();
        for host in hosts.borrow().iter() {
            documents.extend(host.docs.borrow().iter().map(|(_, ui)| ui.clone()));
        }
        documents
    });
    for ui in documents.into_iter().filter(|u| !u.closed.get()) {
        let dirty = ui.dirty.get();
        let anchor = capture_reading_anchor(&ui);
        if let Some(doc) = ui.current.borrow_mut().as_mut() {
            doc.session.relayout();
        }
        refresh_after_content_change(&ui);
        ui.dirty.set(dirty);
        sync_tab(&ui);
        restore_reading_anchor(&ui, anchor);
        if let Some(controls) = ui.text_controls.borrow().as_ref() {
            controls.invalidate_font_catalog();
        }
    }
    document_fonts::refresh_open(ui);
    let missing = report(ui);
    status.set_text(&if missing.is_empty() {
        wordcraft_gnome::t!("All fonts used by this document are now available.").into()
    } else {
        format!(
            "{} {}",
            wordcraft_gnome::t!("Still unavailable:"),
            missing
                .iter()
                .map(|f| if f.faces.is_empty() {
                    f.font.clone()
                } else {
                    format!("{} ({})", f.font, f.faces.join(", "))
                })
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
}
pub(super) fn show(ui: &Rc<Ui>) {
    if let Some(dialog) = ui.font_install_dialog.borrow().as_ref() {
        dialog.present(Some(&ui.document_box));
        return;
    }
    let Some(id) = current_id(ui) else {
        return;
    };
    let dialog = adw::Dialog::builder()
        .title(wordcraft_gnome::t!("Document Fonts"))
        .content_width(520)
        .content_height(560)
        .build();
    dialog.set_widget_name("font-install-dialog");
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();
    group.set_description(Some(wordcraft_gnome::t!("Install each required style in your system font viewer. Opening the viewer does not install a font or replace the document's font names.")));
    let status = gtk::Label::new(None);
    status.set_wrap(true);
    status.set_xalign(0.);
    status.set_widget_name("font-install-status");
    let cancelled = Arc::new(AtomicBool::new(false));
    for font in report(ui) {
        let row = adw::ActionRow::new();
        row.set_title(&font.font);
        row.set_subtitle(&format!(
            "{} {}",
            wordcraft_gnome::t!("Not available; displayed as"),
            font.shown_as.join(", ")
        ));
        group.add(&row);
        if fonts::catalog_file(&font.font, "Regular").is_some() {
            for style in fonts::STYLES {
                let row = adw::ActionRow::new();
                row.set_title(match *style {
                    "Bold" => wordcraft_gnome::t!("Bold"),
                    "Italic" => wordcraft_gnome::t!("Italic"),
                    "BoldItalic" => wordcraft_gnome::t!("Bold Italic"),
                    _ => wordcraft_gnome::t!("Regular"),
                });
                row.set_subtitle("Google Fonts · SIL Open Font License");
                let button = gtk::Button::with_label(wordcraft_gnome::t!("Download and Open"));
                button.set_valign(gtk::Align::Center);
                button.set_widget_name(&format!("font-download-{style}"));
                row.add_suffix(&button);
                group.add(&row);
                let family = font.font.clone();
                let style = style.to_string();
                let weak = dialog.downgrade();
                let status = status.downgrade();
                let cancel = cancelled.clone();
                let path = Rc::new(RefCell::new(None::<PathBuf>));
                button.connect_clicked(move |button| {
                    let (Some(dialog), Some(status)) = (weak.upgrade(), status.upgrade()) else {
                        return;
                    };
                    if let Some(file) = path.borrow().as_ref() {
                        launch(&dialog, file, &status);
                    } else {
                        download(
                            button,
                            &status,
                            &dialog,
                            family.clone(),
                            style.clone(),
                            cancel.clone(),
                            path.clone(),
                        );
                    }
                });
            }
        } else {
            row.set_subtitle(&format!("{}\n{}", row.subtitle().unwrap_or_default(), wordcraft_gnome::t!("Not in the download catalog. Use a local font file or choose a replacement in Document Properties.")));
        }
    }
    let local = gtk::Button::with_label(wordcraft_gnome::t!("Open Local Font File…"));
    local.set_widget_name("font-install-local");
    let weak = dialog.downgrade();
    let status_ref = status.downgrade();
    local.connect_clicked(move |_| {
        if let (Some(dialog), Some(status)) = (weak.upgrade(), status_ref.upgrade()) {
            local_file(&dialog, &status);
        }
    });
    group.add(&local);
    let check = gtk::Button::with_label(wordcraft_gnome::t!("Check Installed Fonts"));
    check.set_widget_name("font-install-check");
    let weak = Rc::downgrade(ui);
    let label = status.downgrade();
    check.connect_clicked(move |_| {
        if let (Some(ui), Some(status)) = (
            weak.upgrade()
                .filter(|u| !u.closed.get() && current_id(u) == Some(id)),
            label.upgrade(),
        ) {
            refresh(&ui, &status);
        }
    });
    group.add(&check);
    group.add(&status);
    page.add(&group);
    toolbar.set_content(Some(&page));
    dialog.set_child(Some(&toolbar));
    if let Some(host) = ui.host.upgrade() {
        let button = check.downgrade();
        let signal = host.window.connect_is_active_notify(move |window| {
            if window.is_active() {
                if let Some(button) = button.upgrade().filter(|b| b.is_mapped()) {
                    button.emit_clicked();
                }
            }
        });
        let window = host.window.downgrade();
        let signal = RefCell::new(Some(signal));
        dialog.connect_closed(move |_| {
            if let (Some(window), Some(signal)) = (window.upgrade(), signal.borrow_mut().take()) {
                window.disconnect(signal);
            }
        });
    }
    let weak = Rc::downgrade(ui);
    dialog.connect_closed(move |_| {
        cancelled.store(true, Ordering::Release);
        if let Some(ui) = weak.upgrade() {
            ui.font_install_dialog.borrow_mut().take();
        }
    });
    ui.font_install_dialog.replace(Some(dialog.clone()));
    dialog.present(Some(&ui.document_box));
}
