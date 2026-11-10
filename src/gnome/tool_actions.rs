//! Stable toolbar identities route to the same native actions and property pages.
use super::*;
use serde_json::json;

pub(super) fn split_tool(group: &str, id: &str) -> bool {
    (group == "text"
        && matches!(
            id,
            "underline" | "strike" | "ink" | "highlight" | "language" | "case"
        ))
        || (group == "paragraph"
            && matches!(
                id,
                "alignment"
                    | "list"
                    | "columns"
                    | "indents"
                    | "spacing"
                    | "borders"
                    | "shading"
                    | "sort"
            ))
}
pub(super) fn toggle_command(group: &str, id: &str) -> Option<(&'static str, &'static str)> {
    Some(match (group, id) {
        ("text", "smallCaps") => ("format.smallCaps", "smallCaps"),
        ("text", "caps") => ("format.allCaps", "caps"),
        ("text", "hidden") => ("format.hidden", "hidden"),
        ("text", "outline") => ("format.outline", "outline"),
        ("text", "shadow") => ("format.shadow", "shadow"),
        ("text", "emboss") => ("format.emboss", "emboss"),
        ("text", "engrave") => ("format.engrave", "engrave"),
        ("text", "text-border") => ("format.border", "border"),
        ("paragraph", "keepNext") => ("para.keepNext", "keepNext"),
        ("paragraph", "keepLines") => ("para.keepLines", "keepLines"),
        ("paragraph", "pageBreakBefore") => ("para.pageBreakBefore", "pageBreakBefore"),
        ("paragraph", "widowControl") => ("para.widowControl", "widowControl"),
        ("paragraph", "marks") => ("view.marks", "marks"),
        _ => return None,
    })
}
pub(super) fn source_name(id: &str) -> Option<&'static str> {
    Some(match id {
        "table-select" => "table-select",
        "table-add" => "table-add",
        "table-delete" => "table-delete",
        "table-merge" => "table-merge",
        "table-split" => "table-split",
        "table-properties" => "table-properties",
        "table-caption" => "table-show-caption",
        "image-original" => "picture-original",
        "image-replace" => "picture-replace",
        "image-delete" => "picture-delete",
        "image-caption" => "picture-show-caption",
        "image-move" => "picture-move-with-text",
        _ => return None,
    })
}
pub(super) fn source_widget(ui: &Rc<Ui>, id: &str) -> Option<gtk::Widget> {
    let name = source_name(id)?;
    if id.starts_with("image-") {
        return ui
            .picture
            .borrow()
            .as_ref()
            .and_then(|p| find_widget(&p.context, name));
    }
    let pop = ui.table_popover.borrow().clone()?;
    find_widget(&pop, name).or_else(|| {
        ui.elements
            .borrow()
            .as_ref()
            .and_then(|p| p.body_for(&pop))
            .and_then(|body| find_widget(&body, name))
    })
}
pub(super) fn enabled(ui: &Rc<Ui>, group: &str, id: &str) -> bool {
    if ui.closed.get() {
        return false;
    }
    let current = ui.current.borrow();
    let Some(d) = current.as_ref() else {
        return false;
    };
    if wordcraft_engine::cmd::protect::enforced(&d.session.doc) && id != "marks" {
        return false;
    }
    if group == "elements" && (id.starts_with("table-") || id.starts_with("cell-")) {
        return (wordcraft_gnome::tables::in_table(&d.session)
            || wordcraft_gnome::captions::table_target(&d.session).is_some())
            && source_widget(ui, id).is_none_or(|w| w.is_sensitive());
    }
    if group == "elements" && id.starts_with("image-") {
        return picture_properties_target(&d.session).is_some()
            && source_widget(ui, id).is_none_or(|w| w.is_sensitive());
    }
    true
}
pub(super) fn direct(group: &str, id: &str) -> bool {
    toggle_command(group, id).is_some()
        || matches!(
            (group, id),
            ("text", "clear")
                | ("paragraph", "reset")
                | ("paragraph", "indent")
                | ("paragraph", "outdent")
        )
        || (group == "elements"
            && !matches!(
                id,
                "image"
                    | "table"
                    | "toc"
                    | "fields"
                    | "breaks"
                    | "table-select"
                    | "table-add"
                    | "table-delete"
                    | "table-split"
                    | "image-size"
                    | "image-spacing"
                    | "image-wrap"
            ))
}
pub(super) fn activate(ui: &Rc<Ui>, group: &str, id: &str) {
    if !enabled(ui, group, id) {
        return;
    }
    if let Some((command, key)) = toggle_command(group, id) {
        let state = if group == "text" {
            selection_character(ui)
        } else {
            selection_paragraph(ui)
        };
        let on = if id == "marks" {
            ui.current
                .borrow()
                .as_ref()
                .is_some_and(|d| d.session.view.marks)
        } else if key == "border" {
            state[key].is_object()
        } else {
            state[key].as_bool().unwrap_or(false)
        };
        if let Err(e) = format_execute(ui, command, json!({"value":!on})) {
            ui.toasts.add_toast(Toast::new(&e));
        }
        return;
    }
    if group == "text" {
        if let Some(tools) = ui.text_tools.borrow().as_ref() {
            match id {
                "ink" | "highlight" => {
                    if let Some(p) = tools.palette.borrow().as_ref() {
                        p.reapply_color(id == "ink");
                    }
                }
                "language" => tools.reapply_language(),
                "case" => tools.reapply_case(),
                "clear" => tools.execute("format.clear", json!({})),
                _ => {}
            }
        }
    } else if group == "paragraph" && matches!(id, "indent" | "outdent") {
        let _ = format_execute(
            ui,
            if id == "indent" {
                "para.indent"
            } else {
                "para.outdent"
            },
            json!({}),
        );
    } else if group == "paragraph" && id != "reset" {
        if id == "sort"
            && ui.current.borrow().as_ref().is_none_or(|d| {
                wordcraft_gnome::text_properties::paragraph_positions(&d.session).len() < 2
            })
        {
            return;
        }
        if let Some(p) = ui.paragraph.borrow().as_ref() {
            p.repeat_group(id);
        }
    } else if group == "paragraph" && id == "reset" {
        if let Err(e) = format_execute(ui, "para.reset", json!({})) {
            ui.toasts.add_toast(Toast::new(&e));
        }
    } else if group == "elements" {
        if let Some(source) = source_widget(ui, id) {
            if let Some(button) = source.downcast_ref::<gtk::Button>() {
                button.emit_clicked();
            } else if let Some(check) = source.downcast_ref::<gtk::CheckButton>() {
                check.set_active(!check.is_active() || check.is_inconsistent());
            }
            return;
        }
        match id {
            "table-distribute-columns" => table_command(ui, "table.distributeColumns", json!({})),
            "table-distribute-rows" => table_command(ui, "table.distributeRows", json!({})),
            "table-auto-height" => table_command(ui, "table.properties", json!({"rowHeight":null})),
            id if id.starts_with("table-") || id.starts_with("cell-") => {
                open_table_property(ui, id)
            }
            _ => {
                let action = match id {
                    "header" => "insert-header",
                    "footer" => "insert-footer",
                    _ => "",
                };
                let action = if action.is_empty() {
                    format!("insert-{id}")
                } else {
                    action.into()
                };
                if let Some(host) = ui.host.upgrade() {
                    gio::prelude::ActionGroupExt::activate_action(&host.window, &action, None);
                }
            }
        }
    }
}
fn open_table_property(ui: &Rc<Ui>, id: &str) {
    let Some(host) = ui.host.upgrade() else {
        return;
    };
    let dialog = show_table_properties(ui, &host.window);
    let name = match id {
        "table-alignment" => "tool-control-table.properties-align",
        "table-autofit" => "tool-control-table.autofit-mode",
        "table-column-width" => "tool-control-table.columnWidth-width",
        "table-row-height" => "tool-control-table.rowHeight-height",
        "table-header-row" => "tool-control-table.properties-headerRow",
        "table-split-rows" => "tool-control-table.properties-allowBreak",
        "cell-alignment" => "tool-control-table.cellAlign-value",
        "cell-direction" => "tool-control-table.textDirection-value",
        "cell-shading" => "element-background-chooser",
        "table-borders" => "tool-control-table.borders-kind",
        _ => return,
    };
    if let Some(widget) = find_widget(&dialog, name) {
        if let Some(page) = widget
            .ancestor(adw::PreferencesPage::static_type())
            .and_downcast::<adw::PreferencesPage>()
        {
            dialog.set_visible_page(&page);
        }
        let weak = widget.downgrade();
        glib::idle_add_local_once(move || {
            if let Some(w) = weak.upgrade() {
                w.grab_focus();
            }
        });
    }
}
pub(super) fn sizing_control(ui: &Rc<Ui>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    row.add_css_class("linked");
    let spin = gtk::SpinButton::with_range(1., 1638., 0.5);
    spin.set_digits(2);
    spin.set_width_chars(4);
    spin.set_widget_name("format-quick-size-input");
    let value = selection_character(ui)["size"].as_f64();
    if let Some(v) = value {
        spin.set_value(v);
    } else {
        spin.set_text("");
    }
    accessible_name(&spin, wordcraft_gnome::t!("Font Size"));
    row.append(&spin);
    let weak = Rc::downgrade(ui);
    bind_numeric(ui, &spin, move |value| {
        let ui = weak.upgrade().ok_or("Document closed")?;
        immediate_format(&ui, "format.set", json!({"props":{"size":value}}))
    });
    for (label, command, tip) in [
        (
            "−",
            "format.shrinkFont",
            wordcraft_gnome::t!("Decrease Size"),
        ),
        ("+", "format.growFont", wordcraft_gnome::t!("Increase Size")),
    ] {
        let button = gtk::Button::with_label(label);
        accessible_name(&button, tip);
        button.set_tooltip_text(Some(tip));
        row.append(&button);
        let weak = Rc::downgrade(ui);
        button.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                if let Err(e) = format_execute(&ui, command, json!({})) {
                    ui.toasts.add_toast(Toast::new(&e));
                }
            }
        });
    }
    let choices = gtk::MenuButton::new();
    choices.set_icon_name("pan-up-symbolic");
    choices.set_direction(gtk::ArrowType::Up);
    choices.set_tooltip_text(Some(wordcraft_gnome::t!("Font Size")));
    row.append(&choices);
    let model = gio::Menu::new();
    for size in [8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 28, 32, 36, 48, 72] {
        let item = gio::MenuItem::new(Some(&format!("{size} pt")), None);
        item.set_action_and_target_value(Some("sizing.choose"), Some(&(size as f64).to_variant()));
        model.append_item(&item);
    }
    let actions = gio::SimpleActionGroup::new();
    let choose = gio::SimpleAction::new("choose", Some(&f64::static_variant_type()));
    let weak = Rc::downgrade(ui);
    choose.connect_activate(move |_, value| {
        if let (Some(ui), Some(value)) = (weak.upgrade(), value.and_then(|v| v.get::<f64>())) {
            if let Err(e) = format_execute(&ui, "format.set", json!({"props":{"size":value}})) {
                ui.toasts.add_toast(Toast::new(&e));
            }
        }
    });
    actions.add_action(&choose);
    row.insert_action_group("sizing", Some(&actions));
    choices.set_menu_model(Some(&model));
    row
}
