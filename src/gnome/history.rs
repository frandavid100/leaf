use super::*;
use wordcraft_gnome::history as engine_history;

pub(super) struct HistoryControls {
    undo: gtk::MenuButton,
    redo: gtk::MenuButton,
    undo_model: gio::Menu,
    redo_model: gio::Menu,
    pub(super) document_model: gio::Menu,
    pub(super) actions: gio::SimpleActionGroup,
}
pub(super) fn remember_repeat(ui: &Rc<Ui>) {
    *ui.repeat_action.borrow_mut() = ui
        .current
        .borrow()
        .as_ref()
        .and_then(|d| engine_history::RepeatAction::capture(&d.session));
}
pub(super) fn repeat_available(ui: &Ui) -> bool {
    ui.current.borrow().as_ref().is_some_and(|d| {
        ui.repeat_action
            .borrow()
            .as_ref()
            .is_some_and(|r| r.applies_to(&d.session))
    })
}
pub(super) fn repeat_last(ui: &Rc<Ui>) {
    let op = ui.repeat_action.borrow().clone();
    let result = ui
        .current
        .borrow_mut()
        .as_mut()
        .zip(op.as_ref())
        .map(|(d, r)| r.apply(&mut d.session));
    match result {
        Some(Ok(true)) => after_content_change(ui),
        Some(Ok(false)) => refresh_selection(ui),
        Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
        _ => {}
    }
    ui.document_box.grab_focus();
}
pub(super) fn redo_or_repeat(ui: &Rc<Ui>) {
    if ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| d.session.can_redo())
    {
        apply_user_edit(ui, Edit::Redo);
    } else {
        repeat_last(ui);
    }
    ui.document_box.grab_focus();
}
fn populate(model: &gio::Menu, labels: &[String], identity: (u64, u64), redo: bool) {
    model.remove_all();
    for (index, label) in labels.iter().enumerate() {
        let description = engine_history::description(label);
        let text = if index == 0 {
            description
        } else {
            wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("{action} ({count} steps)"),
                &[("action", description), ("count", (index + 1).to_string())],
            )
        };
        let item = gio::MenuItem::new(Some(&text), None);
        item.set_action_and_target_value(
            Some("history.travel"),
            Some(&(identity.0, identity.1, redo, (index + 1) as u32).to_variant()),
        );
        if index == 0 {
            item.set_attribute_value(
                "accel",
                Some(
                    &if redo {
                        "<Control><Shift>z"
                    } else {
                        "<Control>z"
                    }
                    .to_variant(),
                ),
            );
        }
        model.append_item(&item);
    }
}
impl HistoryControls {
    pub(super) fn refresh(&self, ui: &Rc<Ui>) {
        let (undo, redo, identity) = ui
            .current
            .borrow()
            .as_ref()
            .map(|d| {
                (
                    d.session.undo_labels(),
                    // Walk redo snapshots only when a history picker is actually open.
                    if self.redo.popover().is_some_and(|p| p.is_visible())
                        || ui.document_menu_open.get()
                    {
                        engine_history::labels(&d.session, true)
                    } else {
                        d.session
                            .redo_label()
                            .map(|s| vec![s.to_string()])
                            .unwrap_or_default()
                    },
                    (d.session.document_id(), d.session.rev()),
                )
            })
            .unwrap_or_default();
        self.undo.set_sensitive(!undo.is_empty());
        self.redo.set_sensitive(!redo.is_empty());
        populate(&self.undo_model, &undo, identity, false);
        populate(&self.redo_model, &redo, identity, true);
        self.document_model.remove_all();
        self.document_model
            .append_submenu(Some(wordcraft_gnome::t!("Undo")), &self.undo_model);
        if redo.is_empty() && repeat_available(ui) {
            menu_item(
                &self.document_model,
                wordcraft_gnome::t!("Repeat Last Action"),
                "repeat",
                None,
            );
        } else {
            self.document_model
                .append_submenu(Some(wordcraft_gnome::t!("Redo")), &self.redo_model);
        }
        let repeat = redo.is_empty() && repeat_available(ui);
        ui.redo_button.set_sensitive(!redo.is_empty() || repeat);
        accessible_name(
            &ui.redo_button,
            if repeat {
                wordcraft_gnome::t!("Repeat Last Action")
            } else {
                wordcraft_gnome::t!("Redo")
            },
        );
        ui.redo_button.set_tooltip_text(Some(if repeat {
            wordcraft_gnome::t!("Repeat Last Action")
        } else {
            wordcraft_gnome::t!("Redo (Ctrl+Shift+Z)")
        }));
    }
}
pub(super) fn install_history_controls(ui: &Rc<Ui>, toolbar: &gtk::Box) -> Rc<HistoryControls> {
    let undo = gtk::MenuButton::new();
    let redo = gtk::MenuButton::new();
    let undo_model = gio::Menu::new();
    let redo_model = gio::Menu::new();
    let actions = gio::SimpleActionGroup::new();
    let travel = gio::SimpleAction::new(
        "travel",
        Some(&<(u64, u64, bool, u32)>::static_variant_type()),
    );
    let weak = Rc::downgrade(ui);
    travel.connect_activate(move |_, value| {
        let (Some(ui), Some((id, rev, redo, count))) = (
            weak.upgrade(),
            value.and_then(|v| v.get::<(u64, u64, bool, u32)>()),
        ) else {
            return;
        };
        if ui.closed.get()
            || document_identity(&ui) != Some((id, rev))
            || !ui
                .host
                .upgrade()
                .and_then(|h| selected_ui(&h))
                .is_some_and(|u| Rc::ptr_eq(&u, &ui))
        {
            return;
        }
        let result = ui
            .current
            .borrow_mut()
            .as_mut()
            .map(|d| engine_history::travel(&mut d.session, redo, count as usize));
        match result {
            Some(Ok(n)) if n > 0 => after_history_change(&ui),
            Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
            _ => {}
        }
        if let Some(h) = ui.history_controls.borrow().as_ref() {
            for b in [&h.undo, &h.redo] {
                b.popdown();
            }
        }
        if let Some(p) = ui.document_menu.borrow().as_ref().and_then(|w| w.upgrade()) {
            p.popdown();
        }
        ui.document_box.grab_focus();
    });
    actions.add_action(&travel);
    clear_box(toolbar);
    for (body, chevron, model, is_redo) in [
        (&ui.undo_button, &undo, &undo_model, false),
        (&ui.redo_button, &redo, &redo_model, true),
    ] {
        let linked = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        linked.add_css_class("linked");
        chevron.set_child(Some(&format_chevron()));
        chevron.set_widget_name(if is_redo {
            "redo-history"
        } else {
            "undo-history"
        });
        let title = if is_redo {
            wordcraft_gnome::t!("Redo History")
        } else {
            wordcraft_gnome::t!("Undo History")
        };
        accessible_name(chevron, title);
        chevron.set_tooltip_text(Some(title));
        let pop = gtk::PopoverMenu::from_model(Some(model));
        pop.insert_action_group("history", Some(&actions));
        let weak = Rc::downgrade(ui);
        pop.connect_show(move |_| {
            if let Some(ui) = weak.upgrade() {
                refresh_history_controls(&ui);
            }
        });
        chevron.set_popover(Some(&pop));
        linked.append(body);
        linked.append(chevron);
        toolbar.append(&linked);
    }
    Rc::new(HistoryControls {
        undo,
        redo,
        undo_model,
        redo_model,
        document_model: gio::Menu::new(),
        actions,
    })
}
pub(super) fn refresh_history_controls(ui: &Rc<Ui>) {
    if let Some(h) = ui.history_controls.borrow().as_ref() {
        h.refresh(ui);
    }
}
