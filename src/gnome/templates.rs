//! Transient template/recovery rows belong to each untouched blank document.
use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) struct Gallery {
    ui: std::rc::Weak<Ui>,
    identity: Option<(u64, u64)>,
    strip: gtk::Box,
    template_group: gtk::Box,
    recovery_group: gtk::Box,
    recovery_strip: gtk::Box,
    inventory: RefCell<Vec<(String, String, String)>>,
    warning: RefCell<Option<String>>,
    cancelled: Arc<AtomicBool>,
    source: RefCell<Option<glib::SourceId>>,
    scan: RefCell<Option<glib::JoinHandle<()>>>,
    rescan: Cell<bool>,
    previews: RefCell<Option<glib::JoinHandle<()>>>,
    generation: Cell<u64>,
    load: RefCell<Option<glib::JoinHandle<()>>>,
}
impl Drop for Gallery {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(source) = self.source.get_mut().take() {
            source.remove();
        }
        for task in [&mut self.scan, &mut self.load, &mut self.previews] {
            if let Some(task) = task.get_mut().take() {
                task.abort();
            }
        }
    }
}

// The sidebar Stack owns the reversible transition; no page-scroll
// compensation or frame-clock worker is needed when the document stays put.
pub(super) fn keeps_start_state(ui: &Ui, document: u64) -> bool {
    ui.template_eligible.get() && ui.template_document.get() == Some(document)
}
fn clear(ui: &Ui) {
    if ui.sidebar_pages.visible_child_name().as_deref() == Some("start") {
        ui.sidebar_pages.set_visible_child_name("navigation");
    }
    if ui.sidebar_pages.is_transition_running() {
        return;
    }
    ui.template_revealer.set_reveal_child(false);
    ui.template_revealer.set_child(None::<&gtk::Widget>);
    ui.template_slot.set_visible(false);
}
pub(super) fn finish_transition(ui: &Ui) {
    if !ui.sidebar_pages.is_transition_running()
        && ui.sidebar_pages.visible_child_name().as_deref() != Some("start")
        && ui.template_gallery.borrow().is_none()
    {
        clear(ui);
    }
}
pub(super) fn dismiss(ui: &Ui) {
    ui.template_eligible.set(false);
    ui.template_document.set(None);
    ui.template_gallery.borrow_mut().take();
    clear(ui);
}
pub(super) fn sync_after_edit(ui: &Rc<Ui>) {
    if !ui.template_eligible.get() || ui.closed.get() {
        return;
    }
    let empty = ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| wordcraft_gnome::templates::is_empty(&d.session));
    if empty {
        let fresh = ui
            .template_gallery
            .borrow()
            .as_ref()
            .is_none_or(|g| g.identity != document_identity(ui));
        if fresh {
            reload(ui);
        }
    } else {
        ui.template_gallery.borrow_mut().take();
        clear(ui);
    }
}
pub(super) fn offer(ui: &Rc<Ui>) {
    dismiss(ui);
    ui.template_eligible.set(true);
    ui.template_document
        .set(document_identity(ui).map(|id| id.0));
    reload(ui);
}
fn row(_title: &str, name: &str) -> (gtk::Box, gtk::Box) {
    let group = gtk::Box::new(gtk::Orientation::Vertical, 0);
    group.set_widget_name(name);
    group.set_visible(false);
    let strip = gtk::Box::new(gtk::Orientation::Vertical, 0);
    strip.set_valign(gtk::Align::Start);
    // Both kinds live in the one outer Start scroller; no section chrome.
    if name == "start-templates" {
        strip.set_widget_name("personal-template-gallery");
    }
    group.append(&strip);
    (group, strip)
}
fn card_layout() -> gtk::Box {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
    content.set_halign(gtk::Align::Center);
    content.set_vexpand(false);
    content
}
fn thumbnail_button(button: &gtk::Button) {
    button.add_css_class("flat");
    button.add_css_class("thumbnail-button");
    button.set_halign(gtk::Align::Center);
    button.set_valign(gtk::Align::Start);
    button.set_vexpand(false);
    button.set_margin_top(6);
    button.set_margin_bottom(10);
}
// A label's natural width must never stretch the paper background. The outer
// slot is common to all cards; only the proportionally sized page owns a card.
fn preview_slot() -> gtk::Box {
    let slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    slot.set_size_request(168, 238);
    slot.set_vexpand(false);
    slot.set_valign(gtk::Align::Center);
    let icon = gtk::Image::from_icon_name("text-x-generic-symbolic");
    icon.set_pixel_size(32);
    icon.set_vexpand(false);
    slot.append(&icon);
    slot
}
fn set_preview(slot: &gtk::Box, image: &wordcraft_gnome::PageImage) {
    let scale = viewer_layout::thumbnail_scale(image.width as f32) as f64;
    let width = (image.width as f64 * scale).ceil().max(1.) as i32;
    let height = (image.height as f64 * scale).ceil().max(1.) as i32;
    slot.set_size_request(width, height);
    let paintable = page_paintable::new(
        wordcraft_gnome::PageImage {
            width: image.width,
            height: image.height,
            rgba: image.rgba.clone(),
            total_pages: image.total_pages,
        },
        width,
        height,
    );
    let picture = gtk::Picture::for_paintable(&paintable);
    picture.set_size_request(width, height);
    picture.set_halign(gtk::Align::Center);
    picture.set_valign(gtk::Align::Center);
    picture.set_vexpand(false);
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.add_css_class("thumbnail-sheet");
    picture.set_overflow(gtk::Overflow::Hidden);
    clear_box(slot);
    slot.append(&picture);
}
fn card_title(name: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(name));
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.set_xalign(0.5);
    label.set_hexpand(true);
    label.set_max_width_chars(20);
    label.set_width_chars(1);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label.set_lines(2);
    label
}
fn reload(ui: &Rc<Ui>) {
    if !ui.template_eligible.get()
        || ui.closed.get()
        || !ui
            .current
            .borrow()
            .as_ref()
            .is_some_and(|d| wordcraft_gnome::templates::is_empty(&d.session))
    {
        return;
    }
    ui.template_gallery.borrow_mut().take();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let (template_group, strip) = row(wordcraft_gnome::t!("Templates"), "start-templates");
    let (recovery_group, recovery_strip) = row(
        wordcraft_gnome::t!("Documents to Recover"),
        "start-recovery",
    );
    content.append(&recovery_group);
    content.append(&template_group);
    ui.template_revealer.set_transition_duration(0);
    ui.template_revealer.set_child(Some(&content));
    ui.template_revealer.set_sensitive(true);
    ui.template_slot.set_visible(false);
    ui.template_revealer.set_reveal_child(true);
    ui.template_revealer.set_transition_duration(0);
    let cancelled = Arc::new(AtomicBool::new(false));
    let gallery = Rc::new(Gallery {
        ui: Rc::downgrade(ui),
        identity: document_identity(ui),
        strip,
        template_group,
        recovery_group,
        recovery_strip,
        inventory: RefCell::new(Vec::new()),
        warning: RefCell::new(None),
        cancelled: cancelled.clone(),
        source: RefCell::new(None),
        scan: RefCell::new(None),
        rescan: Cell::new(false),
        previews: RefCell::new(None),
        generation: Cell::new(0),
        load: RefCell::new(None),
    });
    ui.template_gallery.replace(Some(gallery.clone()));
    gallery.refresh_recovery();
    let folder = wordcraft_gnome::templates::folder();
    gallery.folder_footer(folder.as_deref());
    gallery.present();
    let Some(folder) = folder else { return };
    let (send, receive) = std::sync::mpsc::channel();
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
        if gallery.valid_ui().is_none() {
            gallery.source.borrow_mut().take();
            return glib::ControlFlow::Break;
        }
        for _ in 0..4 {
            match receive.try_recv() {
                Ok((path, image)) => {
                    gallery.add(&path, &image);
                    gallery.template_group.set_visible(true);
                    gallery.present();
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
    fn present(&self) {
        let Some(ui) = self.valid_ui() else { return };
        // Empty Start lists still retain their fixed folder action.
        ui.template_slot.set_visible(true);
        ui.sidebar_pages.set_visible_child_name("start");
    }
    fn folder_footer(self: &Rc<Self>, folder: Option<&Path>) {
        let Some(ui) = self.valid_ui() else { return };
        clear_box(&ui.start_footer);
        let button = gtk::Button::new();
        let label = gtk::Label::new(Some(wordcraft_gnome::t!("Open Templates Folder")));
        label.set_wrap(true);
        label.set_max_width_chars(20);
        label.set_width_chars(1);
        button.set_child(Some(&label));
        button.set_widget_name("start-open-template-folder");
        button.set_margin_start(6);
        button.set_margin_end(6);
        button.set_margin_top(5);
        button.set_margin_bottom(5);
        button.set_tooltip_text(folder.map(|p| p.display().to_string()).as_deref());
        accessible_name(&button, wordcraft_gnome::t!("Open Templates Folder"));
        ui.start_footer.append(&button);
        let path = folder.map(Path::to_path_buf);
        let weak = Rc::downgrade(self);
        button.connect_clicked(move |_| {
            let Some(ui) = weak.upgrade().and_then(|g| g.valid_ui()) else {
                return;
            };
            let Some(host) = ui.host.upgrade() else {
                return;
            };
            let Some(path) = path.as_ref().filter(|p| p.is_dir()) else {
                ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                    "The Templates folder is unavailable."
                )));
                return;
            };
            let launcher = gtk::FileLauncher::new(Some(&gio::File::for_path(&path)));
            launcher.set_writable(false);
            let weak = Rc::downgrade(&ui);
            launcher.launch(Some(&host.window), gio::Cancellable::NONE, move |result| {
                if std::env::args().any(|arg| arg == "--workflows40-folder-test") {
                    eprintln!("WORKFLOWS40_FOLDER_RESULT success={}", result.is_ok());
                }
                if let (Some(ui), Err(error)) = (weak.upgrade().filter(|u| !u.closed.get()), result)
                {
                    ui.toasts.add_toast(Toast::new(&error.to_string()));
                }
            });
        });
    }
    fn confirm_discard(self: &Rc<Self>, id: &str, saved_at: &str, name: &str) {
        let Some(ui) = self.valid_ui() else { return };
        let Some(host) = ui.host.upgrade() else {
            return;
        };
        let body = wordcraft_gnome::i18n::interpolate(
            wordcraft_gnome::t!(
                "The recovery copy of “{name}” will be permanently deleted. This cannot be undone."
            ),
            &[("name", name.into())],
        );
        let dialog =
            adw::AlertDialog::new(Some(wordcraft_gnome::t!("Discard Recovery?")), Some(&body));
        dialog.add_response("cancel", wordcraft_gnome::t!("Cancel"));
        dialog.add_response("discard", wordcraft_gnome::t!("Discard"));
        dialog.set_response_appearance("discard", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");
        let weak = Rc::downgrade(self);
        let (id, stamp) = (id.to_owned(), saved_at.to_owned());
        dialog.connect_response(None, move |_, response| {
            if response != "discard" {
                return;
            }
            let Some(gallery) = weak.upgrade().filter(|g| g.valid_ui().is_some()) else {
                return;
            };
            let weak = Rc::downgrade(&gallery);
            let root = wordcraft_gnome::preferences::recovery_root();
            let (id, stamp) = (id.clone(), stamp.clone());
            glib::spawn_future_local(async move {
                let result = gio::spawn_blocking(move || {
                    wordcraft_gnome::recovery::discard(&root, &id, &stamp)
                })
                .await;
                if let Some(gallery) = weak.upgrade() {
                    match result {
                        Ok(Ok(())) => {}
                        Ok(Err(error)) => gallery.warning(Some(error)),
                        Err(_) => gallery.warning(Some(
                            wordcraft_gnome::t!("Could Not Load Recovery List").into(),
                        )),
                    }
                }
                refresh_recoveries();
            });
        });
        dialog.present(Some(&host.window));
    }
    pub(super) fn inventory_ready(&self) -> bool {
        self.scan.borrow().is_none()
    }
    fn valid_ui(&self) -> Option<Rc<Ui>> {
        self.ui.upgrade().filter(|u| {
            !u.closed.get() && u.template_eligible.get() && document_identity(u) == self.identity
        })
    }
    fn warning(&self, message: Option<String>) {
        if *self.warning.borrow() == message {
            return;
        }
        if let (Some(ui), Some(message)) = (self.valid_ui(), message.as_ref()) {
            ui.toasts.add_toast(Toast::new(message));
        }
        self.warning.replace(message);
    }
    pub(super) fn refresh_recovery(self: &Rc<Self>) {
        if self.valid_ui().is_none() {
            return;
        }
        if self.scan.borrow().is_some() {
            self.rescan.set(true);
            return;
        }
        let weak = Rc::downgrade(self);
        let root = wordcraft_gnome::preferences::recovery_root();
        let task = glib::spawn_future_local(async move {
            let result =
                gio::spawn_blocking(move || wordcraft_gnome::recovery::inventory(&root)).await;
            let Some(gallery) = weak.upgrade().filter(|g| g.valid_ui().is_some()) else {
                return;
            };
            gallery.scan.borrow_mut().take();
            if gallery.rescan.replace(false) {
                // A restore/discard in another tab supersedes this inventory.
                // Never briefly expose stale rows (or their discard buttons).
                gallery.refresh_recovery();
                return;
            }
            match result {
                Ok(Ok(mut inventory)) => {
                    inventory
                        .documents
                        .sort_by(|a, b| b.saved_at.cmp(&a.saved_at).then_with(|| a.id.cmp(&b.id)));
                    gallery.warning(inventory.warning);
                    let key: Vec<_> = inventory
                        .documents
                        .iter()
                        .map(|d| (d.id.clone(), d.name.clone(), d.saved_at.clone()))
                        .collect();
                    if *gallery.inventory.borrow() == key {
                        return;
                    }
                    gallery.inventory.replace(key);
                    let generation = gallery.generation.get().wrapping_add(1);
                    gallery.generation.set(generation);
                    if let Some(task) = gallery.previews.borrow_mut().take() {
                        task.abort();
                    }
                    clear_box(&gallery.recovery_strip);
                    gallery
                        .recovery_group
                        .set_visible(!inventory.documents.is_empty());
                    let mut previews = Vec::new();
                    for record in inventory.documents {
                        let button = gtk::Button::new();
                        button.set_widget_name("start-recovery-document");
                        thumbnail_button(&button);
                        let content = card_layout();
                        let slot = preview_slot();
                        slot.set_widget_name("recovery-preview");
                        content.append(&slot);
                        previews.push((
                            record.id.clone(),
                            record.saved_at.clone(),
                            slot.downgrade(),
                        ));
                        let name = if record.name == "Untitled draft" {
                            wordcraft_gnome::t!("Recovered Document").to_owned()
                        } else {
                            record.name.clone()
                        };
                        let labels = gtk::Box::new(gtk::Orientation::Vertical, 4);
                        labels.set_hexpand(true);
                        labels.set_valign(gtk::Align::Center);
                        labels.append(&card_title(&name));
                        let date = glib::DateTime::from_iso8601(&record.saved_at, None)
                            .and_then(|d| d.to_local())
                            .and_then(|d| d.format("%x %X"))
                            .map(|d| d.to_string())
                            .unwrap_or_else(|_| record.saved_at.clone());
                        let timestamp = gtk::Label::new(Some(&date));
                        timestamp.set_max_width_chars(20);
                        timestamp.set_xalign(0.5);
                        timestamp.set_ellipsize(gtk::pango::EllipsizeMode::End);
                        timestamp.add_css_class("dim-label");
                        labels.append(&timestamp);
                        content.append(&labels);
                        button.set_child(Some(&content));
                        button.set_tooltip_text(Some(&format!("{name} — {date}")));
                        accessible_name(&button, &format!("{name} — {date}"));
                        let row = gtk::Overlay::new();
                        row.set_child(Some(&button));
                        let discard = gtk::Button::from_icon_name("user-trash-symbolic");
                        discard.add_css_class("flat");
                        discard.set_valign(gtk::Align::Start);
                        discard.set_halign(gtk::Align::End);
                        discard.set_margin_top(6);
                        discard.set_margin_end(6);
                        discard.set_widget_name("start-recovery-discard");
                        let label = wordcraft_gnome::i18n::interpolate(
                            wordcraft_gnome::t!("Discard recovery of “{name}”"),
                            &[("name", name.clone())],
                        );
                        discard.set_tooltip_text(Some(&label));
                        accessible_name(&discard, &label);
                        row.add_overlay(&discard);
                        let weak = Rc::downgrade(&gallery);
                        let (id, stamp) = (record.id.clone(), record.saved_at.clone());
                        discard.connect_clicked(move |_| {
                            if let Some(gallery) = weak.upgrade() {
                                gallery.confirm_discard(&id, &stamp, &name);
                            }
                        });
                        let weak = Rc::downgrade(&gallery);
                        button.connect_clicked(move |_| {
                            if let Some(gallery) = weak.upgrade() {
                                gallery.recover(record.id.clone());
                            }
                        });
                        gallery.recovery_strip.append(&row);
                    }
                    gallery.present();
                    let weak = Rc::downgrade(&gallery);
                    let root = wordcraft_gnome::preferences::recovery_root();
                    let task = glib::spawn_future_local(async move {
                        for (id, stamp, slot) in previews {
                            let Some(g) = weak.upgrade().filter(|g| {
                                g.valid_ui().is_some() && g.generation.get() == generation
                            }) else {
                                return;
                            };
                            drop(g);
                            let root = root.clone();
                            let result = gio::spawn_blocking(move || {
                                wordcraft_gnome::recovery::preview(&root, &id, &stamp)
                            })
                            .await;
                            let Some(g) = weak.upgrade().filter(|g| {
                                g.valid_ui().is_some() && g.generation.get() == generation
                            }) else {
                                return;
                            };
                            if let (Some(slot), Ok(Ok(image))) = (slot.upgrade(), result) {
                                set_preview(&slot, &image);
                            }
                            drop(g);
                        }
                    });
                    gallery.previews.replace(Some(task));
                }
                Ok(Err(error)) => gallery.warning(Some(error)),
                Err(_) => gallery.warning(Some(
                    wordcraft_gnome::t!("Could Not Load Recovery List").into(),
                )),
            }
        });
        self.scan.replace(Some(task));
    }
    fn add(self: &Rc<Self>, path: &Path, image: &wordcraft_gnome::PageImage) {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let button = gtk::Button::new();
        button.set_widget_name("personal-template");
        thumbnail_button(&button);
        button.set_tooltip_text(Some(&name));
        accessible_name(&button, &name);
        let content = card_layout();
        let slot = preview_slot();
        slot.set_widget_name("template-preview");
        set_preview(&slot, image);
        content.append(&slot);
        content.append(&card_title(&name));
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
    fn accept(&self, session: Session, recovered: bool) {
        let Some(ui) = self.valid_ui() else {
            return;
        };
        opened(
            &ui,
            session,
            if recovered {
                wordcraft_gnome::t!("Recovered Document")
            } else {
                wordcraft_gnome::t!("New Document")
            },
        );
        if recovered {
            ui.dirty.set(true);
            sync_tab(&ui);
            refresh_recoveries();
        }
        if ui
            .host
            .upgrade()
            .and_then(|h| selected_ui(&h))
            .is_some_and(|s| Rc::ptr_eq(&s, &ui))
        {
            ui.document_box.grab_focus();
        }
    }
    fn choose(self: &Rc<Self>, path: PathBuf) {
        if self.valid_ui().is_none() {
            return;
        }
        if let Some(old) = self.load.borrow_mut().take() {
            old.abort();
        }
        let weak = Rc::downgrade(self);
        let task = glib::spawn_future_local(async move {
            let result = gio::spawn_blocking(move || wordcraft_gnome::templates::open(&path)).await;
            let Some(gallery) = weak.upgrade().filter(|g| g.valid_ui().is_some()) else {
                return;
            };
            gallery.load.borrow_mut().take();
            match result {
                Ok(Ok(session)) => gallery.accept(session, false),
                Ok(Err(e)) => gallery.warning(Some(e)),
                Err(_) => {
                    gallery.warning(Some(wordcraft_gnome::t!("Could Not Open Template").into()))
                }
            }
        });
        self.load.replace(Some(task));
    }
    fn recover(self: &Rc<Self>, id: String) {
        if self.valid_ui().is_none() || self.load.borrow().is_some() {
            return;
        }
        let weak = Rc::downgrade(self);
        let root = wordcraft_gnome::preferences::recovery_root();
        let task = glib::spawn_future_local(async move {
            let result =
                gio::spawn_blocking(move || wordcraft_gnome::recovery::restore(&root, &id)).await;
            let Some(gallery) = weak.upgrade().filter(|g| g.valid_ui().is_some()) else {
                return;
            };
            gallery.load.borrow_mut().take();
            match result {
                Ok(Ok(session)) => gallery.accept(session, true),
                Ok(Err(e)) => {
                    gallery.warning(Some(e));
                    gallery.refresh_recovery();
                }
                Err(_) => gallery.warning(Some(
                    wordcraft_gnome::t!("Could Not Load Recovery List").into(),
                )),
            }
        });
        self.load.replace(Some(task));
    }
}
pub(super) fn refresh_if_visible(ui: &Ui) {
    if ui.template_slot.is_mapped() {
        if let Some(gallery) = ui.template_gallery.borrow().as_ref() {
            gallery.refresh_recovery();
        }
    }
}
fn refresh_recoveries() {
    WORKSPACES.with(|hosts| {
        for host in hosts.borrow().iter() {
            for (_, ui) in host.docs.borrow().iter() {
                if let Some(gallery) = ui.template_gallery.borrow().as_ref() {
                    gallery.refresh_recovery();
                }
            }
        }
    });
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
