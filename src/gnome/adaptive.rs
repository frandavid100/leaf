use super::*;

// Each group keeps a stable toolbar anchor; only shortcut visibility adapts.
pub(super) struct AdaptiveTools {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) history: gtk::Box,
    pub(super) formats: gtk::Box,
    pub(super) top_history: gtk::Box,
    pub(super) top_formats: gtk::Box,
    pub(super) bottom: gtk::Box,
    pub(super) separator: gtk::Separator,
    pub(super) narrow: Cell<bool>,
    pub(super) width_band: Cell<u8>,
    pub(super) generation: Cell<u64>,
    pub(super) pending: RefCell<Vec<(glib::WeakRef<gtk::MenuButton>, Option<(u64, u64)>)>>,
}
impl AdaptiveTools {
    pub(super) fn sync_width(self: &Rc<Self>, width: i32) {
        if width <= 0 {
            return;
        }
        let Some(ui) = self.ui.upgrade() else { return };
        if ui.template_gallery.borrow().is_some() {
            templates::align_start(&ui);
        }
        self.narrow.set(width < 640);
        let band =
            u8::from(width >= 520) | (u8::from(width >= 680) << 1) | (u8::from(width >= 850) << 2);
        let changed = self.width_band.replace(band) != band;
        if changed {
            let identity = document_identity(&ui);
            let mut pending = self.pending.borrow_mut();
            for menu in ui.format_buttons.borrow().iter() {
                if menu.popover().is_some_and(|p| p.get_visible()) {
                    if !pending
                        .iter()
                        .any(|(m, _)| m.upgrade().as_ref() == Some(menu))
                    {
                        pending.push((menu.downgrade(), identity));
                    }
                    menu.popdown();
                }
            }
            ui.format_reflowing.set(!pending.is_empty());
        }
        // History stays in the header; document formatting always has the same
        // bottom anchor. Resizing never reparents an open or pinned popover.
        if self.history.parent().as_ref() != Some(self.top_history.upcast_ref()) {
            if let Some(parent) = self.history.parent().and_downcast::<gtk::Box>() {
                parent.remove(&self.history);
            }
            self.top_history.append(&self.history);
        }
        if self.formats.parent().as_ref() != Some(self.bottom.upcast_ref()) {
            if let Some(parent) = self.formats.parent().and_downcast::<gtk::Box>() {
                parent.remove(&self.formats);
            }
            self.bottom.append(&self.formats);
        }
        self.top_formats.set_visible(false);
        self.separator.set_visible(false);
        ui.format_toolbar.set_visible(true);
        for menu in ui.format_buttons.borrow().iter() {
            menu.set_direction(gtk::ArrowType::Up);
        }
        if let Some(tools) = ui.quick_tools.borrow().as_ref() {
            tools.width(width);
        }
        if changed && !self.pending.borrow().is_empty() {
            // GTK can unmap a popup while its scrolling toolbar is allocated.
            // Let the sidebar's adaptive animation settle before restoring the
            // same menu and subpage; an early restore can target an unmapped anchor.
            let generation = self.generation.get().wrapping_add(1);
            self.generation.set(generation);
            let weak = Rc::downgrade(self);
            glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
                let Some(tools) = weak.upgrade().filter(|a| a.generation.get() == generation)
                else {
                    return;
                };
                let pending = tools.pending.take();
                if let Some(ui) = tools.ui.upgrade().filter(|u| !u.closed.get()) {
                    for (menu, identity) in pending {
                        if identity == document_identity(&ui) {
                            if let Some(menu) = menu.upgrade() {
                                if std::env::args().any(|a| a == "--workflows40-ui-test") {
                                    eprintln!(
                                        "WORKFLOWS40_REFLOW anchor={} mapped={} width={width}",
                                        menu.widget_name(),
                                        menu.is_mapped()
                                    );
                                }
                                if menu.is_mapped() {
                                    menu.popup();
                                }
                            }
                        }
                    }
                    ui.format_reflowing.set(false);
                }
            });
        }
    }
}
pub(super) fn history_icon(redo: bool) -> gtk::Image {
    let icon = gtk::Image::from_icon_name(if redo {
        "edit-redo-symbolic"
    } else {
        "edit-undo-symbolic"
    });
    icon.set_pixel_size(16);
    icon
}
pub(super) fn format_icon_button(menu: &gtk::MenuButton, label: &str, kind: &str) {
    let name = match kind {
        "style" => "headings-allcap-symbolic",
        "paragraph" => "paragraph-symbolic",
        _ => "font-symbolic",
    };
    menu.set_icon_name(name);
    menu.add_css_class("flat");
    menu.add_css_class("image-button");
    menu.set_tooltip_text(Some(label));
    menu.update_property(&[gtk::accessible::Property::Label(label)]);
}
pub(super) fn format_button(ui: &Ui, name: &str) -> Option<gtk::MenuButton> {
    ui.format_buttons
        .borrow()
        .iter()
        .find(|m| m.widget_name() == name)
        .cloned()
}

