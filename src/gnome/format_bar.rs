//! Native toolbar placement derived from persistent application preferences.
//! Existing panels and Session commands own every action in either location.
use super::*;
use wordcraft_gnome::toolbar::Placement;

pub(super) struct QuickTools {
    style_name: gtk::Label,
    alignment: gtk::Button,
    ui: std::rc::Weak<Ui>,
    width: Cell<i32>,
    opening: Cell<bool>,
    groups: Vec<(&'static str, gtk::Box, Vec<(String, gtk::Widget)>)>,
    toggles: Vec<(gtk::Button, gtk::Label, &'static str)>,
    pub(super) overflow: gtk::MenuButton,
    overflow_pop: gtk::Popover,
    overflow_main: gtk::Box,
    origins: Vec<(&'static str, gtk::Popover)>,
    swatches: Vec<(&'static str, gtk::DrawingArea)>,
}

fn icon_button(icon: &str, label: &str, name: &str) -> gtk::Button {
    let button = gtk::Button::from_icon_name(icon);
    button.add_css_class("flat");
    button.set_widget_name(name);
    button.set_tooltip_text(Some(label));
    accessible_name(&button, label);
    button
}

fn shortcut(
    ui: &Rc<Ui>,
    button: gtk::Button,
    icon: &str,
    label: &str,
    key: &'static str,
    command: &'static str,
) -> (gtk::Button, gtk::Label, &'static str) {
    button.set_widget_name(&format!("format-quick-{key}"));
    button.set_tooltip_text(Some(label));
    accessible_name(&button, label);
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&gtk::Image::from_icon_name(icon)));
    let mixed = gtk::Label::new(Some("−"));
    mixed.add_css_class("heading");
    mixed.set_halign(gtk::Align::End);
    mixed.set_valign(gtk::Align::End);
    mixed.set_visible(false);
    mixed.set_widget_name(&format!("format-quick-{key}-mixed"));
    overlay.add_overlay(&mixed);
    button.set_child(Some(&overlay));
    let weak = Rc::downgrade(ui);
    button.connect_clicked(move |_| {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if matches!(key, "underline" | "strike" | "superscript") {
            let palette = ui
                .text_tools
                .borrow()
                .as_ref()
                .and_then(|t| t.palette.borrow().clone());
            if let Some(palette) = palette {
                let check = match key {
                    "underline" => &palette.underline_check,
                    "strike" => &palette.strike_check,
                    _ => &palette.superscript,
                };
                check.set_active(!check.is_active() || check.is_inconsistent());
            }
            ui.document_box.grab_focus();
            caret_ensure_visible(&ui);
            return;
        }
        let args = if matches!(key, "bullets" | "numbering") {
            serde_json::json!({})
        } else {
            let state = selection_character(&ui);
            let active = if key == "underline" {
                state[key].as_str().is_some_and(|v| v != "none")
            } else {
                state[key].as_bool().unwrap_or(false)
            };
            serde_json::json!({"value": !active})
        };
        if let Err(error) = format_execute(&ui, command, args) {
            ui.toasts.add_toast(Toast::new(&error));
        }
        ui.document_box.grab_focus();
        caret_ensure_visible(&ui);
    });
    (button, mixed, key)
}

impl QuickTools {
    pub(super) fn width(&self, width: i32) {
        if self.width.replace(width) != width {
            self.layout();
        }
    }

    pub(super) fn layout(&self) {
        let mut measured = Vec::new();
        let mut pinned = 0;
        for (group, _, tools) in &self.groups {
            for (id, mode) in wordcraft_gnome::preferences::toolbar_placements(group) {
                let widget = &tools.iter().find(|(key, _)| key == &id).unwrap().1;
                widget.set_visible(true);
                let cost = widget.measure(gtk::Orientation::Horizontal, -1).1.max(34) + 6;
                if mode == Placement::Pinned {
                    pinned += cost;
                }
                measured.push((*group, id, mode, cost));
            }
        }
        // Style, fixed overflow, group separators and normal native spacing.
        let mut available = (self.width.get() - 270 - pinned).max(0);
        for (group, container, tools) in &self.groups {
            let mut previous: Option<gtk::Widget> = None;
            for (_, id, mode, cost) in measured.iter().filter(|(key, _, _, _)| key == group) {
                let widget = &tools.iter().find(|(key, _)| key == id).unwrap().1;
                container.reorder_child_after(widget, previous.as_ref());
                previous = Some(widget.clone());
                let visible = *mode == Placement::Pinned
                    || (*mode == Placement::Adaptive && *cost <= available);
                widget.set_visible(visible);
                if visible && *mode == Placement::Adaptive {
                    available -= cost;
                }
            }
        }
        self.filter_menus();
        self.rebuild_overflow();
    }

    fn rebuild_overflow(&self) {
        clear_box(&self.overflow_main);
        for (index, (group, _, tools)) in self.groups.iter().enumerate() {
            if index > 0 {
                self.overflow_main
                    .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            }
            let label = match *group {
                "text" => wordcraft_gnome::t!("Text"),
                "paragraph" => wordcraft_gnome::t!("Paragraph"),
                _ => wordcraft_gnome::t!("Elements"),
            };
            let title = gtk::Label::new(Some(label));
            title.set_xalign(0.);
            title.add_css_class("heading");
            title.set_widget_name(&format!("overflow-section-{group}"));
            self.overflow_main.append(&title);
            for (id, mode) in wordcraft_gnome::preferences::toolbar_placements(group) {
                let widget = &tools.iter().find(|(key, _)| key == &id).unwrap().1;
                if mode != Placement::Adaptive || widget.get_visible() {
                    continue;
                }
                let catalog = wordcraft_gnome::toolbar::tools(group);
                let tool = catalog.iter().find(|t| t.id == id).unwrap();
                if tool_actions::split_tool(group, &id) {
                    let apply = style_menu_button(tool.label);
                    apply.set_widget_name(&format!("overflow-tool-{group}-{id}"));
                    let open = gtk::Button::from_icon_name("go-next-symbolic");
                    let (weak, g, key) = (self.ui.clone(), *group, id.clone());
                    apply.connect_clicked(move |_| {
                        if let Some(ui) = weak.upgrade() {
                            tool_actions::activate(&ui, g, &key);
                        }
                    });
                    let (weak, g, key) = (self.ui.clone(), *group, id.clone());
                    open.connect_clicked(move |_| {
                        if let Some(ui) = weak.upgrade() {
                            if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                                tools.open_group(g, Some(&key));
                            }
                        }
                    });
                    let split = property_split(&apply, &open);
                    split
                        .last_child()
                        .unwrap()
                        .downcast::<gtk::MenuButton>()
                        .unwrap()
                        .set_icon_name("go-next-symbolic");
                    self.overflow_main.append(&split);
                    continue;
                }
                let button = style_menu_button(tool.label);
                button.set_widget_name(&format!("overflow-tool-{group}-{id}"));
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                row.append(&gtk::Image::from_icon_name(tool.icon));
                let label = gtk::Label::new(Some(tool.label));
                label.set_hexpand(true);
                label.set_xalign(0.);
                row.append(&label);
                let state = gtk::Label::new(None);
                state.set_widget_name(&format!("overflow-state-{group}-{id}"));
                row.append(&state);
                button.set_child(Some(&row));
                let (weak, group, id, widget) = (self.ui.clone(), *group, id, widget.clone());
                button.connect_clicked(move |_| {
                    let Some(ui) = weak.upgrade() else { return };
                    if let Some(button) = widget.downcast_ref::<gtk::Button>() {
                        button.emit_clicked();
                        if !ui.format_group.pinned.get() {
                            if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                                tools.overflow.popdown();
                            }
                        }
                    } else if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                        tools.open_group(group, Some(&id));
                    }
                });
                self.overflow_main.append(&button);
            }
            let label = match *group {
                "text" => wordcraft_gnome::t!("More Text Options…"),
                "paragraph" => wordcraft_gnome::t!("More Paragraph Options…"),
                _ => wordcraft_gnome::t!("More Elements Options…"),
            };
            let more = style_menu_button(label);
            more.set_widget_name(&format!("overflow-more-{group}"));
            let (weak, group) = (self.ui.clone(), *group);
            more.connect_clicked(move |_| {
                if let Some(ui) = weak.upgrade() {
                    if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                        tools.open_group(group, None);
                    }
                }
            });
            self.overflow_main.append(&more);
        }
        self.overflow_main
            .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        let customize = style_menu_button(wordcraft_gnome::t!("Customize Toolbar"));
        customize.set_widget_name("overflow-customize-toolbar");
        let weak = self.ui.clone();
        customize.connect_clicked(move |_| {
            let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                return;
            };
            if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                tools.overflow.popdown();
            }
            if let Some(host) = ui.host.upgrade() {
                let dialog = show_preferences(&host.window);
                dialog.set_visible_page_name("toolbar");
            }
        });
        self.overflow_main.append(&customize);
    }

    pub(super) fn root(&self) {
        if let Some(pop) = self.overflow.popover() {
            pop.set_visible(false);
        }
        if let Some(owner) = self.overflow_pop.parent().and_downcast::<gtk::MenuButton>() {
            owner.set_popover(None::<&gtk::Popover>);
        }
        self.overflow.set_popover(Some(&self.overflow_pop));
        self.overflow.popup();
    }
    pub(super) fn open_group(&self, group: &str, page: Option<&str>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        let Some((_, pop)) = self.origins.iter().find(|(key, _)| *key == group) else {
            return;
        };
        if let Some(current) = self.overflow.popover() {
            current.set_visible(false);
        }
        pop.set_visible(false);
        if let Some(owner) = pop.parent().and_downcast::<gtk::MenuButton>() {
            owner.set_popover(None::<&gtk::Popover>);
        }
        // Restore any body leased by a context menu before preparing its page.
        ui.format_group.show(pop);
        let header = ui
            .format_group
            .entries
            .borrow()
            .iter()
            .find(|(p, _)| p.upgrade().as_ref() == Some(pop))
            .map(|(_, h)| h.clone())
            .unwrap();
        header.entry_page.replace(page.map(str::to_owned));
        let weak = self.ui.clone();
        header.parent_back.replace(Some(Rc::new(move || {
            if let Some(ui) = weak.upgrade() {
                if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                    tools.root();
                }
            }
        })));
        pop.set_pointing_to(None::<&gdk::Rectangle>);
        self.overflow.set_popover(Some(pop));
        match group {
            "text" => {
                let panel = ui.text_tools.borrow().as_ref().unwrap().clone();
                let transition = panel.stack.transition_type();
                panel
                    .stack
                    .set_transition_type(gtk::StackTransitionType::None);
                reset_entry_stack(&panel.stack, "main", "detail");
                if let Some(page) = page {
                    panel.page(page);
                } else {
                    panel.home();
                }
                panel.stack.set_transition_type(transition);
            }
            "paragraph" => {
                let panel = ui.paragraph.borrow().as_ref().unwrap().clone();
                let transition = panel.stack.transition_type();
                panel
                    .stack
                    .set_transition_type(gtk::StackTransitionType::None);
                reset_entry_stack(&panel.stack, "main", "detail");
                if let Some(page) = page {
                    panel.page(page);
                } else {
                    panel.home();
                }
                panel.stack.set_transition_type(transition);
            }
            _ => {
                let panel = ui.elements.borrow().as_ref().unwrap().clone();
                let transition = panel.stack.transition_type();
                panel
                    .stack
                    .set_transition_type(gtk::StackTransitionType::None);
                reset_entry_stack(&panel.stack, "home", "toc");
                if let Some(page) = page {
                    panel.requested_page.replace(Some(page.into()));
                    panel.prepare(page);
                } else {
                    panel.home();
                }
                panel.stack.set_transition_type(transition);
            }
        }
        self.filter_menus();
        self.opening.set(true);
        self.overflow.popup();
        self.opening.set(false);
    }

    pub(super) fn filter_menus(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        for (group, _, tools) in &self.groups {
            let root = match *group {
                "paragraph" => ui
                    .paragraph
                    .borrow()
                    .as_ref()
                    .map(|p| p.stack.clone().upcast::<gtk::Widget>()),
                "text" => ui
                    .text_tools
                    .borrow()
                    .as_ref()
                    .map(|t| t.stack.clone().upcast::<gtk::Widget>()),
                _ => ui.elements.borrow().as_ref().and_then(|p| p.pop.child()),
            };
            if let Some(root) = root {
                for (id, _widget) in tools {
                    if let Some(row) = find_widget(&root, &format!("format-row-{group}-{id}")) {
                        let mode = wordcraft_gnome::preferences::toolbar_placements(group)
                            .into_iter()
                            .find(|(key, _)| key == id)
                            .unwrap()
                            .1;
                        row.set_visible(mode == Placement::More);
                    }
                }
            }
        }
    }

    pub(super) fn sync(&self, ui: &Rc<Ui>, character: &serde_json::Value) {
        let current = ui.current.borrow();
        let style = current
            .as_ref()
            .and_then(|d| wordcraft_gnome::styles::current(&d.session));
        let name = current
            .as_ref()
            .and_then(|d| d.session.doc.styles.get(style.as_deref()?))
            .map(|s| s.name.as_str())
            .unwrap_or(wordcraft_gnome::t!("Mixed"));
        self.style_name.set_text(name);
        self.style_name.set_tooltip_text(Some(name));
        if let Some(menu) = self
            .style_name
            .ancestor(gtk::MenuButton::static_type())
            .and_downcast::<gtk::MenuButton>()
        {
            menu.set_tooltip_text(Some(name));
            accessible_name(&menu, name);
        }
        drop(current);
        for (group, _, tools) in &self.groups {
            if *group != "text" {
                continue;
            }
            let palette = ui
                .text_tools
                .borrow()
                .as_ref()
                .and_then(|t| t.palette.borrow().clone());
            if let Some(palette) = palette {
                for (id, widget) in tools {
                    let source = match id.as_str() {
                        "ink" => Some(&palette.ink),
                        "highlight" => Some(&palette.highlight),
                        _ => None,
                    };
                    if let Some(source) = source {
                        widget.set_tooltip_text(source.tooltip_text().as_deref());
                    }
                }
            }
        }
        for (key, swatch) in &self.swatches {
            let saved = ui
                .text_tools
                .borrow()
                .as_ref()
                .and_then(|t| t.palette.borrow().clone())
                .map(|p| {
                    if *key == "ink" {
                        p.last_ink.borrow().clone()
                    } else {
                        p.last_highlight.borrow().clone()
                    }
                })
                .unwrap_or_default();
            let color = if saved.len() == 6 && saved.bytes().all(|b| b.is_ascii_hexdigit()) {
                format!("#{saved}")
            } else {
                saved.clone()
            };
            let rgb = gdk::RGBA::parse(&color)
                .ok()
                .map(|c| (c.red() as f64, c.green() as f64, c.blue() as f64));
            let none = *key == "highlight" && saved == "none";
            swatch.set_draw_func(move |w, c, width, height| {
                if let Some((r, g, b)) = rgb {
                    c.set_source_rgb(r, g, b);
                } else {
                    let color = w.color();
                    c.set_source_rgba(
                        color.red() as f64,
                        color.green() as f64,
                        color.blue() as f64,
                        color.alpha() as f64,
                    );
                }
                if none {
                    c.set_line_width(1.);
                    c.move_to(0., height as f64);
                    c.line_to(width as f64, 0.);
                    let _ = c.stroke();
                } else {
                    c.rectangle(0., 0., width as f64, height as f64);
                    let _ = c.fill();
                }
            });
            swatch.queue_draw();
        }
        for (group, _, tools) in &self.groups {
            for (id, widget) in tools {
                widget.set_sensitive(tool_actions::enabled(ui, group, id));
                if *group == "elements"
                    && (id.starts_with("image-")
                        || id.starts_with("table-")
                        || id.starts_with("cell-"))
                {
                    if !widget.is_sensitive() {
                        widget.set_tooltip_text(Some(if id.starts_with("image-") {
                            wordcraft_gnome::t!("Select an image in the document to edit it.")
                        } else {
                            wordcraft_gnome::t!("Place the caret in a table.")
                        }));
                    }
                }
                if id == "language" {
                    if let Some(body) = find_widget(widget, "format-quick-language") {
                        body.set_sensitive(
                            ui.text_tools
                                .borrow()
                                .as_ref()
                                .is_some_and(|t| t.language_remembered()),
                        );
                    }
                }
                if id == "size" {
                    if let Some(spin) = find_widget(widget, "format-quick-size-input")
                        .and_downcast::<gtk::SpinButton>()
                        .filter(|s| !control_has_focus(s))
                    {
                        let old = ui.format_syncing.replace(true);
                        if let Some(size) = character["size"].as_f64() {
                            spin.set_value(size);
                        } else {
                            spin.set_text("");
                        }
                        ui.format_syncing.set(old);
                    }
                }
            }
        }
        let paragraph = selection_paragraph(ui);
        let remembered = ui
            .paragraph
            .borrow()
            .as_ref()
            .and_then(|p| p.groups.borrow().get("alignment").cloned());
        let alignment = remembered
            .as_ref()
            .and_then(|v| v["props"]["align"].as_str());
        let (icon, label) = match alignment {
            Some("left") => (
                "format-justify-left-symbolic",
                wordcraft_gnome::t!("Align Left"),
            ),
            Some("right") => (
                "format-justify-right-symbolic",
                wordcraft_gnome::t!("Align Right"),
            ),
            Some("center") => (
                "format-justify-center-symbolic",
                wordcraft_gnome::t!("Center"),
            ),
            Some("justify" | "distribute") => (
                "format-justify-fill-symbolic",
                wordcraft_gnome::t!("Justify"),
            ),
            _ => ("format-justify-left-symbolic", wordcraft_gnome::t!("Mixed")),
        };
        if let Some(image) = self
            .alignment
            .child()
            .and_then(|w| w.downcast::<gtk::Overlay>().ok())
            .and_then(|w| w.child())
            .and_downcast::<gtk::Image>()
        {
            image.set_icon_name(Some(icon));
        }
        self.alignment
            .set_tooltip_text(Some(&wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Alignment: {value}"),
                &[("value", label.into())],
            )));
        let list = ui.current.borrow().as_ref().and_then(|d| {
            let p = d.session.doc.para_at(&d.session.sel.focus)?;
            let n = d.session.doc.styles.resolve_para(&p.props).numbering?;
            d.session.doc.numbering.level(n.num, n.level).map(|level| {
                format!("{:?}", level.format)
                    .to_lowercase()
                    .contains("bullet")
            })
        });
        let mixed_list = paragraph["_mixed"]
            .as_array()
            .is_some_and(|fields| fields.iter().any(|p| p == "numbering"));
        for (button, marker, key) in &self.toggles {
            let (active, mixed) = match *key {
                "bullets" => (list == Some(true), mixed_list),
                "numbering" => (list == Some(false), mixed_list),
                "strike" => (
                    character["strike"] == true || character["doubleStrike"] == true,
                    character["strike"].is_null() || character["doubleStrike"].is_null(),
                ),
                "superscript" => (
                    character["vertAlign"] == "superscript",
                    character["vertAlign"].is_null(),
                ),
                "underline" => (
                    character[*key].as_str().is_some_and(|v| v != "none"),
                    character[*key].is_null(),
                ),
                "keepNext" | "keepLines" | "pageBreakBefore" | "widowControl" => {
                    (paragraph[*key] == true, paragraph[*key].is_null())
                }
                "marks" => (
                    ui.current
                        .borrow()
                        .as_ref()
                        .is_some_and(|d| d.session.view.marks),
                    false,
                ),
                "text-border" => (character["border"].is_object(), false),
                _ => (
                    character[*key].as_bool().unwrap_or(false),
                    character[*key].is_null(),
                ),
            };
            if let Ok(toggle) = button.clone().downcast::<gtk::ToggleButton>() {
                toggle.set_active(active && !mixed);
            } else if active && !mixed {
                button.set_state_flags(gtk::StateFlags::CHECKED, false);
            } else {
                button.unset_state_flags(gtk::StateFlags::CHECKED);
            }
            marker.set_visible(mixed);
            if let Some(split) = button.parent().and_downcast::<adw::SplitButton>() {
                if active && !mixed {
                    split.add_css_class("split-active");
                } else {
                    split.remove_css_class("split-active");
                }
            }
            if let Some(status) =
                find_widget(&self.overflow_main, &format!("overflow-state-text-{key}"))
                    .and_downcast::<gtk::Label>()
            {
                status.set_text(if mixed {
                    "−"
                } else if active {
                    "✓"
                } else {
                    ""
                });
            }
            if let Some(proxy) =
                find_widget(&self.overflow_main, &format!("overflow-tool-text-{key}"))
            {
                proxy.update_state(&[gtk::accessible::State::Pressed(if mixed {
                    gtk::AccessibleTristate::Mixed
                } else if active {
                    gtk::AccessibleTristate::True
                } else {
                    gtk::AccessibleTristate::False
                })]);
            }
            button.update_state(&[gtk::accessible::State::Pressed(if mixed {
                gtk::AccessibleTristate::Mixed
            } else if active {
                gtk::AccessibleTristate::True
            } else {
                gtk::AccessibleTristate::False
            })]);
        }
    }
}

