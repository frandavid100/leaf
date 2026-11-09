//! Native toolbar placement derived from persistent application preferences.
//! Existing panels and Session commands own every action in either location.
use super::*;

pub(super) struct QuickTools {
    style_name: gtk::Label,
    style_actions: gtk::MenuButton,
    alignment: gtk::Button,
    ui: std::rc::Weak<Ui>,
    width: Cell<i32>,
    groups: Vec<(&'static str, gtk::Box, Vec<(String, gtk::Widget)>)>,
    toggles: Vec<(gtk::ToggleButton, gtk::Label, &'static str)>,
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
    icon: &str,
    label: &str,
    key: &'static str,
    command: &'static str,
) -> (gtk::ToggleButton, gtk::Label, &'static str) {
    let button = gtk::ToggleButton::new();
    button.add_css_class("flat");
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
        let mut available = if self.width.get() < 520 {
            0
        } else {
            (self.width.get() - 330).max(0)
        };
        for (group, container, tools) in &self.groups {
            let order = wordcraft_gnome::preferences::toolbar(group);
            let mut previous: Option<gtk::Widget> = None;
            for (id, enabled) in order {
                let Some((_, widget)) = tools.iter().find(|(key, _)| key == &id) else {
                    continue;
                };
                container.reorder_child_after(widget, previous.as_ref());
                previous = Some(widget.clone());
                // Measure the widget even when it was overflowed on the last frame.
                widget.set_visible(true);
                let cost = widget.measure(gtk::Orientation::Horizontal, -1).1.max(34);
                let visible = enabled && cost <= available;
                widget.set_visible(visible);
                if visible {
                    available -= cost;
                }
            }
        }
        self.filter_menus();
    }

    pub(super) fn filter_menus(&self) {
        let Some(ui) = self.ui.upgrade() else { return };
        for (group, _, tools) in &self.groups {
            let root = match *group {
                "paragraph" => ui
                    .paragraph
                    .borrow()
                    .as_ref()
                    .map(|p| p.main.clone().upcast::<gtk::Widget>()),
                "text" => ui
                    .text_tools
                    .borrow()
                    .as_ref()
                    .and_then(|t| t.stack.child_by_name("main")),
                _ => ui.elements.borrow().as_ref().and_then(|p| p.pop.child()),
            };
            if let Some(root) = root {
                for (id, widget) in tools {
                    if let Some(row) = find_widget(&root, &format!("format-row-{group}-{id}")) {
                        row.set_visible(!widget.get_visible());
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
        self.style_actions.set_sensitive(style.is_some());
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
            let value = if *key == "ink" {
                &character["color"]["Rgb"]
            } else {
                &character["highlight"]
            };
            let rgb = value.as_array().filter(|v| v.len() == 3).map(|v| {
                (
                    v[0].as_f64().unwrap_or(0.) / 255.,
                    v[1].as_f64().unwrap_or(0.) / 255.,
                    v[2].as_f64().unwrap_or(0.) / 255.,
                )
            });
            let none = *key == "highlight" && rgb.is_none();
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
        let paragraph = selection_paragraph(ui);
        let alignment = paragraph["align"].as_str();
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
            .and_then(|w| w.first_child())
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
                _ => (
                    character[*key].as_bool().unwrap_or(false),
                    character[*key].is_null(),
                ),
            };
            button.set_active(active && !mixed);
            marker.set_visible(mixed);
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
    style_group.add_css_class("linked");
    style_group.add_css_class("style-control");
    style_group.set_width_request(184);
    style_group.set_hexpand(false);
    let style_name = gtk::Label::new(Some("Normal"));
    style_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    style_name.set_max_width_chars(16);
    style_name.set_width_chars(16);
    style.set_hexpand(true);
    style.set_child(Some(&style_name));
    style.set_always_show_arrow(false);
    style_group.append(style);
    let actions = gtk::MenuButton::new();
    actions.set_widget_name("style-actions");
    actions.set_icon_name("pan-down-symbolic");
    actions.set_direction(gtk::ArrowType::Up);
    accessible_name(&actions, wordcraft_gnome::t!("Current Style Actions"));
    actions.set_tooltip_text(Some(wordcraft_gnome::t!("Current Style Actions")));
    let pop = gtk::Popover::new();
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);
    format_popover_header(&pop, &root, wordcraft_gnome::t!("Style"), &ui.format_group);
    let choices = gtk::Box::new(gtk::Orientation::Vertical, 4);
    root.append(&choices);
    pop.set_child(Some(&root));
    actions.set_popover(Some(&pop));
    let weak_ui = Rc::downgrade(ui);
    let weak_style = style.downgrade();
    let weak_pop = pop.downgrade();
    pop.connect_show(move |_| {
        clear_box(&choices);
        let Some(ui) = weak_ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        let Some(id) = ui
            .current
            .borrow()
            .as_ref()
            .and_then(|d| wordcraft_gnome::styles::current(&d.session))
        else {
            return;
        };
        let identity = document_identity(&ui);
        for (name, label) in [
            (
                "style-current-update",
                wordcraft_gnome::t!("Update from Selection"),
            ),
            ("style-current-edit", wordcraft_gnome::t!("Edit Style")),
        ] {
            let button = style_menu_button(label);
            button.set_widget_name(name);
            choices.append(&button);
            let (weak_ui, weak_style, weak_pop, id) = (
                Rc::downgrade(&ui),
                weak_style.clone(),
                weak_pop.clone(),
                id.clone(),
            );
            button.connect_clicked(move |_| {
                let Some(ui) = weak_ui.upgrade().filter(|u| !u.closed.get()) else {
                    return;
                };
                let valid = document_identity(&ui) == identity
                    && ui.current.borrow().as_ref().is_some_and(|d| {
                        wordcraft_gnome::styles::current(&d.session).as_ref() == Some(&id)
                    });
                if !valid {
                    ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                        "The document has changed; open this dialog again."
                    )));
                    return;
                }
                if let Some(pop) = weak_pop.upgrade() {
                    close_format_popover(&pop);
                }
                if name == "style-current-edit" {
                    if let Some(menu) = weak_style.upgrade() {
                        menu.popup();
                    }
                    if let Some(panel) = ui.style.borrow().as_ref() {
                        panel.show_detail(&id);
                    }
                } else {
                    let panel = ui.style.borrow().clone();
                    if let Some(panel) = panel {
                        if let Err(error) =
                            panel.run(wordcraft_gnome::styles::Edit::Update(id.clone()))
                        {
                            ui.toasts.add_toast(Toast::new(&error));
                        }
                    }
                    ui.document_box.grab_focus();
                }
            });
        }
    });
    style_group.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    actions.add_css_class("flat");
    style_group.append(&actions);
    formats.append(&style_group);
    let paragraph_group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let text_group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let elements_group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    for (group, name) in [
        (&paragraph_group, "paragraph"),
        (&text_group, "text"),
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
        ("paragraph", &paragraph_group),
        ("text", &text_group),
        ("elements", &elements_group),
    ] {
        let shortcuts = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.append(&shortcuts);
        let mut tools = Vec::new();
        for tool in wordcraft_gnome::toolbar::tools(group) {
            let widget: gtk::Widget = if group == "text"
                && matches!(
                    tool.id,
                    "bold" | "italic" | "underline" | "strike" | "superscript"
                ) {
                let command = match tool.id {
                    "bold" => "format.bold",
                    "italic" => "format.italic",
                    "underline" => "format.underline",
                    "strike" => "format.strikethrough",
                    _ => "format.superscript",
                };
                let item = shortcut(ui, tool.icon, tool.label, tool.id, command);
                let widget = if matches!(tool.id, "underline" | "strike") {
                    let split = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                    split.add_css_class("linked");
                    split.append(&item.0);
                    let options = icon_button(
                        "pan-down-symbolic",
                        &wordcraft_gnome::i18n::interpolate(
                            wordcraft_gnome::t!("Options for {label}"),
                            &[("label", tool.label.into())],
                        ),
                        &format!("format-quick-{}-options", tool.id),
                    );
                    connect_picker(ui, &options, text, group, tool.id);
                    split.append(&options);
                    split.upcast()
                } else {
                    item.0.clone().upcast()
                };
                toggles.push(item);
                widget
            } else {
                let button =
                    icon_button(tool.icon, tool.label, &format!("format-quick-{}", tool.id));
                if group == "paragraph" && tool.id == "alignment" {
                    alignment = Some(button.clone());
                }
                if group == "elements" && !matches!(tool.id, "image" | "table" | "toc") {
                    button.set_action_name(Some(&format!("win.insert-{}", tool.id)));
                } else {
                    // One target opens choices; the small indicator never changes click semantics.
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
                    let arrow = gtk::Image::from_icon_name("pan-down-symbolic");
                    arrow.set_pixel_size(10);
                    content.append(&arrow);
                    button.set_child(Some(&content));
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
                        tool.id,
                    );
                }
                button.upcast()
            };
            shortcuts.append(&widget);
            tools.push((tool.id.to_owned(), widget));
        }
        groups.push((group, shortcuts, tools));
    }
    for (group, menu, label) in [
        (
            &paragraph_group,
            paragraph,
            wordcraft_gnome::t!("More paragraph options"),
        ),
        (&text_group, text, wordcraft_gnome::t!("More text options")),
        (
            &elements_group,
            &elements.menu,
            wordcraft_gnome::t!("More elements"),
        ),
    ] {
        menu.set_icon_name("view-more-symbolic");
        menu.set_tooltip_text(Some(label));
        accessible_name(menu, label);
        group.append(menu);
    }
    ui.format_buttons.borrow_mut().push(actions.clone());
    Rc::new(QuickTools {
        style_name,
        style_actions: actions,
        alignment: alignment.unwrap(),
        ui: Rc::downgrade(ui),
        width: Cell::new(0),
        groups,
        toggles,
        swatches,
    })
}

