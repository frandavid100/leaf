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
    load: RefCell<Option<glib::JoinHandle<()>>>,
}
impl Drop for Gallery {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Relaxed);
        if let Some(source) = self.source.get_mut().take() {
            source.remove();
        }
        for task in [&mut self.scan, &mut self.load] {
            if let Some(task) = task.get_mut().take() {
                task.abort();
            }
        }
    }
}

pub(super) struct Dismissal {
    document: u64,
    height: f64,
    scroll: f64,
    last_scroll: Cell<f64>,
    user_scrolled: Cell<bool>,
    clock: gdk::FrameClock,
    revealer: gtk::Revealer,
    update: RefCell<Option<glib::SignalHandlerId>>,
    done: RefCell<Option<glib::SignalHandlerId>>,
}
impl Drop for Dismissal {
    fn drop(&mut self) {
        if let Some(id) = self.update.get_mut().take() {
            self.clock.disconnect(id);
        }
        if let Some(id) = self.done.get_mut().take() {
            self.revealer.disconnect(id);
        }
    }
}
fn compensate(ui: &Ui, animation: &Dismissal, height: f64) {
    let adjustment = ui.scroll.vadjustment();
    if (adjustment.value() - animation.last_scroll.get()).abs() > 1.1 {
        animation.user_scrolled.set(true);
    }
    if !animation.user_scrolled.get() {
        let max = (adjustment.upper() - adjustment.page_size()).max(0.);
        adjustment.set_value((animation.scroll - (animation.height - height)).clamp(0., max));
    }
    animation.last_scroll.set(adjustment.value());
}
fn finish_dismissal(ui: &Rc<Ui>, animation: &Rc<Dismissal>) {
    let owns = ui
        .template_dismissal
        .borrow()
        .as_ref()
        .is_some_and(|d| Rc::ptr_eq(d, animation));
    if owns {
        compensate(ui, animation, 0.);
        ui.template_dismissal.borrow_mut().take();
        clear(ui);
        refresh_pages(ui);
    }
}
pub(super) fn keeps_dismissal(ui: &Ui, document: u64) -> bool {
    ui.template_dismissal
        .borrow()
        .as_ref()
        .is_some_and(|d| d.document == document)
}
fn clear(ui: &Ui) {
    ui.template_revealer.set_reveal_child(false);
    ui.template_revealer.set_child(None::<&gtk::Widget>);
    ui.template_slot.set_visible(false);
}
pub(super) fn dismiss(ui: &Ui) {
    let top = document_scroll_y(ui).max(0.);
    ui.template_eligible.set(false);
    ui.template_gallery.borrow_mut().take();
    ui.template_dismissal.borrow_mut().take();
    clear(ui);
    ui.scroll.vadjustment().set_value(top);
}
pub(super) fn dismiss_after_edit(ui: &Rc<Ui>) {
    if !ui.template_eligible.replace(false) {
        return;
    }
    ui.template_gallery.borrow_mut().take();
    let height = document_content_top(ui);
    if height <= 0.
        || !ui.template_slot.is_mapped()
        || !ui.template_slot.settings().is_gtk_enable_animations()
    {
        dismiss(ui);
        return;
    }
    let Some((document, _)) = document_identity(ui) else {
        dismiss(ui);
        return;
    };
    let scroll = ui.scroll.vadjustment().value();
    let clock = ui.template_revealer.frame_clock().unwrap();
    let animation = Rc::new(Dismissal {
        document,
        height,
        scroll,
        last_scroll: Cell::new(scroll),
        user_scrolled: Cell::new(false),
        clock: clock.clone(),
        revealer: ui.template_revealer.clone(),
        update: RefCell::new(None),
        done: RefCell::new(None),
    });
    ui.template_dismissal.replace(Some(animation.clone()));
    ui.template_revealer.set_sensitive(false);
    let weak = Rc::downgrade(ui);
    let running = Rc::downgrade(&animation);
    // Observe GTK's new reveal request after its update handlers, before layout.
    // Allocated height belongs to the previous frame and causes a visible rebound.
    let id = clock.connect_local("update", true, move |_| {
        if let (Some(ui), Some(animation)) = (weak.upgrade(), running.upgrade()) {
            if !ui.closed.get() {
                let height = ui
                    .template_revealer
                    .measure(gtk::Orientation::Vertical, ui.template_revealer.width())
                    .1 as f64;
                compensate(&ui, &animation, height);
            }
        }
        None
    });
    animation.update.replace(Some(id));
    let weak = Rc::downgrade(ui);
    let running = Rc::downgrade(&animation);
    let id = ui
        .template_revealer
        .connect_child_revealed_notify(move |revealer| {
            if !revealer.reveals_child() && !revealer.is_child_revealed() {
                if let (Some(ui), Some(animation)) = (weak.upgrade(), running.upgrade()) {
                    finish_dismissal(&ui, &animation);
                }
            }
        });
    animation.done.replace(Some(id));
    ui.template_revealer.set_reveal_child(false);
}
pub(super) fn offer(ui: &Rc<Ui>) {
    dismiss(ui);
    ui.template_eligible.set(true);
    reload(ui);
}
fn heading(text: &str, name: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_widget_name(name);
    label.set_xalign(0.);
    label.add_css_class("heading");
    label.set_margin_start(20);
    label.set_margin_end(20);
    label.set_margin_top(12);
    label.set_margin_bottom(8);
    label
}
fn row(title: &str, name: &str) -> (gtk::Box, gtk::Box) {
    let group = gtk::Box::new(gtk::Orientation::Vertical, 0);
    group.set_widget_name(name);
    group.set_visible(false);
    group.append(&heading(title, &format!("{name}-heading")));
    let strip = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    strip.set_margin_start(16);
    strip.set_margin_end(16);
    strip.set_margin_bottom(12);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&strip));
    if name == "start-templates" {
        scroll.set_widget_name("personal-template-gallery");
    }
    group.append(&scroll);
    (group, strip)
}
fn reload(ui: &Rc<Ui>) {
    if !ui.template_eligible.get() || ui.closed.get() {
        return;
    }
    ui.template_gallery.borrow_mut().take();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let (template_group, strip) = row(wordcraft_gnome::t!("Templates"), "start-templates");
    let (recovery_group, recovery_strip) = row(
        wordcraft_gnome::t!("Documents to Recover"),
        "start-recovery",
    );
    content.append(&template_group);
    content.append(&recovery_group);
    content.append(&heading(
        wordcraft_gnome::t!("Blank Document"),
        "start-blank-heading",
    ));
    ui.template_revealer.set_transition_duration(0);
    ui.template_revealer.set_child(Some(&content));
    ui.template_revealer.set_sensitive(true);
    ui.template_slot.set_visible(true);
    ui.template_revealer.set_reveal_child(true);
    ui.template_revealer.set_transition_duration(180);
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
        load: RefCell::new(None),
    });
    ui.template_gallery.replace(Some(gallery.clone()));
    gallery.refresh_recovery();
    let Some(folder) = wordcraft_gnome::templates::folder() else {
        return;
    };
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
                gallery.refresh_recovery();
            }
            match result {
                Ok(Ok(inventory)) => {
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
                    clear_box(&gallery.recovery_strip);
                    gallery
                        .recovery_group
                        .set_visible(!inventory.documents.is_empty());
                    for record in inventory.documents {
                        let button = gtk::Button::new();
                        button.set_widget_name("start-recovery-document");
                        button.add_css_class("flat");
                        let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
                        let icon = gtk::Image::from_icon_name("document-open-recent-symbolic");
                        icon.set_pixel_size(48);
                        content.append(&icon);
                        let name = if record.name == "Untitled draft" {
                            wordcraft_gnome::t!("Recovered Document").to_owned()
                        } else {
                            record.name.clone()
                        };
                        let title = gtk::Label::new(Some(&name));
                        title.set_max_width_chars(24);
                        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
                        content.append(&title);
                        let date = glib::DateTime::from_iso8601(&record.saved_at, None)
                            .and_then(|d| d.to_local())
                            .and_then(|d| d.format("%x %X"))
                            .map(|d| d.to_string())
                            .unwrap_or(record.saved_at);
                        let timestamp = gtk::Label::new(Some(&date));
                        timestamp.add_css_class("dim-label");
                        content.append(&timestamp);
                        button.set_child(Some(&content));
                        accessible_name(&button, &format!("{name} — {date}"));
                        let weak = Rc::downgrade(&gallery);
                        button.connect_clicked(move |_| {
                            if let Some(gallery) = weak.upgrade() {
                                gallery.recover(record.id.clone());
                            }
                        });
                        gallery.recovery_strip.append(&button);
                    }
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
