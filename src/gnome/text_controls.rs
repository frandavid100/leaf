use super::*;

pub(super) struct FormatSequence {
    document: u64,
    revision: u64,
    selection: String,
    command: String,
    time: std::time::Instant,
}

/// Apply a validated control without closing/rebuilding its popover or moving focus.
pub(super) fn immediate_format(
    ui: &Rc<Ui>,
    command: &str,
    args: serde_json::Value,
) -> Result<(), String> {
    let focus = ui
        .host
        .upgrade()
        .and_then(|h| gtk::prelude::GtkWindowExt::focus(&h.window))
        .map(|w| w.downgrade());
    let scroll = (
        ui.scroll.hadjustment().value(),
        ui.scroll.vadjustment().value(),
    );
    let changed = {
        let mut current = ui.current.borrow_mut();
        let d = current.as_mut().ok_or(wordcraft_gnome::t!("No document"))?;
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
    if changed {
        after_content_change(ui);
    } else {
        refresh_selection(ui);
    }
    let identity = document_identity(ui);
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
                    w.grab_focus();
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
    font_choices: gtk::Box,
    size_choices: gtk::Box,
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
            self.font.add_css_class("error");
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
    fn show_fonts(self: &Rc<Self>) {
        clear_box(&self.font_choices);
        let size = self
            .ui
            .upgrade()
            .and_then(|ui| selection_character(&ui)["size"].as_f64())
            .unwrap_or(12.0);
        for family in self.family_names() {
            let button = gtk::Button::new();
            button.add_css_class("flat");
            let body = gtk::Box::new(gtk::Orientation::Vertical, 4);
            let caption = gtk::Label::new(Some(&family));
            caption.set_xalign(0.0);
            caption.add_css_class("caption");
            body.append(&caption);
            let preview = gtk::Label::new(Some(&family));
            preview.set_widget_name("font-preview");
            preview.set_xalign(0.0);
            preview.set_ellipsize(gtk::pango::EllipsizeMode::End);
            preview.set_max_width_chars(28);
            let mut description = gtk::pango::FontDescription::new();
            description.set_family(&family);
            description.set_size((size * gtk::pango::SCALE as f64).round() as i32);
            let attrs = gtk::pango::AttrList::new();
            attrs.insert(gtk::pango::AttrFontDesc::new(&description));
            preview.set_attributes(Some(&attrs));
            let clip = gtk::ScrolledWindow::new();
            clip.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never);
            clip.set_max_content_height(80);
            clip.set_propagate_natural_height(true);
            clip.set_child(Some(&preview));
            body.append(&clip);
            button.set_child(Some(&body));
            accessible_name(&button, &family);
            let weak = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if let Some(ui) = s.ui.upgrade() {
                        ui.format_syncing.set(true);
                        s.font.set_text(&family);
                        ui.format_syncing.set(false);
                    }
                    s.apply_font(&family);
                    s.font_menu.popdown();
                }
            });
            self.font_choices.append(&button);
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
            let button = gtk::Button::with_label(&format!("{size} pt"));
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
    scroll.set_min_content_width(180);
    scroll.set_child(Some(&content));
    let popover = gtk::Popover::new();
    popover.set_child(Some(&scroll));
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
        font_source: RefCell::new(None),
        size_source: RefCell::new(None),
    });
    validate_numeric_input(&s.size);
    let weak = Rc::downgrade(&s);
    s.font.connect_changed(move |_| {
        if let Some(s) = weak.upgrade() {
            s.schedule(true);
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
