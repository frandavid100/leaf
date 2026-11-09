//! Native reorderable preference lists. Changes affect all open documents.
use super::*;

fn refresh_windows() {
    WORKSPACES.with(|hosts| {
        for host in hosts.borrow().iter() {
            for (_, ui) in host.docs.borrow().iter() {
                if let Some(tools) = ui.quick_tools.borrow().as_ref() {
                    tools.layout();
                }
            }
        }
    });
}
fn save(dialog: &adw::PreferencesDialog, group: &str, order: &[(String, bool)]) -> bool {
    match wordcraft_gnome::preferences::set_toolbar(group, order) {
        Ok(()) => {
            refresh_windows();
            true
        }
        Err(e) => {
            dialog.add_toast(Toast::new(&e));
            false
        }
    }
}
fn move_to(
    dialog: &adw::PreferencesDialog,
    list: &gtk::ListBox,
    group: &'static str,
    id: &str,
    target: usize,
) {
    let mut order = wordcraft_gnome::preferences::toolbar(group);
    let Some(old) = order.iter().position(|(k, _)| k == id) else {
        return;
    };
    if target >= order.len() || old == target {
        return;
    }
    let item = order.remove(old);
    order.insert(target, item);
    if save(dialog, group, &order) {
        populate(dialog, list, group);
        if let Some(row) = list.row_at_index(target as i32) {
            row.grab_focus();
        }
    }
}
fn populate(dialog: &adw::PreferencesDialog, list: &gtk::ListBox, group: &'static str) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let catalog = wordcraft_gnome::toolbar::tools(group);
    let order = wordcraft_gnome::preferences::toolbar(group);
    for (index, (id, on)) in order.iter().enumerate() {
        let Some(tool) = catalog.iter().find(|t| t.id == id) else {
            continue;
        };
        let row = adw::ActionRow::new();
        row.set_title(tool.label);
        row.set_widget_name(&format!("toolbar-preference-{group}-{id}"));
        row.set_focusable(true);
        let handle = gtk::Image::from_icon_name("list-drag-handle-symbolic");
        handle.set_cursor_from_name(Some("grab"));
        row.add_prefix(&handle);
        row.add_prefix(&gtk::Image::from_icon_name(tool.icon));
        let switch = gtk::Switch::new();
        switch.set_widget_name(&format!("toolbar-visible-{group}-{id}"));
        switch.set_valign(gtk::Align::Center);
        switch.set_active(*on);
        accessible_name(
            &switch,
            &wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Show {label} in toolbar"),
                &[("label", tool.label.into())],
            ),
        );
        row.add_suffix(&switch);
        let weak = dialog.downgrade();
        let id = id.clone();
        let old = *on;
        switch.connect_active_notify(move |toggle| {
            let Some(dialog) = weak.upgrade() else { return };
            let mut order = wordcraft_gnome::preferences::toolbar(group);
            if let Some((_, on)) = order.iter_mut().find(|(k, _)| k == &id) {
                *on = toggle.is_active();
            }
            if !save(&dialog, group, &order) && toggle.is_active() != old {
                toggle.set_active(old);
            }
        });
        // Explicit native actions allow keyboard reordering without dragging.
        for (direction, icon, label) in [
            (-1, "go-up-symbolic", wordcraft_gnome::t!("Move Up")),
            (1, "go-down-symbolic", wordcraft_gnome::t!("Move Down")),
        ] {
            let button = gtk::Button::from_icon_name(icon);
            button.add_css_class("flat");
            button.set_valign(gtk::Align::Center);
            button.set_widget_name(&format!("toolbar-move-{group}-{}-{direction}", tool.id));
            accessible_name(&button, label);
            button.set_tooltip_text(Some(label));
            let target = index as i32 + direction;
            button.set_sensitive(target >= 0 && target < order.len() as i32);
            let (dialog, list, id) = (dialog.downgrade(), list.downgrade(), tool.id);
            button.connect_clicked(move |_| {
                if let (Some(dialog), Some(list)) = (dialog.upgrade(), list.upgrade()) {
                    move_to(&dialog, &list, group, id, target as usize);
                }
            });
            row.add_suffix(&button);
        }
        let source = gtk::DragSource::new();
        source.set_actions(gdk::DragAction::MOVE);
        source.set_content(Some(&gdk::ContentProvider::for_value(
            &format!("leaf-toolbar:{group}:{}", tool.id).to_value(),
        )));
        handle.add_controller(source);
        let target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
        let (weak_dialog, weak_list) = (dialog.downgrade(), list.downgrade());
        target.connect_drop(move |_, value, _, _| {
            let (Some(dialog), Some(list), Ok(value)) = (
                weak_dialog.upgrade(),
                weak_list.upgrade(),
                value.get::<String>(),
            ) else {
                return false;
            };
            let Some(id) = value.strip_prefix(&format!("leaf-toolbar:{group}:")) else {
                return false;
            };
            if !wordcraft_gnome::toolbar::tools(group)
                .iter()
                .any(|t| t.id == id)
            {
                return false;
            }
            move_to(&dialog, &list, group, id, index);
            true
        });
        row.add_controller(target);
        list.append(&row);
    }
}
pub(super) fn add(dialog: &adw::PreferencesDialog) {
    let page = adw::PreferencesPage::new();
    page.set_title(wordcraft_gnome::t!("Bottom Toolbar"));
    page.set_icon_name(Some("preferences-desktop-display-symbolic"));
    page.set_widget_name("toolbar-preferences");
    for (key, title) in [
        ("paragraph", wordcraft_gnome::t!("Paragraph")),
        ("text", wordcraft_gnome::t!("Text")),
        ("elements", wordcraft_gnome::t!("Elements")),
    ] {
        let group = adw::PreferencesGroup::new();
        group.set_title(title);
        if key == "paragraph" {
            group.set_description(Some(wordcraft_gnome::t!("Enabled items appear from left to right in list order. Other items are available in each group’s ⋮ menu. Drag rows or use Move Up and Move Down to reorder them.")));
        }
        let list = gtk::ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(gtk::SelectionMode::None);
        list.set_widget_name(&format!("toolbar-preferences-{key}"));
        populate(dialog, &list, key);
        group.add(&list);
        page.add(&group);
    }
    dialog.add(&page);
}
