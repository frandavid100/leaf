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
    // Preferences initialize the next blank document; existing documents own theirs.
    wordcraft_gnome::preferences::set_check_spelling(on)
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
    choices: RefCell<Vec<Vec<String>>>,
    ignored: RefCell<Vec<ProofIssue>>,
    ignored_document: Cell<Option<u64>>,
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
        let document = identity.map(|(id, _)| id);
        if self.ignored_document.replace(document) != document {
            self.ignored.borrow_mut().clear();
        }
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
            let issues = proofing::issues(&session);
            let entries = issues
                .into_iter()
                .map(|issue| {
                    let choices = proofing::suggestions(&mut session, &issue);
                    (issue, choices)
                })
                .collect::<Vec<_>>();
            let _ = tx.send(entries);
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
                        let (issues, choices): (Vec<_>, Vec<_>) = issues
                            .into_iter()
                            .filter(|(i, _)| !s.ignored.borrow().contains(i))
                            .unzip();
                        s.issues.replace(issues);
                        s.choices.replace(choices);
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
        if let Some(review) = ui.review.borrow().as_ref() {
            review.schedule();
        }
    }
    pub(super) fn render_list(self: &Rc<Self>) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        if self.identity.get().is_none() {
            return;
        }
        let issues = self.issues.borrow();
        self.status.set_visible(issues.is_empty());
        self.status.set_text(wordcraft_gnome::t!(
            "No corrections found in text with available proofing tools."
        ));
        self.status.set_tooltip_text(Some(wordcraft_gnome::t!(
            "Languages without installed dictionaries are skipped."
        )));
        for (n, issue) in issues.iter().enumerate() {
            let row = gtk::ListBoxRow::new();
            row.set_widget_name(&format!("proofing-issue-{n}"));
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            line.add_css_class("card");
            let body = gtk::Box::new(gtk::Orientation::Vertical, 5);
            body.set_hexpand(true);
            body.set_margin_start(6);
            body.set_margin_end(6);
            body.set_margin_top(8);
            body.set_margin_bottom(8);
            let go = gtk::Button::new();
            go.add_css_class("flat");
            go.set_widget_name(&format!("proofing-go-{n}"));
            let heading = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            heading.append(&review_type_icon(ReviewKind::Correction));
            let text = gtk::Label::new(Some(&issue.text));
            text.set_xalign(0.);
            text.set_wrap(true);
            text.set_wrap_mode(gtk::pango::WrapMode::WordChar);
            text.set_max_width_chars(1);
            text.set_hexpand(true);
            text.add_css_class("heading");
            heading.append(&text);
            go.set_child(Some(&heading));
            accessible_name(
                &go,
                if issue.spelling {
                    wordcraft_gnome::t!("Spelling Correction")
                } else {
                    wordcraft_gnome::t!("Grammar Correction")
                },
            );
            let weak = Rc::downgrade(self);
            let target = issue.clone();
            let identity = self.identity.get();
            go.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.select_issue(&target, identity);
                }
            });
            body.append(&go);
            let message = gtk::Label::new(Some(&issue.message));
            message.set_xalign(0.);
            message.set_wrap(true);
            message.set_wrap_mode(gtk::pango::WrapMode::WordChar);
            message.set_max_width_chars(1);
            body.append(&message);
            let proposal = self
                .choices
                .borrow()
                .get(n)
                .and_then(|c| c.first())
                .cloned();
            if let Some(proposal) = proposal.as_ref() {
                let shown = if proposal.is_empty() {
                    wordcraft_gnome::t!("Remove").to_owned()
                } else {
                    proposal.clone()
                };
                let label = gtk::Label::new(Some(&wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Suggestion: {text}"),
                    &[("text", shown)],
                )));
                label.set_xalign(0.);
                label.set_wrap(true);
                label.set_max_width_chars(1);
                label.add_css_class("dim-label");
                body.append(&label);
            }
            let actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
            apply.set_widget_name(&format!("proofing-apply-{n}"));
            apply.set_hexpand(true);
            apply.set_sensitive(proposal.is_some());
            let weak = Rc::downgrade(self);
            let target = issue.clone();
            let identity = self.identity.get();
            apply.connect_clicked(move |_| {
                let Some(s) = weak
                    .upgrade()
                    .filter(|s| s.current_issue(&target, identity))
                else {
                    return;
                };
                let Some(ui) = s.ui.upgrade() else { return };
                if let Some(text) = proposal.as_deref() {
                    proof_replace(&ui, &target, text);
                }
            });
            actions.append(&apply);
            let ignore = gtk::Button::with_label(wordcraft_gnome::t!("Ignore"));
            ignore.set_widget_name(&format!("proofing-ignore-{n}"));
            ignore.set_hexpand(true);
            accessible_name(&ignore, wordcraft_gnome::t!("Ignore This Occurrence"));
            let weak = Rc::downgrade(self);
            let target = issue.clone();
            let identity = self.identity.get();
            ignore.connect_clicked(move |_| {
                let Some(s) = weak
                    .upgrade()
                    .filter(|s| s.current_issue(&target, identity))
                else {
                    return;
                };
                s.ignored.borrow_mut().push(target.clone());
                let index = s.issues.borrow().iter().position(|i| i == &target);
                if let Some(n) = index {
                    s.issues.borrow_mut().remove(n);
                    let choice_len = s.choices.borrow().len();
                    if n < choice_len {
                        s.choices.borrow_mut().remove(n);
                    }
                    s.index.set(None);
                    s.render_list();
                }
            });
            actions.append(&ignore);
            body.append(&actions);
            line.append(&body);
            let menu = gtk::MenuButton::new();
            menu.set_icon_name("view-more-symbolic");
            menu.add_css_class("flat");
            menu.set_valign(gtk::Align::Start);
            menu.set_margin_top(6);
            menu.set_widget_name(&format!("proofing-menu-{n}"));
            accessible_name(&menu, wordcraft_gnome::t!("Correction Options"));
            let weak = Rc::downgrade(self);
            let issue = issue.clone();
            let identity = self.identity.get();
            menu.set_create_popup_func(move |button| {
                let Some(s) = weak.upgrade().filter(|s| s.current_issue(&issue, identity)) else {
                    return;
                };
                let Some(ui) = s.ui.upgrade() else { return };
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
        if let Some(ui) = self.ui.upgrade() {
            if let Some(review) = ui.review.borrow().as_ref() {
                review.schedule();
            }
        }
    }
    fn current_issue(&self, issue: &ProofIssue, identity: Option<(u64, u64)>) -> bool {
        self.identity.get() == identity
            && self
                .ui
                .upgrade()
                .is_some_and(|ui| !ui.closed.get() && document_identity(&ui) == identity)
            && self.issues.borrow().contains(issue)
    }
    fn select_issue(self: &Rc<Self>, issue: &ProofIssue, identity: Option<(u64, u64)>) {
        if !self.current_issue(issue, identity) {
            return;
        }
        let index = self.issues.borrow().iter().position(|i| i == issue);
        if let Some(n) = index {
            self.select(n);
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
        if let Some(review) = ui.review.borrow().as_ref() {
            review.show(Some(ReviewKind::Correction));
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
        if let Some(review) = ui.review.borrow().as_ref() {
            review.show(Some(ReviewKind::Correction));
            return;
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
        choices: RefCell::new(Vec::new()),
        ignored: RefCell::new(Vec::new()),
        ignored_document: Cell::new(None),
        identity: Cell::new(None),
        source: RefCell::new(None),
        debounce: RefCell::new(None),
        generation: Cell::new(0),
        queued: Cell::new(false),
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
pub(super) fn show_text_language(ui: &Rc<Ui>, _parent: &ApplicationWindow) {
    if let Some(menu) = format_button(ui, "text-menu") {
        menu.popup();
    }
    if let Some(tools) = ui.text_tools.borrow().as_ref() {
        tools.page("language");
    }
}
