use super::*;
use serde_json::json;

// Remembered choices are UI command preferences. Current document formatting is
// always read from WordCraft, and read-only synchronization never writes them.
pub(super) struct TextPalette {
    ui: std::rc::Weak<Ui>,
    pub(super) root: gtk::Box,
    pub(super) properties: gtk::Box,
    pub(super) superscript: gtk::CheckButton,
    position: gtk::Button,
    words_only: gtk::CheckButton,
    underline_color: gtk::Button,
    choices: RefCell<Vec<(String, String, gtk::Button)>>,
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
            button.set_widget_name(if button == &ui.format_bold {
                "format-row-text-bold"
            } else {
                "format-row-text-italic"
            });
            root.append(button);
        }
        let superscript = gtk::CheckButton::with_label(wordcraft_gnome::t!("Superscript"));
        superscript.set_widget_name("text-superscript");
        check_icon(
            &superscript,
            wordcraft_gnome::t!("Superscript"),
            "format-text-rich-symbolic",
        );
        let superscript_row = gtk::Box::new(gtk::Orientation::Vertical, 0);
        superscript_row.set_widget_name("format-row-text-superscript");
        let position = style_menu_button(wordcraft_gnome::t!("Text Position"));
        position.set_widget_name("text-position");
        superscript_row.append(&position);
        root.append(&superscript_row);
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
            row.set_widget_name(if name == "text-underline" {
                "format-row-text-underline"
            } else {
                "format-row-text-strike"
            });
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
            open.set_widget_name(name);
            let row = gtk::Box::new(gtk::Orientation::Vertical, 0);
            row.set_widget_name(if name == "text-ink" {
                "format-row-text-ink"
            } else {
                "format-row-text-highlight"
            });
            row.append(&reapply_split(&open, &apply));
            properties.append(&row);
            configs.push((open, apply, value));
        }
        let (ink, ink_apply, ink_value) = configs.remove(0);
        let (highlight, highlight_apply, highlight_value) = configs.remove(0);
        let s = Rc::new(Self {
            ui: Rc::downgrade(ui),
            root,
            properties,
            superscript,
            position,
            words_only: gtk::CheckButton::with_label(wordcraft_gnome::t!("Words Only")),
            underline_color: style_menu_button(wordcraft_gnome::t!("Automatic Color")),
            choices: RefCell::new(Vec::new()),
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
            (&s.position, "superscript"),
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
        if matches!(name, "ink" | "highlight" | "underline-color") {
            self.color_page(match name {
                "ink" => "ink",
                "highlight" => "highlight",
                _ => "underline-color",
            });
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
            self.refresh_last();
            true
        } else {
            false
        }
    }
    pub(super) fn choose_highlight(&self, value: &str) -> bool {
        if self.apply("leaf.highlight", json!({"color":value})) {
            self.last_highlight.replace(value.into());
            self.refresh_last();
            true
        } else {
            false
        }
    }
    fn refresh_last(&self) {
        self.displayed.replace(serde_json::Value::Null);
        self.sync();
        if let Some(ui) = self.ui.upgrade() {
            if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                tools.sync(&ui, &selection_character(&ui));
            }
        }
    }
    pub(super) fn reapply_color(&self, ink: bool) {
        let value = if ink {
            self.last_ink.borrow().clone()
        } else {
            self.last_highlight.borrow().clone()
        };
        if ink {
            self.choose_ink(&value);
        } else {
            self.choose_highlight(&value);
        }
    }
    fn home(&self) {
        if let Some(ui) = self.ui.upgrade() {
            if let Some(tools) = ui.text_tools.borrow().as_ref() {
                if !ui.format_group.pinned.get() {
                    if let Some(pop) = tools.popover.upgrade() {
                        close_format_popover(&pop);
                    }
                    ui.document_box.grab_focus();
                }
            }
        }
    }
    fn panel(&self, name: &'static str) -> gtk::Box {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 4);
        self.panels.borrow_mut().insert(name, root.clone());
        root
    }
    fn choice_row(
        self: &Rc<Self>,
        root: &gtk::Box,
        group: &str,
        value: &str,
        title: &str,
        name: &str,
    ) -> gtk::Button {
        let button = decoration_row(
            title,
            name,
            if group == "strike" {
                if value == "double" {
                    "strike-double"
                } else if value == "single" {
                    "strike-single"
                } else {
                    "none"
                }
            } else {
                value
            },
        );
        root.append(&button);
        self.choices
            .borrow_mut()
            .push((group.into(), value.into(), button.clone()));
        button
    }
    fn options(self: &Rc<Self>) {
        let root = self.panel("underline");
        for (title, style) in [
            (wordcraft_gnome::t!("No underline"), "none"),
            (wordcraft_gnome::t!("Single"), "single"),
            (wordcraft_gnome::t!("Double"), "double"),
            (wordcraft_gnome::t!("Thick"), "thick"),
            (wordcraft_gnome::t!("Dotted"), "dotted"),
            (wordcraft_gnome::t!("Dashed"), "dash"),
            (wordcraft_gnome::t!("Wavy"), "wave"),
            (wordcraft_gnome::t!("Dot Dash"), "dotDash"),
            (wordcraft_gnome::t!("Dot Dot Dash"), "dotDotDash"),
        ] {
            let b = self.choice_row(
                &root,
                "underline",
                style,
                title,
                &format!("underline-choice-{style}"),
            );
            let weak = Rc::downgrade(self);
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    // A style choice preserves this target's independent color.
                    if s.apply(
                        "format.underline",
                        json!({"value":style != "none", "style":style}),
                    ) {
                        if style != "none" {
                            s.last_underline.replace(style.into());
                        }
                        s.home();
                    }
                }
            });
        }
        root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        self.words_only.set_widget_name("underline-words-only");
        self.words_only.set_tooltip_text(Some(wordcraft_gnome::t!("WordCraft supports words-only with a single underline; other line styles cannot yet combine with it.")));
        root.append(&self.words_only);
        let weak = Rc::downgrade(self);
        self.words_only.connect_toggled(move |check| {
            if let Some(s) = weak.upgrade().filter(|s| !s.syncing.get()) {
                s.apply(
                    "format.underline",
                    json!({"value":true,"style":if check.is_active(){"words"}else{"single"}}),
                );
            }
        });
        self.underline_color
            .set_widget_name("underline-color-property");
        root.append(&self.underline_color);
        let weak = Rc::downgrade(self);
        self.underline_color.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade().and_then(|s| s.ui.upgrade()) {
                if let Some(tools) = ui.text_tools.borrow().as_ref() {
                    tools.page("underline-color");
                }
            }
        });
        self.panel("underline-color");
        let root = self.panel("strike");
        for (title, value) in [
            (wordcraft_gnome::t!("No strikethrough"), "none"),
            (wordcraft_gnome::t!("Single"), "single"),
            (wordcraft_gnome::t!("Double"), "double"),
        ] {
            let b = self.choice_row(&root, "strike", value, title, &format!("strike-{value}"));
            let weak = Rc::downgrade(self);
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if s.apply(
                        "format.set",
                        json!({"props":{"strike":value=="single","doubleStrike":value=="double"}}),
                    ) {
                        if value != "none" {
                            s.last_strike.set(value == "double");
                        }
                        s.home();
                    }
                }
            });
        }
        let root = self.panel("superscript");
        for (title, value) in [
            (wordcraft_gnome::t!("Normal"), "baseline"),
            (wordcraft_gnome::t!("Superscript"), "superscript"),
            (wordcraft_gnome::t!("Subscript"), "subscript"),
        ] {
            let b = self.choice_row(
                &root,
                "script",
                value,
                title,
                &format!("script-choice-{value}"),
            );
            let weak = Rc::downgrade(self);
            b.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    if s.apply("format.set", json!({"props":{"vertAlign":value}})) {
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
        if self.pickers.borrow().contains_key(name) {
            self.sync_pickers();
            return;
        }
        clear_box(&root);
        let clear = style_menu_button(if name != "highlight" {
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
                } else if name == "underline-color" {
                    s.apply("format.underlineColor", json!({"color":"auto"}));
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
                } else if name == "underline-color" {
                    s.apply("format.underlineColor", json!({"color":hex}))
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
            ("underline-color", p.underline_color),
        ] {
            if let Some(picker) = self.pickers.borrow().get(name) {
                let hex = color.map(|c| format!("#{}", c.hex()));
                let mixed = resolved.iter().skip(1).any(|r| match name {
                    "ink" => r.color != p.color,
                    "highlight" => r.highlight.or(r.shading) != background,
                    _ => r.underline_color != p.underline_color,
                });
                picker.sync_state(hex.as_deref(), mixed);
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
        let underline = state["underline"].as_str();
        self.words_only.set_inconsistent(underline.is_none());
        self.words_only.set_active(underline == Some("words"));
        self.words_only
            .set_sensitive(matches!(underline, Some("none" | "single" | "words")));
        let underline_color = state["underlineColor"]
            .as_array()
            .map(|rgb| {
                format!(
                    "#{:02X}{:02X}{:02X}",
                    rgb[0].as_u64().unwrap_or(0),
                    rgb[1].as_u64().unwrap_or(0),
                    rgb[2].as_u64().unwrap_or(0)
                )
            })
            .unwrap_or_else(|| wordcraft_gnome::t!("Automatic Color").into());
        self.underline_color.set_label(&format!(
            "{}: {}",
            wordcraft_gnome::t!("Color"),
            underline_color
        ));
        for (group, value, button) in self.choices.borrow().iter() {
            let current = match group.as_str() {
                "underline" => underline.map(|s| if s == "words" { "single" } else { s }),
                "script" => state["vertAlign"].as_str(),
                _ => {
                    if state["strike"].is_null() || state["doubleStrike"].is_null() {
                        None
                    } else if state["doubleStrike"] == true {
                        Some("double")
                    } else if state["strike"] == true {
                        Some("single")
                    } else {
                        Some("none")
                    }
                }
            };
            let selected = current == Some(value.as_str());
            if selected {
                button.add_css_class("current-style");
            } else {
                button.remove_css_class("current-style");
            }
            button.update_state(&[gtk::accessible::State::Pressed(if selected {
                gtk::AccessibleTristate::True
            } else {
                gtk::AccessibleTristate::False
            })]);
            if let Some(mark) = button.child().and_then(|c| c.last_child()) {
                mark.set_visible(selected);
            }
        }
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
        for (button, text, title, value) in [
            (
                &self.ink,
                &self.ink_value,
                wordcraft_gnome::t!("Text Color"),
                if self.last_ink.borrow().as_str() == "auto" {
                    wordcraft_gnome::t!("Automatic Color").into()
                } else {
                    self.last_ink.borrow().clone()
                },
            ),
            (
                &self.highlight,
                &self.highlight_value,
                wordcraft_gnome::t!("Highlight"),
                if self.last_highlight.borrow().as_str() == "none" {
                    wordcraft_gnome::t!("No Highlight").into()
                } else {
                    self.last_highlight.borrow().clone()
                },
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

fn decoration_row(title: &str, name: &str, style: &str) -> gtk::Button {
    let button = style_menu_button(title);
    button.set_widget_name(name);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let label = gtk::Label::new(Some(title));
    label.set_xalign(0.);
    label.set_hexpand(true);
    row.append(&label);
    let sample = gtk::DrawingArea::new();
    sample.set_content_width(72);
    sample.set_content_height(24);
    let style = style.to_owned();
    sample.set_draw_func(move |w, c, width, height| {
        let color = w.color();
        c.set_source_rgba(
            color.red() as f64,
            color.green() as f64,
            color.blue() as f64,
            color.alpha() as f64,
        );
        if style == "none" {
            return;
        }
        if matches!(style.as_str(), "baseline" | "superscript" | "subscript") {
            c.set_font_size(18.);
            c.move_to(12., 19.);
            let _ = c.show_text(match style.as_str() {
                "superscript" => "x²",
                "subscript" => "x₂",
                _ => "x",
            });
            return;
        }
        let strike = style.starts_with("strike");
        if strike {
            c.set_font_size(17.);
            c.move_to(15., 19.);
            let _ = c.show_text("Aa");
        }
        let y = if strike {
            height as f64 / 2. + 1.
        } else {
            height as f64 - 6.
        };
        c.set_line_width(if style == "thick" { 2.5 } else { 1. });
        match style.as_str() {
            "dotted" => c.set_dash(&[1., 3.], 0.),
            "dash" => c.set_dash(&[6., 3.], 0.),
            "dotDash" => c.set_dash(&[1., 3., 6., 3.], 0.),
            "dotDotDash" => c.set_dash(&[1., 3., 1., 3., 6., 3.], 0.),
            _ => {}
        }
        if style == "wave" {
            for x in 5..width - 5 {
                let y = y + ((x - 5) as f64 * 0.6).sin() * 1.5;
                if x == 5 {
                    c.move_to(x as f64, y);
                } else {
                    c.line_to(x as f64, y);
                }
            }
        } else {
            c.move_to(5., y);
            c.line_to(width as f64 - 5., y);
        }
        let _ = c.stroke();
        if matches!(style.as_str(), "double" | "strike-double") {
            c.move_to(5., y - 3.);
            c.line_to(width as f64 - 5., y - 3.);
            let _ = c.stroke();
        }
    });
    row.append(&sample);
    let selected = gtk::Image::from_icon_name("object-select-symbolic");
    selected.set_visible(false);
    row.append(&selected);
    button.set_child(Some(&row));
    button
}
