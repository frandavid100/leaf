use super::*;
use wordcraft_gnome::proofing::{self, Issue as ProofIssue};

pub(super) fn refresh_all_proofing(ui: &Rc<Ui>) {
    let mut docs = WORKSPACES.with(|hosts| {
        hosts
            .borrow()
            .iter()
            .flat_map(|h| {
                h.docs
                    .borrow()
                    .iter()
                    .map(|(_, u)| u.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    });
    if docs.is_empty() {
        docs.push(ui.clone());
    }
    for u in docs {
        if u.closed.get() {
            continue;
        }
        if let Some(d) = u.current.borrow_mut().as_mut() {
            d.session.relayout();
        }
        refresh_document_appearance(&u);
        if let Some(panel) = u.spelling.borrow().as_ref() {
            panel.invalidate();
        }
    }
}

pub(super) fn set_live_proofing(on: bool) -> Result<(), String> {
    wordcraft_gnome::preferences::set_check_spelling(on)?;
    let docs = WORKSPACES.with(|hosts| {
        hosts
            .borrow()
            .iter()
            .flat_map(|h| {
                h.docs
                    .borrow()
                    .iter()
                    .map(|(_, u)| u.clone())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    });
    for ui in &docs {
        if ui.closed.get() {
            continue;
        }
        if let Some(d) = ui.current.borrow_mut().as_mut() {
            d.session
                .run("review.proofing", &serde_json::json!({"value": on}))
                .map_err(|e| e.to_string())?;
        }
    }
    if let Some(ui) = docs.first() {
        refresh_all_proofing(ui);
    }
    Ok(())
}

pub(super) fn proof_replace(ui: &Rc<Ui>, issue: &ProofIssue, text: &str) {
    let result = {
        let mut v = ui.current.borrow_mut();
        let Some(d) = v.as_mut() else { return };
        proofing::replace(&mut d.session, issue, text)
    };
    match result {
        Ok(true) => after_content_change(ui),
        Ok(false) => {}
        Err(e) => ui.toasts.add_toast(Toast::new(&e)),
    }
}

/// A tab's derived issue list, with one worker at a time and a coalesced rescan.
/// Navigation is explicit: publishing scan results never moves the document cursor.
pub(super) struct SpellReview {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) toggle: gtk::ToggleButton,
    pub(super) list: gtk::ListBox,
    pub(super) status: gtk::Label,
    pub(super) index: Cell<Option<usize>>,
    pub(super) issues: RefCell<Vec<ProofIssue>>,
    pub(super) identity: Cell<Option<(u64, u64)>>,
    pub(super) source: RefCell<Option<glib::SourceId>>,
    pub(super) debounce: RefCell<Option<glib::SourceId>>,
    pub(super) generation: Cell<u64>,
    pub(super) queued: Cell<bool>,
}
impl SpellReview {
    pub(super) fn stop(&self) {
        if let Some(id) = self.source.borrow_mut().take() {
            id.remove();
        }
        if let Some(id) = self.debounce.borrow_mut().take() {
            id.remove();
        }
        self.queued.set(false);
    }
    pub(super) fn invalidate(self: &Rc<Self>) {
        self.generation.set(self.generation.get().wrapping_add(1));
        self.identity.set(None);
        self.index.set(None);
        self.issues.borrow_mut().clear();
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        self.status.set_visible(true);
        self.status.set_text(wordcraft_gnome::t!("Checking…"));
        if let Some(id) = self.debounce.borrow_mut().take() {
            id.remove();
        }
        self.queued.set(false);
        if self.toggle.is_active() {
            let weak = Rc::downgrade(self);
            let id =
                glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                    if let Some(s) = weak.upgrade() {
                        s.debounce.borrow_mut().take();
                        s.scan();
                    }
                });
            self.debounce.replace(Some(id));
        }
    }
    pub(super) fn scan(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if let Some(id) = self.debounce.borrow_mut().take() {
            id.remove();
        }
        if self.source.borrow().is_some() {
            self.queued.set(true);
            return;
        }
        self.queued.set(false);
        let Some((doc, prefs)) = ui
            .current
            .borrow()
            .as_ref()
            .map(|d| (d.session.doc.clone(), d.session.prefs.clone()))
        else {
            return;
        };
        let identity = document_identity(&ui);
        let generation = self.generation.get();
        self.identity.set(None);
        self.index.set(None);
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        self.status.set_visible(true);
        self.status.set_text(wordcraft_gnome::t!("Checking…"));
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut session = Session::new(doc);
            session.prefs = prefs;
            let _ = tx.send(proofing::issues(&session));
        });
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local(std::time::Duration::from_millis(25), move || {
            let Some(s) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let Some(ui) = s.ui.upgrade().filter(|u| !u.closed.get()) else {
                s.source.borrow_mut().take();
                return glib::ControlFlow::Break;
            };
            match rx.try_recv() {
                Ok(issues) => {
                    s.source.borrow_mut().take();
                    if document_identity(&ui) == identity && s.generation.get() == generation {
                        s.identity.set(identity);
                        s.issues.replace(issues);
                        s.render_list();
                    }
                    if s.queued.replace(false) {
                        s.scan();
                    }
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(_) => {
                    s.source.borrow_mut().take();
                    s.status
                        .set_text(wordcraft_gnome::t!("The spelling check could not finish."));
                    glib::ControlFlow::Break
                }
            }
        });
        self.source.replace(Some(id));
    }
    fn render_list(self: &Rc<Self>) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        let issues = self.issues.borrow();
        self.status.set_visible(issues.is_empty());
        self.status.set_text(wordcraft_gnome::t!(
            "No issues found in text with available proofing tools."
        ));
        self.status.set_tooltip_text(Some(wordcraft_gnome::t!(
            "Languages without installed dictionaries are skipped."
        )));
        for (n, issue) in issues.iter().enumerate() {
            let row = gtk::ListBoxRow::new();
            row.set_widget_name(&format!("proofing-issue-{n}"));
            let content = gtk::Box::new(gtk::Orientation::Vertical, 3);
            content.set_margin_start(10);
            content.set_margin_end(10);
            content.set_margin_top(8);
            content.set_margin_bottom(8);
            let text = gtk::Label::new(Some(&issue.text));
            text.set_xalign(0.0);
            text.set_wrap(true);
            text.set_wrap_mode(gtk::pango::WrapMode::WordChar);
            text.set_max_width_chars(1);
            text.add_css_class("heading");
            content.append(&text);
            let message = gtk::Label::new(Some(&issue.message));
            message.set_xalign(0.0);
            message.set_wrap(true);
            message.set_wrap_mode(gtk::pango::WrapMode::WordChar);
            message.set_max_width_chars(1);
            message.add_css_class("dim-label");
            content.append(&message);
            content.set_hexpand(true);
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            line.append(&content);
            let menu = gtk::MenuButton::new();
            menu.set_icon_name("view-more-symbolic");
            menu.add_css_class("flat");
            menu.set_widget_name(&format!("proofing-menu-{n}"));
            menu.set_valign(gtk::Align::Start);
            accessible_name(&menu, wordcraft_gnome::t!("Spelling actions"));
            let weak = Rc::downgrade(self);
            let issue = issue.clone();
            let identity = self.identity.get();
            menu.set_create_popup_func(move |button| {
                let Some(panel) = weak.upgrade() else { return };
                let Some(ui) = panel
                    .ui
                    .upgrade()
                    .filter(|u| !u.closed.get() && document_identity(u) == identity)
                else {
                    return;
                };
                let root = gio::Menu::new();
                let group = gio::SimpleActionGroup::new();
                let pop = gtk::PopoverMenu::from_model(Some(&root));
                pop.insert_action_group("context", Some(&group));
                append_proofing_actions(&root, &group, &pop, &ui, issue.clone());
                button.set_popover(Some(&pop));
            });
            line.append(&menu);
            row.set_child(Some(&line));
            self.list.append(&row);
        }
    }
    pub(super) fn item(&self) -> Option<ProofIssue> {
        self.issues.borrow().get(self.index.get()?).cloned()
    }
    pub(super) fn select(self: &Rc<Self>, index: usize) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if document_identity(&ui) != self.identity.get() {
            self.invalidate();
            return;
        }
        self.index.set(Some(index));
        let Some(issue) = self.item() else { return };
        {
            let mut v = ui.current.borrow_mut();
            let Some(d) = v.as_mut() else { return };
            if let Err(e) = proofing::select(&mut d.session, &issue) {
                ui.toasts.add_toast(Toast::new(&e));
                return;
            }
        }
        refresh_selection(&ui);
        caret_ensure_visible(&ui);
        ui.document_box.grab_focus();
    }
    pub(super) fn show(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if let Some(search) = ui.search.borrow().as_ref() {
            search.toggle.set_active(false);
        }
        let already_active = self.toggle.is_active();
        self.toggle.set_active(true);
        ui.sidebar_pages.set_visible_child_name("navigation");
        ui.navigation_pages.set_visible_child_name("proofing");
        ui.sidebar_button.set_active(true);
        if already_active {
            self.scan();
        }
    }
}

