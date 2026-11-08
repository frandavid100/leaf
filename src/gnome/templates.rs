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
    ui.template_eligible.set(false);
    ui.template_gallery.borrow_mut().take();
    clear_box(&ui.template_slot);
    ui.template_slot.set_visible(false);
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
pub(super) fn preferences(parent: &adw::PreferencesWindow) -> adw::PreferencesGroup {
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
        let Some(parent) = weak.upgrade() else {
            return;
        };
        let chooser = gtk::FileDialog::new();
        chooser.set_title(wordcraft_gnome::t!("Choose Templates Folder"));
        chooser.set_accept_label(Some(wordcraft_gnome::t!("_Use Folder")));
        let cancel = gio::Cancellable::new();
        let cancel_ref = cancel.clone();
        parent.connect_visible_notify(move |p| {
            if !p.is_visible() {
                cancel_ref.cancel();
            }
        });
        let parent_ref = parent.downgrade();
        let row_ref = row_ref.clone();
        let reset_ref = reset_ref.clone();
        chooser.select_folder(Some(&parent), Some(&cancel), move |result| {
            let Ok(file) = result else { return };
            let Some(path) = file.path() else { return };
            glib::spawn_future_local(async move {
                let result = read_only_grant(&path).await;
                let Some(parent) = parent_ref.upgrade().filter(|p| p.is_visible()) else {
                    return;
                };
                match result.and_then(|()| wordcraft_gnome::preferences::set_template_folder(&path))
                {
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

// GTK's generic OpenFile folder chooser may grant write access. Templates only
// need read: narrow this app's selected document-portal entry before persisting it.
async fn read_only_grant(path: &Path) -> Result<(), String> {
    let Ok(app_id) = std::env::var("FLATPAK_ID") else {
        return Ok(());
    };
    let connection = gio::bus_get_future(gio::BusType::Session)
        .await
        .map_err(|e| e.to_string())?;
    let bus = "org.freedesktop.portal.Documents";
    let object = "/org/freedesktop/portal/documents";
    let mount = connection
        .call_future(
            Some(bus),
            object,
            bus,
            "GetMountPoint",
            None,
            None,
            gio::DBusCallFlags::NONE,
            5000,
        )
        .await
        .map_err(|e| e.to_string())?;
    let (bytes,) = mount
        .get::<(Vec<u8>,)>()
        .ok_or_else(|| "Invalid document portal mount point".to_owned())?;
    use std::os::unix::ffi::OsStringExt;
    let mount = PathBuf::from(std::ffi::OsString::from_vec(
        bytes.into_iter().take_while(|b| *b != 0).collect(),
    ));
    if let Ok(relative) = path.strip_prefix(mount) {
        let id = relative
            .components()
            .next()
            .and_then(|v| v.as_os_str().to_str())
            .ok_or_else(|| "Invalid document portal path".to_owned())?;
        let parameters = (id, app_id, vec!["write", "delete", "grant-permissions"]).to_variant();
        connection
            .call_future(
                Some(bus),
                object,
                bus,
                "RevokePermissions",
                Some(&parameters),
                None,
                gio::DBusCallFlags::NONE,
                5000,
            )
            .await
            .map_err(|e| e.to_string())?;
    }
    let info = gio::File::for_path(path)
        .query_info_future(
            "access::can-read,access::can-write",
            gio::FileQueryInfoFlags::NONE,
            glib::Priority::DEFAULT,
        )
        .await
        .map_err(|e| e.to_string())?;
    if !info.boolean("access::can-read") || info.boolean("access::can-write") {
        return Err(wordcraft_gnome::t!(
            "The templates folder must be readable without write access."
        )
        .into());
    }
    Ok(())
}
