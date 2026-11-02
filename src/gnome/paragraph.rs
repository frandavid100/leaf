use super::*;

// Paragraph controls share the styles popover's internal-page navigation.
pub(super) struct ParagraphPopover {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) popover: glib::WeakRef<gtk::Popover>,
    pub(super) stack: gtk::Stack,
    pub(super) main: gtk::Box,
    pub(super) detail: gtk::Box,
    pub(super) message: gtk::Label,
    pub(super) header: Rc<FormatHeader>,
    pub(super) spacing: Cell<f64>,
    pub(super) shade: RefCell<String>,
    pub(super) border: RefCell<serde_json::Value>,
    active_page: RefCell<Option<String>>,
    displayed: RefCell<serde_json::Value>,
}
pub(super) fn paragraph_icon(name: &str) -> gtk::Widget {
    let area = gtk::DrawingArea::new();
    area.set_content_width(18);
    area.set_content_height(18);
    area.set_halign(gtk::Align::Center);
    area.set_valign(gtk::Align::Center);
    let kind = name.to_string();
    area.set_draw_func(move |w, cr, _, _| {
        let c = w.color();
        cr.set_source_rgba(
            c.red() as f64,
            c.green() as f64,
            c.blue() as f64,
            c.alpha() as f64,
        );
        cr.set_line_width(1.4);
        match kind.as_str() {
            "format-justify-left-symbolic"
            | "format-justify-center-symbolic"
            | "format-justify-right-symbolic"
            | "format-justify-fill-symbolic" => {
                for (i, width) in [14.0, 10.0, 14.0, 8.0].into_iter().enumerate() {
                    let width = if kind.contains("fill") { 14.0 } else { width };
                    let x = if kind.contains("center") {
                        (18.0 - width) / 2.0
                    } else if kind.contains("right") {
                        16.0 - width
                    } else {
                        2.0
                    };
                    cr.move_to(x, 3.0 + i as f64 * 4.0);
                    cr.line_to(x + width, 3.0 + i as f64 * 4.0);
                }
                let _ = cr.stroke();
            }
            "view-list-bullet-symbolic" | "view-list-ordered-symbolic" => {
                for i in 0..3 {
                    let y = 4.0 + i as f64 * 5.0;
                    cr.move_to(7.0, y);
                    cr.line_to(17.0, y);
                    let _ = cr.stroke();
                    if kind.contains("bullet") {
                        cr.arc(2.0, y, 1.2, 0.0, std::f64::consts::TAU);
                        let _ = cr.fill();
                    } else {
                        cr.set_font_size(6.5);
                        cr.move_to(0.0, y + 2.0);
                        let _ = cr.show_text(&(i + 1).to_string());
                    }
                }
            }
            "format-indent-less-symbolic" | "format-indent-more-symbolic" => {
                for y in [2.0, 16.0] {
                    cr.move_to(1.0, y);
                    cr.line_to(17.0, y);
                }
                for y in [6.0, 10.0] {
                    cr.move_to(8.0, y);
                    cr.line_to(17.0, y);
                }
                if kind.contains("less") {
                    cr.move_to(5.0, 5.0);
                    cr.line_to(1.0, 9.0);
                    cr.line_to(5.0, 13.0);
                } else {
                    cr.move_to(1.0, 5.0);
                    cr.line_to(5.0, 9.0);
                    cr.line_to(1.0, 13.0);
                }
                let _ = cr.stroke();
            }
            "replace" => {
                cr.move_to(1.0, 5.0);
                cr.line_to(16.0, 5.0);
                cr.line_to(12.0, 1.0);
                cr.move_to(16.0, 5.0);
                cr.line_to(12.0, 9.0);
                cr.move_to(17.0, 13.0);
                cr.line_to(2.0, 13.0);
                cr.line_to(6.0, 9.0);
                cr.move_to(2.0, 13.0);
                cr.line_to(6.0, 17.0);
                let _ = cr.stroke();
            }
            "spacing" => {
                for y in [4.0, 9.0, 14.0] {
                    cr.move_to(8.0, y);
                    cr.line_to(17.0, y);
                }
                cr.move_to(3.0, 2.0);
                cr.line_to(3.0, 16.0);
                cr.move_to(0.5, 5.0);
                cr.line_to(3.0, 2.0);
                cr.line_to(5.5, 5.0);
                cr.move_to(0.5, 13.0);
                cr.line_to(3.0, 16.0);
                cr.line_to(5.5, 13.0);
                let _ = cr.stroke();
            }
            "shading" => {
                cr.rectangle(2.0, 3.0, 14.0, 12.0);
                let _ = cr.stroke();
                cr.rectangle(4.0, 8.0, 10.0, 5.0);
                let _ = cr.fill();
            }
            _ => {
                cr.set_dash(&[2.0, 2.0], 0.0);
                cr.rectangle(2.0, 2.0, 14.0, 14.0);
                let _ = cr.stroke();
                cr.set_dash(&[], 0.0);
                cr.move_to(2.0, 16.0);
                cr.line_to(16.0, 16.0);
                let _ = cr.stroke();
            }
        }
    });
    area.upcast()
}
impl ParagraphPopover {
    pub(super) fn sync_selection(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade() else { return };
        let Some(pop) = self.popover.upgrade().filter(|p| p.is_visible()) else {
            return;
        };
        let mut state = selection_paragraph(&ui);
        if self.active_page.borrow().as_deref() == Some("columns") {
            state["target"] = serde_json::json!(ui.current.borrow().as_ref().map(|d| (
                d.session.document_id(),
                d.session.rev(),
                format!("{:?}", d.session.sel)
            )));
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
        let scroll = self
            .detail
            .ancestor(gtk::ScrolledWindow::static_type())
            .and_then(|w| w.downcast::<gtk::ScrolledWindow>().ok());
        let y = scroll.as_ref().map(|s| s.vadjustment().value());
        if let Some(page) = page {
            self.page(&page);
        } else {
            self.home();
        }
        if let (Some(scroll), Some(y)) = (scroll, y) {
            let weak = scroll.downgrade();
            glib::idle_add_local_once(move || {
                if let Some(s) = weak.upgrade() {
                    s.vadjustment().set_value(y);
                }
            });
        }
    }
    fn number_action(
        self: &Rc<Self>,
        n: &gtk::SpinButton,
        command: &'static str,
        key: &'static str,
        property: &'static str,
    ) {
        validate_numeric_input(n);
        let apply = {
            let weak = Rc::downgrade(self);
            move |n: &gtk::SpinButton| {
                let Some(s) = weak.upgrade() else { return };
                let Some(ui) = s.ui.upgrade().filter(|u| !u.closed.get()) else {
                    return;
                };
                if ui.format_syncing.get() {
                    return;
                }
                let text = n.text().trim().replace(',', ".");
                let Some(mut value) = (!text.ends_with('.'))
                    .then(|| text.parse::<f64>().ok())
                    .flatten()
                    .filter(|v| {
                        v.is_finite()
                            && *v >= n.adjustment().lower()
                            && *v <= n.adjustment().upper()
                    })
                else {
                    n.add_css_class("error");
                    return;
                };
                n.remove_css_class("error");
                if key == "level" {
                    value -= 1.0;
                }
                let state = selection_paragraph(&ui);
                let old = property
                    .split('.')
                    .try_fold(&state, |v, k| v.get(k))
                    .and_then(|v| v.as_f64());
                let rule_ok =
                    command != "para.lineSpacing" || state["lineSpacing"]["rule"] == "exact";
                if rule_ok && old.is_some_and(|old| (old - value).abs() < 0.001) {
                    return;
                }
                if let Err(e) = immediate_format(&ui, command, serde_json::json!({key:value})) {
                    s.message.set_text(&e);
                    s.message.set_visible(true);
                }
            }
        };
        let action: Rc<dyn Fn(&gtk::SpinButton)> = Rc::new(apply);
        let execute = action.clone();
        n.connect_value_changed(move |n| execute(n));
        let generation = Rc::new(Cell::new(0u64));
        let weak_ui = self.ui.clone();
        let weak_n = n.downgrade();
        n.connect_changed(move |_| {
            let Some(ui) = weak_ui.upgrade() else { return };
            if ui.format_syncing.get() {
                return;
            }
            let next = generation.get().wrapping_add(1);
            generation.set(next);
            let generation = generation.clone();
            let weak_n = weak_n.clone();
            let action = action.clone();
            let weak_ui = Rc::downgrade(&ui);
            let identity = document_identity(&ui);
            let selected = ui.current.borrow().as_ref().map(|d| d.session.sel.clone());
            glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
                if generation.get() != next {
                    return;
                }
                let (Some(ui), Some(n)) = (weak_ui.upgrade(), weak_n.upgrade()) else {
                    return;
                };
                if ui.closed.get()
                    || document_identity(&ui) != identity
                    || ui.current.borrow().as_ref().map(|d| d.session.sel.clone()) != selected
                {
                    return;
                }
                action(&n);
            });
        });
    }
    pub(super) fn execute(self: &Rc<Self>, command: &str, args: serde_json::Value) {
        if let Some(ui) = self.ui.upgrade() {
            match format_execute(&ui, command, args) {
                Ok(()) => {
                    self.message.set_visible(false);
                    if self.stack.visible_child_name().as_deref() == Some("main") {
                        self.home();
                    }
                    if let Some(p) = self.popover.upgrade() {
                        close_format_popover(&p);
                    }
                    ui.document_box.grab_focus();
                    caret_ensure_visible(&ui);
                }
                Err(e) => {
                    self.message.set_text(&e);
                    self.message.set_visible(true);
                }
            }
        }
    }
    pub(super) fn action(
        self: &Rc<Self>,
        label: &str,
        name: &str,
        command: &'static str,
        args: serde_json::Value,
    ) -> gtk::Button {
        let b = style_menu_button(label);
        b.set_widget_name(name);
        let weak = Rc::downgrade(self);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.execute(command, args.clone());
            }
        });
        b
    }
    pub(super) fn icon_action(
        self: &Rc<Self>,
        icon: &str,
        label: &str,
        name: &str,
        active: bool,
        command: &'static str,
        args: serde_json::Value,
    ) -> gtk::ToggleButton {
        let b = gtk::ToggleButton::new();
        b.set_child(Some(&paragraph_icon(icon)));
        b.set_tooltip_text(Some(label));
        accessible_name(&b, label);
        b.set_widget_name(name);
        b.set_active(active);
        b.set_hexpand(true);
        b.add_css_class("flat");
        let weak = Rc::downgrade(self);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.execute(command, args.clone());
            }
        });
        b
    }
    pub(super) fn split(
        self: &Rc<Self>,
        row: &gtk::Box,
        button: &gtk::ToggleButton,
        page: &'static str,
        label: &str,
    ) {
        // AdwSplitButton owns native shared hover, separator and half-button states.
        let split = adw::SplitButton::new();
        split.add_css_class("flat");
        split.set_hexpand(true);
        let icon = button.child().unwrap();
        button.set_child(None::<&gtk::Widget>);
        split.set_child(Some(&icon));
        split.add_css_class("image-button");
        split.set_tooltip_text(button.tooltip_text().as_deref());
        split.set_dropdown_tooltip(&wordcraft_gnome::i18n::interpolate(
            wordcraft_gnome::t!("Options for {label}"),
            &[("label", label.to_string())],
        ));
        // The documented CSS hierarchy exposes the two GTK controls. Route the
        // dropdown's lazy popup callback to our existing stack, not another popup.
        let main = split
            .first_child()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap();
        accessible_name(&main, button.tooltip_text().as_deref().unwrap_or(label));
        main.set_widget_name(&button.widget_name());
        main.add_css_class("split-action");
        if button.is_active() {
            main.set_state_flags(gtk::StateFlags::CHECKED, false);
        }
        let action = button.clone();
        split.connect_clicked(move |_| action.emit_clicked());
        let more = main
            .next_sibling()
            .unwrap()
            .next_sibling()
            .unwrap()
            .downcast::<gtk::MenuButton>()
            .unwrap();
        accessible_name(
            &more,
            &wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Options for {label}"),
                &[("label", label.to_string())],
            ),
        );
        more.first_child()
            .unwrap()
            .set_widget_name(&format!("paragraph-more-{page}"));
        let weak = Rc::downgrade(self);
        more.set_create_popup_func(move |m| {
            m.popdown();
            if let Some(s) = weak.upgrade() {
                s.page(page);
                if std::env::args().any(|a| a == "--split-states-demo") {
                    let weak = Rc::downgrade(&s);
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(200),
                        move || {
                            if let Some(s) = weak.upgrade() {
                                assert_eq!(s.stack.visible_child_name().as_deref(), Some("detail"));
                                eprintln!("SPLIT_SUBPAGE_OK");
                            }
                        },
                    );
                }
            }
        });
        row.append(&split);
    }
    pub(super) fn home(self: &Rc<Self>) {
        self.active_page.replace(None);
        self.header.home();
        clear_box(&self.main);
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = selection_paragraph(&ui);
        self.displayed.replace(state.clone());
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        for (icon, label, value) in [
            (
                "format-justify-left-symbolic",
                wordcraft_gnome::t!("Align Left"),
                "left",
            ),
            (
                "format-justify-center-symbolic",
                wordcraft_gnome::t!("Center"),
                "center",
            ),
            (
                "format-justify-right-symbolic",
                wordcraft_gnome::t!("Align Right"),
                "right",
            ),
            (
                "format-justify-fill-symbolic",
                wordcraft_gnome::t!("Justify"),
                "justify",
            ),
        ] {
            row.append(&self.icon_action(
                icon,
                label,
                &format!("paragraph-align-{value}"),
                state["align"].as_str().unwrap_or("left") == value,
                "para.align",
                serde_json::json!({"value":value}),
            ));
        }
        self.main.append(&row);
        self.main
            .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let list_kind = {
            let value = ui.current.borrow();
            value.as_ref().and_then(|d| {
                let p = d.session.doc.para_at(&d.session.sel.focus)?;
                let props = d.session.doc.styles.resolve_para(&p.props);
                let n = props.numbering?;
                d.session
                    .doc
                    .numbering
                    .level(n.num, n.level)
                    .map(|l| format!("{:?}", l.format))
            })
        };
        let bullet = list_kind
            .as_ref()
            .is_some_and(|s| s.to_lowercase().contains("bullet"));
        let numbered = list_kind.is_some() && !bullet;
        let b = self.icon_action(
            "view-list-bullet-symbolic",
            wordcraft_gnome::t!("Bulleted List"),
            "paragraph-bullets",
            bullet,
            "para.bullets",
            serde_json::json!({}),
        );
        self.split(&row, &b, "bullets", wordcraft_gnome::t!("bullets"));
        let b = self.icon_action(
            "view-list-ordered-symbolic",
            wordcraft_gnome::t!("Numbered List"),
            "paragraph-numbering",
            numbered,
            "para.numbering",
            serde_json::json!({}),
        );
        self.split(&row, &b, "numbering", wordcraft_gnome::t!("numbering"));
        row.append(&self.icon_action(
            "format-indent-less-symbolic",
            wordcraft_gnome::t!("Decrease Indent"),
            "paragraph-outdent",
            false,
            "para.outdent",
            serde_json::json!({}),
        ));
        let b = self.icon_action(
            "format-indent-more-symbolic",
            wordcraft_gnome::t!("Increase Indent"),
            "paragraph-indent",
            false,
            "para.indent",
            serde_json::json!({}),
        );
        self.split(&row, &b, "indents", wordcraft_gnome::t!("indents"));
        self.main.append(&row);
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let b = self.icon_action(
            "spacing",
            &wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Apply Line Spacing {}"),
                &[("0", format!("{}", self.spacing.get()))],
            ),
            "paragraph-spacing",
            state["lineSpacing"]["rule"].as_str() == Some("multiple")
                && state["lineSpacing"]["value"].as_f64() == Some(self.spacing.get()),
            "para.lineSpacing",
            serde_json::json!({"value":self.spacing.get()}),
        );
        self.split(&row, &b, "spacing", "interlineado");
        let b = self.icon_action(
            "shading",
            wordcraft_gnome::t!("Apply Last Shading"),
            "paragraph-shading",
            state["shading"].is_array(),
            "para.shading",
            serde_json::json!({"color":self.shade.borrow().as_str()}),
        );
        self.split(&row, &b, "shading", "sombreado");
        let b = self.icon_action(
            "borders",
            wordcraft_gnome::t!("Apply Last Border"),
            "paragraph-borders",
            state["borders"].is_object(),
            "para.borders",
            self.border.borrow().clone(),
        );
        self.split(&row, &b, "borders", "bordes");
        self.main.append(&row);
        self.main
            .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let columns = style_menu_button(wordcraft_gnome::t!("Columns…"));
        columns.set_widget_name("paragraph-columns");
        self.main.append(&columns);
        let weak = Rc::downgrade(self);
        columns.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.page("columns");
            }
        });
        let more = style_menu_button(wordcraft_gnome::t!("More Paragraph Options…"));
        more.set_widget_name("paragraph-more-options");
        self.main.append(&more);
        let weak = Rc::downgrade(self);
        more.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.page("options");
            }
        });
        self.stack.set_visible_child_name("main");
    }
    pub(super) fn title(&self, label: &str) {
        let l = gtk::Label::new(Some(label));
        l.set_xalign(0.0);
        l.add_css_class("heading");
        l.set_margin_top(8);
        self.detail.append(&l);
    }
    pub(super) fn number(&self, label: &str, value: f64, min: f64, max: f64) -> gtk::SpinButton {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let l = gtk::Label::new(Some(label));
        l.set_xalign(0.0);
        l.set_hexpand(true);
        l.set_wrap(true);
        row.append(&l);
        let n = gtk::SpinButton::with_range(min, max, 0.5);
        n.set_digits(2);
        n.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
        n.set_width_chars(6);
        n.set_value(value);
        accessible_name(&n, label);
        row.append(&n);
        self.detail.append(&row);
        n
    }
    pub(super) fn button(
        self: &Rc<Self>,
        label: &str,
        name: &str,
        command: &'static str,
        args: serde_json::Value,
    ) {
        self.detail.append(&self.action(label, name, command, args));
    }
    pub(super) fn page(self: &Rc<Self>, page: &str) {
        self.active_page.replace(Some(page.to_string()));
        let weak = Rc::downgrade(self);
        self.header.back(wordcraft_gnome::t!("Paragraph"), move || {
            if let Some(s) = weak.upgrade() {
                s.home();
            }
        });
        clear_box(&self.detail);
        self.message.set_visible(false);
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = selection_paragraph(&ui);
        match page {
            "columns" => {
                self.title(wordcraft_gnome::t!("Columns"));
                let weak = Rc::downgrade(self);
                let editor = columns_editor(&ui, move || {
                    if let Some(s) = weak.upgrade() {
                        s.home();
                        if let Some(pop) = s.popover.upgrade() {
                            close_format_popover(&pop);
                        }
                    }
                });
                self.detail.append(&editor.root);
                self.detail.append(&editor.apply);
            }
            "bullets" | "numbering" => {
                self.title(if page == "bullets" {
                    wordcraft_gnome::t!("Bullets")
                } else {
                    wordcraft_gnome::t!("Numbering")
                });
                let (command, choices): (&'static str, Vec<(&str, &str)>) = if page == "bullets" {
                    (
                        "para.bullets",
                        vec![
                            (wordcraft_gnome::t!("• Disc"), "•"),
                            (wordcraft_gnome::t!("○ Circle"), "○"),
                            (wordcraft_gnome::t!("▪ Square"), "▪"),
                            (wordcraft_gnome::t!("– Dash"), "–"),
                        ],
                    )
                } else {
                    (
                        "para.numbering",
                        vec![
                            ("1. 2. 3.", "numbered"),
                            ("1) 2) 3)", "numberedParen"),
                            ("A. B. C.", "upperLetter"),
                            ("a. b. c.", "lowerLetter"),
                            ("i. ii. iii.", "lowerRoman"),
                        ],
                    )
                };
                for (label, kind) in choices {
                    self.button(
                        label,
                        &format!("paragraph-list-{kind}"),
                        command,
                        serde_json::json!({"kind":kind}),
                    );
                }
                self.button(
                    wordcraft_gnome::t!("Remove List"),
                    "paragraph-list-none",
                    command,
                    serde_json::json!({"off":true}),
                );
                self.title(wordcraft_gnome::t!("Multilevel List"));
                self.button(
                    "1 / 1.1 / 1.1.1",
                    "paragraph-list-legal",
                    "para.multilevel",
                    serde_json::json!({"kind":"legal"}),
                );
                self.button(
                    wordcraft_gnome::t!("Outline"),
                    "paragraph-list-outline",
                    "para.multilevel",
                    serde_json::json!({"kind":"outline"}),
                );
                let level = self.number(
                    wordcraft_gnome::t!("Level (1–9)"),
                    state["numbering"]["level"].as_f64().unwrap_or(0.0) + 1.0,
                    1.0,
                    9.0,
                );
                level.set_digits(0);
                level.set_increments(1.0, 1.0);
                self.number_action(&level, "para.listLevel", "level", "numbering.level");
                self.button(
                    wordcraft_gnome::t!("Restart Numbering at 1"),
                    "paragraph-list-restart",
                    "para.restartNumbering",
                    serde_json::json!({}),
                );
            }
            "indents" => {
                self.title(wordcraft_gnome::t!("Indents (pt)"));
                let left = self.number(
                    wordcraft_gnome::t!("Left"),
                    state["indentLeft"].as_f64().unwrap_or(0.0),
                    -1584.0,
                    1584.0,
                );
                let right = self.number(
                    wordcraft_gnome::t!("Right"),
                    state["indentRight"].as_f64().unwrap_or(0.0),
                    -1584.0,
                    1584.0,
                );
                let first = self.number(
                    wordcraft_gnome::t!("First Line (−: Hanging)"),
                    state["indentFirst"].as_f64().unwrap_or(0.0),
                    -1584.0,
                    1584.0,
                );
                for (n, key, property) in [
                    (&left, "left", "indentLeft"),
                    (&right, "right", "indentRight"),
                    (&first, "firstLine", "indentFirst"),
                ] {
                    n.set_widget_name(&format!("paragraph-{property}"));
                    if state[property].is_null() {
                        n.set_text("");
                    }
                    self.number_action(n, "para.indents", key, property);
                }
            }
            "spacing" => {
                self.title(wordcraft_gnome::t!("Line Spacing"));
                for (value, label) in [
                    (1.0, wordcraft_gnome::t!("Single")),
                    (1.15, wordcraft_gnome::t!("1.15 Lines")),
                    (1.5, wordcraft_gnome::t!("1.5 Lines")),
                    (2.0, wordcraft_gnome::t!("Double")),
                ] {
                    let b = style_menu_button(label);
                    b.set_widget_name(&format!("paragraph-line-{value}"));
                    self.detail.append(&b);
                    let weak = Rc::downgrade(self);
                    b.connect_clicked(move |_| {
                        if let Some(s) = weak.upgrade() {
                            s.spacing.set(value);
                            s.execute("para.lineSpacing", serde_json::json!({"value":value}));
                        }
                    });
                }
                let exact = self.number(
                    wordcraft_gnome::t!("Exactly (pt)"),
                    state["lineSpacing"]["value"].as_f64().unwrap_or(14.0),
                    1.0,
                    500.0,
                );
                exact.set_widget_name("paragraph-exact-line-spacing");
                if state["lineSpacing"]["rule"] != "exact" {
                    exact.set_text("");
                }
                self.number_action(&exact, "para.lineSpacing", "exactly", "lineSpacing.value");
                self.title(wordcraft_gnome::t!("Paragraph Spacing (pt)"));
                let before = self.number(
                    wordcraft_gnome::t!("Before"),
                    state["spaceBefore"].as_f64().unwrap_or(0.0),
                    0.0,
                    1584.0,
                );
                let after = self.number(
                    wordcraft_gnome::t!("After"),
                    state["spaceAfter"].as_f64().unwrap_or(0.0),
                    0.0,
                    1584.0,
                );
                for (n, key, property) in [
                    (&before, "before", "spaceBefore"),
                    (&after, "after", "spaceAfter"),
                ] {
                    n.set_widget_name(&format!("paragraph-{property}"));
                    if state[property].is_null() {
                        n.set_text("");
                    }
                    self.number_action(n, "para.spacing", key, property);
                }
            }
            "shading" => {
                self.title(wordcraft_gnome::t!("Shading"));
                let grid = gtk::Grid::new();
                grid.set_column_spacing(6);
                grid.set_row_spacing(6);
                for (i, (label, color)) in [
                    (wordcraft_gnome::t!("Light Yellow"), "FFF2CC"),
                    (wordcraft_gnome::t!("Light Blue"), "DDEBF7"),
                    (wordcraft_gnome::t!("Light Green"), "E2F0D9"),
                    (wordcraft_gnome::t!("Light Pink"), "FCE4D6"),
                    (wordcraft_gnome::t!("Light Gray"), "E7E6E6"),
                    (wordcraft_gnome::t!("Dark Gray"), "808080"),
                    (wordcraft_gnome::t!("Blue"), "4472C4"),
                    (wordcraft_gnome::t!("Black"), "000000"),
                ]
                .into_iter()
                .enumerate()
                {
                    let b = gtk::Button::new();
                    let l = gtk::Label::new(None);
                    l.set_markup(&format!("<span background=\"#{color}\">     </span>"));
                    b.set_child(Some(&l));
                    b.set_tooltip_text(Some(label));
                    accessible_name(&b, label);
                    b.set_widget_name(&format!("paragraph-shade-{color}"));
                    grid.attach(&b, (i % 4) as i32, (i / 4) as i32, 1, 1);
                    let weak = Rc::downgrade(self);
                    b.connect_clicked(move |_| {
                        if let Some(s) = weak.upgrade() {
                            s.shade.replace(color.into());
                            s.execute("para.shading", serde_json::json!({"color":color}));
                        }
                    });
                }
                self.detail.append(&grid);
                self.color_entry(wordcraft_gnome::t!("Apply RGB Color"), false);
                self.button(
                    wordcraft_gnome::t!("No Shading"),
                    "paragraph-shade-none",
                    "para.shading",
                    serde_json::json!({"color":null}),
                );
            }
            "borders" => {
                self.title(wordcraft_gnome::t!("Borders"));
                let width = self.number(wordcraft_gnome::t!("Width (pt)"), 0.5, 0.25, 12.0);
                let color = gtk::Entry::new();
                color.set_placeholder_text(Some(wordcraft_gnome::t!(
                    "RGB Color, for Example 2266AA"
                )));
                color.set_text("000000");
                self.detail.append(&color);
                for (label, kind) in [
                    (wordcraft_gnome::t!("No Borders"), "none"),
                    (wordcraft_gnome::t!("All"), "all"),
                    (wordcraft_gnome::t!("Outside"), "outside"),
                    (wordcraft_gnome::t!("Inside"), "inside"),
                    (wordcraft_gnome::t!("Top"), "top"),
                    (wordcraft_gnome::t!("Bottom"), "bottom"),
                    (wordcraft_gnome::t!("Left"), "left"),
                    (wordcraft_gnome::t!("Right"), "right"),
                    (wordcraft_gnome::t!("Horizontal Line"), "horizontalLine"),
                ] {
                    let b = style_menu_button(label);
                    b.set_widget_name(&format!("paragraph-border-{kind}"));
                    self.detail.append(&b);
                    let weak = Rc::downgrade(self);
                    let width = width.clone();
                    let color = color.clone();
                    b.connect_clicked(move |_|{if let Some(s)=weak.upgrade(){width.update();if let Some(hex)=s.valid_color(&color){let args=serde_json::json!({"kind":kind,"width":width.value(),"color":hex});s.border.replace(args.clone());s.execute("para.borders",args);}}});
                }
            }
            _ => {
                self.title(wordcraft_gnome::t!("Direction and Formatting"));
                self.button(
                    wordcraft_gnome::t!("Distributed Alignment"),
                    "paragraph-distribute",
                    "para.align",
                    serde_json::json!({"value":"distribute"}),
                );
                self.button(
                    wordcraft_gnome::t!("Left to Right"),
                    "paragraph-ltr",
                    "para.ltr",
                    serde_json::json!({"value":true}),
                );
                self.button(
                    wordcraft_gnome::t!("Right to Left"),
                    "paragraph-rtl",
                    "para.rtl",
                    serde_json::json!({"value":true}),
                );
                let marks = ui
                    .current
                    .borrow()
                    .as_ref()
                    .is_some_and(|d| d.session.view.marks);
                self.toggle(
                    wordcraft_gnome::t!("Show Formatting Marks (¶)"),
                    marks,
                    "view.marks",
                );
                self.title(wordcraft_gnome::t!("Breaks and Pagination"));
                for (label, key, cmd) in [
                    (
                        wordcraft_gnome::t!("Keep with Next"),
                        "keepNext",
                        "para.keepNext",
                    ),
                    (
                        wordcraft_gnome::t!("Keep Lines Together"),
                        "keepLines",
                        "para.keepLines",
                    ),
                    (
                        wordcraft_gnome::t!("Page Break Before"),
                        "pageBreakBefore",
                        "para.pageBreakBefore",
                    ),
                    (
                        wordcraft_gnome::t!("Widow and Orphan Control"),
                        "widowControl",
                        "para.widowControl",
                    ),
                ] {
                    self.toggle(label, state[key].as_bool().unwrap_or(false), cmd);
                }
                self.title(wordcraft_gnome::t!("Sort Selected Paragraphs"));
                self.button(
                    wordcraft_gnome::t!("Ascending (A–Z)"),
                    "paragraph-sort-asc",
                    "para.sort",
                    serde_json::json!({"descending":false}),
                );
                self.button(
                    wordcraft_gnome::t!("Descending (Z–A)"),
                    "paragraph-sort-desc",
                    "para.sort",
                    serde_json::json!({"descending":true}),
                );
                self.title(wordcraft_gnome::t!("Tab Stops"));
                let pos = self.number(wordcraft_gnome::t!("Position (pt)"), 36.0, 0.0, 1584.0);
                let b = style_menu_button(wordcraft_gnome::t!("Add Left Tab Stop"));
                self.detail.append(&b);
                let weak = Rc::downgrade(self);
                b.connect_clicked(move |_| {
                    if let Some(s) = weak.upgrade() {
                        pos.update();
                        s.execute(
                            "para.tabs",
                            serde_json::json!({"set":[{"pos":pos.value(),"align":"left"}]}),
                        );
                    }
                });
                self.button(
                    wordcraft_gnome::t!("Clear Tab Stops"),
                    "paragraph-tabs-clear",
                    "para.tabs",
                    serde_json::json!({"clearAll":true}),
                );
                self.button(
                    wordcraft_gnome::t!("Reset Paragraph Formatting"),
                    "paragraph-reset",
                    "para.reset",
                    serde_json::json!({}),
                );
            }
        }
        self.stack.set_visible_child_name("detail");
    }
    pub(super) fn toggle(self: &Rc<Self>, label: &str, active: bool, command: &'static str) {
        let b = gtk::CheckButton::with_label(label);
        b.set_active(active);
        if let Some(ui) = self.ui.upgrade() {
            let state = selection_paragraph(&ui);
            let key = command.rsplit('.').next().unwrap_or("");
            b.set_inconsistent(state.get(key).is_some_and(|v| v.is_null()));
        }
        self.detail.append(&b);
        let weak = Rc::downgrade(self);
        b.connect_toggled(move |b| {
            b.set_inconsistent(false);
            if let Some(s) = weak.upgrade() {
                s.execute(command, serde_json::json!({"value":b.is_active()}));
            }
        });
    }
    pub(super) fn valid_color(&self, e: &gtk::Entry) -> Option<String> {
        let c = e.text().trim().trim_start_matches('#').to_string();
        if c.len() == 6 && c.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(c)
        } else {
            self.message.set_text(wordcraft_gnome::t!(
                "Enter Six RGB Digits, for Example 2266AA"
            ));
            self.message.set_visible(true);
            None
        }
    }
    pub(super) fn color_entry(self: &Rc<Self>, label: &str, _border: bool) {
        let e = gtk::Entry::new();
        e.set_placeholder_text(Some("RGB hexadecimal: FFF2CC"));
        self.detail.append(&e);
        let _ = label;
        let weak = Rc::downgrade(self);
        e.connect_changed(move |entry| {
            if let Some(s) = weak.upgrade() {
                let c = entry.text().trim().trim_start_matches('#').to_string();
                if c.len() == 6 && c.bytes().all(|b| b.is_ascii_hexdigit()) {
                    entry.remove_css_class("error");
                    s.shade.replace(c.clone());
                    if let Some(ui) = s.ui.upgrade() {
                        if let Err(e) =
                            immediate_format(&ui, "para.shading", serde_json::json!({"color":c}))
                        {
                            s.message.set_text(&e);
                            s.message.set_visible(true);
                        }
                    }
                } else if !c.is_empty() {
                    entry.add_css_class("error");
                }
            }
        });
    }
}
pub(super) fn install_paragraph_popover(
    ui: &Rc<Ui>,
    menu: &gtk::MenuButton,
) -> Rc<ParagraphPopover> {
    let pop = gtk::Popover::new();
    pop.add_css_class("format-popover");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_width_request(300);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);
    let header = format_popover_header(
        &pop,
        &root,
        wordcraft_gnome::t!("Paragraph"),
        &ui.format_group,
    );
    header.button.set_widget_name("paragraph-back");
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    stack.set_vhomogeneous(false);
    stack.set_hhomogeneous(false);
    stack.set_interpolate_size(true);
    let main = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let detail = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(420);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&detail));
    let detail_page = gtk::Box::new(gtk::Orientation::Vertical, 6);
    detail_page.append(&scroll);
    stack.add_named(&main, Some("main"));
    stack.add_named(&detail_page, Some("detail"));
    root.append(&stack);
    let message = gtk::Label::new(None);
    message.set_wrap(true);
    message.set_max_width_chars(35);
    message.set_visible(false);
    root.append(&message);
    pop.set_child(Some(&root));
    let s = Rc::new(ParagraphPopover {
        ui: Rc::downgrade(ui),
        popover: pop.downgrade(),
        stack,
        main,
        detail,
        message,
        header,
        spacing: Cell::new(1.5),
        shade: RefCell::new("FFF2CC".into()),
        border: RefCell::new(serde_json::json!({"kind":"outside","width":0.5,"color":"000000"})),
        active_page: RefCell::new(None),
        displayed: RefCell::new(serde_json::Value::Null),
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
    menu.set_popover(Some(&pop));
    s
}
pub(super) fn show_paragraph_options(ui: &Rc<Ui>, _parent: &ApplicationWindow) {
    if let Some(s) = ui.paragraph.borrow().as_ref() {
        if let Some(p) = s.popover.upgrade() {
            p.popup();
        }
        s.page("options");
    }
}
