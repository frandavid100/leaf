use super::*;

// Per-tab sidebar search; weak callbacks never keep a closed document alive.
pub(super) struct SearchPanel {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) toggle: gtk::ToggleButton,
    pub(super) query: gtk::SearchEntry,
    pub(super) replacement: gtk::Entry,
    pub(super) replacement_row: gtk::Box,
    pub(super) replace_all: gtk::Button,
    pub(super) _actions_width: gtk::SizeGroup,
    pub(super) match_case: gtk::CheckButton,
    pub(super) replace_mode: gtk::CheckButton,
    pub(super) whole_word: gtk::CheckButton,
    pub(super) regex: gtk::CheckButton,
    pub(super) preserve_case: gtk::CheckButton,
    pub(super) scope: gtk::DropDown,
    range: RefCell<Option<wordcraft_gnome::search::Range>>,
    pub(super) list: gtk::ListBox,
    pub(super) status: gtk::Label,
    pub(super) hits: RefCell<Vec<wordcraft_gnome::search::Hit>>,
    pub(super) pending: Cell<bool>,
}
impl SearchPanel {
    fn options(&self) -> wordcraft_gnome::search::Options {
        use wordcraft_gnome::search::{Options, Scope};
        Options {
            match_case: self.match_case.is_active(),
            whole_word: self.whole_word.is_active(),
            special: false,
            regex: self.regex.is_active(),
            preserve_case: self.preserve_case.is_active(),
            scope: match self.scope.selected() {
                1 => Scope::Body,
                2 => Scope::Notes,
                3 => Scope::HeadersFooters,
                4 => Scope::Selection,
                _ => Scope::Document,
            },
        }
    }
    fn capture_range(&self) {
        if let Some(ui) = self.ui.upgrade() {
            self.range.replace(
                ui.current
                    .borrow()
                    .as_ref()
                    .and_then(|d| wordcraft_gnome::search::Range::capture(&d.session)),
            );
        }
    }
    pub(super) fn show(&self, replace: bool) {
        self.capture_range();
        if replace {
            self.replace_mode.set_active(true);
        }
        self.toggle.set_active(true);
        if let Some(ui) = self.ui.upgrade() {
            ui.sidebar.set_show_sidebar(true);
            ui.sidebar_pages.set_visible_child_name("search");
        }
        self.query.grab_focus();
        self.query.select_region(0, -1);
    }
    pub(super) fn schedule(self: &Rc<Self>) {
        if !self.toggle.is_active() {
            return;
        }
        if self.pending.replace(true) {
            return;
        }
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(160), move || {
            if let Some(s) = weak.upgrade() {
                s.pending.set(false);
                s.refresh();
            }
        });
    }
    pub(super) fn refresh(self: &Rc<Self>) {
        let _perf = perf_scope("leaf.search_refresh");
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        // Retain the query, but rebuild its results only when they are used.
        // Toggling search back on always refreshes against the current revision.
        if !self.toggle.is_active() {
            return;
        }
        let result = {
            let mut state = ui.current.borrow_mut();
            state.as_mut().map(|d| {
                let _p = perf_scope("boundary.search_query");
                wordcraft_gnome::search::query_with(
                    &mut d.session,
                    &self.query.text(),
                    &self.options(),
                    self.range.borrow().as_ref(),
                )
            })
        };
        let (hits, error) = match result {
            Some(Ok(h)) => (h, None),
            Some(Err(e)) => (Vec::new(), Some(e)),
            None => (Vec::new(), None),
        };
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        ui.search_highlights.borrow_mut().clear();
        if self.toggle.is_active() {
            *ui.search_highlights.borrow_mut() =
                hits.iter().flat_map(|h| h.rects.clone()).collect();
        }
        self.status.set_visible(hits.is_empty());
        self.status
            .set_text(error.as_deref().unwrap_or(if self.query.text().is_empty() {
                wordcraft_gnome::t!("Type to Search")
            } else {
                wordcraft_gnome::t!("No Matches")
            }));
        self.replace_all.set_sensitive(!hits.is_empty());
        for (index, hit) in hits.iter().enumerate() {
            perf_count("search_rows", 1);
            let row = gtk::ListBoxRow::new();
            row.set_widget_name(&format!("search-hit-{index}"));
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            content.set_margin_top(1);
            content.set_margin_bottom(1);
            content.set_margin_start(8);
            content.set_margin_end(6);
            let snippet = gtk::Label::new(None);
            snippet.set_xalign(0.0);
            snippet.set_hexpand(true);
            snippet.set_ellipsize(gtk::pango::EllipsizeMode::End);
            snippet.set_max_width_chars(1);
            let escaped = |s: &str| glib::markup_escape_text(&s.replace('\n', " ")).to_string();
            let before = hit
                .before
                .chars()
                .rev()
                .take(8)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>();
            let before = if before.len() < hit.before.len() {
                format!("…{before}")
            } else {
                before
            };
            snippet.set_markup(&format!(
                "{}<b>{}</b>{}",
                escaped(&before),
                escaped(&hit.matched),
                escaped(&hit.after)
            ));
            snippet.set_tooltip_text(Some(&format!("{}{}{}", hit.before, hit.matched, hit.after)));
            content.append(&snippet);
            let page = gtk::Label::new(Some(
                &hit.page
                    .map(|p| (p + 1).to_string())
                    .unwrap_or_else(|| "—".into()),
            ));
            page.add_css_class("dim-label");
            page.set_tooltip_text(Some(wordcraft_gnome::t!("Page")));
            content.append(&page);
            if self.replace_mode.is_active() {
                let b = gtk::Button::new();
                b.set_child(Some(&paragraph_icon("replace")));
                b.add_css_class("flat");
                b.add_css_class("search-replace");
                b.set_tooltip_text(Some(wordcraft_gnome::t!("Replace This Match")));
                b.set_widget_name(&format!("search-replace-{index}"));
                accessible_name(&b, wordcraft_gnome::t!("Replace This Match"));
                content.append(&b);
                let weak = Rc::downgrade(self);
                b.connect_clicked(move |_| {
                    if let Some(s) = weak.upgrade() {
                        s.replace_hit(index);
                    }
                });
            }
            row.set_child(Some(&content));
            self.list.append(&row);
        }
        *self.hits.borrow_mut() = hits;
        refresh_selection(&ui);
    }
    pub(super) fn activate(&self, index: usize) {
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let Some(hit) = self.hits.borrow().get(index).cloned() else {
            return;
        };
        let result = {
            let mut state = ui.current.borrow_mut();
            state.as_mut().map(|d| {
                let r = wordcraft_gnome::search::select(&mut d.session, &hit);
                if r.is_ok() {
                    d.session.page_hint = hit.page.unwrap_or(0);
                }
                r
            })
        };
        if let Some(Err(e)) = result {
            ui.toasts.add_toast(Toast::new(&e));
            return;
        }
        refresh_selection(&ui);
        caret_ensure_visible(&ui);
        if let Some(row) = self.list.row_at_index(index as i32) {
            self.list.select_row(Some(&row));
        }
    }
    pub(super) fn replace_all(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let result = {
            let mut value = ui.current.borrow_mut();
            value.as_mut().map(|d| {
                let mut range = self.range.borrow_mut();
                wordcraft_gnome::search::replace_with(
                    &mut d.session,
                    &self.hits.borrow(),
                    &self.query.text(),
                    &self.replacement.text(),
                    &self.options(),
                    if self.scope.selected() == 4 {
                        range.as_mut()
                    } else {
                        None
                    },
                )
            })
        };
        match result {
            Some(Ok(n)) if n > 0 => after_content_change(&ui),
            Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
            _ => {}
        }
        self.refresh();
        ui.document_box.grab_focus();
    }
    pub(super) fn replace_hit(self: &Rc<Self>, index: usize) {
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let Some(hit) = self.hits.borrow().get(index).cloned() else {
            return;
        };
        let result = {
            let mut state = ui.current.borrow_mut();
            state.as_mut().map(|d| {
                let mut range = self.range.borrow_mut();
                wordcraft_gnome::search::replace_with(
                    &mut d.session,
                    std::slice::from_ref(&hit),
                    &self.query.text(),
                    &self.replacement.text(),
                    &self.options(),
                    if self.scope.selected() == 4 {
                        range.as_mut()
                    } else {
                        None
                    },
                )
            })
        };
        match result {
            Some(Ok(_)) => after_content_change(&ui),
            Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
            None => {}
        }
        self.refresh();
        let count = self.hits.borrow().len();
        if count > 0 {
            self.activate(index.min(count - 1));
        }
        ui.document_box.grab_focus();
    }
}
pub(super) fn install_search_panel(
    ui: &Rc<Ui>,
    toggle: &gtk::ToggleButton,
    compact: &gtk::Button,
) -> Rc<SearchPanel> {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_margin_top(6);
    let query = gtk::SearchEntry::new();
    query.set_placeholder_text(Some(wordcraft_gnome::t!("Search Document")));
    query.set_hexpand(true);
    query.set_width_chars(1);
    query.set_widget_name("document-search");
    let pasted_help = wordcraft_gnome::t!(
        "Paste tabs or line endings directly. Plain-text line endings find both soft breaks and paragraph boundaries."
    );
    query.set_tooltip_text(Some(pasted_help));
    query.update_property(&[gtk::accessible::Property::Description(pasted_help)]);
    accessible_name(&query, wordcraft_gnome::t!("Search Document"));
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    line.set_margin_start(6);
    line.set_margin_end(6);
    let field = gtk::Overlay::new();
    field.set_hexpand(true);
    query.add_css_class("search-with-options");
    field.set_child(Some(&query));
    line.append(&field);
    let options = gtk::MenuButton::new();
    options.set_icon_name("nautilus-search-filters-symbolic");
    options.set_widget_name("search-options");
    accessible_name(&options, wordcraft_gnome::t!("Search Options"));
    options.add_css_class("flat");
    options.add_css_class("search-options");
    options.set_tooltip_text(Some(wordcraft_gnome::t!("Search Options")));
    options.set_halign(gtk::Align::End);
    options.set_valign(gtk::Align::Center);
    options.set_margin_end(3);
    field.add_overlay(&options);
    root.append(&line);
    let pop = gtk::Popover::new();
    let choices = gtk::Box::new(gtk::Orientation::Vertical, 8);
    choices.set_widget_name("search-option-choices");
    choices.set_margin_top(8);
    choices.set_margin_bottom(8);
    choices.set_margin_start(8);
    choices.set_margin_end(8);
    let case = gtk::CheckButton::with_label(wordcraft_gnome::t!("Case Sensitive"));
    let replace = gtk::CheckButton::with_label(wordcraft_gnome::t!("Replace"));
    choices.append(&replace);
    choices.append(&case);
    let whole_word = gtk::CheckButton::with_label(wordcraft_gnome::t!("Whole Words"));
    whole_word.set_widget_name("search-whole-words");
    choices.append(&whole_word);
    let scope = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("Entire Document"),
        wordcraft_gnome::t!("Body Text"),
        wordcraft_gnome::t!("Notes"),
        wordcraft_gnome::t!("Headers and Footers"),
        wordcraft_gnome::t!("Selected Text"),
    ]);
    scope.set_widget_name("search-scope");
    accessible_name(&scope, wordcraft_gnome::t!("Search In"));
    let regex = gtk::CheckButton::with_label(wordcraft_gnome::t!("Regular Expressions"));
    regex.set_widget_name("search-regex");
    let preserve_case =
        gtk::CheckButton::with_label(wordcraft_gnome::t!("Preserve Capitalization"));
    preserve_case.set_widget_name("search-preserve-case");
    preserve_case.set_sensitive(false);
    let regex_help = wordcraft_gnome::t!(
        "Example: ([0-9]+), replacement: $1. Patterns must select text; empty matches cannot be replaced."
    );
    regex.set_tooltip_text(Some(regex_help));
    regex.update_property(&[gtk::accessible::Property::Description(regex_help)]);
    let scope_help = wordcraft_gnome::t!(
        "Selected Text uses the document selection. Edits outside this search invalidate that range."
    );
    scope.set_tooltip_text(Some(scope_help));
    scope.update_property(&[gtk::accessible::Property::Description(scope_help)]);
    choices.append(&preserve_case);
    choices.append(&regex);
    choices.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    choices.append(&scope);
    pop.set_child(Some(&choices));
    options.set_popover(Some(&pop));
    let replacement = gtk::Entry::new();
    replacement.set_placeholder_text(Some(wordcraft_gnome::t!("Replace With")));
    replacement.set_hexpand(true);
    replacement.set_width_chars(1);
    let replacement_help = wordcraft_gnome::t!(
        "Pasted line endings create paragraphs. Text such as ^p and $1 stays literal unless Regular Expressions is enabled."
    );
    replacement.set_tooltip_text(Some(replacement_help));
    replacement.update_property(&[gtk::accessible::Property::Description(replacement_help)]);
    accessible_name(&replacement, wordcraft_gnome::t!("Replace With"));
    let replacement_row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    replacement_row.set_margin_start(6);
    replacement_row.set_margin_end(6);
    replacement_row.append(&replacement);
    let replace_all = gtk::Button::new();
    accessible_name(&replace_all, wordcraft_gnome::t!("Replace All Matches"));
    replace_all.set_child(Some(&paragraph_icon("replace")));
    replace_all.add_css_class("flat");
    replace_all.set_tooltip_text(Some(wordcraft_gnome::t!("Replace All Matches")));
    replace_all.set_widget_name("search-replace-all");
    replacement_row.append(&replace_all);
    replacement_row.set_visible(false);
    root.append(&replacement_row);
    let actions_width = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
    actions_width.add_widget(&options);
    actions_width.add_widget(&replace_all);
    let status = gtk::Label::new(Some(wordcraft_gnome::t!("Type to Search")));
    status.add_css_class("dim-label");
    status.set_wrap(true);
    root.append(&status);
    let list = gtk::ListBox::new();
    list.set_activate_on_single_click(true);
    list.add_css_class("navigation-sidebar");
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&list));
    root.append(&scroll);
    ui.sidebar_pages.add_named(&root, Some("search"));
    let s = Rc::new(SearchPanel {
        ui: Rc::downgrade(ui),
        toggle: toggle.clone(),
        query: query.clone(),
        replacement: replacement.clone(),
        replacement_row,
        replace_all: replace_all.clone(),
        _actions_width: actions_width,
        match_case: case.clone(),
        replace_mode: replace.clone(),
        whole_word: whole_word.clone(),
        regex: regex.clone(),
        preserve_case: preserve_case.clone(),
        scope: scope.clone(),
        range: RefCell::new(None),
        list: list.clone(),
        status,
        hits: RefCell::new(Vec::new()),
        pending: Cell::new(false),
    });
    for button in [&whole_word, &regex, &preserve_case] {
        let weak = Rc::downgrade(&s);
        button.connect_toggled(move |_| {
            if let Some(s) = weak.upgrade() {
                s.refresh();
            }
        });
    }
    let weak = Rc::downgrade(&s);
    scope.connect_selected_notify(move |scope| {
        if let Some(s) = weak.upgrade() {
            if scope.selected() == 4 {
                s.capture_range();
            }
            s.refresh();
        }
    });
    let weak = Rc::downgrade(&s);
    replace_all.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            s.replace_all();
        }
    });
    let weak = Rc::downgrade(&s);
    query.connect_search_changed(move |_| {
        if let Some(s) = weak.upgrade() {
            s.refresh();
        }
    });
    let weak = Rc::downgrade(&s);
    case.connect_toggled(move |_| {
        if let Some(s) = weak.upgrade() {
            s.refresh();
        }
    });
    let weak = Rc::downgrade(&s);
    replace.connect_toggled(move |b| {
        if let Some(s) = weak.upgrade() {
            s.replacement_row.set_visible(b.is_active());
            s.preserve_case.set_sensitive(b.is_active());
            s.refresh();
        }
    });
    let weak = Rc::downgrade(&s);
    toggle.connect_toggled(move |b| {
        if let Some(s) = weak.upgrade() {
            if let Some(ui) = s.ui.upgrade() {
                ui.sidebar_pages.set_visible_child_name(if b.is_active() {
                    "search"
                } else {
                    "navigation"
                });
                if b.is_active() {
                    s.capture_range();
                    ui.sidebar.set_show_sidebar(true);
                    s.query.grab_focus();
                    s.refresh();
                } else {
                    ui.search_highlights.borrow_mut().clear();
                    refresh_selection(&ui);
                    refresh_thumbnails(&ui);
                }
            }
        }
    });
    let weak = Rc::downgrade(&s);
    compact.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            s.show(false);
        }
    });
    let weak = Rc::downgrade(&s);
    list.connect_row_activated(move |_, r| {
        if let Some(s) = weak.upgrade() {
            s.activate(r.index() as usize);
        }
    });
    let weak = Rc::downgrade(&s);
    query.connect_activate(move |_| {
        if let Some(s) = weak.upgrade() {
            let next = s
                .list
                .selected_row()
                .map(|r| r.index() as usize + 1)
                .unwrap_or(0);
            let n = s.hits.borrow().len();
            if n > 0 {
                s.activate(next % n);
            }
        }
    });
    let weak = Rc::downgrade(&s);
    query.connect_stop_search(move |_| {
        if let Some(s) = weak.upgrade() {
            s.toggle.set_active(false);
            if let Some(ui) = s.ui.upgrade() {
                ui.document_box.grab_focus();
            }
        }
    });
    s
}
pub(super) fn show_search(ui: &Rc<Ui>, _parent: &ApplicationWindow, replace: bool) {
    if let Some(s) = ui.search.borrow().as_ref() {
        s.show(replace);
    }
}