pub(super) struct TextTools {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) popover: glib::WeakRef<gtk::Popover>,
    pub(super) stack: gtk::Stack,
    pub(super) detail: gtk::Box,
    pub(super) header: Rc<FormatHeader>,
    active_page: RefCell<Option<String>>,
    displayed: RefCell<serde_json::Value>,
    pub(super) palette: RefCell<Option<Rc<TextPalette>>>,
    language_value: gtk::Label,
    font_value: gtk::Label,
    font_page: gtk::Box,
    language_page: gtk::Box,
    last_font: RefCell<Option<String>>,
    last_language: RefCell<Option<String>>,
}
impl TextTools {
    pub(super) fn remember_property(&self, command: &str, args: &serde_json::Value) {
        if let Some(pop) = self.popover.upgrade() {
            if let Some(button) = find_widget(
                &pop,
                if command == "format.font" {
                    "text-font-apply"
                } else if command == "leaf.proofText" {
                    "text-language-apply"
                } else {
                    ""
                },
            ) {
                button.set_sensitive(true);
            }
        }
        if command == "format.font" {
            if let Some(name) = args["name"].as_str() {
                self.last_font.replace(Some(name.into()));
            }
        }
        if command == "leaf.proofText" {
            if let Some(choice) = args["choice"].as_str() {
                self.last_language.replace(Some(choice.into()));
            }
        }
    }
    pub(super) fn language_value_text(&self) -> glib::GString {
        self.language_value.text()
    }
    pub(super) fn sync_selection(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade() else { return };
        if let Some(palette) = self.palette.borrow().as_ref() {
            palette.sync();
        }
        if let Some(d) = ui.current.borrow().as_ref() {
            let value = wordcraft_gnome::proofing_defaults::choice(&d.session);
            let name = |value: &str| -> String {
                match value {
                    "off" => wordcraft_gnome::t!("Do not check spelling or grammar").into(),
                    "mixed" => wordcraft_gnome::t!("Mixed").into(),
                    "en-US" => wordcraft_gnome::t!("English (United States)").into(),
                    "en-GB" => wordcraft_gnome::t!("English (United Kingdom)").into(),
                    "es-ES" => wordcraft_gnome::t!("Spanish (Spain)").into(),
                    "es-MX" => wordcraft_gnome::t!("Spanish (Mexico)").into(),
                    "es-AR" => wordcraft_gnome::t!("Spanish (Argentina)").into(),
                    _ => TEXT_LANGUAGES
                        .iter()
                        .find(|(tag, _)| *tag == value)
                        .map_or(value, |(_, label)| *label)
                        .into(),
                }
            };
            let label = if value == "default" {
                wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Default — {language}"),
                    &[(
                        "language",
                        name(&wordcraft_gnome::proofing_defaults::effective_choice(
                            &d.session,
                        )),
                    )],
                )
            } else {
                name(&value)
            };
            let label = wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("{label}: {value}"),
                &[
                    ("label", wordcraft_gnome::t!("Language").into()),
                    ("value", label),
                ],
            );
            if self.active_page.borrow().as_deref() == Some("language") {
                self.sync_language_rows(&ui);
            }
            self.language_value.set_text(&label);
            self.language_value.set_tooltip_text(Some(&label));
            if let Some(button) = self
                .language_value
                .ancestor(gtk::Button::static_type())
                .and_then(|w| w.downcast::<gtk::Button>().ok())
            {
                accessible_name(
                    &button,
                    &wordcraft_gnome::i18n::interpolate(
                        wordcraft_gnome::t!("Text Language: {language}"),
                        &[("language", label)],
                    ),
                );
            }
        }
        let state = selection_character(&ui);
        let font_label = format!(
            "{}: {}",
            wordcraft_gnome::t!("Font"),
            state["font"]
                .as_str()
                .unwrap_or(wordcraft_gnome::t!("Mixed"))
        );
        self.font_value.set_text(&font_label);
        self.font_value.set_tooltip_text(Some(&font_label));
        let Some(origin) = self.popover.upgrade() else {
            return;
        };
        let pop = embedded_format(&origin)
            .map(|(host, _)| host)
            .unwrap_or(origin);
        if !pop.is_visible() {
            return;
        }
        if let Some(controls) = ui.text_controls.borrow().as_ref() {
            controls.mark_current_font(state["font"].as_str());
        }
        if *self.displayed.borrow() == state {
            return;
        }
        if ui
            .host
            .upgrade()
            .and_then(|h| gtk::prelude::GtkWindowExt::focus(&h.window))
            .is_some_and(|w| w.is_ancestor(&pop))
        {
            return;
        }
        self.displayed.replace(state);
        let page = self.active_page.borrow().clone();
        if let Some(page) = page {
            let scroll = self
                .detail
                .ancestor(gtk::ScrolledWindow::static_type())
                .and_then(|w| w.downcast::<gtk::ScrolledWindow>().ok());
            let y = scroll.as_ref().map(|s| s.vadjustment().value());
            self.page(&page);
            if let (Some(scroll), Some(y)) = (scroll, y) {
                let weak = scroll.downgrade();
                glib::idle_add_local_once(move || {
                    if let Some(s) = weak.upgrade() {
                        s.vadjustment().set_value(y);
                    }
                });
            }
        }
    }
    pub(super) fn home(self: &Rc<Self>) {
        self.active_page.replace(None);
        self.header.home();
        self.stack.set_visible_child_name("main");
    }
    pub(super) fn execute(&self, cmd: &str, args: serde_json::Value) {
        if let Some(ui) = self.ui.upgrade() {
            match format_execute(&ui, cmd, args) {
                Ok(()) => {
                    if let Some(p) = self.popover.upgrade() {
                        close_format_popover(&p);
                    }
                    if !ui.format_group.pinned.get() {
                        ui.document_box.grab_focus();
                        caret_ensure_visible(&ui);
                    }
                }
                Err(e) => ui.toasts.add_toast(Toast::new(&e)),
            }
        }
    }
    pub(super) fn action(self: &Rc<Self>, label: &str, cmd: &'static str, args: serde_json::Value) {
        let b = style_menu_button(label);
        let weak = Rc::downgrade(self);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.execute(cmd, args.clone());
            }
        });
        self.detail.append(&b);
    }
    pub(super) fn heading(&self, label: &str) {
        let l = gtk::Label::new(Some(label));
        l.set_xalign(0.0);
        l.add_css_class("heading");
        l.set_margin_top(8);
        self.detail.append(&l);
    }
    pub(super) fn choice(
        self: &Rc<Self>,
        title: &str,
        options: &[(&str, &str)],
        cmd: &'static str,
        key: &str,
    ) {
        self.heading(title);
        for (label, value) in options {
            self.action(label, cmd, serde_json::json!({key:*value}));
        }
    }
    pub(super) fn toggle(self: &Rc<Self>, label: &str, on: bool, cmd: &'static str) {
        let b = gtk::CheckButton::with_label(label);
        b.set_active(on);
        if let Some(ui) = self.ui.upgrade() {
            let state = selection_character(&ui);
            let key = match cmd {
                "format.strikethrough" => "strike",
                "format.doubleStrikethrough" => "doubleStrike",
                "format.allCaps" => "caps",
                _ => cmd.rsplit('.').next().unwrap_or(""),
            };
            b.set_inconsistent(state.get(key).is_some_and(|v| v.is_null()));
        }
        self.detail.append(&b);
        let weak = Rc::downgrade(self);
        b.connect_toggled(move |b| {
            if let Some(s) = weak.upgrade() {
                b.set_inconsistent(false);
                s.execute(cmd, serde_json::json!({"value":b.is_active()}));
            }
        });
    }
    pub(super) fn number(&self, label: &str, value: f64, min: f64, max: f64) -> gtk::SpinButton {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let l = gtk::Label::new(Some(label));
        l.set_xalign(0.0);
        l.set_hexpand(true);
        l.set_wrap(true);
        row.append(&l);
        let n = gtk::SpinButton::with_range(min, max, 0.5);
        n.set_digits(1);
        n.set_value(value);
        accessible_name(&n, label);
        row.append(&n);
        self.detail.append(&row);
        n
    }
    pub(super) fn page(self: &Rc<Self>, page: &str) {
        // Never unparent a mapped editable merely to reopen its current page.
        // GTK owns its native focus/clipboard target across pinning and typing.
        if matches!(page, "font" | "language")
            && self.active_page.borrow().as_deref() == Some(page)
            && self.stack.visible_child_name().as_deref() == Some("detail")
            && (if page == "font" {
                &self.font_page
            } else {
                &self.language_page
            })
            .parent()
            .as_ref()
                == Some(self.detail.upcast_ref())
        {
            return;
        }
        if let Some(pop) = self.popover.upgrade() {
            if let Some(root) = pop.child() {
                root.set_width_request(if matches!(page, "underline" | "strike" | "superscript") {
                    240
                } else {
                    300
                });
            }
        }
        self.active_page.replace(Some(page.into()));
        clear_box(&self.detail);
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = selection_character(&ui);
        if let Some(scroll) = self
            .detail
            .ancestor(gtk::ScrolledWindow::static_type())
            .and_downcast::<gtk::ScrolledWindow>()
        {
            let height = ui.host.upgrade().map_or(850, |h| h.window.height());
            scroll.set_max_content_height(if page == "font" {
                (height - 150).clamp(220, 650)
            } else {
                (height - 150).clamp(180, 460)
            });
        }
        let weak = Rc::downgrade(self);
        let home = matches!(
            page,
            "options" | "language" | "font" | "underline" | "strike" | "ink" | "highlight"
        );
        let title = match page {
            "underline" => wordcraft_gnome::t!("Underline"),
            "underline-color" => wordcraft_gnome::t!("Color"),
            "superscript" => wordcraft_gnome::t!("Text Position"),
            "strike" => wordcraft_gnome::t!("Strikethrough"),
            "ink" => wordcraft_gnome::t!("Text Color"),
            "highlight" => wordcraft_gnome::t!("Highlight"),
            "font" => wordcraft_gnome::t!("Font"),
            "language" => wordcraft_gnome::t!("Language"),
            "character" => wordcraft_gnome::t!("Font and Characters"),
            "color" => wordcraft_gnome::t!("Color and Effects"),
            "case" => wordcraft_gnome::t!("Change Case"),
            "annotations" => wordcraft_gnome::t!("Annotations"),
            _ => wordcraft_gnome::t!("More Text Options…"),
        };
        let page_owned = page.to_owned();
        self.header.page(page, title, move || {
            if let Some(s) = weak.upgrade() {
                if page_owned == "underline-color" {
                    s.page("underline");
                } else if home {
                    s.home();
                } else {
                    s.page("options");
                }
            }
        });
        if let Some(panel) = self
            .palette
            .borrow()
            .as_ref()
            .and_then(|p| p.options_page(page))
        {
            if let Some(parent) = panel.parent().and_then(|p| p.downcast::<gtk::Box>().ok()) {
                parent.remove(&panel);
            }
            self.detail.append(&panel);
            self.stack.set_visible_child_name("detail");
            return;
        }
        match page {
            "font" => {
                if find_widget(&self.font_page, "text-font-list").is_none() {
                    let Some(controls) = ui.text_controls.borrow().as_ref().cloned() else {
                        return;
                    };
                    // A single searchable selection page, with the existing
                    // editable and command/debounce controller retained.
                    if let Some(parent) = ui
                        .font_input
                        .parent()
                        .and_then(|p| p.downcast::<gtk::Box>().ok())
                    {
                        parent.remove(&ui.font_input);
                    }
                    clear_box(&self.font_page);
                    ui.font_input.set_icon_from_icon_name(
                        gtk::EntryIconPosition::Primary,
                        Some("edit-find-symbolic"),
                    );
                    ui.font_input
                        .set_placeholder_text(Some(wordcraft_gnome::t!("Search fonts")));
                    accessible_name(&ui.font_input, wordcraft_gnome::t!("Search fonts"));
                    self.font_page.append(&ui.font_input);
                    let choices = controls.font_choices.clone();
                    if let Some(parent) = choices
                        .parent()
                        .and_then(|p| p.downcast::<gtk::ScrolledWindow>().ok())
                    {
                        parent.set_child(None::<&gtk::Widget>);
                    } else if let Some(parent) = choices
                        .parent()
                        .and_then(|p| p.downcast::<gtk::Viewport>().ok())
                    {
                        parent.set_child(None::<&gtk::Widget>);
                    }
                    let list = gtk::ScrolledWindow::new();
                    list.set_widget_name("text-font-list");
                    list.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
                    list.set_min_content_height(160);
                    list.set_max_content_height(
                        ui.host
                            .upgrade()
                            .map_or(400, |h| (h.window.height() - 230).clamp(160, 560)),
                    );
                    list.set_propagate_natural_height(true);
                    list.set_child(Some(&choices));
                    self.font_page.append(&list);
                }
                if let Some(controls) = ui.text_controls.borrow().as_ref() {
                    let syncing = ui.format_syncing.replace(true);
                    ui.font_input.set_text("");
                    ui.font_input.remove_css_class("error");
                    ui.format_syncing.set(syncing);
                    controls.show_fonts();
                }
                if let Some(parent) = self
                    .font_page
                    .parent()
                    .and_then(|p| p.downcast::<gtk::Box>().ok())
                {
                    parent.remove(&self.font_page);
                }
                self.detail.append(&self.font_page);
            }
            "language" => {
                self.build_languages(&ui);
                if let Some(parent) = self.language_page.parent().and_downcast::<gtk::Box>() {
                    parent.remove(&self.language_page);
                }
                self.detail.append(&self.language_page);
                self.sync_language_rows(&ui);
            }
            "case" => self.choice(
                wordcraft_gnome::t!("Change Case"),
                &[
                    (wordcraft_gnome::t!("Sentence case"), "sentence"),
                    (wordcraft_gnome::t!("lowercase"), "lower"),
                    (wordcraft_gnome::t!("UPPERCASE"), "upper"),
                    (wordcraft_gnome::t!("Capitalize Each Word"), "title"),
                    (wordcraft_gnome::t!("Invert"), "toggle"),
                ],
                "format.changeCase",
                "mode",
            ),
            "options" => {
                for (label, page) in [
                    (wordcraft_gnome::t!("Font and Characters"), "character"),
                    (wordcraft_gnome::t!("Color and Effects"), "color"),
                    (wordcraft_gnome::t!("Annotations"), "annotations"),
                    (wordcraft_gnome::t!("Change Case"), "case"),
                ] {
                    let b = style_menu_button(label);
                    b.set_widget_name(&format!("text-options-{page}"));
                    let weak = Rc::downgrade(self);
                    b.connect_clicked(move |_| {
                        if let Some(s) = weak.upgrade() {
                            s.page(page);
                        }
                    });
                    self.detail.append(&b);
                }
                let clear = style_menu_button(wordcraft_gnome::t!("Clear Text Formatting"));
                clear.set_widget_name("text-clear-formatting");
                let weak = Rc::downgrade(self);
                clear.connect_clicked(move |_| {
                    if let Some(panel) = weak.upgrade() {
                        panel.execute("format.clear", serde_json::json!({}));
                    }
                });
                self.detail.append(&clear);
            }
            "character" => {
                self.heading(wordcraft_gnome::t!("Character Formatting"));
                for (label, key, cmd) in [
                    (
                        wordcraft_gnome::t!("Small Caps"),
                        "smallCaps",
                        "format.smallCaps",
                    ),
                    (
                        wordcraft_gnome::t!("Capitalization"),
                        "caps",
                        "format.allCaps",
                    ),
                    (
                        wordcraft_gnome::t!("Hidden Text"),
                        "hidden",
                        "format.hidden",
                    ),
                ] {
                    self.toggle(label, state[key].as_bool().unwrap_or(false), cmd);
                }
                for (label, cmd) in [
                    (wordcraft_gnome::t!("Subscript"), "format.subscript"),
                    (wordcraft_gnome::t!("Increase Size"), "format.growFont"),
                    (wordcraft_gnome::t!("Decrease Size"), "format.shrinkFont"),
                ] {
                    self.action(label, cmd, serde_json::json!({}));
                }
                self.heading(wordcraft_gnome::t!("Character Spacing"));
                let spacing = self.number(
                    wordcraft_gnome::t!("Spacing (pt)"),
                    state["spacing"].as_f64().unwrap_or(0.0),
                    -100.0,
                    100.0,
                );
                let scale = self.number(
                    wordcraft_gnome::t!("Scale (%)"),
                    state["scale"].as_f64().unwrap_or(100.0),
                    1.0,
                    600.0,
                );
                let position = self.number(
                    wordcraft_gnome::t!("Offset (pt)"),
                    state["position"].as_f64().unwrap_or(0.0),
                    -100.0,
                    100.0,
                );
                for (n, key) in [
                    (&spacing, "spacing"),
                    (&scale, "scale"),
                    (&position, "position"),
                ] {
                    n.set_widget_name(&format!("text-{key}"));
                    if state[key].is_null() {
                        n.set_text("");
                    }
                    let weak = Rc::downgrade(&ui);
                    bind_numeric(&ui, n, move |value| {
                        let ui = weak.upgrade().ok_or("Document closed")?;
                        immediate_format(
                            &ui,
                            "format.set",
                            serde_json::json!({"props":{key:value}}),
                        )
                    });
                }
            }
            "color" => {
                self.heading(wordcraft_gnome::t!("Effects"));
                for (label, key, cmd) in [
                    (wordcraft_gnome::t!("Outline"), "outline", "format.outline"),
                    (wordcraft_gnome::t!("Shadow"), "shadow", "format.shadow"),
                    (wordcraft_gnome::t!("Embossed"), "emboss", "format.emboss"),
                    (wordcraft_gnome::t!("Engraved"), "engrave", "format.engrave"),
                    (
                        wordcraft_gnome::t!("Text Border"),
                        "border",
                        "format.border",
                    ),
                ] {
                    self.toggle(label, state[key].as_bool().unwrap_or(false), cmd);
                }
            }
            _ => {
                self.choice(
                    wordcraft_gnome::t!("Enclosed Characters"),
                    &[
                        (wordcraft_gnome::t!("Circle"), "circle"),
                        (wordcraft_gnome::t!("Square"), "square"),
                        (wordcraft_gnome::t!("Triangle"), "triangle"),
                        (wordcraft_gnome::t!("Diamond"), "diamond"),
                    ],
                    "format.enclose",
                    "shape",
                );
                self.action(
                    wordcraft_gnome::t!("Remove Shape"),
                    "format.enclose",
                    serde_json::json!({"shape":null}),
                );
                self.heading(wordcraft_gnome::t!("Phonetic Guide"));
                let ruby = gtk::Entry::new();
                ruby.set_placeholder_text(Some(wordcraft_gnome::t!("Annotation")));
                accessible_name(&ruby, wordcraft_gnome::t!("Phonetic Annotation"));
                self.detail.append(&ruby);
                let size = self.number(wordcraft_gnome::t!("Size (pt)"), 8.0, 1.0, 96.0);
                let offset = self.number(wordcraft_gnome::t!("Spacing (pt)"), 0.0, -20.0, 100.0);
                let b = style_menu_button(wordcraft_gnome::t!("Apply Phonetic Guide"));
                self.detail.append(&b);
                let weak = Rc::downgrade(self);
                b.connect_clicked(move |_| {
                    if let Some(panel) = weak.upgrade() {
                        size.update();
                        offset.update();
                        panel.execute(
                            "format.phonetic",
                            serde_json::json!({
                                "ruby": ruby.text().as_str(),
                                "size": size.value(),
                                "offset": offset.value(),
                                "align": "center",
                            }),
                        );
                    }
                });
                self.action(
                    wordcraft_gnome::t!("Remove Phonetic Guide"),
                    "format.phonetic",
                    serde_json::json!({"ruby":null}),
                );
            }
        }
        self.stack.set_visible_child_name("detail");
    }
}
pub(super) fn install_text_pages(
    ui: &Rc<Ui>,
    pop: &gtk::Popover,
    root: &gtk::Box,
    _inline: &gtk::Box,
    header: &Rc<FormatHeader>,
    more: &gtk::Button,
) -> Rc<TextTools> {
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    stack.set_interpolate_size(true);
    stack.set_vhomogeneous(false);
    stack.set_hhomogeneous(false);
    let main = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let mut child = root
        .first_child()
        .and_then(|w| w.next_sibling())
        .and_then(|w| w.next_sibling());
    while let Some(w) = child {
        child = w.next_sibling();
        root.remove(&w);
        main.append(&w);
    }
    let detail = gtk::Box::new(gtk::Orientation::Vertical, 6);
    detail.set_margin_bottom(8);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(420);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&detail));
    let main_scroll = gtk::ScrolledWindow::new();
    main_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    main_scroll.set_max_content_height(430);
    main_scroll.set_propagate_natural_height(true);
    main_scroll.set_child(Some(&main));
    stack.add_named(&main_scroll, Some("main"));
    stack.add_named(&scroll, Some("detail"));
    root.append(&stack);
    root.set_width_request(320);
    let language_button = find_widget(&main, "text-language")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
    let language_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    language_button.add_css_class("flat");
    let language_value = gtk::Label::new(Some(wordcraft_gnome::t!("Default language")));
    language_value.set_widget_name("text-language-value");
    language_value.set_xalign(0.0);
    language_value.set_hexpand(true);
    language_value.set_ellipsize(gtk::pango::EllipsizeMode::End);
    language_row.append(&gtk::Image::from_icon_name(
        "preferences-desktop-locale-symbolic",
    ));
    language_row.append(&language_value);
    language_button.set_child(Some(&language_row));
    more.set_widget_name("text-more-options");
    let footer = gtk::Label::new(Some(wordcraft_gnome::t!("More Text Options…")));
    footer.set_xalign(0.);
    more.set_child(Some(&footer));
    main.remove(more);
    main.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    main.append(more);
    let (font_content, font_value) = property_content(wordcraft_gnome::t!("Font"), "font-symbolic");
    let font_page = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let s = Rc::new(TextTools {
        ui: Rc::downgrade(ui),
        popover: pop.downgrade(),
        stack,
        detail,
        header: header.clone(),
        palette: RefCell::new(None),
        language_value,
        font_value,
        font_page,
        language_page: gtk::Box::new(gtk::Orientation::Vertical, 6),
        last_font: RefCell::new(None),
        last_language: RefCell::new(None),
        active_page: RefCell::new(None),
        displayed: RefCell::new(serde_json::Value::Null),
    });
    let palette = TextPalette::new(ui, _inline);
    main.remove(_inline);
    let font_selector = ui
        .font_input
        .parent()
        .unwrap()
        .downcast::<gtk::Box>()
        .unwrap();
    let size_selector = ui
        .size_input
        .parent()
        .unwrap()
        .downcast::<gtk::Box>()
        .unwrap();
    for widget in [&font_selector, &size_selector] {
        if let Some(parent) = widget.parent().and_then(|p| p.downcast::<gtk::Box>().ok()) {
            parent.remove(widget);
        }
    }
    if let Some(parent) = language_button
        .parent()
        .and_then(|p| p.downcast::<gtk::Box>().ok())
    {
        parent.remove(&language_button);
    }
    main.remove(more);
    clear_box(&main);
    main.append(&palette.root);
    let font_open = style_menu_button(wordcraft_gnome::t!("Font"));
    font_open.remove_css_class("flat");
    font_open.set_widget_name("text-font-property");
    font_open.set_child(Some(&font_content));
    font_open.set_hexpand(true);
    s.font_page.append(&font_selector);
    let font_repeat = gtk::Button::from_icon_name("view-refresh-symbolic");
    font_repeat.set_widget_name("text-font-apply");
    font_repeat.set_sensitive(false);

    accessible_name(&font_repeat, wordcraft_gnome::t!("Apply Last Font"));
    font_repeat.set_tooltip_text(Some(wordcraft_gnome::t!("Apply Last Font")));
    let font_row = property_split(&font_open, &font_repeat);
    font_row.set_widget_name("format-row-text-font");
    main.append(&font_row);
    let size_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    size_row.append(&gtk::Image::from_icon_name("zoom-in-symbolic"));
    size_row.append(&gtk::Label::new(Some(wordcraft_gnome::t!("Size"))));
    ui.size_input.set_hexpand(true);
    size_selector.set_hexpand(true);
    size_selector.set_halign(gtk::Align::Fill);
    size_selector.remove_css_class("linked");
    size_selector.add_css_class("size-controls");
    size_row.append(&size_selector);
    let size_field = gtk::Frame::new(None);
    size_field.set_widget_name("text-size-field");
    size_row.set_margin_start(12);
    size_row.set_margin_end(4);
    size_row.set_margin_top(3);
    size_row.set_margin_bottom(3);
    size_field.add_css_class("size-field");
    size_field.set_child(Some(&size_row));
    main.append(&size_field);
    language_button.remove_css_class("flat");
    language_button.set_hexpand(true);
    let language_repeat = gtk::Button::from_icon_name("view-refresh-symbolic");

    language_repeat.set_widget_name("text-language-apply");
    language_repeat.set_sensitive(false);
    accessible_name(&language_repeat, wordcraft_gnome::t!("Apply Last Language"));
    language_repeat.set_tooltip_text(Some(wordcraft_gnome::t!("Apply Last Language")));
    let language_row = property_split(&language_button, &language_repeat);
    language_row.set_widget_name("format-row-text-language");
    main.append(&language_row);
    main.append(&palette.properties);
    main.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    main.append(more);
    s.palette.replace(Some(palette));
    let weak = Rc::downgrade(&s);
    font_open.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            s.page("font");
        }
    });
    let weak = Rc::downgrade(&s);
    font_repeat.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            let value = s.last_font.borrow().clone();
            if let Some(name) = value {
                s.execute("format.font", serde_json::json!({"name":name}));
            }
        }
    });
    let weak = Rc::downgrade(&s);
    language_repeat.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            let value = s.last_language.borrow().clone();
            if let Some(choice) = value {
                s.execute("leaf.proofText", serde_json::json!({"choice":choice}));
            }
        }
    });
    let weak = Rc::downgrade(&s);
    language_button.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            s.page("language");
        }
    });
    let weak = Rc::downgrade(&s);
    more.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            s.page("options");
        }
    });
    let state = s.clone();
    pop.connect_show(glib::clone!(
        #[weak]
        state,
        move |_| {
            if !state.ui.upgrade().is_some_and(|u| u.format_reflowing.get()) {
                let entry = state.header.entry_page.borrow().clone();
                let transition = state.stack.transition_type();
                state
                    .stack
                    .set_transition_type(gtk::StackTransitionType::None);
                if let Some(page) = entry {
                    state.page(&page);
                } else {
                    state.home();
                }
                state.stack.set_transition_type(transition);
            }
            state.sync_selection();
        }
    ));
    s.home();
    s
}

