use super::*;
use serde_json::json;

// Remembered choices are UI command preferences. Current document formatting is
// always read from WordCraft, and read-only synchronization never writes them.
pub(super) struct TextPalette {
    ui: std::rc::Weak<Ui>,
    pub(super) root: gtk::Box,
    pub(super) properties: gtk::Box,
    pub(super) superscript: gtk::CheckButton,
    pub(super) underline_check: gtk::CheckButton,
    pub(super) strike_check: gtk::CheckButton,
    pub(super) underline: gtk::Button,
    pub(super) strike: gtk::Button,
    pub(super) ink: gtk::Button,
    pub(super) highlight: gtk::Button,
    pub(super) ink_apply: gtk::Button,
    pub(super) highlight_apply: gtk::Button,
    ink_value: gtk::Label,
    highlight_value: gtk::Label,
    displayed: RefCell<serde_json::Value>,
    pub(super) last_underline: RefCell<String>,
    last_underline_color: RefCell<Option<String>>,
    pub(super) last_strike: Cell<bool>,
    pub(super) last_ink: RefCell<String>,
    pub(super) last_highlight: RefCell<String>,
    panels: RefCell<std::collections::HashMap<&'static str, gtk::Box>>,
    pickers: RefCell<std::collections::HashMap<&'static str, Rc<InlineColor>>>,
    syncing: Cell<bool>,
}
impl TextPalette {
    pub(super) fn new(ui: &Rc<Ui>, old: &gtk::Box) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 5);
        let properties = gtk::Box::new(gtk::Orientation::Vertical, 4);
        root.set_widget_name("text-basic-tools");
        for (button, title) in [
            (&ui.format_bold, wordcraft_gnome::t!("Bold")),
            (&ui.format_italic, wordcraft_gnome::t!("Italic")),
        ] {
            old.remove(button);
            button.set_label(Some(title));
            button.set_hexpand(true);
            check_icon(
                button,
                title,
                if button == &ui.format_bold {
                    "format-text-bold-symbolic"
                } else {
                    "format-text-italic-symbolic"
                },
            );
            root.append(button);
        }
        let superscript = gtk::CheckButton::with_label(wordcraft_gnome::t!("Superscript"));
        superscript.set_widget_name("text-superscript");
        check_icon(
            &superscript,
            wordcraft_gnome::t!("Superscript"),
            "format-text-rich-symbolic",
        );
        let underline_check = gtk::CheckButton::with_label(wordcraft_gnome::t!("Underline"));
        let strike_check = gtk::CheckButton::with_label(wordcraft_gnome::t!("Strikethrough"));
        let mut buttons = Vec::new();
        for (check, name, title) in [
            (
                &underline_check,
                "text-underline",
                wordcraft_gnome::t!("Underline"),
            ),
            (
                &strike_check,
                "text-strike",
                wordcraft_gnome::t!("Strikethrough"),
            ),
        ] {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            check.set_widget_name(&format!("{name}-check"));
            check.set_hexpand(true);
            check_icon(
                check,
                title,
                if name == "text-underline" {
                    "format-text-underline-symbolic"
                } else {
                    "format-text-strikethrough-symbolic"
                },
            );
            row.append(check);
            let button = gtk::Button::from_icon_name("go-next-symbolic");
            button.add_css_class("flat");
            button.set_widget_name(name);
            let label = wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Options for {label}"),
                &[("label", title.into())],
            );
            button.set_tooltip_text(Some(&label));
            accessible_name(&button, &label);
            row.append(&button);
            root.append(&row);
            buttons.push(button);
        }
        let underline = buttons.remove(0);
        let strike = buttons.remove(0);
        root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let mut configs = Vec::new();
        for (title, name, tip) in [
            (
                wordcraft_gnome::t!("Text Color"),
                "text-ink",
                wordcraft_gnome::t!("Apply Last Text Color"),
            ),
            (
                wordcraft_gnome::t!("Highlight"),
                "text-highlight",
                wordcraft_gnome::t!("Apply Last Highlight"),
            ),
        ] {
            let open = style_menu_button(title);
            open.remove_css_class("flat");
            open.set_widget_name(name);
            open.set_hexpand(true);
            let apply = gtk::Button::from_icon_name("view-refresh-symbolic");
            apply.set_widget_name(&format!("{name}-apply"));
            // Native linked buttons share one continuous background.
            apply.set_tooltip_text(Some(tip));
            accessible_name(&apply, tip);
            // Assemble after setting the property content below.
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            content.append(&gtk::Image::from_icon_name(if name == "text-ink" {
                "font-select-symbolic"
            } else {
                "color-select-symbolic"
            }));
            let value = gtk::Label::new(Some(title));
            value.set_xalign(0.);
            value.set_hexpand(true);
            value.set_ellipsize(gtk::pango::EllipsizeMode::End);
            value.set_max_width_chars(28);
            content.append(&value);
            // Lower property rows have no navigation arrow.
            open.set_child(Some(&content));
            properties.append(&property_split(&open, &apply));
            configs.push((open, apply, value));
        }
        let (ink, ink_apply, ink_value) = configs.remove(0);
        let (highlight, highlight_apply, highlight_value) = configs.remove(0);
        let s = Rc::new(Self {
            ui: Rc::downgrade(ui),
            root,
            properties,
            superscript,
            underline_check,
            strike_check,
            underline,
            strike,
            ink,
            highlight,
            ink_apply,
            highlight_apply,
            ink_value,
            highlight_value,
            displayed: RefCell::new(serde_json::Value::Null),
            last_underline: RefCell::new("single".into()),
            last_underline_color: RefCell::new(None),
            last_strike: Cell::new(false),
            last_ink: RefCell::new("auto".into()),
            last_highlight: RefCell::new("yellow".into()),
            panels: RefCell::new(std::collections::HashMap::new()),
            pickers: RefCell::new(std::collections::HashMap::new()),
            syncing: Cell::new(false),
        });
        let weak = Rc::downgrade(&s);
        s.superscript.connect_toggled(move |b| {
            if let Some(s) = weak.upgrade().filter(|s| !s.syncing.get()) {
                let on = b.is_active() || b.is_inconsistent();
                b.set_inconsistent(false);
                s.apply("format.superscript", json!({"value":on}));
            }
        });
        let weak = Rc::downgrade(&s);
        s.underline_check.connect_toggled(move |b| {
            if let Some(s) = weak.upgrade().filter(|s| !s.syncing.get()) {
                let on = b.is_active() || b.is_inconsistent();
                b.set_inconsistent(false);
                let style = s.last_underline.borrow().clone();
                s.apply_underline(on, &style);
            }
        });
        let weak = Rc::downgrade(&s);
        s.strike_check.connect_toggled(move |b| {
            if let Some(s) = weak.upgrade().filter(|s| !s.syncing.get()) {
                let on = b.is_active() || b.is_inconsistent();
                b.set_inconsistent(false);
                let double = s.last_strike.get();
                s.apply(
                    "format.set",
                    json!({"props":{"strike":on&&!double,"doubleStrike":on&&double}}),
                );
            }
        });
        for (button, page) in [
            (&s.underline, "underline"),
            (&s.strike, "strike"),
            (&s.ink, "ink"),
            (&s.highlight, "highlight"),
        ] {
            let weak = Rc::downgrade(&s);
            button.connect_clicked(move |_| {
                if let Some(ui) = weak.upgrade().and_then(|s| s.ui.upgrade()) {
                    if let Some(tools) = ui.text_tools.borrow().as_ref() {
                        tools.page(page);
                    }
                }
            });
        }
        let weak = Rc::downgrade(&s);
        s.ink_apply.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                let value = s.last_ink.borrow().clone();
                s.choose_ink(&value);
            }
        });
        let weak = Rc::downgrade(&s);
        s.highlight_apply.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                let value = s.last_highlight.borrow().clone();
                s.choose_highlight(&value);
            }
        });
        for b in [
            &ui.format_bold,
            &ui.format_italic,
            &s.superscript,
            &s.underline_check,
            &s.strike_check,
        ] {
            if let Some(label) = b.last_child().and_then(|w| w.downcast::<gtk::Label>().ok()) {
                label.set_hexpand(true);
                label.set_xalign(0.);
            }
        }
        s.options();
        s.sync();
        s
    }
    pub(super) fn options_page(self: &Rc<Self>, name: &str) -> Option<gtk::Box> {
        if matches!(name, "ink" | "highlight") {
            self.color_page(if name == "ink" { "ink" } else { "highlight" });
        }
        self.panels.borrow().get(name).cloned()
    }
    fn apply(&self, command: &str, args: serde_json::Value) -> bool {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return false;
        };
        let result = if matches!(
            command,
            "format.color" | "format.underlineColor" | "leaf.highlight"
        ) {
            immediate_format(&ui, command, args.clone())
        } else {
            format_execute(&ui, command, args.clone())
        };
        match result {
            Ok(()) => {
                if command == "format.underlineColor" {
                    self.last_underline_color
                        .replace(args["color"].as_str().map(str::to_owned));
                }
                true
            }
            Err(e) => {
                ui.toasts.add_toast(Toast::new(&e));
                false
            }
        }
    }
    fn apply_underline(&self, on: bool, style: &str) -> bool {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return false;
        };
        let color = self.last_underline_color.borrow().clone();
        tools_change(&ui, |s| {
            let original = s.edit_snapshot();
            let depth = s.undo_depth();
            let result = (|| {
                s.run("format.underline", &json!({"value":on,"style":style}))
                    .map_err(|e| e.to_string())?;
                if on {
                    if let Some(color) = color {
                        if s.undo_depth() > depth {
                            s.join_next_undo();
                        }
                        s.run("format.underlineColor", &json!({"color":color}))
                            .map_err(|e| e.to_string())?;
                    }
                }
                Ok(())
            })();
            if result.is_err() {
                s.restore(original);
            }
            result
        })
    }
    pub(super) fn choose_ink(&self, value: &str) -> bool {
        if self.apply("format.color", json!({"color":value})) {
            self.last_ink.replace(value.into());
            true
        } else {
            false
        }
    }
    pub(super) fn choose_highlight(&self, value: &str) -> bool {
        if self.apply("leaf.highlight", json!({"color":value})) {
            if value != "none" {
                self.last_highlight.replace(value.into());
            }
            true
        } else {
            false
        }
    }
    fn home(&self) {
        if let Some(ui) = self.ui.upgrade() {
            if let Some(tools) = ui.text_tools.borrow().as_ref() {
                tools.home();
            }
        }
    }
    fn panel(&self, name: &'static str) -> gtk::Box {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 4);
        self.panels.borrow_mut().insert(name, root.clone());
        root
    }
    fn options(self: &Rc<Self>) {
        let root = self.panel("underline");
        for (title, style) in [
            (wordcraft_gnome::t!("Single"), "single"),
            (wordcraft_gnome::t!("Double"), "double"),
            (wordcraft_gnome::t!("Thick"), "thick"),
            (wordcraft_gnome::t!("Dotted"), "dotted"),
            (wordcraft_gnome::t!("Dashed"), "dash"),
            (wordcraft_gnome::t!("Wavy"), "wave"),
            (wordcraft_gnome::t!("Dot Dash"), "dotDash"),
            (wordcraft_gnome::t!("Dot Dot Dash"), "dotDotDash"),
            (wordcraft_gnome::t!("Words Only"), "words"),
        ] {
            let b = style_menu_button(title);
            b.set_widget_name(&format!("underline-choice-{style}"));
            root.append(&b);
            let weak = Rc::downgrade(self);
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if s.apply_underline(true, style) {
                        s.last_underline.replace(style.into());
                        s.home();
                    }
                }
            });
        }
        let weak = Rc::downgrade(self);
        let picker = embedded_color_chooser("text-underline-chooser", None, move |hex| {
            weak.upgrade()
                .is_some_and(|s| s.apply("format.underlineColor", json!({"color":hex})))
        });
        root.append(&picker.widget);
        self.pickers.borrow_mut().insert("underline", picker);
        let auto = style_menu_button(wordcraft_gnome::t!("Automatic Color"));
        root.append(&auto);
        let weak = Rc::downgrade(self);
        auto.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.apply("format.underlineColor", json!({"color":"auto"}));
            }
        });
        let root = self.panel("strike");
        for (title, double) in [
            (wordcraft_gnome::t!("Single"), false),
            (wordcraft_gnome::t!("Double"), true),
        ] {
            let b = style_menu_button(title);
            b.set_widget_name(if double {
                "strike-double"
            } else {
                "strike-single"
            });
            root.append(&b);
            let weak = Rc::downgrade(self);
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if s.apply(
                        "format.set",
                        json!({"props":{"strike":!double,"doubleStrike":double}}),
                    ) {
                        s.last_strike.set(double);
                        s.home();
                    }
                }
            });
        }
        self.panel("ink");
        self.panel("highlight");
    }
    fn color_page(self: &Rc<Self>, name: &'static str) {
        let root = self.panels.borrow().get(name).cloned().unwrap();
        clear_box(&root);
        let clear = style_menu_button(if name == "ink" {
            wordcraft_gnome::t!("Automatic Color")
        } else {
            wordcraft_gnome::t!("No Highlight")
        });
        clear.set_widget_name(if name == "ink" {
            "ink-choice-auto"
        } else {
            "highlight-choice-none"
        });
        root.append(&clear);
        let weak = Rc::downgrade(self);
        clear.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                if name == "ink" {
                    s.choose_ink("auto");
                } else {
                    s.choose_highlight("none");
                }
            }
        });
        let weak = Rc::downgrade(self);
        let picker = embedded_color_chooser(&format!("text-{name}-chooser"), None, move |hex| {
            weak.upgrade().is_some_and(|s| {
                if name == "ink" {
                    s.choose_ink(hex)
                } else {
                    s.choose_highlight(hex)
                }
            })
        });
        root.append(&picker.widget);
        self.pickers.borrow_mut().insert(name, picker);
        self.sync_pickers();
    }
    fn sync_pickers(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        let current = ui.current.borrow();
        let Some(d) = current.as_ref() else { return };
        let resolved = wordcraft_gnome::text_properties::selected_resolved(&d.session);
        let Some(p) = resolved.first() else { return };
        let ink = match p.color {
            wordcraft_doc::props::TextColor::Rgb(c) => Some(c),
            _ => None,
        };
        let background = p.highlight.or(p.shading);
        for (name, color) in [
            ("ink", ink),
            ("highlight", background),
            ("underline", p.underline_color),
        ] {
            if let Some(picker) = self.pickers.borrow().get(name) {
                let hex = color.map(|c| format!("#{}", c.hex()));
                picker.sync(hex.as_deref());
            }
        }
    }

    pub(super) fn sync(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        let state = selection_character(&ui);
        if *self.displayed.borrow() == state {
            return;
        }
        self.displayed.replace(state.clone());
        self.sync_pickers();
        self.syncing.set(true);
        for (button, key, on) in [
            (
                &self.superscript,
                "vertAlign",
                state["vertAlign"] == "superscript",
            ),
            (
                &self.underline_check,
                "underline",
                state["underline"].as_str().is_some_and(|v| v != "none"),
            ),
            (
                &self.strike_check,
                "strike",
                state["strike"] == true || state["doubleStrike"] == true,
            ),
        ] {
            button.set_active(on);
            button.set_inconsistent(
                state.get(key).is_some_and(|v| v.is_null())
                    || (key == "strike" && state["doubleStrike"].is_null()),
            );
        }
        let color = if state["color"].is_null() {
            wordcraft_gnome::t!("Mixed").into()
        } else if let Some(rgb) = state["color"]["Rgb"].as_array() {
            format!(
                "#{:02X}{:02X}{:02X}",
                rgb[0].as_u64().unwrap_or(0),
                rgb[1].as_u64().unwrap_or(0),
                rgb[2].as_u64().unwrap_or(0)
            )
        } else {
            wordcraft_gnome::t!("Automatic Color").into()
        };
        let current = ui.current.borrow();
        let resolved = current
            .as_ref()
            .map(|d| wordcraft_gnome::text_properties::selected_resolved(&d.session))
            .unwrap_or_default();
        let highlight = if resolved
            .windows(2)
            .any(|w| w[0].highlight.or(w[0].shading) != w[1].highlight.or(w[1].shading))
        {
            wordcraft_gnome::t!("Mixed").into()
        } else if let Some(c) = resolved.first().and_then(|p| p.highlight.or(p.shading)) {
            format!("#{:02X}{:02X}{:02X}", c.0, c.1, c.2)
        } else {
            wordcraft_gnome::t!("No Highlight").into()
        };
        for (button, text, title, value) in [
            (
                &self.ink,
                &self.ink_value,
                wordcraft_gnome::t!("Text Color"),
                color,
            ),
            (
                &self.highlight,
                &self.highlight_value,
                wordcraft_gnome::t!("Highlight"),
                highlight,
            ),
        ] {
            let label = wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("{label}: {value}"),
                &[("label", title.into()), ("value", value)],
            );
            if text.text() != label {
                text.set_text(&label);
            }
            button.set_tooltip_text(Some(&label));
            accessible_name(button, &label);
        }
        self.syncing.set(false);
    }
}
