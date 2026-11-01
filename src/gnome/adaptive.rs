use super::*;

// Shared controls move between two real toolbar positions; no duplicate actions.
pub(super) struct AdaptiveTools {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) history: gtk::Box,
    pub(super) formats: gtk::Box,
    pub(super) top_history: gtk::Box,
    pub(super) top_formats: gtk::Box,
    pub(super) bottom: gtk::Box,
    pub(super) separator: gtk::Separator,
    pub(super) narrow: Cell<bool>,
    pub(super) generation: Cell<u64>,
}
impl AdaptiveTools {
    pub(super) fn sync_width(self: &Rc<Self>, width: i32) {
        if width <= 0 {
            return;
        }
        let narrow = tool_diagnostics()
            || if self.narrow.get() {
                width < 680
            } else {
                width < 640
            };
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        if self.narrow.replace(narrow) == narrow {
            ui.format_toolbar.set_visible(narrow);
            return;
        }
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        ui.format_reflowing.set(true);
        let open = ui
            .format_buttons
            .borrow()
            .iter()
            .filter_map(|m| {
                m.popover().filter(|p| p.is_visible()).map(|p| {
                    p.popdown();
                    m.clone()
                })
            })
            .collect::<Vec<_>>();
        for group in [&self.history, &self.formats] {
            if let Some(parent) = group.parent().and_then(|p| p.downcast::<gtk::Box>().ok()) {
                parent.remove(group);
            }
        }
        if narrow {
            self.bottom.prepend(&self.history);
            self.bottom
                .insert_child_after(&self.formats, Some(&self.separator));
        } else {
            self.top_history.append(&self.history);
            self.top_formats.append(&self.formats);
        }
        ui.format_toolbar.set_visible(narrow);
        self.separator.set_visible(narrow);
        for m in ui.format_buttons.borrow().iter() {
            m.set_direction(if narrow {
                gtk::ArrowType::Up
            } else {
                gtk::ArrowType::Down
            });
        }
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(40), move || {
            if let Some(a) = weak.upgrade() {
                if a.generation.get() != generation {
                    return;
                }
                if let Some(ui) = a.ui.upgrade() {
                    for menu in open {
                        menu.popup();
                    }
                    ui.format_reflowing.set(false);
                }
            }
        });
    }
}
pub(super) fn history_icon(redo: bool) -> gtk::DrawingArea {
    let a = gtk::DrawingArea::new();
    a.set_content_width(18);
    a.set_content_height(18);
    a.set_draw_func(move |w, c, _, _| {
        let f = w.color();
        c.set_source_rgba(
            f.red() as f64,
            f.green() as f64,
            f.blue() as f64,
            f.alpha() as f64,
        );
        if redo {
            c.translate(18.0, 0.0);
            c.scale(-1.0, 1.0);
        }
        c.set_line_width(1.6);
        c.set_line_cap(gtk::cairo::LineCap::Round);
        c.set_line_join(gtk::cairo::LineJoin::Round);
        c.move_to(6.0, 3.0);
        c.line_to(2.0, 7.0);
        c.line_to(6.0, 11.0);
        c.move_to(2.0, 7.0);
        c.line_to(11.0, 7.0);
        c.curve_to(17.0, 7.0, 17.0, 15.0, 11.0, 15.0);
        let _ = c.stroke();
    });
    a
}
pub(super) fn format_icon_button(menu: &gtk::MenuButton, label: &str, kind: &str) {
    let name = match kind {
        "style" => "format-text-rich-symbolic",
        "paragraph" => "format-justify-left-symbolic",
        _ => "font-select-symbolic",
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
}
impl TextTools {
    pub(super) fn home(self: &Rc<Self>) {
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
                    ui.document_box.grab_focus();
                    caret_ensure_visible(&ui);
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
    pub(super) fn color(self: &Rc<Self>, title: &str, cmd: &'static str) {
        self.heading(title);
        let choose = style_menu_button(wordcraft_gnome::t!("Choose Color…"));
        self.detail.append(&choose);
        let weak = Rc::downgrade(self);
        choose.connect_clicked(move |_| {
            let Some(s) = weak.upgrade() else {
                return;
            };
            let Some(ui) = s.ui.upgrade() else {
                return;
            };
            let Some(host) = ui.host.upgrade() else {
                return;
            };
            let identity = document_identity(&ui);
            if let Some(p) = s.popover.upgrade() {
                close_format_popover(&p);
            }
            let dialog = gtk::ColorDialog::new();
            dialog.set_title(wordcraft_gnome::t!("Choose Color"));
            dialog.set_with_alpha(false);
            let weak = Rc::downgrade(&s);
            dialog.choose_rgba(
                Some(&host.window),
                None::<&gdk::RGBA>,
                None::<&gio::Cancellable>,
                move |result| {
                    let Some(s) = weak.upgrade() else {
                        return;
                    };
                    let Some(ui) = s.ui.upgrade() else {
                        return;
                    };
                    if ui.closed.get() || document_identity(&ui) != identity {
                        return;
                    }
                    match result {
                        Ok(c) => {
                            let color = format!(
                                "{:02X}{:02X}{:02X}",
                                (c.red() * 255.0).round() as u8,
                                (c.green() * 255.0).round() as u8,
                                (c.blue() * 255.0).round() as u8
                            );
                            s.execute(cmd, serde_json::json!({"color":color}));
                        }
                        Err(e)
                            if e.matches(gtk::DialogError::Cancelled)
                                || e.matches(gtk::DialogError::Dismissed) => {}
                        Err(e) => ui.toasts.add_toast(Toast::new(&e.to_string())),
                    }
                },
            );
        });
        let e = gtk::Entry::new();
        e.set_placeholder_text(Some("RGB: 2266AA"));
        accessible_name(&e, wordcraft_gnome::t!("Hexadecimal RGB Color"));
        self.detail.append(&e);
        let b = style_menu_button(wordcraft_gnome::t!("Apply Color"));
        self.detail.append(&b);
        let weak = Rc::downgrade(self);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                let c = e.text().trim().trim_start_matches('#').to_string();
                if c.len() == 6 && c.bytes().all(|b| b.is_ascii_hexdigit()) {
                    s.execute(cmd, serde_json::json!({"color":c}));
                } else if let Some(ui) = s.ui.upgrade() {
                    ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                        "Enter Six Hexadecimal Digits"
                    )));
                }
            }
        });
    }
    pub(super) fn toggle(self: &Rc<Self>, label: &str, on: bool, cmd: &'static str) {
        let b = gtk::CheckButton::with_label(label);
        b.set_active(on);
        self.detail.append(&b);
        let weak = Rc::downgrade(self);
        b.connect_toggled(move |b| {
            if let Some(s) = weak.upgrade() {
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
        clear_box(&self.detail);
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = selection_character(&ui);
        let weak = Rc::downgrade(self);
        let home = page == "options";
        self.header.back(
            if home {
                wordcraft_gnome::t!("Text")
            } else {
                wordcraft_gnome::t!("Text Options")
            },
            move || {
                if let Some(s) = weak.upgrade() {
                    if home {
                        s.home();
                    } else {
                        s.page("options");
                    }
                }
            },
        );
        match page {
            "options" => {
                for (label, page) in [
                    (wordcraft_gnome::t!("Font and Characters"), "character"),
                    (wordcraft_gnome::t!("Color and Effects"), "color"),
                    (wordcraft_gnome::t!("Annotations"), "annotations"),
                ] {
                    let b = style_menu_button(label);
                    let weak = Rc::downgrade(self);
                    b.connect_clicked(move |_| {
                        if let Some(s) = weak.upgrade() {
                            s.page(page);
                        }
                    });
                    self.detail.append(&b);
                }
            }
            "character" => {
                self.heading(wordcraft_gnome::t!("Character Formatting"));
                for (label, key, cmd) in [
                    (wordcraft_gnome::t!("Bold"), "bold", "format.bold"),
                    (wordcraft_gnome::t!("Italic"), "italic", "format.italic"),
                    (
                        wordcraft_gnome::t!("Strikethrough"),
                        "strike",
                        "format.strikethrough",
                    ),
                    (
                        wordcraft_gnome::t!("Double Strikethrough"),
                        "doubleStrike",
                        "format.doubleStrikethrough",
                    ),
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
                    (wordcraft_gnome::t!("Superscript"), "format.superscript"),
                    (wordcraft_gnome::t!("Increase Size"), "format.growFont"),
                    (wordcraft_gnome::t!("Decrease Size"), "format.shrinkFont"),
                    (wordcraft_gnome::t!("Clear Text Formatting"), "format.clear"),
                ] {
                    self.action(label, cmd, serde_json::json!({}));
                }
                self.choice(
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
                );
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
                let b = style_menu_button(wordcraft_gnome::t!("Apply Spacing"));
                self.detail.append(&b);
                let weak = Rc::downgrade(self);
                b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){spacing.update();scale.update();position.update();s.execute("format.set",serde_json::json!({"props":{"spacing":spacing.value(),"scale":scale.value(),"position":position.value()}}));}});
            }
            "color" => {
                self.color(wordcraft_gnome::t!("Text Color"), "format.color");
                self.action(
                    wordcraft_gnome::t!("Automatic Color"),
                    "format.color",
                    serde_json::json!({"color":"auto"}),
                );
                self.choice(
                    wordcraft_gnome::t!("Highlight"),
                    &[
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
                    ],
                    "format.highlight",
                    "color",
                );
                self.action(
                    wordcraft_gnome::t!("Remove Underline"),
                    "format.underline",
                    serde_json::json!({"value":false}),
                );
                self.choice(
                    wordcraft_gnome::t!("Underline"),
                    &[
                        (wordcraft_gnome::t!("Single"), "single"),
                        (wordcraft_gnome::t!("Double"), "double"),
                        (wordcraft_gnome::t!("Thick"), "thick"),
                        (wordcraft_gnome::t!("Dotted"), "dotted"),
                        (wordcraft_gnome::t!("Dashed"), "dash"),
                        (wordcraft_gnome::t!("Dot Dash"), "dotDash"),
                        (wordcraft_gnome::t!("Dot Dot Dash"), "dotDotDash"),
                        (wordcraft_gnome::t!("Wavy"), "wave"),
                        (wordcraft_gnome::t!("Words Only"), "words"),
                    ],
                    "format.underline",
                    "style",
                );
                self.color(
                    wordcraft_gnome::t!("Underline Color"),
                    "format.underlineColor",
                );
                self.action(
                    wordcraft_gnome::t!("Automatic Underline Color"),
                    "format.underlineColor",
                    serde_json::json!({"color":"auto"}),
                );
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
                b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){size.update();offset.update();s.execute("format.phonetic",serde_json::json!({"ruby":ruby.text().as_str(),"size":size.value(),"offset":offset.value(),"align":"center"}));}});
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
    let main = gtk::Box::new(gtk::Orientation::Vertical, 8);
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
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(420);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&detail));
    stack.add_named(&main, Some("main"));
    stack.add_named(&scroll, Some("detail"));
    root.append(&stack);
    root.set_width_request(330);
    let s = Rc::new(TextTools {
        ui: Rc::downgrade(ui),
        popover: pop.downgrade(),
        stack,
        detail,
        header: header.clone(),
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
                state.home();
            }
        }
    ));
    s.home();
    s
}