pub(super) fn localized_language(tag: &str) -> String {
    match tag {
        "off" => wordcraft_gnome::t!("Do not check spelling or grammar"),
        "mixed" => wordcraft_gnome::t!("Mixed"),
        "en-US" => wordcraft_gnome::t!("English (United States)"),
        "en-GB" => wordcraft_gnome::t!("English (United Kingdom)"),
        "es-ES" => wordcraft_gnome::t!("Spanish (Spain)"),
        "es-MX" => wordcraft_gnome::t!("Spanish (Mexico)"),
        "es-AR" => wordcraft_gnome::t!("Spanish (Argentina)"),
        "fr-FR" => wordcraft_gnome::t!("French"),
        "de-DE" => wordcraft_gnome::t!("German"),
        "it-IT" => wordcraft_gnome::t!("Italian"),
        "pt-BR" => wordcraft_gnome::t!("Portuguese (Brazil)"),
        "pl-PL" => wordcraft_gnome::t!("Polish"),
        "ru-RU" => wordcraft_gnome::t!("Russian"),
        "ca-ES" => wordcraft_gnome::t!("Catalan"),
        "eu-ES" => wordcraft_gnome::t!("Basque"),
        "gl-ES" => wordcraft_gnome::t!("Galician"),
        _ => tag,
    }
    .into()
}
impl TextTools {
    pub(super) fn reapply_language(&self) {
        if let Some(choice) = self.last_language.borrow().clone() {
            self.execute("leaf.proofText", serde_json::json!({"choice":choice}));
        }
    }
    pub(super) fn language_remembered(&self) -> bool {
        self.last_language.borrow().is_some()
    }
    fn language_title(ui: &Rc<Ui>, choice: &str) -> String {
        if choice == "default" {
            let effective = ui
                .current
                .borrow()
                .as_ref()
                .map(|d| wordcraft_gnome::proofing_defaults::effective_choice(&d.session))
                .unwrap_or_else(|| "mixed".into());
            wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Default — {language}"),
                &[("language", localized_language(&effective))],
            )
        } else if choice == "off" {
            localized_language(choice)
        } else {
            let native = TEXT_LANGUAGES
                .iter()
                .find(|(tag, _)| *tag == choice)
                .map(|(_, name)| *name)
                .unwrap_or(choice);
            let translated = localized_language(choice);
            let title = if native == translated {
                native.to_owned()
            } else {
                format!("{native} — {translated}")
            };
            if wordcraft_gnome::dictionaries::has_dictionary_files(choice) {
                title
            } else {
                format!(
                    "{title}\n{}",
                    wordcraft_gnome::t!("Dictionary not installed")
                )
            }
        }
    }
    fn sync_language_rows(&self, ui: &Rc<Ui>) {
        let current = ui
            .current
            .borrow()
            .as_ref()
            .map(|d| wordcraft_gnome::proofing_defaults::choice(&d.session));
        let mut rows = self.language_page.first_child();
        let Some(choices) = find_widget(&self.language_page, "language-choices") else {
            return;
        };
        while let Some(row) = rows {
            rows = row.next_sibling();
        }
        let mut child = choices.first_child();
        while let Some(row) = child {
            child = row.next_sibling();
            let Some(button) = row.downcast_ref::<gtk::Button>() else {
                continue;
            };
            let name = button.widget_name();
            let Some(choice) = name.strip_prefix("language-choice-") else {
                continue;
            };
            let selected = current.as_deref() == Some(choice);
            let title = Self::language_title(ui, choice);
            button.set_label(&if selected {
                format!("✓ {title}")
            } else {
                title.clone()
            });
            accessible_name(button, &title);
            button.update_state(&[gtk::accessible::State::Pressed(if selected {
                gtk::AccessibleTristate::True
            } else {
                gtk::AccessibleTristate::False
            })]);
            if selected {
                button.add_css_class("current-style");
            } else {
                button.remove_css_class("current-style");
            }
            if let Some(label) = button.child().and_downcast::<gtk::Label>() {
                label.set_wrap(true);
                label.set_max_width_chars(32);
                label.set_width_chars(1);
                label.set_xalign(0.);
            }
        }
        if let Some(label) = find_widget(&self.language_page, "language-mixed") {
            label.set_visible(current.as_deref() == Some("mixed"));
        }
    }
    fn build_languages(self: &Rc<Self>, ui: &Rc<Ui>) {
        if self.language_page.first_child().is_some() {
            return;
        }
        let search = gtk::SearchEntry::new();
        search.set_widget_name("language-search");
        search.set_placeholder_text(Some(wordcraft_gnome::t!("Search languages")));
        accessible_name(&search, wordcraft_gnome::t!("Search languages"));
        self.language_page.append(&search);
        let mixed = gtk::Label::new(Some(wordcraft_gnome::t!("Mixed")));
        mixed.set_widget_name("language-mixed");
        self.language_page.append(&mixed);
        let choices = gtk::Box::new(gtk::Orientation::Vertical, 2);
        choices.set_widget_name("language-choices");
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_min_content_height(90);
        let height = ui.host.upgrade().map_or(850, |h| h.window.height());
        scroll.set_max_content_height((height - 300).clamp(100, 300));
        scroll.set_propagate_natural_height(true);
        scroll.set_child(Some(&choices));
        self.language_page.append(&scroll);
        let mut tags = vec!["off".to_owned(), "default".to_owned()];
        tags.extend(TEXT_LANGUAGES.iter().map(|(tag, _)| tag.to_string()));
        for tag in wordcraft_gnome::dictionaries::additional_languages() {
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }
        for choice in tags {
            let button = style_menu_button(&Self::language_title(ui, &choice));
            button.set_widget_name(&format!("language-choice-{choice}"));
            if !matches!(choice.as_str(), "off" | "default")
                && !wordcraft_gnome::dictionaries::has_dictionary_files(&choice)
            {
                button.set_tooltip_text(Some(wordcraft_gnome::t!("Language can be assigned without a dictionary. Use Dictionary Settings to install proofing support.")));
            }
            choices.append(&button);
            if choice == "default" {
                choices.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            }
            let weak = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.execute("leaf.proofText", serde_json::json!({"choice":choice}));
                }
            });
        }
        let target = choices.downgrade();
        search.connect_search_changed(move |search| {
            let Some(choices) = target.upgrade() else {
                return;
            };
            let query = search.text().to_lowercase();
            let mut child = choices.first_child();
            let mut found = false;
            while let Some(row) = child {
                child = row.next_sibling();
                let name = row.widget_name();
                let Some(tag) = name.strip_prefix("language-choice-") else {
                    continue;
                };
                let native = TEXT_LANGUAGES
                    .iter()
                    .find(|(t, _)| *t == tag)
                    .map(|(_, n)| *n)
                    .unwrap_or(tag);
                let matches = matches!(tag, "off" | "default")
                    || format!("{native} {} {tag}", localized_language(tag))
                        .to_lowercase()
                        .contains(&query);
                row.set_visible(matches);
                found |= matches && !matches!(tag, "off" | "default");
            }
            if let Some(empty) = choices
                .last_child()
                .filter(|w| w.widget_name() == "language-no-results")
            {
                empty.set_visible(!found);
            }
        });
        let empty = gtk::Label::new(Some(wordcraft_gnome::t!("No Results")));
        empty.set_widget_name("language-no-results");
        empty.set_visible(false);
        choices.append(&empty);
        self.language_page
            .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        for (label, name, install) in [
            (
                wordcraft_gnome::t!("Dictionary Settings…"),
                "dictionary-settings",
                false,
            ),
            (
                wordcraft_gnome::t!("Install Additional Languages…"),
                "install-dictionaries",
                true,
            ),
        ] {
            let button = style_menu_button(label);
            button.set_widget_name(name);
            self.language_page.append(&button);
            let weak = Rc::downgrade(self);
            button.connect_clicked(move |_| {
                let Some(ui) = weak.upgrade().and_then(|s| s.ui.upgrade()) else {
                    return;
                };
                let Some(host) = ui.host.upgrade() else {
                    return;
                };
                if install {
                    import_dictionary_files(&ui, &host.window);
                } else {
                    show_dictionary_tools(&ui);
                }
            });
        }
    }
}