fn connect_picker(
    ui: &Rc<Ui>,
    button: &gtk::Button,
    menu: &gtk::MenuButton,
    group: &'static str,
    page: &'static str,
) {
    let weak = Rc::downgrade(ui);
    let menu = menu.downgrade();
    button.connect_clicked(move |button| {
        let (Some(ui), Some(menu)) = (weak.upgrade().filter(|u| !u.closed.get()), menu.upgrade())
        else {
            return;
        };
        let Some(pop) = menu.popover() else { return };
        // Point in the original parent's coordinates, retaining its lifetime,
        // pin and close handlers. Native popup placement follows the launcher.
        if let Some(bounds) = button.compute_bounds(&menu) {
            pop.set_pointing_to(Some(&gdk::Rectangle::new(
                bounds.x().round() as i32,
                bounds.y().round() as i32,
                bounds.width().round() as i32,
                bounds.height().round() as i32,
            )));
        }
        let header = ui
            .format_group
            .entries
            .borrow()
            .iter()
            .find(|(p, _)| p.upgrade().as_ref() == Some(&pop))
            .map(|(_, h)| h.clone());
        if let Some(header) = header {
            header.entry_page.replace(Some(page.into()));
        }
        // Elements prepares and presents its borrowed body itself. Opening its
        // empty host first can race the previous element's closed callback.
        if group != "elements" {
            menu.popup();
        }
        match group {
            "paragraph" => {
                let panel = ui.paragraph.borrow().clone();
                if let Some(p) = panel {
                    p.page(page);
                }
            }
            "text" => {
                let panel = ui.text_tools.borrow().clone();
                if let Some(p) = panel {
                    p.page(page);
                }
            }
            _ => {
                let panel = ui.elements.borrow().clone();
                if let Some(p) = panel {
                    p.show(page);
                }
            }
        }
    });
}
