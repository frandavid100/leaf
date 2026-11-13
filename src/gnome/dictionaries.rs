//! Application-wide dictionary resources, independent of document language/Undo.
use super::*;
use wordcraft_gnome::dictionaries;

pub(super) struct DictionaryTools {
    pub(super) window: adw::Dialog,
    ui: std::rc::Weak<Ui>,
    folder: gtk::Label,
    forget: gtk::Button,
    status: gtk::Label,
    closed: Cell<bool>,
}
fn cancel_dictionary_chooser(ui: &Ui) {
    let previous = ui.dictionary_chooser.borrow_mut().take();
    if let Some(c) = previous {
        c.cancel();
    }
}
fn begin_chooser(ui: &Ui) -> gio::Cancellable {
    cancel_dictionary_chooser(ui);
    let c = gio::Cancellable::new();
    ui.dictionary_chooser.replace(Some(c.clone()));
    c
}
fn finish_chooser(ui: &Ui, c: &gio::Cancellable) -> bool {
    let owns = ui.dictionary_chooser.borrow().as_ref() == Some(c);
    if owns {
        ui.dictionary_chooser.borrow_mut().take();
    }
    owns && !c.is_cancelled() && !ui.closed.get()
}
pub(super) fn stop_dictionary_tools(ui: &Ui) {
    cancel_dictionary_chooser(ui);
    let tools = ui.dictionary_tools.borrow_mut().take();
    if let Some(tools) = tools {
        tools.window.close();
    }
}
fn resources_changed(ui: &Rc<Ui>) {
    refresh_all_proofing(ui);
    // Rebuild only the language choices; changing resources never changes a
    // document's language or the user's canonical selection/history.
    let text = ui.text_tools.borrow().clone();
    if let Some(tools) = text {
        tools.page("language");
    }
    let panel = ui.dictionary_tools.borrow().clone();
    if let Some(panel) = panel {
        panel.summary();
    }
}
pub(super) fn import_dictionary_files(ui: &Rc<Ui>, parent: &impl IsA<gtk::Window>) {
    let chooser = gtk::FileDialog::new();
    chooser.set_title(wordcraft_gnome::t!(
        "Select Matching Hunspell .aff and .dic Files"
    ));
    let cancellable = begin_chooser(ui);
    let weak = Rc::downgrade(ui);
    let lease = cancellable.clone();
    chooser.open_multiple(Some(parent), Some(&cancellable), move |result| {
        let Some(ui) = weak.upgrade() else { return };
        if !finish_chooser(&ui, &lease) {
            return;
        }
        let files = match result {
            Ok(files) => files,
            Err(e)
                if e.matches(gtk::DialogError::Dismissed)
                    || e.matches(gtk::DialogError::Cancelled) =>
            {
                return;
            }
            Err(e) => {
                ui.toasts.add_toast(Toast::new(&e.to_string()));
                return;
            }
        };
        let paths = (0..files.n_items().min(3))
            .filter_map(|i| files.item(i)?.downcast::<gio::File>().ok()?.path())
            .collect::<Vec<_>>();
        match proofing::install_dictionary(&paths) {
            Ok(()) => {
                resources_changed(&ui);
                ui.toasts
                    .add_toast(Toast::new(wordcraft_gnome::t!("Dictionary installed.")));
            }
            Err(e) => ui.toasts.add_toast(Toast::new(&e)),
        }
    });
}
impl DictionaryTools {
    fn summary(&self) {
        let selected = dictionaries::folder();
        self.forget.set_sensitive(selected.is_some());
        let text = if let Some(folder) = selected {
            self.folder
                .set_tooltip_text(Some(&folder.to_string_lossy()));
            let name = folder
                .file_name()
                .unwrap_or(folder.as_os_str())
                .to_string_lossy();
            if folder.read_dir().is_ok() {
                wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Dictionary folder: {folder}"),
                    &[("folder", name.into_owned())],
                )
            } else {
                wordcraft_gnome::t!(
                    "The selected folder is unavailable. Other dictionaries remain available."
                )
                .into()
            }
        } else {
            self.folder.set_tooltip_text(None);
            wordcraft_gnome::t!("No additional dictionary folder selected.").into()
        };
        self.folder.set_text(&text);
    }
    fn show_result(&self, result: Result<bool, String>) {
        if self.closed.get() {
            return;
        }
        match result {
            Ok(changed) => {
                self.status.remove_css_class("error");
                self.status
                    .set_text(wordcraft_gnome::t!("Dictionaries refreshed."));
                if changed {
                    if let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) {
                        resources_changed(&ui);
                    }
                }
            }
            Err(e) => {
                self.status.add_css_class("error");
                self.status.set_text(&e);
            }
        }
        self.summary();
    }
    fn choose_folder(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        let chooser = gtk::FileDialog::new();
        chooser.set_title(wordcraft_gnome::t!("Choose a Dictionary Folder"));
        chooser.set_accept_label(Some(wordcraft_gnome::t!("_Use This Folder")));
        if let Some(path) = dictionaries::folder() {
            chooser.set_initial_folder(Some(&gio::File::for_path(path)));
        }
        let c = begin_chooser(&ui);
        let lease = c.clone();
        let weak = Rc::downgrade(self);
        chooser.select_folder(
            self.window.root().and_downcast::<gtk::Window>().as_ref(),
            Some(&c),
            move |result| {
                let Some(panel) = weak.upgrade().filter(|p| !p.closed.get()) else {
                    return;
                };
                let Some(ui) = panel.ui.upgrade() else { return };
                if !finish_chooser(&ui, &lease) {
                    return;
                }
                let file = match result {
                    Ok(file) => file,
                    Err(e)
                        if e.matches(gtk::DialogError::Dismissed)
                            || e.matches(gtk::DialogError::Cancelled) =>
                    {
                        return;
                    }
                    Err(e) => {
                        panel.show_result(Err(e.to_string()));
                        return;
                    }
                };
                if diagnostics::flag("--dictionary84-ui-test") {
                    eprintln!("DICTIONARY84_SELECTED path={:?}", file.path());
                }
                let result = file
                    .path()
                    .ok_or_else(|| {
                        wordcraft_gnome::t!("Choose a local dictionary folder.").to_owned()
                    })
                    .and_then(|p| dictionaries::use_folder(Some(&p)))
                    .map(|_| true);
                panel.show_result(result);
            },
        );
    }
}
/// Ubuntu Language Support installs Hunspell packages visible to native Leaf.
/// A Flatpak must not offer that launcher: installing on the host does not grant
/// its sandbox access to those files. Unsupported desktops retain folder/import.
fn system_language_settings() -> Option<gio::AppInfo> {
    if std::env::var_os("FLATPAK_ID").is_some() {
        return None;
    }
    let os = std::fs::read_to_string("/etc/os-release").ok()?;
    if !os.lines().any(|l| l == "ID=ubuntu" || l == "ID=\"ubuntu\"")
        || glib::find_program_in_path("gnome-language-selector").is_none()
    {
        return None;
    }
    gio::AppInfo::all()
        .into_iter()
        .find(|a| a.id().as_deref() == Some("gnome-language-selector.desktop"))
}
pub(super) fn show_dictionary_tools(ui: &Rc<Ui>) -> Option<Rc<DictionaryTools>> {
    if ui.closed.get() {
        return None;
    }
    if let Some(panel) = ui.dictionary_tools.borrow().as_ref() {
        if let Some(host) = ui.host.upgrade() {
            panel.window.present(Some(&host.window));
        }
        return Some(panel.clone());
    }
    let window = adw::Dialog::builder()
        .title(wordcraft_gnome::t!("Dictionaries"))
        .content_width(460)
        .content_height(480)
        .build();

    let toolbar = ToolbarView::new();
    toolbar.add_top_bar(&HeaderBar::new());
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(18);
    content.set_margin_bottom(18);
    content.set_margin_start(18);
    content.set_margin_end(18);
    let description = gtk::Label::new(Some(wordcraft_gnome::t!(
        "Use readable system dictionaries, an additional folder or imported Hunspell files. The document language stays independent of the interface language."
    )));
    description.set_wrap(true);
    description.set_max_width_chars(48);
    description.set_xalign(0.);
    content.append(&description);
    let folder = gtk::Label::new(None);
    folder.set_wrap(true);
    folder.set_max_width_chars(48);
    folder.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    folder.set_xalign(0.);
    folder.set_widget_name("dictionary-folder-status");
    content.append(&folder);
    let choose = gtk::Button::with_label(wordcraft_gnome::t!("Choose Dictionary Folder…"));
    choose.set_widget_name("dictionary-folder-choose");
    choose.add_css_class("suggested-action");
    content.append(&choose);
    let forget = gtk::Button::with_label(wordcraft_gnome::t!("Stop Using This Folder"));
    forget.set_widget_name("dictionary-folder-forget");
    content.append(&forget);
    let refresh = gtk::Button::with_label(wordcraft_gnome::t!("Refresh Dictionaries"));
    refresh.set_widget_name("dictionary-refresh");
    content.append(&refresh);
    let import = gtk::Button::with_label(wordcraft_gnome::t!("Import Dictionary Files…"));
    import.set_widget_name("dictionary-import");
    content.append(&import);
    if let Some(app) = system_language_settings() {
        let button = gtk::Button::with_label(wordcraft_gnome::t!("System Language Settings…"));
        button.set_widget_name("dictionary-system-settings");
        content.append(&button);
        let weak = Rc::downgrade(ui);
        button.connect_clicked(move |_| {
            if let Err(e) = app.launch(&[], None::<&gio::AppLaunchContext>) {
                if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
                    ui.toasts.add_toast(Toast::new(&e.to_string()));
                }
            }
        });
    }
    let status = gtk::Label::new(None);
    status.set_wrap(true);
    status.set_max_width_chars(48);
    status.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    status.set_xalign(0.);
    status.set_widget_name("dictionary-result");
    content.append(&status);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_min_content_height(280);
    scroll.set_max_content_height(500);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&content));
    toolbar.set_content(Some(&scroll));
    window.set_child(Some(&toolbar));
    let panel = Rc::new(DictionaryTools {
        window,
        ui: Rc::downgrade(ui),
        folder,
        forget: forget.clone(),
        status,
        closed: Cell::new(false),
    });
    let weak = Rc::downgrade(&panel);
    choose.connect_clicked(move |_| {
        if let Some(p) = weak.upgrade().filter(|p| !p.closed.get()) {
            p.choose_folder();
        }
    });
    let weak = Rc::downgrade(&panel);
    forget.connect_clicked(move |_| {
        if let Some(p) = weak.upgrade().filter(|p| !p.closed.get()) {
            p.show_result(dictionaries::use_folder(None).map(|_| true));
        }
    });
    let weak = Rc::downgrade(&panel);
    refresh.connect_clicked(move |_| {
        if let Some(p) = weak.upgrade().filter(|p| !p.closed.get()) {
            p.show_result(dictionaries::refresh(true));
        }
    });
    let weak = Rc::downgrade(&panel);
    import.connect_clicked(move |_| {
        if let Some(p) = weak.upgrade().filter(|p| !p.closed.get()) {
            if let Some(ui) = p.ui.upgrade().filter(|u| !u.closed.get()) {
                if let Some(host) = ui.host.upgrade() {
                    import_dictionary_files(&ui, &host.window);
                }
            }
        }
    });
    // Reload explicitly or when opening the language list. Rebuilding language
    // controls from a window focus notification can re-enter GTK focus handling.
    let weak = Rc::downgrade(&panel);
    panel.window.connect_closed(move |_| {
        if let Some(p) = weak.upgrade().filter(|p| !p.closed.get()) {
            p.closed.set(true);
            if let Some(ui) = p.ui.upgrade() {
                cancel_dictionary_chooser(&ui);
                ui.dictionary_tools.borrow_mut().take();
            }
        }
    });
    panel.summary();
    ui.dictionary_tools.replace(Some(panel.clone()));
    if let Some(host) = ui.host.upgrade() {
        panel.window.present(Some(&host.window));
    }
    Some(panel)
}