pub(super) fn install_quick_tools(
    ui: &Rc<Ui>,
    formats: &gtk::Box,
    style: &gtk::MenuButton,
    paragraph: &gtk::MenuButton,
    text: &gtk::MenuButton,
    elements: &Rc<ElementsPanel>,
) -> Rc<QuickTools> {
    for menu in [style, paragraph, text, &elements.menu] {
        formats.remove(menu);
    }
    formats.set_spacing(6);
    let style_group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    style_group.set_widget_name("format-style-group");
    style_group.set_width_request(184);
    style_group.set_hexpand(false);
    style.remove_css_class("flat");
    style.set_has_frame(true);
    style.add_css_class("raised");
    style.remove_css_class("image-button");
    style.set_label("Normal");
    style.set_always_show_arrow(true);
    style.set_hexpand(true);
    let style_name = first_label(style.upcast_ref()).expect("native style dropdown label");
    style_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    style_name.set_max_width_chars(16);
    style_name.set_width_chars(16);
    style_group.append(style);
    formats.append(&style_group);
    let paragraph_group = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let text_group = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let elements_group = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    for (group, name) in [
        (&text_group, "text"),
        (&paragraph_group, "paragraph"),
        (&elements_group, "elements"),
    ] {
        formats.append(&gtk::Separator::new(gtk::Orientation::Vertical));
        group.set_widget_name(&format!("format-{name}-group"));
        formats.append(group);
    }
    let mut groups = Vec::new();
    let mut toggles = Vec::new();
    let mut alignment = None;
    let mut swatches = Vec::new();
    for (group, container) in [
        ("text", &text_group),
        ("paragraph", &paragraph_group),
        ("elements", &elements_group),
    ] {
        let shortcuts = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        container.append(&shortcuts);
        let mut tools = Vec::new();
        for tool in wordcraft_gnome::toolbar::tools(group) {
            let widget: gtk::Widget = if group == "text"
                && matches!(tool.id, "bold" | "italic" | "underline" | "strike")
            {
                let command = match tool.id {
                    "bold" => "format.bold",
                    "italic" => "format.italic",
                    "underline" => "format.underline",
                    "strike" => "format.strikethrough",
                    _ => "format.superscript",
                };
                if matches!(tool.id, "underline" | "strike") {
                    let split = adw::SplitButton::new();
                    split.add_css_class("flat");
                    split.add_css_class("image-button");
                    split.add_css_class("leaf-format-split");
                    split.set_valign(gtk::Align::Center);
                    split.set_direction(gtk::ArrowType::Up);
                    // The documented native hierarchy is the same as Undo.
                    let body = split
                        .first_child()
                        .unwrap()
                        .downcast::<gtk::Button>()
                        .unwrap();
                    let options = body
                        .next_sibling()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .downcast::<gtk::MenuButton>()
                        .unwrap();
                    options.set_widget_name(&format!("format-quick-{}-options", tool.id));
                    let label = wordcraft_gnome::i18n::interpolate(
                        wordcraft_gnome::t!("Options for {label}"),
                        &[("label", tool.label.into())],
                    );
                    accessible_name(&options, &label);
                    options.set_tooltip_text(Some(&label));
                    connect_picker(
                        ui,
                        &options,
                        if group == "paragraph" {
                            paragraph
                        } else {
                            text
                        },
                        group,
                        Some(tool.id),
                    );
                    toggles.push(shortcut(ui, body, tool.icon, tool.label, tool.id, command));
                    split.upcast()
                } else {
                    let toggle = gtk::ToggleButton::new();
                    toggle.add_css_class("flat");
                    let item = shortcut(
                        ui,
                        toggle.clone().upcast(),
                        tool.icon,
                        tool.label,
                        tool.id,
                        command,
                    );
                    toggles.push(item);
                    toggle.upcast()
                }
            } else if tool_actions::split_tool(group, tool.id) {
                let split = adw::SplitButton::new();
                split.add_css_class("flat");
                split.add_css_class("image-button");
                split.add_css_class("leaf-format-split");
                split.set_direction(gtk::ArrowType::Up);
                split.set_valign(gtk::Align::Center);
                let body = split
                    .first_child()
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap();
                let options = split
                    .last_child()
                    .unwrap()
                    .downcast::<gtk::MenuButton>()
                    .unwrap();
                body.set_widget_name(&format!("format-quick-{}", tool.id));
                body.set_tooltip_text(Some(tool.label));
                accessible_name(&body, tool.label);
                let icon = gtk::Overlay::new();
                icon.set_child(Some(&gtk::Image::from_icon_name(tool.icon)));
                if matches!(tool.id, "ink" | "highlight") {
                    let swatch = gtk::DrawingArea::new();
                    swatch.set_widget_name(&format!("format-quick-{}-swatch", tool.id));
                    swatch.set_content_width(16);
                    swatch.set_content_height(3);
                    swatch.set_valign(gtk::Align::End);
                    icon.add_overlay(&swatch);
                    swatches.push((tool.id, swatch));
                }
                body.set_child(Some(&icon));
                let (weak, id) = (Rc::downgrade(ui), tool.id);
                let group = group;
                body.connect_clicked(move |_| {
                    if let Some(ui) = weak.upgrade() {
                        tool_actions::activate(&ui, group, id);
                    }
                });
                if group == "paragraph" && tool.id == "alignment" {
                    alignment = Some(body.clone());
                }
                options.set_widget_name(&format!("format-quick-{}-options", tool.id));
                let label = wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Options for {label}"),
                    &[("label", tool.label.into())],
                );
                accessible_name(&options, &label);
                options.set_tooltip_text(Some(&label));
                connect_picker(
                    ui,
                    &options,
                    if group == "paragraph" {
                        paragraph
                    } else {
                        text
                    },
                    group,
                    Some(tool.id),
                );
                split.upcast()
            } else if group == "text" && tool.id == "size" {
                let control = tool_actions::sizing_control(ui);
                control.set_widget_name("format-quick-size");
                control.upcast()
            } else if tool_actions::direct(group, tool.id) {
                let button =
                    icon_button(tool.icon, tool.label, &format!("format-quick-{}", tool.id));
                let (weak, group, id) = (Rc::downgrade(ui), group, tool.id);
                button.connect_clicked(move |_| {
                    if let Some(ui) = weak.upgrade() {
                        tool_actions::activate(&ui, group, id);
                    }
                });
                if tool_actions::toggle_command(group, tool.id).is_some() {
                    let mixed = gtk::Label::new(Some("−"));
                    mixed.set_visible(false);
                    let icon = gtk::Overlay::new();
                    icon.set_child(Some(&gtk::Image::from_icon_name(tool.icon)));
                    icon.add_overlay(&mixed);
                    mixed.set_halign(gtk::Align::End);
                    mixed.set_valign(gtk::Align::End);
                    button.set_child(Some(&icon));
                    toggles.push((button.clone(), mixed, tool.id));
                }
                button.upcast()
            } else {
                let button = gtk::MenuButton::new();
                button.set_widget_name(&format!("format-quick-{}", tool.id));
                button.add_css_class("flat");
                button.set_has_frame(false);
                button.set_direction(gtk::ArrowType::Up);
                button.set_tooltip_text(Some(tool.label));
                accessible_name(&button, tool.label);

                let content = gtk::Box::new(gtk::Orientation::Horizontal, 2);
                let image = gtk::Image::from_icon_name(tool.icon);
                if group == "text" && matches!(tool.id, "ink" | "highlight") {
                    let icon = gtk::Overlay::new();
                    icon.set_child(Some(&image));
                    let swatch = gtk::DrawingArea::new();
                    swatch.set_widget_name(&format!("format-quick-{}-swatch", tool.id));
                    swatch.set_content_width(16);
                    swatch.set_content_height(3);
                    swatch.set_valign(gtk::Align::End);
                    icon.add_overlay(&swatch);
                    swatches.push((tool.id, swatch));
                    content.append(&icon);
                } else {
                    content.append(&image);
                }
                let arrow = gtk::Image::from_icon_name("pan-up-symbolic");
                arrow.set_pixel_size(16);
                content.append(&arrow);
                button.set_child(Some(&content));
                if group == "elements" && matches!(tool.id, "fields" | "breaks") {
                    let model = elements_menu()
                        .item_link(2, "section")
                        .unwrap()
                        .item_link(if tool.id == "fields" { 1 } else { 2 }, "submenu")
                        .unwrap();
                    button.set_menu_model(Some(&model));
                } else if let Some(source) =
                    tool_actions::source_widget(ui, tool.id).and_downcast::<gtk::MenuButton>()
                {
                    connect_existing_picker(ui, &button, &source, tool.id);
                } else {
                    connect_picker(
                        ui,
                        &button,
                        if group == "paragraph" {
                            paragraph
                        } else if group == "text" {
                            text
                        } else {
                            &elements.menu
                        },
                        group,
                        Some(tool.id),
                    );
                }
                button.upcast()
            };
            shortcuts.append(&widget);
            tools.push((tool.id.to_owned(), widget));
        }
        groups.push((group, shortcuts, tools));
    }
    let origins = vec![
        ("text", text.popover().unwrap()),
        ("paragraph", paragraph.popover().unwrap()),
        ("elements", elements.pop.clone()),
    ];
    // Existing internal category launchers (including keyboard/demo callers)
    // route to the one visible anchor; they no longer occupy toolbar slots.
    for (menu, group) in [
        (text, "text"),
        (paragraph, "paragraph"),
        (&elements.menu, "elements"),
    ] {
        let weak = Rc::downgrade(ui);
        menu.set_create_popup_func(move |_| {
            if let Some(ui) = weak.upgrade().filter(|ui| !ui.closed.get()) {
                if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                    tools.open_group(group, None);
                }
            }
        });
    }
    let overflow = gtk::MenuButton::new();
    overflow.set_widget_name("format-overflow-menu");
    overflow.set_icon_name("view-more-symbolic");
    overflow.set_direction(gtk::ArrowType::Up);
    overflow.add_css_class("flat");
    accessible_name(&overflow, wordcraft_gnome::t!("More Formatting Options"));
    let pop = gtk::Popover::new();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let header = format_popover_header(
        &pop,
        &root,
        wordcraft_gnome::t!("Formatting"),
        &ui.format_group,
    );
    header.home();
    let main = gtk::Box::new(gtk::Orientation::Vertical, 6);
    main.set_width_request(290);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(480);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&main));
    root.append(&scroll);
    pop.set_child(Some(&root));
    overflow.set_popover(Some(&pop));
    let weak = Rc::downgrade(ui);
    pop.connect_show(move |_| {
        if let Some(ui) = weak.upgrade() {
            if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                tools.sync(&ui, &selection_character(&ui));
            }
        }
    });
    ui.format_buttons.borrow_mut().push(overflow.clone());
    // Only the shortcuts scroll. The single menu stays fixed at the far right.
    let fixed = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let scroll_bar = gtk::ScrolledWindow::new();
    scroll_bar.set_widget_name("format-pinned-scroll");
    scroll_bar.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    scroll_bar.set_hexpand(true);
    scroll_bar.set_min_content_width(0);
    scroll_bar.set_propagate_natural_width(false);
    ui.format_toolbar.set_min_content_width(0);
    ui.format_toolbar.set_propagate_natural_width(false);
    ui.format_toolbar.set_child(None::<&gtk::Widget>);
    scroll_bar.set_child(Some(&formats.parent().unwrap()));
    fixed.append(&scroll_bar);
    fixed.append(&overflow);
    ui.format_toolbar
        .set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never);
    ui.format_toolbar.set_child(Some(&fixed));
    // Prepare the common page on the next activation, after GTK has finished
    // closing the previous surface. Never reparent from a closed idle callback.
    let weak = Rc::downgrade(ui);
    overflow.set_create_popup_func(move |button| {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        let Some(tools) = ui.quick_tools.borrow().as_ref().cloned() else {
            return;
        };
        if tools.opening.get() || ui.format_reflowing.get() {
            return;
        }
        if let Some(current) = button.popover() {
            if current != tools.overflow_pop {
                current.set_visible(false);
                button.set_popover(Some(&tools.overflow_pop));
            }
        } else {
            button.set_popover(Some(&tools.overflow_pop));
        }
    });
    Rc::new(QuickTools {
        style_name,
        alignment: alignment.unwrap(),
        ui: Rc::downgrade(ui),
        width: Cell::new(0),
        opening: Cell::new(false),
        groups,
        toggles,
        swatches,
        overflow,
        overflow_pop: pop,
        overflow_main: main,
        origins,
    })
}

