//! Persistent document tools. Narrow layouts hide shortcuts, while each
//! group's existing menu retains every operation and its original callbacks.
use super::*;

pub(super) struct QuickTools {
    style_name: gtk::Label,
    style_actions: gtk::MenuButton,
    alignment: gtk::Button,
    paragraph: gtk::Box,
    text: gtk::Box,
    elements: gtk::Box,
    toggles: Vec<(gtk::ToggleButton, gtk::Label, &'static str)>,
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
        // The menus never move or unmap, including when pinned during resize.
        self.elements.set_visible(width >= 850);
        self.paragraph.set_visible(width >= 680);
        self.text.set_visible(width >= 520);
        self.style_name
            .set_max_width_chars(if width < 520 { 11 } else { 22 });
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
        self.alignment.set_icon_name(icon);
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
    let style_name = gtk::Label::new(Some("Normal"));
    style_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    style_name.set_max_width_chars(22);
    style_name.set_width_chars(7);
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
    let paragraph_shortcuts = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    paragraph_group.append(&paragraph_shortcuts);
    let alignment = icon_button(
        "format-justify-left-symbolic",
        wordcraft_gnome::t!("Alignment"),
        "format-quick-alignment",
    );
    paragraph_shortcuts.append(&alignment);
    let weak = Rc::downgrade(ui);
    let menu = paragraph.downgrade();
    alignment.connect_clicked(move |_| {
        if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
            if let Some(menu) = menu.upgrade() {
                menu.popup();
            }
            if let Some(panel) = ui.paragraph.borrow().as_ref() {
                panel.page("alignment");
            }
        }
    });
    let mut toggles = Vec::new();
    for (icon, label, key, command) in [
        (
            "view-list-bullet-symbolic",
            wordcraft_gnome::t!("Bullets"),
            "bullets",
            "para.bullets",
        ),
        (
            "view-list-ordered-symbolic",
            wordcraft_gnome::t!("Numbering"),
            "numbering",
            "para.numbering",
        ),
    ] {
        let item = shortcut(ui, icon, label, key, command);
        paragraph_shortcuts.append(&item.0);
        toggles.push(item);
    }
    let text_shortcuts = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    text_group.append(&text_shortcuts);
    for (icon, label, key, command) in [
        (
            "format-text-bold-symbolic",
            wordcraft_gnome::t!("Bold"),
            "bold",
            "format.bold",
        ),
        (
            "format-text-italic-symbolic",
            wordcraft_gnome::t!("Italic"),
            "italic",
            "format.italic",
        ),
        (
            "format-text-underline-symbolic",
            wordcraft_gnome::t!("Underline"),
            "underline",
            "format.underline",
        ),
    ] {
        let item = shortcut(ui, icon, label, key, command);
        text_shortcuts.append(&item.0);
        toggles.push(item);
    }
    let element_shortcuts = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    elements_group.append(&element_shortcuts);
    for (icon, label, page) in [
        (
            "insert-image-symbolic",
            wordcraft_gnome::t!("Image"),
            "image",
        ),
        ("view-grid-symbolic", wordcraft_gnome::t!("Table"), "table"),
    ] {
        let button = icon_button(icon, label, &format!("format-quick-{page}"));
        element_shortcuts.append(&button);
        let weak = Rc::downgrade(elements);
        button.connect_clicked(move |_| {
            if let Some(panel) = weak.upgrade() {
                panel.show(page);
            }
        });
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
        alignment,
        paragraph: paragraph_shortcuts,
        text: text_shortcuts,
        elements: element_shortcuts,
        toggles,
    })
}
