//! Native toolbar placement derived from persistent application preferences.
//! Existing panels and Session commands own every action in either location.
use super::*;

pub(super) struct QuickTools {
    style_name: gtk::Label,
    alignment: gtk::MenuButton,
    ui: std::rc::Weak<Ui>,
    width: Cell<i32>,
    groups: Vec<(&'static str, gtk::Box, Vec<(String, gtk::Widget)>)>,
    toggles: Vec<(gtk::Button, gtk::Label, &'static str)>,
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
                let cost = widget.measure(gtk::Orientation::Horizontal, -1).1.max(34) + 6;
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
            if let Ok(toggle) = button.clone().downcast::<gtk::ToggleButton>() {
                toggle.set_active(active && !mixed);
            } else if active && !mixed {
                button.set_state_flags(gtk::StateFlags::CHECKED, false);
            } else {
                button.unset_state_flags(gtk::StateFlags::CHECKED);
            }
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
        let shortcuts = gtk::Box::new(gtk::Orientation::Horizontal, 6);
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
                if matches!(tool.id, "underline" | "strike") {
                    let split = adw::SplitButton::new();
                    split.add_css_class("flat");
                    split.add_css_class("image-button");
                    split.set_valign(gtk::Align::Center);
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
                    connect_picker(ui, &options, text, group, Some(tool.id));
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
            } else if group == "elements" && !matches!(tool.id, "image" | "table" | "toc") {
                let button =
                    icon_button(tool.icon, tool.label, &format!("format-quick-{}", tool.id));
                button.set_action_name(Some(&format!("win.insert-{}", tool.id)));
                button.upcast()
            } else {
                let button = gtk::MenuButton::new();
                button.set_widget_name(&format!("format-quick-{}", tool.id));
                button.add_css_class("flat");
                button.set_has_frame(false);
                button.set_direction(gtk::ArrowType::Up);
                button.set_tooltip_text(Some(tool.label));
                accessible_name(&button, tool.label);
                if group == "paragraph" && tool.id == "alignment" {
                    alignment = Some(button.clone());
                }
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
                content.append(&format_chevron());
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
                    Some(tool.id),
                );
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
        if let Some(pop) = menu.popover() {
            connect_picker(
                ui,
                menu,
                menu,
                if menu == paragraph {
                    "paragraph"
                } else if menu == text {
                    "text"
                } else {
                    "elements"
                },
                None,
            );
            // Keep the default group popup attached until a direct launcher takes it.
            menu.set_popover(Some(&pop));
        }
        group.append(menu);
    }
    Rc::new(QuickTools {
        style_name,
        alignment: alignment.unwrap(),
        ui: Rc::downgrade(ui),
        width: Cell::new(0),
        groups,
        toggles,
        swatches,
    })
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