// GtkStack can retain an interrupted transition when reopening the same page.
// Finish it while the popover is hidden; nested navigation keeps its animation.
fn reset_entry_stack(stack: &gtk::Stack, root: &str, alternate: &str) {
    if stack.is_transition_running() && stack.visible_child_name().as_deref() == Some(root) {
        stack.set_visible_child_full(alternate, gtk::StackTransitionType::None);
    }
    stack.set_visible_child_full(root, gtk::StackTransitionType::None);
}

fn first_label(root: &gtk::Widget) -> Option<gtk::Label> {
    if let Ok(label) = root.clone().downcast::<gtk::Label>() {
        return Some(label);
    }
    let mut child = root.first_child();
    while let Some(widget) = child {
        child = widget.next_sibling();
        if let Some(label) = first_label(&widget) {
            return Some(label);
        }
    }
    None
}

// Each launcher is a native MenuButton. Its factory adopts the existing shared
// popover, so GTK owns the correct open state, anchor and second-click dismissal.
// https://docs.gtk.org/gtk4/method.MenuButton.set_create_popup_func.html
fn connect_picker(
    ui: &Rc<Ui>,
    button: &gtk::MenuButton,
    menu: &gtk::MenuButton,
    group: &'static str,
    page: Option<&'static str>,
) {
    let Some(pop) = menu.popover() else { return };
    let (weak, pop) = (Rc::downgrade(ui), pop.downgrade());
    button.set_create_popup_func(move |button| {
        let (Some(ui), Some(pop)) = (weak.upgrade().filter(|u| !u.closed.get()), pop.upgrade())
        else {
            return;
        };
        if let Some(owner) = pop.parent().and_downcast::<gtk::MenuButton>() {
            if owner != *button {
                pop.set_visible(false);
                owner.set_popover(None::<&gtk::Popover>);
            }
        }
        if ui.format_reflowing.get() && pop.parent().as_ref() == Some(button.upcast_ref()) {
            return;
        }
        pop.set_pointing_to(None::<&gdk::Rectangle>);
        if pop.parent().as_ref() != Some(button.upcast_ref()) {
            button.set_popover(Some(&pop));
        }
        let header = ui
            .format_group
            .entries
            .borrow()
            .iter()
            .find(|(p, _)| p.upgrade().as_ref() == Some(&pop))
            .map(|(_, h)| h.clone());
        if let Some(header) = header {
            header.parent_back.borrow_mut().take();
            header.entry_page.replace(page.map(str::to_owned));
        }
        // Existing show handlers prepare overflow home; doing it twice can
        // start a second stack transition when the popup is reopened.
        if page.is_none() && group != "elements" {
            return;
        }
        match group {
            "paragraph" => {
                if let Some(panel) = ui.paragraph.borrow().clone() {
                    let transition = panel.stack.transition_type();
                    panel
                        .stack
                        .set_transition_type(gtk::StackTransitionType::None);
                    reset_entry_stack(&panel.stack, "main", "detail");
                    if let Some(page) = page {
                        panel.page(page);
                    } else {
                        panel.home();
                    }
                    panel.stack.set_transition_type(transition);
                }
            }
            "text" => {
                if let Some(panel) = ui.text_tools.borrow().clone() {
                    let transition = panel.stack.transition_type();
                    panel
                        .stack
                        .set_transition_type(gtk::StackTransitionType::None);
                    reset_entry_stack(&panel.stack, "main", "detail");
                    if let Some(page) = page {
                        panel.page(page);
                    } else {
                        panel.home();
                    }
                    panel.stack.set_transition_type(transition);
                }
            }
            _ => {
                if let Some(panel) = ui.elements.borrow().clone() {
                    let transition = panel.stack.transition_type();
                    panel
                        .stack
                        .set_transition_type(gtk::StackTransitionType::None);
                    reset_entry_stack(&panel.stack, "home", "toc");
                    if let Some(page) = page {
                        panel.prepare(page);
                    } else if let Some(page) = panel.requested_page.take() {
                        panel.prepare(&page);
                    } else {
                        panel.home();
                    }
                    panel.stack.set_transition_type(transition);
                }
            }
        }
    });
    if page.is_some() {
        ui.format_buttons.borrow_mut().push(button.clone());
    }
}

