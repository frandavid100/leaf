use super::*;

// Paragraph controls share the styles popover's internal-page navigation.
pub(super) struct ParagraphPopover {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) popover: glib::WeakRef<gtk::Popover>,
    pub(super) stack: gtk::Stack,
    pub(super) main: gtk::Box,
    pub(super) detail: gtk::Box,
    pub(super) fixed: gtk::Box,
    pub(super) message: gtk::Label,
    pub(super) header: Rc<FormatHeader>,
    pub(super) spacing: Cell<f64>,
    pub(super) shade: RefCell<String>,
    pub(super) border: RefCell<serde_json::Value>,
    last_applied: RefCell<std::collections::HashMap<String, serde_json::Value>>,
    pub(super) groups: RefCell<std::collections::HashMap<String, serde_json::Value>>,
    pub(super) last_columns: RefCell<
        Option<(
            document_properties::ColumnSetup,
            document_properties::ColumnScope,
        )>,
    >,
    active_page: RefCell<Option<String>>,
    displayed: RefCell<serde_json::Value>,
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
        // GTK outputs an adjustment's number again while mapping a SpinButton.
        // An unset/mixed field must stay blank, and that initial output is not
        // a user edit (notably when the current line spacing is proportional).
        let ready = Rc::new(Cell::new(n.is_mapped()));
        let blank = n.text().trim().is_empty();
        let mapped = ready.clone();
        n.connect_map(move |n| {
            mapped.set(false);
            let weak = n.downgrade();
            let mapped = mapped.clone();
            glib::idle_add_local_once(move || {
                if let Some(n) = weak.upgrade().filter(|n| n.is_mapped()) {
                    if blank {
                        n.set_text("");
                    }
                    mapped.set(true);
                }
            });
        });
        let unmapped = ready.clone();
        n.connect_unmap(move |_| unmapped.set(false));
        let apply = {
            let weak = Rc::downgrade(self);
            let ready = ready.clone();
            move |n: &gtk::SpinButton| {
                if !ready.get() || !n.is_mapped() {
                    return;
                }
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
                let args = serde_json::json!({key:value});
                match immediate_format(&ui, command, args.clone()) {
                    Ok(()) => {
                        s.last_applied.borrow_mut().insert(command.into(), args);
                    }
                    Err(e) => {
                        s.message.set_text(&e);
                        s.message.set_visible(true);
                    }
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
            if !ready.get() {
                return;
            }
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
            match format_execute(&ui, command, args.clone()) {
                Ok(()) => {
                    let enabled = args.get("off") != Some(&serde_json::json!(true))
                        && !(command == "para.shading"
                            && args.get("color").is_some_and(|v| v.is_null()))
                        && !(command == "para.borders" && args["kind"] == "none");
                    if enabled {
                        self.last_applied
                            .borrow_mut()
                            .insert(command.into(), args.clone());
                    }
                    self.message.set_visible(false);
                    if self.stack.visible_child_name().as_deref() == Some("main") {
                        self.home();
                    }
                    if let Some(p) = self.popover.upgrade() {
                        close_format_popover(&p);
                    }
                    if !ui.format_group.pinned.get() {
                        ui.document_box.grab_focus();
                        caret_ensure_visible(&ui);
                    }
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
        let icon = if command == "para.align" {
            match args["value"].as_str() {
                Some("center") => "format-justify-center-symbolic",
                Some("right") => "format-justify-right-symbolic",
                Some("justify" | "distribute") => "format-justify-fill-symbolic",
                _ => "format-justify-left-symbolic",
            }
        } else if command == "para.outdent" {
            "format-indent-less-symbolic"
        } else if command.contains("indent") {
            "format-indent-more-symbolic"
        } else if command.contains("bullets")
            || command.contains("numbering")
            || command.contains("list")
        {
            "view-list-symbolic"
        } else if command.contains("shading") {
            "color-select-symbolic"
        } else if command.contains("borders") {
            "view-grid-symbolic"
        } else {
            "paragraph-symbolic"
        };
        if args["off"] == true && matches!(command, "para.bullets" | "para.numbering") {
            b.set_child(Some(&list_choice_content(label, "list-remove")));
        } else {
            b.set_child(Some(&property_content(label, icon).0));
        }
        let weak = Rc::downgrade(self);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.execute(command, args.clone());
            }
        });
        b
    }
    pub(super) fn remember_property(&self, command: &str, args: &serde_json::Value, ui: &Rc<Ui>) {
        let group = match command {
            "para.align" => "alignment",
            "para.bullets" | "para.numbering" | "para.multilevel" | "para.listLevel" => "list",
            "para.indents" => "indents",
            "para.sort" => "sort",
            "para.shading" => "shading",
            "para.borders" => "borders",
            "para.lineSpacing" | "para.spacing" => "spacing",
            _ => return,
        };
        let state = selection_paragraph(ui);
        let keys: &[&str] = match group {
            "alignment" => &["align"],
            "list" => &["numbering"],
            "indents" => &["indentLeft", "indentRight", "indentFirst"],
            "borders" => &["borders"],
            "shading" => &["shading"],
            _ => &["lineSpacing", "spaceBefore", "spaceAfter"],
        };
        let props: serde_json::Map<_, _> = keys
            .iter()
            .filter_map(|k| {
                state
                    .get(*k)
                    .filter(|v| !v.is_null())
                    .map(|v| ((*k).to_string(), v.clone()))
            })
            .collect();
        let config = if command == "para.sort" {
            serde_json::json!({"command":command,"args":args})
        } else if command == "para.listLevel" {
            let saved = self.groups.borrow().get("list").cloned();
            if let Some(mut saved) = saved.filter(|v| v["command"].is_string()) {
                saved["listLevel"] = args["level"].clone();
                saved
            } else {
                // Imported numbering retains its canonical definition in this document.
                serde_json::json!({"props":props,"document":ui.current.borrow().as_ref().map(|d|d.session.document_id())})
            }
        } else if group == "list" {
            serde_json::json!({"command":command,"args":args})
        } else {
            serde_json::json!({"props":props,"clearShading":group=="shading" && state["shading"].is_null()})
        };
        self.groups.borrow_mut().insert(group.into(), config);
    }

    pub(super) fn repeat_group(self: &Rc<Self>, page: &str) {
        if page == "columns" {
            let saved = self.last_columns.borrow().clone();
            if let (Some(ui), Some((setup, scope))) = (self.ui.upgrade(), saved) {
                if tools_change(&ui, |s| {
                    document_properties::apply_columns(s, &setup, scope).map(|_| ())
                }) {
                    if let Some(p) = self.popover.upgrade() {
                        close_format_popover(&p);
                    }
                }
            }
        } else {
            let args = self.groups.borrow().get(page).cloned();
            if let Some(args) = args {
                let Some(ui) = self.ui.upgrade() else { return };
                if args["document"].as_u64().is_some_and(|id| {
                    ui.current
                        .borrow()
                        .as_ref()
                        .is_none_or(|d| d.session.document_id() != id)
                }) {
                    return;
                }
                if let Some(command) = args["command"]
                    .as_str()
                    .filter(|_| args["listLevel"].is_null())
                {
                    self.execute(command, args["args"].clone());
                    return;
                }
                let clear = args["clearShading"] == true;
                if tools_change(&ui, |s| {
                    let snapshot = s.edit_snapshot();
                    let mark = s.undo_mark();
                    let before = s.doc.clone();
                    let result = (|| {
                        if clear {
                            s.run(
                                "para.shading",
                                &serde_json::json!({"color":null,"paragraph":true}),
                            )
                            .map_err(|e| e.to_string())?;
                            s.join_next_undo();
                        }
                        if let Some(command) = args["command"].as_str() {
                            s.run(command, &args["args"]).map_err(|e| e.to_string())?;
                            s.join_next_undo();
                            s.run(
                                "para.listLevel",
                                &serde_json::json!({"level":args["listLevel"]}),
                            )
                            .map_err(|e| e.to_string())?;
                        } else {
                            s.run("para.set", &args).map_err(|e| e.to_string())?;
                        }
                        Ok(())
                    })();
                    if result.is_err() {
                        s.restore(snapshot);
                    } else if s.doc != before {
                        s.merge_undo_since(mark);
                    }
                    result
                }) {
                    if let Some(p) = self.popover.upgrade() {
                        close_format_popover(&p);
                    }
                    if !ui.format_group.pinned.get() {
                        ui.document_box.grab_focus();
                    }
                }
            }
        }
    }
    fn option_row(self: &Rc<Self>, title: &str, name: &str, page: &'static str) {
        let button = style_menu_button(title);
        button.set_widget_name(name);
        button.set_hexpand(true);
        if page != "options" {
            button.remove_css_class("flat");
            let icon = match page {
                "alignment" => "format-justify-left-symbolic",
                "list" => "view-list-symbolic",
                "indents" => "format-indent-more-symbolic",
                "columns" => "view-dual-symbolic",
                _ => "paragraph-symbolic",
            };
            let (content, _) = property_content(title, icon);
            button.set_child(Some(&content));
        }
        button.set_tooltip_text(Some(title));
        accessible_name(&button, title);
        if page == "options" {
            let footer = gtk::Label::new(Some(title));
            footer.set_xalign(0.);
            button.set_child(Some(&footer));
            self.main
                .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            self.main.append(&button);
        } else {
            let repeat = gtk::Button::from_icon_name("view-refresh-symbolic");

            repeat.set_widget_name(&format!("{name}-apply"));
            repeat.set_tooltip_text(Some(wordcraft_gnome::t!("Apply Last Settings")));
            accessible_name(&repeat, wordcraft_gnome::t!("Apply Last Settings"));
            repeat.set_sensitive(if page == "columns" {
                self.last_columns.borrow().is_some()
            } else {
                self.groups.borrow().get(page).is_some_and(|v| {
                    v["document"].as_u64().is_none_or(|id| {
                        self.ui.upgrade().is_some_and(|u| {
                            u.current
                                .borrow()
                                .as_ref()
                                .is_some_and(|d| d.session.document_id() == id)
                        })
                    })
                })
            });
            let weak = Rc::downgrade(self);
            repeat.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.repeat_group(page);
                }
            });
            let row = reapply_split(&button, &repeat);
            row.set_widget_name(&format!("format-row-paragraph-{page}"));
            self.main.append(&row);
        }
        let weak = Rc::downgrade(self);
        button.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.page(page);
                if std::env::args().any(|a| a == "--format136-ui-test") {
                    eprintln!("FORMAT136_PARAGRAPH_OPEN {page}");
                }
                if std::env::args().any(|a| a == "--split-states-demo") {
                    eprintln!("SPLIT_SUBPAGE_OK");
                }
            }
        });
    }
    pub(super) fn home(self: &Rc<Self>) {
        self.active_page.replace(None);
        self.header.home();
        clear_box(&self.main);
        let Some(ui) = self.ui.upgrade() else { return };
        let state = selection_paragraph(&ui);
        self.displayed.replace(state.clone());
        let alignment = match state["align"].as_str() {
            Some("left") => wordcraft_gnome::t!("Align Left"),
            Some("center") => wordcraft_gnome::t!("Center"),
            Some("right") => wordcraft_gnome::t!("Align Right"),
            Some("justify") => wordcraft_gnome::t!("Justify"),
            Some("distribute") => wordcraft_gnome::t!("Distributed Alignment"),
            _ => wordcraft_gnome::t!("Mixed"),
        };
        self.option_row(
            &wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Alignment: {value}"),
                &[("value", alignment.into())],
            ),
            "paragraph-alignment",
            "alignment",
        );
        let list_kind = {
            let value = ui.current.borrow();
            value.as_ref().and_then(|d| {
                let p = d.session.doc.para_at(&d.session.sel.focus)?;
                let n = d.session.doc.styles.resolve_para(&p.props).numbering?;
                d.session
                    .doc
                    .numbering
                    .level(n.num, n.level)
                    .map(|l| format!("{:?}", l.format))
            })
        };
        let mixed_list = state["_mixed"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v == "numbering"));
        let list_label = if mixed_list {
            wordcraft_gnome::t!("Mixed")
        } else if list_kind
            .as_ref()
            .is_some_and(|s| s.to_lowercase().contains("bullet"))
        {
            wordcraft_gnome::t!("Bulleted List")
        } else if list_kind.is_some() {
            wordcraft_gnome::t!("Numbered List")
        } else {
            wordcraft_gnome::t!("None")
        };
        self.option_row(
            &wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("{label}: {value}"),
                &[
                    ("label", wordcraft_gnome::t!("List").into()),
                    ("value", list_label.into()),
                ],
            ),
            "paragraph-list",
            "list",
        );
        self.option_row(
            wordcraft_gnome::t!("Columns"),
            "paragraph-columns",
            "columns",
        );
        self.option_row(
            wordcraft_gnome::t!("Indents"),
            "paragraph-more-indents",
            "indents",
        );
        let mixed = |key: &str| {
            state["_mixed"]
                .as_array()
                .is_some_and(|a| a.iter().any(|v| v == key))
        };
        // The engine stores f32 values; do not show their JSON expansion
        // (for example 1.149999976) as a current selection value.
        let line_value = state["lineSpacing"]["value"]
            .as_f64()
            .map(|v| {
                format!("{v:.2}")
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_owned()
            })
            .unwrap_or_default();
        let spacing_value = if mixed("lineSpacing") {
            wordcraft_gnome::t!("Mixed").to_string()
        } else if state["lineSpacing"]["rule"] == "exact" {
            format!("{line_value} pt")
        } else {
            line_value
        };
        let shade_value = if mixed("shading") {
            wordcraft_gnome::t!("Mixed").into()
        } else if let Some(rgb) = state["shading"].as_array() {
            format!(
                "#{:02X}{:02X}{:02X}",
                rgb[0].as_u64().unwrap_or(0),
                rgb[1].as_u64().unwrap_or(0),
                rgb[2].as_u64().unwrap_or(0)
            )
        } else {
            wordcraft_gnome::t!("None").into()
        };
        let border_value = if mixed("borders") {
            wordcraft_gnome::t!("Mixed")
        } else if state["borders"].is_object()
            && state["borders"]
                .as_object()
                .is_some_and(|o| o.values().any(|v| !v.is_null()))
        {
            wordcraft_gnome::t!("Custom")
        } else {
            wordcraft_gnome::t!("None")
        };
        for (title, value, page, name, command, defaults, tip) in [
            (
                wordcraft_gnome::t!("Line Spacing"),
                spacing_value,
                "spacing",
                "paragraph-spacing",
                "para.lineSpacing",
                serde_json::json!({"value":self.spacing.get()}),
                wordcraft_gnome::t!("Apply Last Line Spacing"),
            ),
            (
                wordcraft_gnome::t!("Borders"),
                border_value.into(),
                "borders",
                "paragraph-borders",
                "para.borders",
                self.border.borrow().clone(),
                wordcraft_gnome::t!("Apply Last Border Settings"),
            ),
            (
                wordcraft_gnome::t!("Shading"),
                shade_value,
                "shading",
                "paragraph-shading",
                "para.shading",
                serde_json::json!({"color":self.shade.borrow().as_str(),"paragraph":true}),
                wordcraft_gnome::t!("Apply Last Settings"),
            ),
        ] {
            let label = wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("{label}: {value}"),
                &[("label", title.into()), ("value", value)],
            );
            let open = style_menu_button(&label);
            open.remove_css_class("flat");
            open.set_widget_name(&format!("paragraph-more-{page}"));
            open.set_hexpand(true);
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let text = gtk::Label::new(Some(&label));
            text.set_xalign(0.);
            text.set_hexpand(true);
            text.set_ellipsize(gtk::pango::EllipsizeMode::End);
            text.set_max_width_chars(28);
            content.append(&gtk::Image::from_icon_name(match page {
                "spacing" => "format-justify-fill-symbolic",
                "shading" => "color-select-symbolic",
                _ => "view-grid-symbolic",
            }));
            content.append(&text);
            open.set_child(Some(&content));
            open.set_tooltip_text(Some(&label));
            accessible_name(&open, &label);

            let weak = Rc::downgrade(self);
            open.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.page(page);
                }
            });
            let _args = self
                .last_applied
                .borrow()
                .get(command)
                .cloned()
                .unwrap_or(defaults);
            let apply = gtk::Button::new();

            apply.set_widget_name(name);
            apply.set_sensitive(self.groups.borrow().contains_key(page));
            let weak = Rc::downgrade(self);
            apply.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.repeat_group(page);
                }
            });
            apply.set_child(Some(&gtk::Image::from_icon_name("view-refresh-symbolic")));
            apply.set_tooltip_text(Some(tip));
            accessible_name(&apply, tip);
            let row = reapply_split(&open, &apply);
            row.set_widget_name(&format!("format-row-paragraph-{page}"));
            self.main.append(&row);
        }

        self.option_row(
            wordcraft_gnome::t!("More Paragraph Options"),
            "paragraph-more-options",
            "options",
        );
        self.stack.set_visible_child_name("main");
        if let Some(tools) = ui.quick_tools.borrow().as_ref() {
            tools.filter_menus();
        }
    }
    fn group_separator(&self) {
        if self.detail.first_child().is_some() {
            self.detail
                .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        }
    }
    pub(super) fn number(&self, label: &str, value: f64, min: f64, max: f64) -> gtk::SpinButton {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let l = gtk::Label::new(Some(label));
        l.set_xalign(0.0);
        l.set_hexpand(true);
        l.set_wrap(true);
        let icon = match self.active_page.borrow().as_deref() {
            Some("borders") => "view-grid-symbolic",
            Some("indents") => "format-indent-more-symbolic",
            Some("spacing") => "format-justify-fill-symbolic",
            Some("bullets" | "numbering") => "view-list-symbolic",
            _ => "paragraph-symbolic",
        };
        row.append(&gtk::Image::from_icon_name(icon));
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
    fn shading_controls(self: &Rc<Self>) {
        self.group_separator();
        let initial = self
            .ui
            .upgrade()
            .and_then(|u| rgb_value_hex(&selection_paragraph(&u)["shading"]));
        let weak = Rc::downgrade(self);
        let picker = embedded_color_chooser(
            "paragraph-shading-chooser",
            initial.as_deref(),
            move |hex| {
                let Some(s) = weak.upgrade() else {
                    return false;
                };
                let Some(ui) = s.ui.upgrade().filter(|u| !u.closed.get()) else {
                    return false;
                };
                let args = serde_json::json!({"color":hex,"paragraph":true});
                match immediate_format(&ui, "para.shading", args.clone()) {
                    Ok(()) => {
                        s.shade.replace(hex.into());
                        s.last_applied
                            .borrow_mut()
                            .insert("para.shading".into(), args);
                        true
                    }
                    Err(e) => {
                        s.message.set_text(&e);
                        s.message.set_visible(true);
                        false
                    }
                }
            },
        );
        self.detail.append(&picker.widget);
        self.button(
            wordcraft_gnome::t!("No Shading"),
            "paragraph-shade-none",
            "para.shading",
            serde_json::json!({"color":null,"paragraph":true}),
        );
    }
    pub(super) fn page(self: &Rc<Self>, page: &str) {
        self.active_page.replace(Some(page.to_string()));
        let weak = Rc::downgrade(self);
        let title = match page {
            "alignment" => wordcraft_gnome::t!("Alignment"),
            "list" => wordcraft_gnome::t!("List"),
            "bullets" => wordcraft_gnome::t!("Bullets"),
            "numbering" => wordcraft_gnome::t!("Numbering"),
            "indents" => wordcraft_gnome::t!("Indents"),
            "spacing" => wordcraft_gnome::t!("Spacing"),
            "direction" => wordcraft_gnome::t!("Text Direction"),
            "sort" => wordcraft_gnome::t!("Sort"),
            "tabs" => wordcraft_gnome::t!("Tab Stops"),
            "borders" => wordcraft_gnome::t!("Borders"),
            "border-color" => wordcraft_gnome::t!("Color"),
            "shading" => wordcraft_gnome::t!("Shading"),
            "columns" => wordcraft_gnome::t!("Columns"),
            _ => wordcraft_gnome::t!("More Paragraph Options"),
        };
        let nested = matches!(page, "bullets" | "numbering" | "border-color");
        let border_color = page == "border-color";
        self.header.page(page, title, move || {
            if let Some(s) = weak.upgrade() {
                if nested {
                    s.page(if border_color { "borders" } else { "list" });
                } else {
                    s.home();
                }
            }
        });
        clear_box(&self.fixed);
        clear_box(&self.detail);
        self.message.set_visible(false);
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let state = selection_paragraph(&ui);
        if paragraph_options::build(self, &ui, page, &state) {
            self.stack.set_visible_child_name("detail");
            if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                tools.filter_menus();
            }
            return;
        }
        match page {
            "list" => {
                for (title, target) in [
                    (wordcraft_gnome::t!("Bulleted List"), "bullets"),
                    (wordcraft_gnome::t!("Numbered List"), "numbering"),
                ] {
                    let b = style_menu_button(title);
                    b.set_child(Some(&list_choice_content(
                        title,
                        if target == "bullets" {
                            "view-list-bullet-symbolic"
                        } else {
                            "view-list-ordered-symbolic"
                        },
                    )));
                    b.set_widget_name(&format!("paragraph-more-{target}"));
                    self.detail.append(&b);
                    let weak = Rc::downgrade(self);
                    b.connect_clicked(move |_| {
                        if let Some(s) = weak.upgrade() {
                            s.page(target);
                        }
                    });
                }
                self.button(
                    wordcraft_gnome::t!("Remove List"),
                    "paragraph-list-none",
                    "para.bullets",
                    serde_json::json!({"off":true}),
                );
            }
            "alignment" => {
                for (title, value) in [
                    (wordcraft_gnome::t!("Align Left"), "left"),
                    (wordcraft_gnome::t!("Center"), "center"),
                    (wordcraft_gnome::t!("Align Right"), "right"),
                    (wordcraft_gnome::t!("Justify"), "justify"),
                    (wordcraft_gnome::t!("Distributed Alignment"), "distribute"),
                ] {
                    self.button(
                        title,
                        &format!("paragraph-align-{value}"),
                        "para.align",
                        serde_json::json!({"value":value}),
                    );
                }
            }
            "columns" => {
                self.group_separator();
                let weak = Rc::downgrade(self);
                let editor = columns_editor(&ui, move || {
                    if let Some(s) = weak.upgrade() {
                        if !s
                            .ui
                            .upgrade()
                            .is_some_and(|ui| ui.format_group.pinned.get())
                        {
                            s.home();
                        }
                        if let Some(pop) = s.popover.upgrade() {
                            close_format_popover(&pop);
                        }
                    }
                });
                if let Some(parent) = editor.scope_header.parent().and_downcast::<gtk::Box>() {
                    parent.remove(&editor.scope_header);
                }
                self.fixed.append(&editor.scope_header);
                self.detail.append(&editor.root);
                self.detail.append(&editor.apply);
            }
            "bullets" | "numbering" => {
                self.group_separator();
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
                self.group_separator();
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
                for (title, name, command) in [
                    (
                        wordcraft_gnome::t!("Increase Indent"),
                        "paragraph-indent",
                        "para.indent",
                    ),
                    (
                        wordcraft_gnome::t!("Decrease Indent"),
                        "paragraph-outdent",
                        "para.outdent",
                    ),
                ] {
                    self.detail
                        .append(&self.action(title, name, command, serde_json::json!({})));
                }

                self.detail
                    .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
                let left = self.number(
                    wordcraft_gnome::t!("Left (pt)"),
                    state["indentLeft"].as_f64().unwrap_or(0.0),
                    -1584.0,
                    1584.0,
                );
                let right = self.number(
                    wordcraft_gnome::t!("Right (pt)"),
                    state["indentRight"].as_f64().unwrap_or(0.0),
                    -1584.0,
                    1584.0,
                );
                let first = self.number(
                    wordcraft_gnome::t!("First Line (pt; −: Hanging)"),
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
                self.group_separator();
                for (value, label) in [
                    (1.0, wordcraft_gnome::t!("Single")),
                    (1.15, wordcraft_gnome::t!("1.15 Lines")),
                    (1.5, wordcraft_gnome::t!("1.5 Lines")),
                    (2.0, wordcraft_gnome::t!("Double")),
                ] {
                    let b = style_menu_button(label);
                    b.set_widget_name(&format!("paragraph-line-{value}"));
                    b.set_child(Some(
                        &property_content(label, "format-justify-fill-symbolic").0,
                    ));
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
                    if state["lineSpacing"]["rule"] == "exact" {
                        state["lineSpacing"]["value"].as_f64().unwrap_or(14.0)
                    } else {
                        14.0
                    },
                    1.0,
                    500.0,
                );
                exact.set_widget_name("paragraph-exact-line-spacing");
                if state["lineSpacing"]["rule"] != "exact" {
                    exact.set_text("");
                }
                self.number_action(&exact, "para.lineSpacing", "exactly", "lineSpacing.value");
                self.group_separator();
                let before = self.number(
                    wordcraft_gnome::t!("Before (pt)"),
                    state["spaceBefore"].as_f64().unwrap_or(0.0),
                    0.0,
                    1584.0,
                );
                let after = self.number(
                    wordcraft_gnome::t!("After (pt)"),
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
                let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
                    "Between paragraphs, the larger adjoining Before or After value is used. Styles can suppress this spacing."
                )));
                hint.set_widget_name("paragraph-spacing-hint");
                hint.set_wrap(true);
                hint.set_max_width_chars(38);
                hint.set_xalign(0.0);
                hint.set_margin_top(6);
                hint.add_css_class("dim-label");
                self.detail.append(&hint);
            }
            "shading" => self.shading_controls(),
            "borders" => {
                let width = self.number(wordcraft_gnome::t!("Width (pt)"), 0.5, 0.25, 12.0);
                if let Some(edges) = state["borders"].as_object() {
                    let widths = edges
                        .values()
                        .filter_map(|v| v["width"].as_f64())
                        .collect::<Vec<_>>();
                    if let Some(first) = widths.first() {
                        if widths.iter().all(|n| (n - first).abs() < 0.001) {
                            width.set_value(*first);
                        } else {
                            width.set_text("");
                            width.set_tooltip_text(Some(wordcraft_gnome::t!("Mixed")));
                        }
                    }
                }
                let color = gtk::Entry::new();
                color.set_placeholder_text(Some(wordcraft_gnome::t!(
                    "RGB Color, for Example 2266AA"
                )));
                color.set_text("000000");
                if let Some(hex) = state["borders"]
                    .as_object()
                    .and_then(|b| b.values().find_map(|v| rgb_value_hex(&v["color"])))
                {
                    color.set_text(hex.trim_start_matches('#'));
                }
                let entry = color.clone();
                let weak = Rc::downgrade(self);
                let picker = embedded_color_chooser(
                    "paragraph-border-chooser",
                    Some(&format!("#{}", color.text())),
                    move |hex| {
                        entry.set_text(hex);
                        let Some(s) = weak.upgrade() else {
                            return false;
                        };
                        let Some(ui) = s.ui.upgrade().filter(|u| !u.closed.get()) else {
                            return false;
                        };
                        let current = selection_paragraph(&ui);
                        let has_border = current["borders"]
                            .as_object()
                            .is_some_and(|b| b.values().any(|v| v.is_object()));
                        let mut args = s.border.borrow().clone();
                        args["color"] = serde_json::json!(hex);
                        s.border.replace(args.clone());
                        if !has_border {
                            return true;
                        }
                        let mut borders = current["borders"].clone();
                        if let Some(edges) = borders.as_object_mut() {
                            for edge in edges.values_mut().filter(|v| v.is_object()) {
                                edge["color"] = serde_json::json!(hex);
                            }
                        }
                        match immediate_format(
                            &ui,
                            "para.set",
                            serde_json::json!({"props":{"borders":borders}}),
                        ) {
                            Ok(()) => {
                                s.remember_property("para.borders", &args, &ui);
                                true
                            }
                            Err(e) => {
                                s.message.set_text(&e);
                                s.message.set_visible(true);
                                false
                            }
                        }
                    },
                );
                self.detail.append(&picker.widget);
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
                    b.set_child(Some(&property_content(label, "view-grid-symbolic").0));
                    self.detail.append(&b);
                    let weak = Rc::downgrade(self);
                    let width = width.clone();
                    let color = color.clone();
                    b.connect_clicked(move |_| {
                        if let Some(panel) = weak.upgrade() {
                            width.update();
                            if let Some(hex) = panel.valid_color(&color) {
                                let args = serde_json::json!({
                                    "kind": kind,
                                    "width": width.value(),
                                    "color": hex,
                                });
                                panel.border.replace(args.clone());
                                panel.execute("para.borders", args);
                            }
                        }
                    });
                }
            }
            _ => {
                self.group_separator();
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
                self.group_separator();
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
                self.group_separator();
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
                self.group_separator();
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
        b.set_widget_name(&format!(
            "format-row-paragraph-{}",
            command.rsplit('.').next().unwrap_or("")
        ));
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
}
pub(super) fn install_paragraph_popover(
    ui: &Rc<Ui>,
    menu: &gtk::MenuButton,
) -> Rc<ParagraphPopover> {
    let pop = gtk::Popover::new();
    pop.add_css_class("format-popover");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_width_request(270);
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
    detail.set_margin_bottom(8);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(420);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&detail));
    let detail_page = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let fixed = gtk::Box::new(gtk::Orientation::Vertical, 6);
    detail_page.append(&fixed);
    detail_page.append(&scroll);
    let main_scroll = gtk::ScrolledWindow::new();
    main_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    main_scroll.set_max_content_height(430);
    main_scroll.set_propagate_natural_height(true);
    main_scroll.set_child(Some(&main));
    stack.add_named(&main_scroll, Some("main"));
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
        fixed,
        message,
        header,
        last_applied: RefCell::new(std::collections::HashMap::new()),
        spacing: Cell::new(1.5),
        shade: RefCell::new("FFF2CC".into()),
        border: RefCell::new(serde_json::json!({"kind":"outside","width":0.5,"color":"000000"})),
        groups: RefCell::new(std::collections::HashMap::new()),
        last_columns: RefCell::new(None),
        active_page: RefCell::new(None),
        displayed: RefCell::new(serde_json::Value::Null),
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
        }
    ));
    menu.set_popover(Some(&pop));
    s
}
pub(super) fn show_paragraph_options(ui: &Rc<Ui>, _parent: &ApplicationWindow) {
    format_bar::open_group(ui, "paragraph", Some("options"));
}
