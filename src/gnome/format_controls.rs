use super::*;

// Native formatting panels. All document mutations use upstream public commands.
pub(super) fn format_execute(
    ui: &Rc<Ui>,
    command: &str,
    args: serde_json::Value,
) -> Result<(), String> {
    let changed = {
        let mut value = ui.current.borrow_mut();
        let doc = value.as_mut().ok_or(wordcraft_gnome::t!("No document"))?;
        if command == "leaf.highlight" {
            wordcraft_gnome::highlight::register(&mut doc.session);
        }
        let rev = doc.session.rev();
        if matches!(
            command,
            "para.bullets" | "para.numbering" | "para.multilevel"
        ) {
            wordcraft_gnome::list_design::apply(&mut doc.session, command, &args)?;
        } else {
            doc.session.run(command, &args).map_err(|e| e.to_string())?;
        }
        doc.session.rev() != rev
    };
    if let Some(t) = ui.text_tools.borrow().as_ref() {
        t.remember_property(command, &args);
    }
    if let Some(p) = ui.paragraph.borrow().as_ref() {
        p.remember_property(command, &args, ui);
    }
    if changed {
        after_content_change(ui);
    } else if command == "view.marks" {
        let dirty = ui.dirty.get();
        let subtitle = ui.title.subtitle().to_string();
        after_history_change(ui);
        ui.dirty.set(dirty);
        ui.title.set_subtitle(&subtitle);
        sync_tab(ui);
    } else {
        if command.starts_with("format.") || command.starts_with("para.") {
            remember_repeat(ui);
        }
        refresh_selection(ui);
    }
    Ok(())
}
/// A modal formatting panel belongs to one document and one selected tab.
/// A hidden/closed dialog or a replaced document cannot apply a late control edit.
pub(super) fn panel_target(
    ui: &Rc<Ui>,
    dialog: &impl IsA<adw::Dialog>,
) -> impl Fn() -> bool + 'static {
    let id = ui
        .current
        .borrow()
        .as_ref()
        .map(|d| d.session.document_id());
    let weak_ui = Rc::downgrade(ui);
    let weak_dialog = dialog.as_ref().downgrade();
    move || {
        let Some(ui) = weak_ui.upgrade().filter(|u| !u.closed.get()) else {
            return false;
        };
        let Some(dialog) = weak_dialog.upgrade().filter(|w| w.is_mapped()) else {
            return false;
        };
        ui.current
            .borrow()
            .as_ref()
            .map(|d| d.session.document_id())
            == id
            && ui.host.upgrade().is_some_and(|h| {
                h.window.visible_dialog().as_ref() == Some(&dialog)
                    && selected_ui(&h).is_some_and(|active| Rc::ptr_eq(&active, &ui))
            })
    }
}

pub(super) fn preferences_dialog(title: &str) -> adw::PreferencesDialog {
    let w = adw::PreferencesDialog::new();
    w.set_title(title);
    w.set_content_width(620);
    w.set_content_height(620);
    w
}
pub(super) fn panel_page(w: &adw::PreferencesDialog, title: &str) -> adw::PreferencesPage {
    let p = adw::PreferencesPage::new();
    p.set_title(title);
    w.add(&p);
    p
}
pub(super) fn panel_group(p: &adw::PreferencesPage, title: &str) -> adw::PreferencesGroup {
    let g = adw::PreferencesGroup::new();
    g.set_title(title);
    p.add(&g);
    g
}
pub(super) fn panel_action(
    g: &adw::PreferencesGroup,
    label: &str,
    ui: &Rc<Ui>,
    w: &adw::PreferencesDialog,
    command: &'static str,
    args: serde_json::Value,
) {
    let b = gtk::Button::with_label(label);
    b.set_widget_name(&format!("tool-control-{command}"));
    b.set_halign(gtk::Align::Fill);
    accessible_name(&b, label);
    g.add(&b);
    let valid = panel_target(ui, w);
    let ui = ui.clone();
    let w = w.clone();
    b.connect_clicked(glib::clone!(
        #[weak]
        ui,
        #[weak]
        w,
        move |_| {
            if !valid() {
                return;
            }
            if let Err(e) = format_execute(&ui, command, args.clone()) {
                w.add_toast(Toast::new(&e));
            }
        }
    ));
}