pub(super) fn show_spelling(ui: &Rc<Ui>, _parent: &ApplicationWindow) {
    if let Some(panel) = ui.spelling.borrow().as_ref() {
        panel.show();
    }
}

pub(super) fn install_spelling(ui: &Rc<Ui>, toggle: &gtk::ToggleButton) -> Rc<SpellReview> {
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    body.set_widget_name("proofing-panel");
    accessible_name(&body, wordcraft_gnome::t!("Spelling and Grammar"));
    let status = gtk::Label::new(None);
    status.set_widget_name("proofing-status");
    status.set_visible(false);
    status.set_wrap(true);
    status.set_xalign(0.0);
    status.set_max_width_chars(1);
    status.set_margin_start(10);
    status.set_margin_end(10);
    status.add_css_class("dim-label");
    body.append(&status);
    let list = gtk::ListBox::new();
    list.set_widget_name("proofing-issues");
    list.set_activate_on_single_click(true);
    list.add_css_class("navigation-sidebar");
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_widget_name("proofing-scroll");
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_min_content_height(70);
    scroll.set_child(Some(&list));
    body.append(&scroll);
    ui.navigation_pages.add_named(&body, Some("proofing"));
    let s = Rc::new(SpellReview {
        ui: Rc::downgrade(ui),
        toggle: toggle.clone(),
        list,
        status,
        index: Cell::new(None),
        issues: RefCell::new(Vec::new()),
        identity: Cell::new(None),
        source: RefCell::new(None),
        debounce: RefCell::new(None),
        generation: Cell::new(0),
        queued: Cell::new(false),
    });
    let weak = Rc::downgrade(&s);
    toggle.connect_toggled(move |b| {
        if b.is_active() {
            if let Some(s) = weak.upgrade() {
                s.show();
            }
        }
    });
    let weak = Rc::downgrade(&s);
    s.list.connect_row_activated(move |_, row| {
        if let Some(s) = weak.upgrade() {
            s.select(row.index() as usize);
        }
    });
    s
}

