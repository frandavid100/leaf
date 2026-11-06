use super::*;
use serde_json::json;

// Remembered choices are UI command preferences. Current document formatting is
// always read from WordCraft, and read-only synchronization never writes them.
pub(super) struct TextPalette {
    ui: std::rc::Weak<Ui>,
    pub(super) root: gtk::Box,
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
    syncing: Cell<bool>,
}
impl TextPalette {
    pub(super) fn new(ui: &Rc<Ui>, old: &gtk::Box) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 5);
        root.set_widget_name("text-basic-tools");
        for (button, title) in [
            (&ui.format_bold, wordcraft_gnome::t!("Bold")),
            (&ui.format_italic, wordcraft_gnome::t!("Italic")),
        ] {
            old.remove(button);
            button.set_label(Some(title));
            button.set_hexpand(true);
            root.append(button);
        }
        let superscript = gtk::CheckButton::with_label(wordcraft_gnome::t!("Superscript"));
        superscript.set_widget_name("text-superscript");
        root.append(&superscript);
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
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let open = style_menu_button(title);
            open.set_widget_name(name);
            open.set_hexpand(true);
            let apply = gtk::Button::from_icon_name("view-refresh-symbolic");
            apply.set_widget_name(&format!("{name}-apply"));
            apply.add_css_class("flat");
            apply.set_tooltip_text(Some(tip));
            accessible_name(&apply, tip);
            row.append(&open);
            row.append(&apply);
            root.append(&row);
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let value = gtk::Label::new(Some(title));
            value.set_xalign(0.);
            value.set_hexpand(true);
            value.set_ellipsize(gtk::pango::EllipsizeMode::End);
            value.set_max_width_chars(28);
            content.append(&value);
            content.append(&gtk::Image::from_icon_name("go-next-symbolic"));
            open.set_child(Some(&content));
            configs.push((open, apply, value));
        }
        let (ink, ink_apply, ink_value) = configs.remove(0);
        let (highlight, highlight_apply, highlight_value) = configs.remove(0);
        let s = Rc::new(Self {
            ui: Rc::downgrade(ui),
            root,
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
    pub(super) fn options_page(&self, name: &str) -> Option<gtk::Box> {
        self.panels.borrow().get(name).cloned()
    }
    fn apply(&self, command: &str, args: serde_json::Value) -> bool {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return false;
        };
        match format_execute(&ui, command, args.clone()) {
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
        if self.apply("format.highlight", json!({"color":value})) {
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
        let color = style_menu_button(wordcraft_gnome::t!("Underline Color…"));
        root.append(&color);
        let weak = Rc::downgrade(self);
        color.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.pick_color("format.underlineColor");
            }
        });
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
        let root = self.panel("ink");
        for (title, value) in [
            (wordcraft_gnome::t!("Automatic Color"), "auto"),
            (wordcraft_gnome::t!("Black"), "000000"),
            (wordcraft_gnome::t!("Red"), "E01B24"),
            (wordcraft_gnome::t!("Blue"), "1C71D8"),
            (wordcraft_gnome::t!("Green"), "26A269"),
            (wordcraft_gnome::t!("Purple"), "813D9C"),
        ] {
            let b = style_menu_button(title);
            b.set_widget_name(&format!("ink-choice-{value}"));
            root.append(&b);
            let weak = Rc::downgrade(self);
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if s.choose_ink(value) {
                        s.home();
                    }
                }
            });
        }
        let color = style_menu_button(wordcraft_gnome::t!("Choose Color…"));
        root.append(&color);
        let weak = Rc::downgrade(self);
        color.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.pick_color("format.color");
            }
        });
        let entry = gtk::Entry::new();
        entry.set_placeholder_text(Some("RGB: 2266AA"));
        accessible_name(&entry, wordcraft_gnome::t!("Hexadecimal RGB Color"));
        root.append(&entry);
        let weak = Rc::downgrade(self);
        entry.connect_changed(move |e| {
            if let Some(s) = weak.upgrade() {
                let value = e.text();
                let hex = value.trim().trim_start_matches('#');
                if hex.len() == 6 && hex.bytes().all(|c| c.is_ascii_hexdigit()) {
                    s.choose_ink(hex);
                    e.remove_css_class("error");
                } else if !hex.is_empty() {
                    e.add_css_class("error");
                }
            }
        });
        let root = self.panel("highlight");
        for (title, value) in [
            (wordcraft_gnome::t!("No Highlight"), "none"),
            (wordcraft_gnome::t!("Yellow"), "yellow"),
            (wordcraft_gnome::t!("Green"), "brightGreen"),
            (wordcraft_gnome::t!("Turquoise"), "turquoise"),
            (wordcraft_gnome::t!("Pink"), "pink"),
            (wordcraft_gnome::t!("Blue"), "blue"),
            (wordcraft_gnome::t!("Red"), "red"),
            (wordcraft_gnome::t!("Dark Blue"), "darkBlue"),
            (wordcraft_gnome::t!("Teal"), "teal"),
            (wordcraft_gnome::t!("Dark Green"), "green"),
            (wordcraft_gnome::t!("Purple"), "violet"),
            (wordcraft_gnome::t!("Dark Red"), "darkRed"),
            (wordcraft_gnome::t!("Dark Yellow"), "darkYellow"),
            (wordcraft_gnome::t!("50% Gray"), "gray50"),
            (wordcraft_gnome::t!("25% Gray"), "gray25"),
            (wordcraft_gnome::t!("Black"), "black"),
            (wordcraft_gnome::t!("White"), "white"),
        ] {
            let b = style_menu_button(title);
            b.set_widget_name(&format!("highlight-choice-{value}"));
            root.append(&b);
            let weak = Rc::downgrade(self);
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if s.choose_highlight(value) {
                        s.home();
                    }
                }
            });
        }
    }
    fn pick_color(self: &Rc<Self>, command: &'static str) {
        let Some(ui) = self.ui.upgrade() else { return };
        let Some(host) = ui.host.upgrade() else {
            return;
        };
        let identity = document_identity(&ui);
        let dialog = gtk::ColorDialog::new();
        dialog.set_with_alpha(false);
        dialog.set_title(wordcraft_gnome::t!("Choose Color"));
        let weak = Rc::downgrade(self);
        dialog.choose_rgba(
            Some(&host.window),
            None::<&gdk::RGBA>,
            None::<&gio::Cancellable>,
            move |result| {
                let Some(s) = weak.upgrade() else { return };
                let Some(ui) =
                    s.ui.upgrade()
                        .filter(|u| !u.closed.get() && document_identity(u) == identity)
                else {
                    return;
                };
                match result {
                    Ok(c) => {
                        let hex = format!(
                            "{:02X}{:02X}{:02X}",
                            (c.red() * 255.).round() as u8,
                            (c.green() * 255.).round() as u8,
                            (c.blue() * 255.).round() as u8
                        );
                        if command == "format.color" {
                            s.choose_ink(&hex);
                        } else {
                            s.apply(command, json!({"color":hex}));
                        }
                    }
                    Err(e)
                        if e.matches(gtk::DialogError::Cancelled)
                            || e.matches(gtk::DialogError::Dismissed) => {}
                    Err(e) => ui.toasts.add_toast(Toast::new(&e.to_string())),
                }
            },
        );
    }
    pub(super) fn sync(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        let state = selection_character(&ui);
        if *self.displayed.borrow() == state {
            return;
        }
        self.displayed.replace(state.clone());
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
            .any(|w| w[0].highlight != w[1].highlight)
        {
            wordcraft_gnome::t!("Mixed").into()
        } else if let Some(c) = resolved.first().and_then(|p| p.highlight) {
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
