use super::*;
use wordcraft_gnome::history as engine_history;

// Use the same native component and documented child hierarchy as Paragraph.
// Keep body references for independent Repeat / empty-history sensitivity.
pub(super) fn history_button(redo: bool) -> (adw::SplitButton, gtk::Button, gtk::MenuButton) {
    let split = adw::SplitButton::new();
    split.add_css_class("flat");
    split.add_css_class("image-button");
    split.set_valign(gtk::Align::Center);
    split.set_child(Some(&history_icon(redo)));
    let body = split
        .first_child()
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
    let more = body
        .next_sibling()
        .unwrap()
        .next_sibling()
        .unwrap()
        .downcast::<gtk::MenuButton>()
        .unwrap();
    (split, body, more)
}
pub(super) struct HistoryControls {
    undo: gtk::MenuButton,
    redo: gtk::MenuButton,
    undo_model: gio::Menu,
    redo_model: gio::Menu,
    pub(super) document_model: gio::Menu,
    pub(super) actions: gio::SimpleActionGroup,
}
pub(super) fn remember_repeat(ui: &Rc<Ui>) {
    if let Some(d) = ui.current.borrow().as_ref() {
        ui.repeat_action.borrow_mut().record(&d.session);
    }
}
pub(super) fn repeat_available(ui: &Ui) -> bool {
    ui.current
        .borrow()
        .as_ref()
        .is_some_and(|d| ui.repeat_action.borrow().available(&d.session))
}
fn repeat_count(ui: &Rc<Ui>, count: usize) {
    let result = ui.current.borrow_mut().as_mut().map(|d| {
        ui.repeat_action
            .borrow_mut()
            .repeat_recent(&mut d.session, count)
    });
    match result {
        Some(Ok(true)) => {
            refresh_after_content_change(ui);
            schedule_derived_refresh(ui);
        }
        Some(Ok(false)) => refresh_selection(ui),
        Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
        _ => {}
    }
    ui.document_box.grab_focus();
}
pub(super) fn repeat_last(ui: &Rc<Ui>) {
    repeat_count(ui, 1);
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
        let repeat = redo.is_empty() && repeat_available(ui);
        self.redo.set_sensitive(!redo.is_empty() || repeat);
        populate(&self.undo_model, &undo, identity, false);
        if repeat {
            self.redo_model.remove_all();
            let history = ui.repeat_action.borrow();
            for (index, label) in history.descriptions().iter().enumerate() {
                let item = gio::MenuItem::new(Some(label), None);
                item.set_action_and_target_value(
                    Some("history.repeat"),
                    Some(
                        &(
                            identity.0,
                            identity.1,
                            history.generation(),
                            (index + 1) as u32,
                        )
                            .to_variant(),
                    ),
                );
                self.redo_model.append_item(&item);
            }
        } else {
            populate(&self.redo_model, &redo, identity, true);
        }
        accessible_name(
            &self.redo,
            if repeat {
                wordcraft_gnome::t!("Repeat History")
            } else {
                wordcraft_gnome::t!("Redo History")
            },
        );
        self.redo.set_tooltip_text(Some(if repeat {
            wordcraft_gnome::t!("Repeat History")
        } else {
            wordcraft_gnome::t!("Redo History")
        }));
        if ui.redo_button.has_css_class("repeat-mode") != repeat {
            if repeat {
                ui.redo_button.add_css_class("repeat-mode");
                ui.redo_button
                    .set_child(Some(&gtk::Image::from_icon_name("view-refresh-symbolic")));
            } else {
                ui.redo_button.remove_css_class("repeat-mode");
                ui.redo_button.set_child(Some(&history_icon(true)));
            }
        }
        self.document_model.remove_all();
        self.document_model
            .append_submenu(Some(wordcraft_gnome::t!("Undo")), &self.undo_model);
        self.document_model.append_submenu(
            Some(if repeat {
                wordcraft_gnome::t!("Repeat Last Action")
            } else {
                wordcraft_gnome::t!("Redo")
            }),
            &self.redo_model,
        );
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
    let undo_split = ui
        .undo_button
        .parent()
        .unwrap()
        .downcast::<adw::SplitButton>()
        .unwrap();
    let redo_split = ui
        .redo_button
        .parent()
        .unwrap()
        .downcast::<adw::SplitButton>()
        .unwrap();
    let undo = undo_split
        .last_child()
        .unwrap()
        .downcast::<gtk::MenuButton>()
        .unwrap();
    let redo = redo_split
        .last_child()
        .unwrap()
        .downcast::<gtk::MenuButton>()
        .unwrap();
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
    let repeat = gio::SimpleAction::new(
        "repeat",
        Some(&<(u64, u64, u64, u32)>::static_variant_type()),
    );
    let weak = Rc::downgrade(ui);
    repeat.connect_activate(move |_, value| {
        let (Some(ui), Some((id, rev, generation, count))) = (
            weak.upgrade(),
            value.and_then(|v| v.get::<(u64, u64, u64, u32)>()),
        ) else {
            return;
        };
        if ui.closed.get()
            || document_identity(&ui) != Some((id, rev))
            || ui.repeat_action.borrow().generation() != generation
            || !ui
                .host
                .upgrade()
                .and_then(|h| selected_ui(&h))
                .is_some_and(|u| Rc::ptr_eq(&u, &ui))
            || ui
                .current
                .borrow()
                .as_ref()
                .is_some_and(|d| d.session.can_redo())
        {
            return;
        }
        repeat_count(&ui, count as usize);
    });
    actions.add_action(&repeat);
    clear_box(toolbar);
    for (split, chevron, model, is_redo) in [
        (&undo_split, &undo, &undo_model, false),
        (&redo_split, &redo, &redo_model, true),
    ] {
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
        split.set_popover(Some(&pop));
        toolbar.append(split);
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
