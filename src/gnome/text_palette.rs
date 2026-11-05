use super::*;
use serde_json::json;

pub(super) struct TextPalette {
    ui: std::rc::Weak<Ui>,
    pub(super) root: gtk::Box,
    pub(super) superscript: gtk::CheckButton,
    pub(super) underline_check: gtk::CheckButton,
    pub(super) strike_check: gtk::CheckButton,
    pub(super) underline: gtk::MenuButton,
    pub(super) strike: gtk::MenuButton,
    pub(super) ink: adw::SplitButton,
    pub(super) highlight: adw::SplitButton,
    pub(super) last_underline: RefCell<String>,
    pub(super) last_strike: Cell<bool>,
    pub(super) last_ink: RefCell<String>,
    pub(super) last_highlight: RefCell<String>,
    syncing: Cell<bool>,
}
fn label(markup: &str) -> gtk::Label {
    let l = gtk::Label::new(None);
    l.set_markup(markup);
    l
}
fn split(markup: &str, name: &str, title: &str) -> adw::SplitButton {
    let b = adw::SplitButton::new();
    b.set_widget_name(name);
    b.set_child(Some(&label(markup)));
    b.set_hexpand(true);
    b.add_css_class("flat");
    b.set_tooltip_text(Some(title));
    b.set_dropdown_tooltip(title);
    accessible_name(&b, title);
    b
}
fn popup_content() -> (gtk::Popover, gtk::Box) {
    let p = gtk::Popover::new();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 4);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(360);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&root));
    p.set_child(Some(&scroll));
    (p, root)
}
impl TextPalette {
    pub(super) fn new(ui: &Rc<Ui>, old: &gtk::Box) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 4);
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
        let mut menus = Vec::new();
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
            check.set_widget_name(&format!("{name}-check"));
            check.set_hexpand(true);
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            row.append(check);
            let menu = gtk::MenuButton::new();
            menu.set_widget_name(name);
            menu.set_icon_name("go-next-symbolic");
            menu.add_css_class("flat");
            let options = wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Options for {label}"),
                &[("label", title.to_string())],
            );
            menu.set_tooltip_text(Some(&options));
            accessible_name(&menu, &options);
            row.append(&menu);
            root.append(&row);
            menus.push(menu);
        }
        let underline = menus.remove(0);
        let strike = menus.remove(0);
        let ink = split(
            wordcraft_gnome::t!("Text Color"),
            "text-ink",
            wordcraft_gnome::t!("Text Color"),
        );
        let highlight = split(
            wordcraft_gnome::t!("Highlight"),
            "text-highlight",
            wordcraft_gnome::t!("Highlight"),
        );
        root.append(&ink);
        root.append(&highlight);
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
            last_underline: RefCell::new("single".into()),
            last_strike: Cell::new(false),
            last_ink: RefCell::new("auto".into()),
            last_highlight: RefCell::new("yellow".into()),
            syncing: Cell::new(false),
        });
        let weak = Rc::downgrade(&s);
        s.superscript.connect_toggled(move |b| {
            if let Some(s) = weak.upgrade() {
                if !s.syncing.get() {
                    let on = b.is_active() || b.is_inconsistent();
                    b.set_inconsistent(false);
                    s.apply("format.superscript", json!({"value":on}));
                }
            }
        });
        let weak = Rc::downgrade(&s);
        s.underline_check.connect_toggled(move |button| {
            if let Some(s) = weak.upgrade() {
                if s.syncing.get() {
                    return;
                }
                let active = !button.is_active() && !button.is_inconsistent();
                button.set_inconsistent(false);
                s.apply(
                    "format.underline",
                    json!({"value":!active,"style":*s.last_underline.borrow()}),
                );
            }
        });
        let weak = Rc::downgrade(&s);
        s.strike_check.connect_toggled(move |button| {
            if let Some(palette) = weak.upgrade() {
                if palette.syncing.get() { return; }
                let active = !button.is_active() && !button.is_inconsistent();
                button.set_inconsistent(false);
                let double = palette.last_strike.get();
                palette.apply(
                    "format.set",
                    json!({"props": {"strike": !active && !double, "doubleStrike": !active && double}}),
                );
            }
        });
        let weak = Rc::downgrade(&s);
        s.ink.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.apply("format.color", json!({"color":*s.last_ink.borrow()}));
            }
        });
        let weak = Rc::downgrade(&s);
        s.highlight.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.apply(
                    "format.highlight",
                    json!({"color":*s.last_highlight.borrow()}),
                );
            }
        });
        s.options();
        s.swatch(false);
        s.swatch(true);
        s.sync();
        s
    }
    fn state(&self) -> serde_json::Value {
        self.ui
            .upgrade()
            .map(|u| selection_character(&u))
            .unwrap_or_default()
    }
    fn apply(&self, command: &str, args: serde_json::Value) -> bool {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return false;
        };
        match format_execute(&ui, command, args) {
            Ok(()) => true,
            Err(e) => {
                ui.toasts.add_toast(Toast::new(&e));
                false
            }
        }
    }
    pub(super) fn choose_ink(&self, value: &str) -> bool {
        if self.apply("format.color", json!({"color":value})) {
            self.last_ink.replace(value.into());
            if let Some(w) = self.ink.child().and_then(|w| w.last_child()) {
                w.queue_draw();
            }
            true
        } else {
            false
        }
    }
    pub(super) fn choose_highlight(&self, value: &str) -> bool {
        if self.apply("format.highlight", json!({"color":value})) {
            self.last_highlight.replace(value.into());
            if let Some(w) = self.highlight.child().and_then(|w| w.last_child()) {
                w.queue_draw();
            }
            true
        } else {
            false
        }
    }
    fn options(self: &Rc<Self>) {
        let (pop, root) = popup_content();
        self.underline.set_popover(Some(&pop));
        for (name, style) in [
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
            let b = style_menu_button(name);
            b.set_widget_name(&format!("underline-choice-{style}"));
            root.append(&b);
            let weak = Rc::downgrade(self);
            let p = pop.downgrade();
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if s.apply("format.underline", json!({"style":style,"value":true})) {
                        s.last_underline.replace(style.into());
                    }
                    if let Some(p) = p.upgrade() {
                        p.popdown();
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
        let (pop, root) = popup_content();
        self.strike.set_popover(Some(&pop));
        for (name, double) in [
            (wordcraft_gnome::t!("Single"), false),
            (wordcraft_gnome::t!("Double"), true),
        ] {
            let b = style_menu_button(name);
            b.set_widget_name(if double {
                "strike-double"
            } else {
                "strike-single"
            });
            root.append(&b);
            let weak = Rc::downgrade(self);
            let p = pop.downgrade();
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if s.apply(
                        "format.set",
                        json!({"props":{"strike":!double,"doubleStrike":double}}),
                    ) {
                        s.last_strike.set(double);
                    }
                    if let Some(p) = p.upgrade() {
                        p.popdown();
                    }
                }
            });
        }
        let (pop, root) = popup_content();
        self.ink.set_popover(Some(&pop));
        for (name, value) in [
            (wordcraft_gnome::t!("Automatic Color"), "auto"),
            (wordcraft_gnome::t!("Black"), "000000"),
            (wordcraft_gnome::t!("Red"), "E01B24"),
            (wordcraft_gnome::t!("Blue"), "1C71D8"),
            (wordcraft_gnome::t!("Green"), "26A269"),
            (wordcraft_gnome::t!("Purple"), "813D9C"),
        ] {
            let b = style_menu_button(name);
            b.set_widget_name(&format!("ink-choice-{value}"));
            root.append(&b);
            let weak = Rc::downgrade(self);
            let p = pop.downgrade();
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.choose_ink(value);
                    if let Some(p) = p.upgrade() {
                        p.popdown();
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
        entry.connect_changed(move |entry| {
            if let Some(s) = weak.upgrade() {
                let value = entry.text();
                let hex = value.trim().trim_start_matches('#');
                if hex.len() == 6 && hex.bytes().all(|c| c.is_ascii_hexdigit()) {
                    s.choose_ink(hex);
                    entry.remove_css_class("error");
                } else if !hex.is_empty() {
                    entry.add_css_class("error");
                }
            }
        });
        let (pop, root) = popup_content();
        self.highlight.set_popover(Some(&pop));
        for (name, value) in [
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
            let b = style_menu_button(name);
            b.set_widget_name(&format!("highlight-choice-{value}"));
            root.append(&b);
            let weak = Rc::downgrade(self);
            let p = pop.downgrade();
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.choose_highlight(value);
                    if let Some(p) = p.upgrade() {
                        p.popdown();
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
                            (c.red() * 255.0).round() as u8,
                            (c.green() * 255.0).round() as u8,
                            (c.blue() * 255.0).round() as u8
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
    fn swatch(self: &Rc<Self>, highlight: bool) {
        let b = if highlight {
            &self.highlight
        } else {
            &self.ink
        };
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let title = gtk::Label::new(Some(if highlight {
            wordcraft_gnome::t!("Highlight")
        } else {
            wordcraft_gnome::t!("Text Color")
        }));
        title.set_xalign(0.0);
        title.set_hexpand(true);
        content.append(&title);
        let a = gtk::DrawingArea::new();
        a.set_content_width(22);
        a.set_content_height(3);
        a.set_halign(gtk::Align::Center);
        content.append(&a);
        b.set_child(Some(&content));
        let weak = Rc::downgrade(self);
        a.set_draw_func(move |w, cr, width, height| {
            if let Some(s) = weak.upgrade() {
                let value = if highlight {
                    s.last_highlight.borrow().clone()
                } else {
                    s.last_ink.borrow().clone()
                };
                let rgb = if highlight {
                    serde_json::from_value::<wordcraft_doc::props::Highlight>(json!(value))
                        .ok()
                        .and_then(|h| h.rgb())
                } else {
                    wordcraft_doc::props::Rgb::parse(&value)
                };
                if let Some(c) = rgb {
                    cr.set_source_rgb(c.0 as f64 / 255.0, c.1 as f64 / 255.0, c.2 as f64 / 255.0);
                } else {
                    let c = w.color();
                    cr.set_source_rgba(c.red() as f64, c.green() as f64, c.blue() as f64, 1.0);
                }
                cr.rectangle(0.0, 0.0, width as f64, height as f64);
                let _ = cr.fill();
            }
        });
    }
    pub(super) fn sync(&self) {
        self.syncing.set(true);
        let state = self.state();
        self.superscript
            .set_active(state["vertAlign"].as_str() == Some("superscript"));
        self.superscript
            .set_inconsistent(state.get("vertAlign").is_some_and(|v| v.is_null()));
        for (button, mixed, active) in [
            (
                &self.underline_check,
                state.get("underline").is_some_and(|v| v.is_null()),
                state["underline"].as_str().is_some_and(|v| v != "none"),
            ),
            (
                &self.strike_check,
                ["strike", "doubleStrike"]
                    .iter()
                    .any(|k| state.get(k).is_some_and(|v| v.is_null())),
                state["strike"] == true || state["doubleStrike"] == true,
            ),
        ] {
            button.set_active(active);
            button.set_inconsistent(mixed);
        }
        self.syncing.set(false);
    }
}
