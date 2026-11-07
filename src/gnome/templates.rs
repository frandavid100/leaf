//! A file-based template chooser, with no second template library.
use super::*;
pub(super) fn show(host: &Rc<Workspace>) {
    let dialog = adw::Window::builder()
        .title(wordcraft_gnome::t!("New from Template"))
        .transient_for(&host.window)
        .modal(true)
        .default_width(440)
        .default_height(420)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.append(&adw::HeaderBar::new());
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    let status = gtk::Label::new(None);
    status.set_wrap(true);
    content.append(&status);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_child(Some(&list));
    content.append(&scroll);
    let folder = wordcraft_gnome::templates::folder();
    match folder.as_ref().map(|p| wordcraft_gnome::templates::list(p)) {
        Some(Ok(paths)) => {
            if paths.is_empty() {
                status.set_text(wordcraft_gnome::t!(
                    "Add DOCX or ODT files to your templates folder."
                ));
            }
            for path in paths {
                let button = gtk::Button::with_label(
                    &path.file_stem().unwrap_or_default().to_string_lossy(),
                );
                button.set_widget_name("personal-template");
                list.append(&button);
                let weak = Rc::downgrade(host);
                let dialog = dialog.downgrade();
                button.connect_clicked(move |_| {
                    let (Some(host), Some(dialog)) = (weak.upgrade(), dialog.upgrade()) else {
                        return;
                    };
                    match wordcraft_gnome::templates::open(&path) {
                        Ok(session) => {
                            if let Some(app) = host_app(&host) {
                                build_tab(
                                    &app,
                                    &host,
                                    false,
                                    true,
                                    false,
                                    false,
                                    false,
                                    false,
                                    false,
                                    Some((session, wordcraft_gnome::t!("New Document").into())),
                                );
                                dialog.close();
                            }
                        }
                        Err(e) => {
                            if let Some(ui) = selected_ui(&host) {
                                ui.toasts.add_toast(Toast::new(&e));
                            }
                        }
                    }
                });
            }
        }
        Some(Err(e)) => status.set_text(&e),
        None => status.set_text(wordcraft_gnome::t!("Choose your templates folder.")),
    }
    let files = gtk::Button::with_label(wordcraft_gnome::t!("Open Templates Folder"));
    content.append(&files);
    files.set_sensitive(folder.is_some());
    let weak = dialog.downgrade();
    files.connect_clicked(move |_| {
        if let (Some(parent), Some(folder)) = (weak.upgrade(), folder.as_ref()) {
            let uri = gio::File::for_path(folder).uri();
            gtk::UriLauncher::new(&uri).launch(Some(&parent), gio::Cancellable::NONE, |_| {});
        }
    });
    let choose = gtk::Button::with_label(wordcraft_gnome::t!("Choose Another Folder…"));
    content.append(&choose);
    let weak = Rc::downgrade(host);
    let parent = dialog.downgrade();
    choose.connect_clicked(move |_| {
        let Some(parent) = parent.upgrade() else {
            return;
        };
        let chooser = gtk::FileDialog::new();
        let cancel = gio::Cancellable::new();
        let cancel_close = cancel.clone();
        parent.connect_close_request(move |_| {
            cancel_close.cancel();
            glib::Propagation::Proceed
        });
        let weak = weak.clone();
        let parent = parent.downgrade();
        chooser.select_folder(parent.upgrade().as_ref(), Some(&cancel), move |result| {
            let (Some(host), Some(parent)) = (weak.upgrade(), parent.upgrade()) else {
                return;
            };
            if !parent.is_visible() {
                return;
            }
            if let Ok(file) = result {
                if let Some(path) = file.path() {
                    match wordcraft_gnome::preferences::set_template_folder(&path) {
                        Ok(()) => {
                            parent.close();
                            show(&host)
                        }
                        Err(e) => {
                            if let Some(ui) = selected_ui(&host) {
                                ui.toasts.add_toast(Toast::new(&e));
                            }
                        }
                    }
                }
            }
        });
    });
    dialog.set_content(Some(&content));
    dialog.present();
}
