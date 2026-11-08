//! A transient gallery belongs to each newly created blank document.
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) struct Gallery {
    ui: std::rc::Weak<Ui>,
    identity: Option<(u64, u64)>,
    strip: gtk::Box,
    cancelled: Arc<AtomicBool>,
    source: RefCell<Option<glib::SourceId>>,
    load: RefCell<Option<glib::JoinHandle<()>>>,
}
impl Drop for Gallery {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(source) = self.source.get_mut().take() {
            source.remove();
        }
        if let Some(load) = self.load.get_mut().take() {
            load.abort();
        }
    }
}
pub(super) fn dismiss(ui: &Ui) {
    let top = document_scroll_y(ui).max(0.);
    ui.template_eligible.set(false);
    ui.template_gallery.borrow_mut().take();
    clear_box(&ui.template_slot);
    ui.template_slot.set_visible(false);
    ui.scroll.vadjustment().set_value(top);
}
pub(super) fn offer(ui: &Rc<Ui>) {
    dismiss(ui);
    ui.template_eligible.set(true);
    reload(ui);
}
fn reload(ui: &Rc<Ui>) {
    if !ui.template_eligible.get() || ui.closed.get() {
        return;
    }
    ui.template_gallery.borrow_mut().take();
    clear_box(&ui.template_slot);
    ui.template_slot.set_visible(false);
    let Some(folder) = wordcraft_gnome::templates::folder() else {
        return;
    };
    let strip = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    for side in [0, 1] {
        if side == 0 {
            strip.set_margin_start(16);
        } else {
            strip.set_margin_end(16);
        }
    }
    strip.set_margin_top(12);
    strip.set_margin_bottom(12);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&strip));
    scroll.set_widget_name("personal-template-gallery");
    ui.template_slot.append(&scroll);
    let cancelled = Arc::new(AtomicBool::new(false));
    let gallery = Rc::new(Gallery {
        ui: Rc::downgrade(ui),
        identity: document_identity(ui),
        strip,
        cancelled: cancelled.clone(),
        source: RefCell::new(None),
        load: RefCell::new(None),
    });
    ui.template_gallery.replace(Some(gallery.clone()));
    let (send, receive) = std::sync::mpsc::channel();
    // Neither directory enumeration nor loading/layout/rendering runs on GTK's thread.
    std::thread::spawn(move || {
        let Ok(paths) = wordcraft_gnome::templates::list(&folder) else {
            return;
        };
        for path in paths {
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            let image = wordcraft_gnome::templates::preview(&path);
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            if let Ok(image) = image {
                if send.send((path, image)).is_err() {
                    break;
                }
            }
        }
    });
    let weak = Rc::downgrade(&gallery);
    let source = glib::timeout_add_local(std::time::Duration::from_millis(35), move || {
        let Some(gallery) = weak.upgrade() else {
            return glib::ControlFlow::Break;
        };
        let Some(ui) = gallery.ui.upgrade().filter(|u| {
            !u.closed.get() && u.template_eligible.get() && document_identity(u) == gallery.identity
        }) else {
            gallery.cancelled.store(true, Ordering::Relaxed);
            gallery.source.borrow_mut().take();
            return glib::ControlFlow::Break;
        };
        for _ in 0..4 {
            match receive.try_recv() {
                Ok((path, image)) => {
                    gallery.add(&path, &image);
                    ui.template_slot.set_visible(true);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    gallery.source.borrow_mut().take();
                    return glib::ControlFlow::Break;
                }
            }
        }
        glib::ControlFlow::Continue
    });
    gallery.source.replace(Some(source));
}
impl Gallery {
    fn add(self: &Rc<Self>, path: &Path, image: &wordcraft_gnome::PageImage) {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let button = gtk::Button::new();
        button.set_widget_name("personal-template");
        button.add_css_class("flat");
        button.set_tooltip_text(Some(&name));
        accessible_name(&button, &name);
        let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let picture = gtk::Picture::for_paintable(&texture(wordcraft_gnome::PageImage {
            width: image.width,
            height: image.height,
            rgba: image.rgba.clone(),
            total_pages: image.total_pages,
        }));
        picture.set_size_request(104, 148);
        picture.set_content_fit(gtk::ContentFit::Contain);
        picture.add_css_class("card");
        content.append(&picture);
        let title = gtk::Label::new(Some(&name));
        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title.set_max_width_chars(16);
        content.append(&title);
        button.set_child(Some(&content));
        self.strip.append(&button);
        let weak = Rc::downgrade(self);
        let path = path.to_path_buf();
        button.connect_clicked(move |_| {
            if let Some(gallery) = weak.upgrade() {
                gallery.choose(path.clone());
            }
        });
    }
    fn choose(self: &Rc<Self>, path: PathBuf) {
        if let Some(old) = self.load.borrow_mut().take() {
            old.abort();
        }
        let weak = Rc::downgrade(self);
        let task = glib::spawn_future_local(async move {
            let result = gio::spawn_blocking(move || wordcraft_gnome::templates::open(&path)).await;
            let Some(gallery) = weak.upgrade() else {
                return;
            };
            let Some(ui) = gallery.ui.upgrade().filter(|u| {
                !u.closed.get()
                    && u.template_eligible.get()
                    && document_identity(u) == gallery.identity
            }) else {
                return;
            };
            gallery.load.borrow_mut().take();
            match result {
                Ok(Ok(session)) => {
                    // The clicked new tab is the only target, regardless of current focus.
                    opened(&ui, session, wordcraft_gnome::t!("New Document"));
                    ui.document_box.grab_focus();
                }
                Ok(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
                Err(_) => ui
                    .toasts
                    .add_toast(Toast::new(wordcraft_gnome::t!("Could Not Open Template"))),
            }
        });
        self.load.replace(Some(task));
    }
}
fn refresh_folders() {
    WORKSPACES.with(|hosts| {
        for host in hosts.borrow().iter() {
            for (_, ui) in host.docs.borrow().iter() {
                reload(ui);
            }
        }
    });
}
pub(super) fn preferences(parent: &adw::PreferencesDialog) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();
    group.set_title(wordcraft_gnome::t!("Templates"));
    let row = adw::ActionRow::new();
    row.set_title(wordcraft_gnome::t!("Templates Folder"));
    row.set_widget_name("preferences-template-folder");
    row.set_subtitle(
        &wordcraft_gnome::templates::folder()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| wordcraft_gnome::t!("Unavailable").into()),
    );
    let choose = gtk::Button::from_icon_name("folder-open-symbolic");
    choose.set_valign(gtk::Align::Center);
    choose.set_widget_name("preferences-template-choose");
    // The folder portal grants write access and cannot revoke its own grant.
    // Keep the production XDG :ro mount until a read-only chooser is supported.
    let sandboxed = std::env::var_os("FLATPAK_ID").is_some();
    choose.set_sensitive(!sandboxed);
    if sandboxed {
        choose.set_tooltip_text(Some(wordcraft_gnome::t!(
            "Custom template folders are unavailable in the sandbox. Use the system Templates folder."
        )));
    }
    accessible_name(&choose, wordcraft_gnome::t!("Choose Templates Folder"));
    let reset = gtk::Button::from_icon_name("edit-undo-symbolic");
    reset.set_valign(gtk::Align::Center);
    reset.set_widget_name("preferences-template-reset");
    accessible_name(&reset, wordcraft_gnome::t!("Use System Templates Folder"));
    reset.set_sensitive(wordcraft_gnome::preferences::template_folder().is_some());
    row.add_suffix(&reset);
    row.add_suffix(&choose);
    group.add(&row);
    let weak = parent.downgrade();
    let row_ref = row.downgrade();
    let reset_ref = reset.downgrade();
    choose.connect_clicked(move |_| {
        if sandboxed {
            return;
        }
        let Some(parent) = weak.upgrade() else {
            return;
        };
        let chooser = gtk::FileDialog::new();
        chooser.set_title(wordcraft_gnome::t!("Choose Templates Folder"));
        chooser.set_accept_label(Some(wordcraft_gnome::t!("_Use Folder")));
        let cancel = gio::Cancellable::new();
        let cancel_ref = cancel.clone();
        parent.connect_closed(move |_| cancel_ref.cancel());
        let lease = cancel.clone();
        let parent_ref = parent.downgrade();
        let row_ref = row_ref.clone();
        let reset_ref = reset_ref.clone();
        let Some(root) = parent.root().and_downcast::<gtk::Window>() else {
            return;
        };
        chooser.select_folder(Some(&root), Some(&cancel), move |result| {
            if lease.is_cancelled() {
                return;
            }
            let Ok(file) = result else { return };
            let Some(path) = file.path() else { return };
            let Some(parent) = parent_ref.upgrade().filter(|p| p.root().is_some()) else {
                return;
            };
            match wordcraft_gnome::preferences::set_template_folder(&path) {
                Ok(()) => {
                    if let Some(row) = row_ref.upgrade() {
                        row.set_subtitle(&path.display().to_string());
                    }
                    if let Some(reset) = reset_ref.upgrade() {
                        reset.set_sensitive(true);
                    }
                    refresh_folders();
                }
                Err(e) => parent.add_toast(Toast::new(&e)),
            }
        });
    });
    let weak = parent.downgrade();
    let row = row.downgrade();
    reset.connect_clicked(move |reset| {
        match wordcraft_gnome::preferences::reset_template_folder() {
            Ok(()) => {
                reset.set_sensitive(false);
                if let Some(row) = row.upgrade() {
                    row.set_subtitle(
                        &wordcraft_gnome::templates::folder()
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|| wordcraft_gnome::t!("Unavailable").into()),
                    );
                }
                refresh_folders();
            }
            Err(e) => {
                if let Some(parent) = weak.upgrade() {
                    parent.add_toast(Toast::new(&e));
                }
            }
        }
    });
    group
}