/// Open a category at the common native anchor (also used by keyboard actions).
pub(super) fn open_group(ui: &Rc<Ui>, group: &str, page: Option<&str>) -> gtk::MenuButton {
    let tools = ui.quick_tools.borrow().as_ref().unwrap().clone();
    tools.open_group(group, page);
    tools.overflow.clone()
}
pub(super) fn group_pop(ui: &Ui, group: &str) -> gtk::Popover {
    ui.quick_tools
        .borrow()
        .as_ref()
        .unwrap()
        .origins
        .iter()
        .find(|(key, _)| *key == group)
        .unwrap()
        .1
        .clone()
}

fn connect_existing_picker(
    ui: &Rc<Ui>,
    button: &gtk::MenuButton,
    source: &gtk::MenuButton,
    id: &'static str,
) {
    let Some(pop) = source.popover() else { return };
    for launcher in [button, source] {
        let (weak, pop) = (Rc::downgrade(ui), pop.clone());
        launcher.set_create_popup_func(move |launcher| {
            let Some(ui) = weak
                .upgrade()
                .filter(|u| tool_actions::enabled(u, "elements", id))
            else {
                return;
            };
            if pop.parent().as_ref() != Some(launcher.upcast_ref()) {
                pop.set_visible(false);
                if let Some(old) = pop.parent().and_downcast::<gtk::MenuButton>() {
                    old.set_popover(None::<&gtk::Popover>);
                }
                launcher.set_popover(Some(&pop));
            }
            let _ = ui;
        });
    }
}