pub(super) fn panel_number(
    g: &adw::PreferencesGroup,
    label: &str,
    value: f64,
    min: f64,
    max: f64,
) -> gtk::SpinButton {
    let row = adw::ActionRow::new();
    row.set_title(label);
    let n = gtk::SpinButton::with_range(min, max, 0.5);
    n.set_digits(1);
    n.set_value(value);
    n.set_valign(gtk::Align::Center);
    accessible_name(&n, label);
    row.set_activatable_widget(Some(&n));
    row.add_suffix(&n);
    g.add(&row);
    n
}
pub(super) fn panel_entry(g: &adw::PreferencesGroup, label: &str, text: &str) -> gtk::Entry {
    let row = adw::ActionRow::new();
    row.set_title(label);
    let e = gtk::Entry::new();
    e.set_text(text);
    e.set_valign(gtk::Align::Center);
    accessible_name(&e, label);
    row.set_activatable_widget(Some(&e));
    row.add_suffix(&e);
    g.add(&row);
    e
}
pub(super) fn panel_choice(
    g: &adw::PreferencesGroup,
    label: &str,
    options: &[(&str, &str)],
    ui: &Rc<Ui>,
    w: &adw::PreferencesDialog,
    command: &'static str,
    key: &'static str,
) {
    let row = adw::ComboRow::new();
    row.set_widget_name(&format!("tool-control-{command}-{key}"));
    row.set_title(label);
    let labels: Vec<&str> = std::iter::once(wordcraft_gnome::t!("Choose…"))
        .chain(options.iter().map(|o| o.0))
        .collect();
    row.set_model(Some(&gtk::StringList::new(&labels)));
    row.set_selected(0);
    g.add(&row);
    let values: Vec<String> = options.iter().map(|x| x.1.to_string()).collect();
    let valid = panel_target(ui, w);
    let ui = ui.clone();
    let w = w.clone();
    row.connect_selected_notify(glib::clone!(
        #[weak]
        ui,
        #[weak]
        w,
        move |r| {
            if !valid() || ui.format_syncing.get() {
                return;
            }
            if let Some(v) = r
                .selected()
                .checked_sub(1)
                .and_then(|i| values.get(i as usize))
            {
                let args = serde_json::json!({key:v});
                if let Err(e) = format_execute(&ui, command, args) {
                    w.add_toast(Toast::new(&e));
                }
            }
        }
    ));
}
pub(super) fn panel_color(
    g: &adw::PreferencesGroup,
    label: &str,
    ui: &Rc<Ui>,
    w: &adw::PreferencesDialog,
    command: &'static str,
) {
    let title = gtk::Label::new(Some(label));
    title.set_xalign(0.0);
    g.add(&title);
    let initial = if command == "table.shading" {
        let mut current = ui.current.borrow_mut();
        current
            .as_mut()
            .and_then(|d| {
                d.session
                    .run("table.properties", &serde_json::json!({}))
                    .ok()
            })
            .and_then(|v| rgb_value_hex(&v["cell"]["shading"]))
    } else {
        None
    };
    let valid = panel_target(ui, w);
    let weak_ui = Rc::downgrade(ui);
    let weak_window = w.downgrade();
    let picker = embedded_color_chooser(
        "element-background-chooser",
        initial.as_deref(),
        move |hex| {
            if !valid() {
                return false;
            }
            let Some(ui) = weak_ui.upgrade().filter(|u| !u.closed.get()) else {
                return false;
            };
            let Some(window) = weak_window.upgrade().filter(|w| w.is_mapped()) else {
                return false;
            };
            match immediate_format(&ui, command, serde_json::json!({"color":hex})) {
                Ok(()) => true,
                Err(e) => {
                    window.add_toast(Toast::new(&e));
                    false
                }
            }
        },
    );
    accessible_name(&picker.widget, label);
    g.add(&picker.widget);
}

pub(super) fn selection_character(ui: &Rc<Ui>) -> serde_json::Value {
    let value = ui.current.borrow();
    let Some(doc) = value.as_ref() else {
        return serde_json::json!({});
    };
    let mut states = wordcraft_gnome::text_properties::selected_resolved(&doc.session)
        .into_iter()
        .filter_map(|p| serde_json::to_value(p).ok());
    let Some(mut state) = states.next() else {
        return serde_json::json!({});
    };
    for next in states {
        if let Some(fields) = state.as_object_mut() {
            for (key, value) in fields {
                if next.get(key) != Some(value) {
                    *value = serde_json::Value::Null;
                }
            }
        }
    }
    state
}
pub(super) fn selection_paragraph(ui: &Rc<Ui>) -> serde_json::Value {
    let value = ui.current.borrow();
    let Some(doc) = value.as_ref() else {
        return serde_json::json!({});
    };
    let mut states = wordcraft_gnome::text_properties::paragraph_positions(&doc.session)
        .into_iter()
        .filter_map(|pos| {
            doc.session.doc.para_at(&pos).and_then(|p| {
                serde_json::to_value(doc.session.doc.styles.resolve_para(&p.props)).ok()
            })
        });
    let Some(mut state) = states.next() else {
        return serde_json::json!({});
    };
    let mut mixed = Vec::<String>::new();
    for next in states {
        if let Some(fields) = state.as_object_mut() {
            for (key, value) in fields {
                if next.get(key) != Some(value) {
                    if !mixed.contains(key) {
                        mixed.push(key.clone());
                    }
                    *value = serde_json::Value::Null;
                }
            }
        }
    }
    state["_mixed"] = serde_json::json!(mixed);
    state
}
pub(super) fn show_text_options(ui: &Rc<Ui>, _parent: &ApplicationWindow) {
    format_bar::open_group(ui, "text", Some("options"));
}

pub(super) fn property_content(label: &str, icon: &str) -> (gtk::Box, gtk::Label) {
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    content.append(&gtk::Image::from_icon_name(icon));
    let text = gtk::Label::new(Some(label));
    text.set_xalign(0.);
    text.set_hexpand(true);
    text.set_ellipsize(gtk::pango::EllipsizeMode::End);
    text.set_max_width_chars(28);
    content.append(&text);
    (content, text)
}
pub(super) fn check_icon(check: &gtk::CheckButton, title: &str, icon: &str) {
    let (content, _) = property_content(title, icon);
    check.set_child(Some(&content));
    accessible_name(check, title);
}
