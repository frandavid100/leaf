//! Native reorderable preference lists. Changes affect all open documents.
use super::*;
use wordcraft_gnome::toolbar::Placement;

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
fn save(dialog: &adw::PreferencesDialog, group: &str, order: &[(String, Placement)]) -> bool {
    match wordcraft_gnome::preferences::set_toolbar_placements(group, order) {
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
    let mut order = wordcraft_gnome::preferences::toolbar_placements(group);
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
fn clear_drag_marks(list: &gtk::ListBox) {
    let mut child = list.first_child();
    while let Some(row) = child {
        child = row.next_sibling();
        row.remove_css_class("outline-drop-before");
        row.remove_css_class("toolbar-drop-after");
    }
}
// This is an internal list reorder, so use a native gesture rather than a
// desktop data transfer. Commit once on release; cancellation changes nothing.
fn install_row_drag(
    dialog: &adw::PreferencesDialog,
    list: &gtk::ListBox,
    handle: &gtk::Image,
    group: &'static str,
    id: &'static str,
    index: usize,
) {
    let drag = gtk::GestureDrag::new();
    drag.set_button(gdk::BUTTON_PRIMARY);
    drag.set_propagation_phase(gtk::PropagationPhase::Capture);
    let start = Rc::new(Cell::new(None::<f64>));
    let (weak_list, weak_handle, begin) = (list.downgrade(), handle.downgrade(), start.clone());
    drag.connect_drag_begin(move |gesture, _, y| {
        if let (Some(list), Some(handle)) = (weak_list.upgrade(), weak_handle.upgrade()) {
            if let Some(bounds) = handle.compute_bounds(&list) {
                begin.set(Some(bounds.y() as f64 + y));
                handle.set_cursor_from_name(Some("grabbing"));
                gesture.set_state(gtk::EventSequenceState::Claimed);
            }
        }
    });
    let (weak_list, moving) = (list.downgrade(), start.clone());
    drag.connect_drag_update(move |_, _, dy| {
        let (Some(list), Some(y)) = (weak_list.upgrade(), moving.get()) else {
            return;
        };
        clear_drag_marks(&list);
        if let Some(row) = list.row_at_y((y + dy).round() as i32) {
            if row.index() != index as i32 {
                row.add_css_class(if row.index() < index as i32 {
                    "outline-drop-before"
                } else {
                    "toolbar-drop-after"
                });
            }
        }
    });
    let (weak_dialog, weak_list, weak_handle, ending) = (
        dialog.downgrade(),
        list.downgrade(),
        handle.downgrade(),
        start.clone(),
    );
    drag.connect_drag_end(move |_, _, dy| {
        let y = ending.take();
        if let Some(handle) = weak_handle.upgrade() {
            handle.set_cursor_from_name(Some("grab"));
        }
        let (Some(list), Some(y)) = (weak_list.upgrade(), y) else {
            return;
        };
        clear_drag_marks(&list);
        let Some(target) = list
            .row_at_y((y + dy).round() as i32)
            .map(|r| r.index() as usize)
        else {
            return;
        };
        let (weak_dialog, weak_list) = (weak_dialog.clone(), weak_list.clone());
        glib::idle_add_local_once(move || {
            if let (Some(dialog), Some(list)) = (weak_dialog.upgrade(), weak_list.upgrade()) {
                move_to(&dialog, &list, group, id, target);
            }
        });
    });
    let (weak_list, weak_handle) = (list.downgrade(), handle.downgrade());
    drag.connect_cancel(move |_, _| {
        start.set(None);
        if let Some(handle) = weak_handle.upgrade() {
            handle.set_cursor_from_name(Some("grab"));
        }
        if let Some(list) = weak_list.upgrade() {
            clear_drag_marks(&list);
        }
    });
    handle.add_controller(drag);
}
fn populate(dialog: &adw::PreferencesDialog, list: &gtk::ListBox, group: &'static str) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
    let catalog = wordcraft_gnome::toolbar::tools(group);
    let order = wordcraft_gnome::preferences::toolbar_placements(group);
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
        handle.set_can_target(true);
        handle.set_size_request(24, 24);
        row.add_prefix(&handle);
        row.add_prefix(&gtk::Image::from_icon_name(tool.icon));
        let placement = gtk::DropDown::from_strings(&[
            wordcraft_gnome::t!("Always in Toolbar"),
            wordcraft_gnome::t!("In Toolbar When Space Allows"),
            wordcraft_gnome::t!("Only in More Options"),
        ]);
        placement.set_widget_name(&format!("toolbar-placement-{group}-{id}"));
        placement.set_valign(gtk::Align::Center);
        placement.set_selected(match on {
            Placement::Pinned => 0,
            Placement::Adaptive => 1,
            Placement::More => 2,
        });
        accessible_name(
            &placement,
            &wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Placement of {label}"),
                &[("label", tool.label.into())],
            ),
        );
        row.add_suffix(&placement);
        let weak = dialog.downgrade();
        let id = id.clone();
        let old = placement.selected();
        placement.connect_selected_notify(move |choice| {
            let Some(dialog) = weak.upgrade() else { return };
            let mut order = wordcraft_gnome::preferences::toolbar_placements(group);
            if let Some((_, mode)) = order.iter_mut().find(|(k, _)| k == &id) {
                *mode = match choice.selected() {
                    0 => Placement::Pinned,
                    1 => Placement::Adaptive,
                    _ => Placement::More,
                };
            }
            if !save(&dialog, group, &order) && choice.selected() != old {
                choice.set_selected(old);
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
        handle.set_widget_name(&format!("toolbar-drag-{group}-{}", tool.id));
        install_row_drag(dialog, list, &handle, group, tool.id, index);
        list.append(&row);
    }
}
pub(super) fn add(dialog: &adw::PreferencesDialog) {
    let page = adw::PreferencesPage::new();
    page.set_name(Some("toolbar"));
    page.set_title(wordcraft_gnome::t!("Bottom Toolbar"));
    page.set_icon_name(Some("preferences-desktop-display-symbolic"));
    page.set_widget_name("toolbar-preferences");
    for (key, title) in [
        ("text", wordcraft_gnome::t!("Text")),
        ("paragraph", wordcraft_gnome::t!("Paragraph")),
        ("elements", wordcraft_gnome::t!("Elements")),
    ] {
        let group = adw::PreferencesGroup::new();
        group.set_title(title);
        if key == "text" {
            group.set_description(Some(wordcraft_gnome::t!("Choose where each tool appears. Pinned tools stay in the scrollable toolbar; other tools fill available space or remain in More Options. Drag rows or use Move Up and Move Down to reorder them.")));
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
