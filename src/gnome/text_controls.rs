use super::*;

pub(super) struct FormatSequence {
    document: u64,
    revision: u64,
    selection: String,
    command: String,
    time: std::time::Instant,
}

/// Native composite controls can own focus through their editable delegate,
/// including popover surfaces with different GTK versions.
pub(super) fn control_has_focus(widget: &impl IsA<gtk::Widget>) -> bool {
    let widget = widget.as_ref();
    if widget.has_focus() || widget.is_focus() {
        return true;
    }
    let mut child = widget.first_child();
    while let Some(w) = child {
        if control_has_focus(&w) {
            return true;
        }
        child = w.next_sibling();
    }
    false
}

/// Apply a validated control without closing/rebuilding its popover or moving focus.
pub(super) fn immediate_format(
    ui: &Rc<Ui>,
    command: &str,
    args: serde_json::Value,
) -> Result<(), String> {
    let focus = if control_has_focus(&ui.font_input) {
        Some(ui.font_input.clone().upcast::<gtk::Widget>())
    } else if control_has_focus(&ui.size_input) {
        Some(ui.size_input.clone().upcast::<gtk::Widget>())
    } else {
        ui.host
            .upgrade()
            .and_then(|h| gtk::prelude::GtkWindowExt::focus(&h.window))
    };
    let edit = focus
        .as_ref()
        .and_then(|w| w.clone().dynamic_cast::<gtk::Editable>().ok())
        .map(|e| {
            (
                e.downgrade(),
                e.text().to_string(),
                e.position(),
                e.selection_bounds(),
            )
        });
    let focus = focus.map(|w| w.downgrade());
    let scroll = (
        ui.scroll.hadjustment().value(),
        ui.scroll.vadjustment().value(),
    );
    let changed = {
        let mut current = ui.current.borrow_mut();
        let d = current.as_mut().ok_or(wordcraft_gnome::t!("No document"))?;
        if command == "leaf.highlight" {
            wordcraft_gnome::highlight::register(&mut d.session);
        }
        if wordcraft_gnome::text_properties::formatting_matches(&d.session, command, &args) {
            return Ok(());
        }
        let before = d.session.doc.clone();
        let control = format!(
            "{}:{:?}",
            command,
            args.get("props")
                .unwrap_or(&args)
                .as_object()
                .map(|m| m.keys().cloned().collect::<Vec<_>>())
        );
        let selection = format!(
            "{:?}|{:?}|{:?}",
            d.session.sel,
            d.session.cell_range(),
            d.session.column_segments()
        );
        let id = d.session.document_id();
        if ui.format_sequence.borrow().as_ref().is_some_and(|last| {
            last.document == id
                && last.revision == d.session.rev()
                && last.selection == selection
                && last.command == control
                && last.time.elapsed() < std::time::Duration::from_millis(900)
        }) {
            d.session.join_next_undo();
        }
        d.session.run(command, &args).map_err(|e| e.to_string())?;
        ui.format_sequence.replace(Some(FormatSequence {
            document: id,
            revision: d.session.rev(),
            selection,
            command: control,
            time: std::time::Instant::now(),
        }));
        d.session.doc != before
    };
    if let Some(t) = ui.text_tools.borrow().as_ref() {
        t.remember_property(command, &args);
    }
    if let Some(p) = ui.paragraph.borrow().as_ref() {
        p.remember_property(command, &args, ui);
    }
    let identity = document_identity(ui);
    if changed {
        after_content_change(ui);
    } else {
        remember_repeat(ui);
        refresh_selection(ui);
    }
    let weak = Rc::downgrade(ui);
    glib::idle_add_local_once(move || {
        if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
            if document_identity(&ui) != identity {
                return;
            }
            ui.scroll.hadjustment().set_value(scroll.0);
            ui.scroll.vadjustment().set_value(scroll.1);
            if ui
                .host
                .upgrade()
                .and_then(|h| selected_ui(&h))
                .is_some_and(|u| Rc::ptr_eq(&ui, &u))
            {
                if let Some(w) = focus.and_then(|w| w.upgrade()).filter(|w| w.is_mapped()) {
                    if !control_has_focus(&w) {
                        w.grab_focus();
                        // Re-focusing a native Entry/SpinButton can select all
                        // text. Keep an in-progress edit's cursor/selection.
                        if let Some((editable, text, position, selection)) = edit {
                            if let Some(e) = editable.upgrade().filter(|e| e.text() == text) {
                                if let Some((a, b)) = selection {
                                    e.select_region(a, b);
                                } else {
                                    e.set_position(position);
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

pub(super) fn numeric_value(n: &gtk::SpinButton) -> Option<f64> {
    let text = n.text().trim().replace(',', ".");
    (!text.ends_with('.'))
        .then(|| text.parse::<f64>().ok())
        .flatten()
        .filter(|v| v.is_finite() && *v >= n.adjustment().lower() && *v <= n.adjustment().upper())
}
pub(super) fn validate_numeric_input(n: &gtk::SpinButton) {
    // Our parser accepts both decimal separators and intermediate input. The
    // native numeric filter can discard punctuation before that parser sees it.
    n.set_numeric(false);
    n.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
    n.connect_input(|n| Some(numeric_value(n).ok_or(())));
}

pub(super) fn bind_numeric(
    ui: &Rc<Ui>,
    n: &gtk::SpinButton,
    action: impl Fn(f64) -> Result<(), String> + 'static,
) {
    validate_numeric_input(n);
    let last = Rc::new(Cell::new(
        n.text().trim().replace(',', ".").parse::<f64>().ok(),
    ));
    let weak = Rc::downgrade(ui);
    let apply: Rc<dyn Fn(&gtk::SpinButton)> = Rc::new(move |n| {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if ui.format_syncing.get() {
            return;
        }
        let text = n.text().trim().replace(',', ".");
        let Some(value) = (!text.ends_with('.'))
            .then(|| text.parse::<f64>().ok())
            .flatten()
            .filter(|v| {
                v.is_finite() && *v >= n.adjustment().lower() && *v <= n.adjustment().upper()
            })
        else {
            n.add_css_class("error");
            return;
        };
        n.remove_css_class("error");
        if last.get().is_some_and(|old| (old - value).abs() < 0.0001) {
            return;
        }
        match action(value) {
            Ok(()) => last.set(Some(value)),
            Err(e) => {
                n.add_css_class("error");
                ui.toasts.add_toast(Toast::new(&e));
            }
        }
    });
    let a = apply.clone();
    n.connect_value_changed(move |n| a(n));
    let weak = Rc::downgrade(ui);
    let target = n.downgrade();
    let generation = Rc::new(Cell::new(0u64));
    n.connect_changed(move |_| {
        let Some(ui) = weak.upgrade() else { return };
        if ui.format_syncing.get() {
            return;
        }
        let next = generation.get().wrapping_add(1);
        generation.set(next);
        let generation = generation.clone();
        let target = target.clone();
        let action = apply.clone();
        let weak = Rc::downgrade(&ui);
        let identity = document_identity(&ui);
        let selection = ui.current.borrow().as_ref().map(|d| d.session.sel.clone());
        glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
            if generation.get() != next {
                return;
            }
            let (Some(ui), Some(n)) = (weak.upgrade(), target.upgrade()) else {
                return;
            };
            if ui.closed.get()
                || document_identity(&ui) != identity
                || ui.current.borrow().as_ref().map(|d| d.session.sel.clone()) != selection
            {
                return;
            }
            action(&n);
        });
    });
}

pub(super) struct TextControls {
    ui: std::rc::Weak<Ui>,
    font: gtk::Entry,
    size: gtk::SpinButton,
    font_menu: gtk::MenuButton,
    size_menu: gtk::MenuButton,
    pub(super) font_choices: gtk::Box,
    size_choices: gtk::Box,
    font_rows: RefCell<Vec<(String, gtk::Button)>>,
    font_document_rows: gtk::Box,
    font_all_rows: gtk::Box,
    font_document_key: RefCell<Vec<String>>,
    font_source: RefCell<Option<glib::SourceId>>,
    size_source: RefCell<Option<glib::SourceId>>,
}
impl TextControls {
    pub(super) fn stop(&self) {
        for source in [&self.font_source, &self.size_source] {
            if let Some(id) = source.borrow_mut().take() {
                id.remove();
            }
        }
    }
    fn family_names(&self) -> Vec<String> {
        let Some(ui) = self.ui.upgrade() else {
            return vec![];
        };
        let mut names: Vec<_> = ui
            .document_box
            .pango_context()
            .list_families()
            .iter()
            .map(|f| f.name().to_string())
            .collect();
        names.sort_by_key(|s| s.to_lowercase());
        names
    }
    fn apply_font(&self, value: &str) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        let value = value.trim();
        let current = selection_character(&ui);
        if current["font"].as_str() == Some(value) {
            return;
        }
        let valid = self
            .family_names()
            .iter()
            .any(|f| f.eq_ignore_ascii_case(value))
            || ["sans", "serif", "monospace"].contains(&value.to_ascii_lowercase().as_str());
        if !valid {
            let query = value.to_lowercase();
            if query.is_empty()
                || self
                    .family_names()
                    .iter()
                    .any(|f| f.to_lowercase().contains(&query))
            {
                self.font.remove_css_class("error");
            } else {
                self.font.add_css_class("error");
            }
            return;
        }
        self.font.remove_css_class("error");
        if let Err(e) = immediate_format(&ui, "format.font", serde_json::json!({"name":value})) {
            ui.toasts.add_toast(Toast::new(&e));
        }
    }
    fn apply_size(&self) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        let value = self.size.text().trim().replace(',', ".");
        let Some(value) = (!value.ends_with('.'))
            .then(|| value.parse::<f64>().ok())
            .flatten()
            .filter(|n| n.is_finite() && (1.0..=1638.0).contains(n))
        else {
            self.size.add_css_class("error");
            return;
        };
        self.size.remove_css_class("error");
        if let Err(e) = immediate_format(
            &ui,
            "format.set",
            serde_json::json!({"props":{"size":value}}),
        ) {
            ui.toasts.add_toast(Toast::new(&e));
        }
    }
    fn schedule(self: &Rc<Self>, font: bool) {
        let Some(ui) = self.ui.upgrade() else { return };
        if ui.format_syncing.get() {
            return;
        }
        let source = if font {
            &self.font_source
        } else {
            &self.size_source
        };
        if let Some(id) = source.borrow_mut().take() {
            id.remove();
        }
        let identity = document_identity(&ui);
        let selection = ui.current.borrow().as_ref().map(|d| d.session.sel.clone());
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
            let Some(s) = weak.upgrade() else { return };
            (if font { &s.font_source } else { &s.size_source })
                .borrow_mut()
                .take();
            let Some(ui) = s.ui.upgrade().filter(|u| !u.closed.get()) else {
                return;
            };
            if document_identity(&ui) != identity
                || ui.current.borrow().as_ref().map(|d| d.session.sel.clone()) != selection
            {
                return;
            }
            if font {
                s.apply_font(&s.font.text());
            } else {
                s.apply_size();
            }
        });
        source.replace(Some(id));
    }
    fn font_row(self: &Rc<Self>, family: &str, installed: bool) -> gtk::Button {
        let button = gtk::Button::new();
        button.add_css_class("flat");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let preview = gtk::Label::new(Some(family));
        preview.set_widget_name("font-preview");
        preview.set_xalign(0.);
        preview.set_hexpand(true);
        preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
        preview.set_width_chars(1);
        preview.set_max_width_chars(28);
        preview.set_tooltip_text(Some(family));
        row.append(&preview);
        if let Some(ui) = self.ui.upgrade().filter(|_| installed) {
            let context = ui.document_box.pango_context();
            let mut desc = gtk::pango::FontDescription::new();
            desc.set_family(family);
            if let Some(font) = context.load_font(&desc).filter(|f| {
                f.describe()
                    .family()
                    .is_some_and(|name| name.eq_ignore_ascii_case(family))
            }) {
                let attr = gtk::pango::AttrList::new();
                attr.insert(gtk::pango::AttrString::new_family(family));
                attr.insert(gtk::pango::AttrInt::new_fallback(false));
                if family
                    .chars()
                    .all(|c| c.is_whitespace() || font.has_char(c))
                {
                    preview.set_attributes(Some(&attr));
                }
                if !"Aa Bb Cc"
                    .chars()
                    .all(|c| c.is_whitespace() || font.has_char(c))
                {
                    if let Some(text) = [
                        "العربية",
                        "עברית",
                        "हिन्दी",
                        "汉字",
                        "日本語",
                        "한글",
                        "αβγ",
                        "абв",
                        "★♫",
                    ]
                    .into_iter()
                    .find(|t| t.chars().all(|c| font.has_char(c)))
                    {
                        let sample = gtk::Label::new(Some(text));
                        sample.set_attributes(Some(&attr));
                        row.append(&sample);
                    }
                }
            }
        }
        let mark = gtk::Image::from_icon_name("object-select-symbolic");
        mark.set_widget_name("font-current-marker");
        mark.set_visible(false);
        row.append(&mark);
        button.set_child(Some(&row));
        accessible_name(&button, family);
        button.set_tooltip_text(Some(if installed {
            family
        } else {
            wordcraft_gnome::t!("This font is not installed. Its document identity is preserved.")
        }));
        button.set_sensitive(installed);
        let (weak, name) = (Rc::downgrade(self), family.to_owned());
        button.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                if let Some(ui) = s.ui.upgrade() {
                    let old = ui.format_syncing.replace(true);
                    s.font.set_text(&name);
                    ui.format_syncing.set(old);
                }
                s.apply_font(&name);
                if let Some(ui) = s.ui.upgrade().filter(|ui| !ui.format_group.pinned.get()) {
                    if let Some(p) = ui
                        .text_tools
                        .borrow()
                        .as_ref()
                        .and_then(|t| t.popover.upgrade())
                    {
                        close_format_popover(&p);
                    }
                }
            }
        });
        self.font_rows
            .borrow_mut()
            .push((family.into(), button.clone()));
        button
    }
    pub(super) fn show_fonts(self: &Rc<Self>) {
        if self.font_all_rows.parent().is_none() {
            clear_box(&self.font_choices);
            let title = gtk::Label::new(Some(wordcraft_gnome::t!("Fonts in This Document")));
            title.set_xalign(0.);
            title.add_css_class("heading");
            self.font_choices.append(&title);
            self.font_choices.append(&self.font_document_rows);
            self.font_choices
                .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            let title = gtk::Label::new(Some(wordcraft_gnome::t!("All Fonts")));
            title.set_xalign(0.);
            title.add_css_class("heading");
            self.font_choices.append(&title);
            self.font_choices.append(&self.font_all_rows);
            for family in self.family_names() {
                self.font_all_rows.append(&self.font_row(&family, true));
            }
            let empty = gtk::Label::new(Some(wordcraft_gnome::t!("No Results")));
            empty.set_widget_name("font-no-results");
            empty.set_visible(false);
            self.font_choices.append(&empty);
        }
        self.sync_document_fonts();
        let current = self.ui.upgrade().map(|u| selection_character(&u));
        self.mark_current_font(current.as_ref().and_then(|v| v["font"].as_str()));
        self.filter_fonts();
        let selected = self
            .font_rows
            .borrow()
            .iter()
            .find(|(name, _)| {
                current
                    .as_ref()
                    .and_then(|v| v["font"].as_str())
                    .is_some_and(|f| f.eq_ignore_ascii_case(name))
            })
            .map(|(_, b)| b.downgrade());
        if let Some(row) = selected {
            glib::idle_add_local_once(move || {
                if let Some(row) = row.upgrade().filter(|r| r.is_mapped()) {
                    reveal_font_row(&row);
                }
            });
        }
    }
    pub(super) fn sync_document_fonts(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade() else { return };
        let mut names = ui
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
        if let Some(current) = selection_character(&ui)["font"].as_str() {
            names.push(current.into());
        }
        names.sort_by_key(|s| s.to_lowercase());
        names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        if *self.font_document_key.borrow() == names {
            return;
        }
        self.font_document_key.replace(names.clone());
        let old = self
            .font_rows
            .borrow()
            .iter()
            .filter(|(_, b)| b.parent().as_ref() == Some(self.font_document_rows.upcast_ref()))
            .cloned()
            .collect::<Vec<_>>();
        for (name, button) in &old {
            if !names.contains(name) {
                self.font_document_rows.remove(button);
            }
        }
        self.font_rows
            .borrow_mut()
            .retain(|(_, b)| b.parent().is_some());
        let installed = self.family_names();
        let mut previous: Option<gtk::Widget> = None;
        for name in names {
            let button = old
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, b)| b.clone())
                .unwrap_or_else(|| {
                    let b = self.font_row(
                        &name,
                        installed.iter().any(|f| f.eq_ignore_ascii_case(&name)),
                    );
                    self.font_document_rows.append(&b);
                    b
                });
            self.font_document_rows
                .reorder_child_after(&button, previous.as_ref());
            previous = Some(button.upcast());
        }
    }
    pub(super) fn mark_current_font(&self, current: Option<&str>) {
        for (name, row) in self.font_rows.borrow().iter() {
            let selected = current.is_some_and(|f| f.eq_ignore_ascii_case(name));
            if selected {
                row.add_css_class("current-style");
            } else {
                row.remove_css_class("current-style");
            }
            row.update_state(&[gtk::accessible::State::Pressed(if selected {
                gtk::AccessibleTristate::True
            } else {
                gtk::AccessibleTristate::False
            })]);
            if let Some(mark) = find_widget(row, "font-current-marker") {
                mark.set_visible(selected);
            }
        }
    }
    fn filter_fonts(&self) {
        let query = self.font.text().trim().to_lowercase();
        let mut found = false;
        for (name, row) in self.font_rows.borrow().iter() {
            let visible = name.to_lowercase().contains(&query);
            row.set_visible(visible);
            found |= visible;
        }
        if let Some(empty) = find_widget(&self.font_choices, "font-no-results") {
            empty.set_visible(!found);
        }
    }
    fn show_sizes(self: &Rc<Self>) {
        clear_box(&self.size_choices);
        let Some(ui) = self.ui.upgrade() else { return };
        let sizes = ui
            .current
            .borrow()
            .as_ref()
            .map(|d| wordcraft_gnome::text_properties::used_sizes(&d.session))
            .unwrap_or_default();
        for size in sizes {
            let button = gtk::Button::with_label(&format!(
                "{} pt",
                format!("{size:.2}")
                    .trim_end_matches('0')
                    .trim_end_matches('.')
            ));
            button.add_css_class("flat");
            button.set_widget_name(&format!("size-choice-{size}"));
            let weak = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.size.set_value(size);
                    s.apply_size();
                    s.size_menu.popdown();
                }
            });
            self.size_choices.append(&button);
        }
    }
}
pub(super) fn text_selector(
    entry: &impl IsA<gtk::Widget>,
    name: &str,
) -> (gtk::Box, gtk::MenuButton, gtk::Box) {
    let linked = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    linked.add_css_class("linked");
    linked.set_hexpand(true);
    linked.append(entry);
    let menu = gtk::MenuButton::new();
    menu.set_icon_name("pan-down-symbolic");
    menu.set_widget_name(name);
    let label = if name == "text-font-choices" {
        wordcraft_gnome::t!("Font")
    } else {
        wordcraft_gnome::t!("Font Size")
    };
    accessible_name(&menu, label);
    menu.set_tooltip_text(Some(label));
    linked.append(&menu);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(320);
    scroll.set_propagate_natural_height(true);
    // The selector allocation owns the width; preview text must never inflate it.
    scroll.set_propagate_natural_width(false);
    scroll.set_child(Some(&content));
    let popover = gtk::Popover::new();
    popover.set_child(Some(&scroll));
    let sizing = scroll.downgrade();
    let anchor = linked.downgrade();
    let button = menu.downgrade();
    let resize = Rc::new(move |pop: &gtk::Popover| {
        if let (Some(scroll), Some(anchor), Some(button)) =
            (sizing.upgrade(), anchor.upgrade(), button.upgrade())
        {
            let width = anchor.width();
            if width <= 0 {
                return;
            }
            // Measure native popover chrome after allocation, with a first-map
            // fallback. Keep the whole popup approximately as wide as the row.
            let chrome = if pop.width() > 0 && scroll.width() > 0 {
                (pop.width() - scroll.width()).max(0)
            } else {
                24
            };
            let target = (width - chrome).max(1);
            if scroll.width_request() != target {
                // Clear the old minimum before changing a bounded range in
                // either direction; GTK rejects min > old max (and vice versa).
                scroll.set_min_content_width(-1);
                scroll.set_max_content_width(target);
                scroll.set_min_content_width(target);
                scroll.set_size_request(target, -1);
            }
            pop.set_pointing_to(Some(&gdk::Rectangle::new(
                button.width() - width,
                0,
                width,
                button.height(),
            )));
        }
    });
    let size_popup = resize.clone();
    popover.connect_show(move |pop| size_popup(pop));
    // Allocation changes have no notify signal. A mapped-only frame callback
    // follows adaptive resizing and is explicitly removed when the popup closes.
    let tick = Rc::new(RefCell::new(None::<gtk::TickCallbackId>));
    let slot = tick.clone();
    popover.connect_map(move |pop| {
        let resize = resize.clone();
        *slot.borrow_mut() = Some(pop.add_tick_callback(move |pop, _| {
            resize(pop);
            glib::ControlFlow::Continue
        }));
    });
    popover.connect_unmap(move |_| {
        if let Some(id) = tick.borrow_mut().take() {
            id.remove();
        }
    });
    menu.set_popover(Some(&popover));
    (linked, menu, content)
}
pub(super) fn install_text_controls(
    ui: &Rc<Ui>,
    font_menu: gtk::MenuButton,
    size_menu: gtk::MenuButton,
    font_choices: gtk::Box,
    size_choices: gtk::Box,
) -> Rc<TextControls> {
    let s = Rc::new(TextControls {
        ui: Rc::downgrade(ui),
        font: ui.font_input.clone(),
        size: ui.size_input.clone(),
        font_menu,
        size_menu,
        font_choices,
        size_choices,
        font_rows: RefCell::new(Vec::new()),
        font_document_rows: gtk::Box::new(gtk::Orientation::Vertical, 2),
        font_all_rows: gtk::Box::new(gtk::Orientation::Vertical, 2),
        font_document_key: RefCell::new(Vec::new()),
        font_source: RefCell::new(None),
        size_source: RefCell::new(None),
    });
    validate_numeric_input(&s.size);
    let weak = Rc::downgrade(&s);
    s.font.connect_changed(move |_| {
        if let Some(s) = weak.upgrade() {
            s.filter_fonts();
        }
    });
    let weak = Rc::downgrade(&s);
    s.font.connect_activate(move |_| {
        if let Some(s) = weak.upgrade() {
            s.apply_font(&s.font.text());
        }
    });
    let weak = Rc::downgrade(&s);
    s.size.connect_changed(move |_| {
        if let Some(s) = weak.upgrade() {
            s.schedule(false);
        }
    });
    let weak = Rc::downgrade(&s);
    s.size.connect_value_changed(move |_| {
        if let Some(s) = weak.upgrade() {
            if s.ui.upgrade().is_some_and(|u| !u.format_syncing.get()) {
                s.apply_size();
            }
        }
    });
    let weak = Rc::downgrade(&s);
    s.size.connect_activate(move |_| {
        if let Some(s) = weak.upgrade() {
            s.apply_size();
        }
    });
    let weak = Rc::downgrade(&s);
    s.font_menu.popover().unwrap().connect_show(move |_| {
        if let Some(s) = weak.upgrade() {
            s.show_fonts();
        }
    });
    let weak = Rc::downgrade(&s);
    s.size_menu.popover().unwrap().connect_show(move |_| {
        if let Some(s) = weak.upgrade() {
            s.show_sizes();
        }
    });
    s
}

fn reveal_font_row(row: &gtk::Button) {
    if let Some(scroll) = row
        .ancestor(gtk::ScrolledWindow::static_type())
        .and_downcast::<gtk::ScrolledWindow>()
    {
        if let Some(bounds) = row.compute_bounds(&scroll) {
            let a = scroll.vadjustment();
            let delta = if bounds.y() < 0. {
                bounds.y()
            } else {
                (bounds.y() + bounds.height() - scroll.height() as f32).max(0.)
            };
            a.set_value(
                (a.value() + delta as f64)
                    .clamp(a.lower(), (a.upper() - a.page_size()).max(a.lower())),
            );
        }
    }
}