pub(super) const TEXT_LANGUAGES: &[(&str, &str)] = &[
    ("en-US", "English (United States)"),
    ("en-GB", "English (United Kingdom)"),
    ("es-ES", "Español (España)"),
    ("es-MX", "Español (México)"),
    ("es-AR", "Español (Argentina)"),
    ("fr-FR", "Français"),
    ("de-DE", "Deutsch"),
    ("it-IT", "Italiano"),
    ("pt-BR", "Português (Brasil)"),
    ("pl-PL", "Polski"),
    ("ru-RU", "Русский"),
    ("ca-ES", "Català"),
    ("eu-ES", "Euskara"),
    ("gl-ES", "Galego"),
];
pub(super) struct LanguageDialog {
    pub(super) window: adw::Window,
    pub(super) tag: gtk::Entry,
    pub(super) no_proof: gtk::Switch,
    pub(super) scope: adw::ComboRow,
    pub(super) apply: gtk::Button,
}
pub(super) fn show_text_language(ui: &Rc<Ui>, parent: &ApplicationWindow) -> LanguageDialog {
    let identity = document_identity(ui);
    let state = {
        let mut v = ui.current.borrow_mut();
        proofing::state(&mut v.as_mut().unwrap().session)
    };
    let window = adw::Window::builder()
        .title(wordcraft_gnome::t!("Text Language"))
        .transient_for(parent)
        .modal(true)
        .default_width(460)
        .default_height(460)
        .build();
    let toolbar = ToolbarView::new();
    let header = HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    header.pack_start(&cancel);
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
    apply.add_css_class("suggested-action");
    header.pack_end(&apply);
    toolbar.add_top_bar(&header);
    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();
    page.add(&group);
    let lang = option_combo(
        &group,
        wordcraft_gnome::t!("Language"),
        &TEXT_LANGUAGES
            .iter()
            .map(|(_, name)| *name)
            .chain(std::iter::once(wordcraft_gnome::t!("Other Language")))
            .collect::<Vec<_>>(),
    );
    let current = state["lang"].as_str().unwrap_or("en-US");
    let i = TEXT_LANGUAGES
        .iter()
        .position(|(tag, _)| tag.eq_ignore_ascii_case(current))
        .unwrap_or(TEXT_LANGUAGES.len());
    lang.set_selected(i as u32);
    let tag = gtk::Entry::new();
    tag.set_text(current);
    tag.set_placeholder_text(Some("es-ES"));
    accessible_name(&tag, wordcraft_gnome::t!("Language Tag"));
    group.add(&tag);
    let hint = gtk::Label::new(None);
    hint.set_wrap(true);
    hint.set_xalign(0.0);
    group.add(&hint);
    let no_proof = option_switch(
        &group,
        wordcraft_gnome::t!("Do Not Check Spelling or Grammar"),
    );
    no_proof.set_active(state["noProof"].as_bool().unwrap_or(false));
    let scope = option_combo(
        &group,
        wordcraft_gnome::t!("Apply To"),
        &[
            wordcraft_gnome::t!("Selection or Typing Position"),
            wordcraft_gnome::t!("Whole Document"),
        ],
    );
    let future = option_switch(&group, wordcraft_gnome::t!("Use for New Documents"));
    let message = gtk::Label::new(None);
    message.add_css_class("error");
    message.set_wrap(true);
    message.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    group.add(&message);
    {
        let tag = tag.downgrade();
        lang.connect_selected_notify(move |r| {
            if let (Some(tag), Some((value, _))) =
                (tag.upgrade(), TEXT_LANGUAGES.get(r.selected() as usize))
            {
                tag.set_text(value);
            }
        });
    }
    let update = move |tag: &gtk::Entry| {
        let value = tag.text();
        hint.set_text(if wordcraft_engine::proof::has_dictionary(Some(&value)) {
            wordcraft_gnome::t!("Spelling tools are available for this language.")
        } else {
            wordcraft_gnome::t!(
                "No dictionary is installed for this language. Its spelling cannot be checked."
            )
        });
    };
    update(&tag);
    tag.connect_changed(update);
    {
        let w = window.downgrade();
        cancel.connect_clicked(move |_| {
            if let Some(w) = w.upgrade() {
                w.close();
            }
        });
    }
    {
        let weak = Rc::downgrade(ui);
        let w = window.downgrade();
        let tag = tag.clone();
        let no_proof = no_proof.clone();
        let scope = scope.clone();
        apply.connect_clicked(move |_| {
            let (Some(ui), Some(w)) = (weak.upgrade(), w.upgrade()) else {
                return;
            };
            if ui.closed.get() || document_identity(&ui) != identity {
                message.set_text(wordcraft_gnome::t!("Checking…"));
                return;
            }
            let result = {
                let mut v = ui.current.borrow_mut();
                proofing::language(
                    &mut v.as_mut().unwrap().session,
                    &tag.text(),
                    no_proof.is_active(),
                    scope.selected() == 1,
                )
            };
            match result {
                Ok(changed) => {
                    if changed {
                        after_content_change(&ui);
                    } else {
                        refresh_selection(&ui);
                    }
                    if future.is_active() {
                        if let Err(e) =
                            wordcraft_gnome::preferences::set_default_language(&tag.text())
                        {
                            message.set_text(&e);
                            return;
                        }
                    }
                    w.close();
                    refresh_all_proofing(&ui);
                    ui.document_box.grab_focus();
                }
                Err(e) => message.set_text(&e),
            }
        });
    }
    toolbar.set_content(Some(&page));
    window.set_content(Some(&toolbar));
    window.present();
    LanguageDialog {
        window,
        tag,
        no_proof,
        scope,
        apply,
    }
}
