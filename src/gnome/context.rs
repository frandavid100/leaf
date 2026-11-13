use super::*;

// Native context menus. Selection and editing remain public engine commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DocumentContext {
    Text,
    Table,
    Picture,
}

pub(super) fn context_prepare(ui: &Rc<Ui>, x: f64, y: f64) -> Option<DocumentContext> {
    let pos = pos_at_canvas(ui, x, y)?;
    let toc = ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| wordcraft_gnome::references::toc_at(&d.session, &pos));
    if toc {
        let mut c = ui.current.borrow_mut();
        selection::set_caret(&mut c.as_mut()?.session, &pos, false).ok()?;
    }

    let keep = {
        let value = ui.current.borrow();
        let d = value.as_ref()?;
        let (a, b) = d.session.sel.ordered();
        let text = !d.session.sel.is_collapsed() && a.story == pos.story && a <= pos && pos < b;
        let cells = pos
            .path
            .cell()
            .zip(d.session.sel.focus.path.cell())
            .is_some_and(|((tp, r, c), (current, _, _))| {
                pos.story == d.session.sel.focus.story
                    && tp == current
                    && d.session.cell_range().is_some()
                    && wordcraft_engine::cmd::table::selected_cells(&d.session).contains(&(r, c))
            });
        text || cells
    };
    let picture = if keep {
        ui.current
            .borrow()
            .as_ref()
            .is_some_and(|d| pictures::selected(&d.session).is_some())
    } else {
        select_picture_at_canvas(ui, x, y)
    };
    if !keep && !picture {
        let mut value = ui.current.borrow_mut();
        let d = value.as_mut()?;
        if let Err(e) = selection::set_caret(&mut d.session, &pos, false) {
            ui.toasts.add_toast(Toast::new(&e));
            return None;
        }
    }
    refresh_selection(ui);
    Some(if picture {
        DocumentContext::Picture
    } else if pos.path.cell().is_some() {
        DocumentContext::Table
    } else {
        DocumentContext::Text
    })
}

pub(super) struct ContextFormatLease {
    pub(super) host: glib::WeakRef<gtk::PopoverMenu>,
    pub(super) entries: RefCell<Vec<(gtk::Popover, gtk::Widget, Rc<FormatHeader>)>>,
    pub(super) restored: Cell<bool>,
}
impl ContextFormatLease {
    pub(super) fn restore(&self) {
        if self.restored.replace(true) {
            return;
        }
        let entries = std::mem::take(&mut *self.entries.borrow_mut());
        for (origin, body, header) in &entries {
            if let Some(host) = self.host.upgrade() {
                host.remove_child(body);
            }
            if body.parent().is_none() {
                origin.set_child(Some(body));
            }
            header.parent_back.borrow_mut().take();
            header.home();
            EMBEDDED_FORMAT.with(|items| {
                items
                    .borrow_mut()
                    .retain(|(p, _, _)| p.upgrade().as_ref() != Some(origin))
            });
        }
    }
}
pub(super) fn native_menu_stack(root: &impl IsA<gtk::Widget>) -> Option<gtk::Stack> {
    let root = root.as_ref();
    if let Some(s) = root.downcast_ref::<gtk::Stack>() {
        return Some(s.clone());
    }
    let mut child = root.first_child();
    while let Some(w) = child {
        if let Some(s) = native_menu_stack(&w) {
            return Some(s);
        }
        child = w.next_sibling();
    }
    None
}
pub(super) fn mount_context_formats(
    ui: &Rc<Ui>,
    host: &gtk::PopoverMenu,
) -> Rc<ContextFormatLease> {
    // Official GtkPopoverMenu custom slots hold the existing widget trees.
    // Keep nonmodality constant, including when pinning inside a custom page.
    host.add_css_class("format-popover");
    host.add_css_class("style-popover");
    host.set_autohide(false);
    ui.format_group.show(host.upcast_ref());
    ui.format_group
        .hosts
        .borrow_mut()
        .retain(|p| p.upgrade().is_some());
    ui.format_group
        .hosts
        .borrow_mut()
        .push(host.clone().upcast::<gtk::Popover>().downgrade());
    let mut entries = Vec::new();
    for (name, label, id) in [
        ("style-menu", wordcraft_gnome::t!("Style"), "context-style"),
        (
            "paragraph-menu",
            wordcraft_gnome::t!("Paragraph"),
            "context-paragraph",
        ),
        ("text-menu", wordcraft_gnome::t!("Text"), "context-text"),
    ] {
        let origin = match name {
            "paragraph-menu" => ui
                .paragraph
                .borrow()
                .as_ref()
                .and_then(|p| p.popover.upgrade()),
            "text-menu" => ui
                .text_tools
                .borrow()
                .as_ref()
                .and_then(|p| p.popover.upgrade()),
            _ => format_button(ui, name).and_then(|m| m.popover()),
        };
        let Some(origin) = origin else {
            continue;
        };
        let header = ui
            .format_group
            .entries
            .borrow()
            .iter()
            .find(|(p, _)| p.upgrade().as_ref() == Some(&origin))
            .map(|(_, h)| h.clone());
        let (Some(body), Some(header)) = (origin.child(), header) else {
            continue;
        };
        origin.set_child(None::<&gtk::Widget>);
        if !host.add_child(&body, id) {
            origin.set_child(Some(&body));
            continue;
        }
        let weak = host.downgrade();
        header.parent_back.replace(Some(Rc::new(move || {
            if let Some(host) = weak.upgrade() {
                host.set_visible_submenu(Some("main"));
            }
        })));
        header.home();
        // GTK generates a native back/title row before every custom submenu.
        // Our shared header already provides back/title/pin in one row. Retain
        // GTK's title for its navigation bookkeeping, but hide this duplicate.
        if let Some(page) = native_menu_stack(host).and_then(|s| s.child_by_name(label)) {
            if let Some(title) = page.first_child() {
                if title.type_().name() == "GtkModelButton" {
                    title.set_visible(false);
                }
            }
        }
        EMBEDDED_FORMAT.with(|items| {
            items.borrow_mut().push((
                origin.downgrade(),
                host.clone().upcast::<gtk::Popover>().downgrade(),
                header.pin.downgrade(),
            ))
        });
        entries.push((origin, body, header));
    }
    if let Some(s) = ui.style.borrow().as_ref() {
        s.main();
    }
    if let Some(s) = ui.paragraph.borrow().as_ref() {
        s.home();
    }
    if let Some(s) = ui.text_tools.borrow().as_ref() {
        s.home();
    }
    let lease = Rc::new(ContextFormatLease {
        host: host.downgrade(),
        entries: RefCell::new(entries),
        restored: Cell::new(false),
    });
    let closed = lease.clone();
    host.connect_closed(move |_| closed.restore());
    lease
}
pub(super) fn context_item(
    menu: &gio::Menu,
    group: &gio::SimpleActionGroup,
    pop: &gtk::PopoverMenu,
    ui: &Rc<Ui>,
    label: &str,
    name: &str,
    enabled: bool,
    callback: impl Fn(&Rc<Ui>) + 'static,
) {
    let action = gio::SimpleAction::new(name, None);
    action.set_enabled(enabled);
    let weak = Rc::downgrade(ui);
    let popup = pop.downgrade();
    action.connect_activate(move |_, _| {
        if let Some(pop) = popup.upgrade() {
            pop.popdown();
        }
        if let Some(ui) = weak.upgrade() {
            if !ui.closed.get() {
                callback(&ui);
            }
        }
    });
    group.add_action(&action);
    let item = gio::MenuItem::new(Some(label), Some(&format!("context.{name}")));
    let accel = match name {
        "link" => Some("<Control>k"),
        "cut" => Some("<Control>x"),
        "copy" => Some("<Control>c"),
        "paste" => Some("<Control>v"),
        "paste-plain" => Some("<Control><Shift>v"),
        "copy-format" => Some("<Control><Shift>c"),
        "select-all" => Some("<Control>a"),
        _ => None,
    };
    if let Some(accel) = accel {
        item.set_attribute_value("accel", Some(&accel.to_variant()));
    }
    menu.append_item(&item);
}
pub(super) fn show_document_context(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    x: i32,
    y: i32,
    kind: DocumentContext,
) -> gtk::PopoverMenu {
    // A single native GMenu supplies separators, submenu navigation and states.
    let root = gio::Menu::new();
    let pop = gtk::PopoverMenu::from_model(Some(&root));
    pop.set_widget_name("document-context-menu");
    pop.set_has_arrow(false);
    // Closing a nested variants popover returns to this menu. Editing actions
    // still explicitly close this owner before dispatch.
    pop.set_cascade_popdown(false);
    pop.set_halign(gtk::Align::Start);
    pop.set_parent(&ui.document_box);
    pop.set_pointing_to(Some(&gdk::Rectangle::new(x, y, 1, 1)));
    if let Some(p) = ui.table_popover.borrow().as_ref() {
        if p.parent().as_ref() == Some(ui.document_box.upcast_ref()) {
            p.set_pointing_to(Some(&gdk::Rectangle::new(x, y, 1, 1)));
        }
    }
    if let Some(s) = ui.picture.borrow().as_ref() {
        if s.pop.parent().as_ref() == Some(ui.document_box.upcast_ref()) {
            s.pop
                .set_pointing_to(Some(&gdk::Rectangle::new(x, y, 1, 1)));
        }
    }
    let group = gio::SimpleActionGroup::new();
    pop.insert_action_group("context", Some(&group));
    let toc = ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| wordcraft_engine::cmd::references::caret_in_toc(&d.session));
    if toc {
        context_item(
            &root,
            &group,
            &pop,
            ui,
            wordcraft_gnome::t!("Edit Table of Contents…"),
            "edit-toc",
            true,
            |ui| {
                if let Some(panel) = ui.elements.borrow().as_ref() {
                    panel.show("toc");
                }
            },
        );
    }

    if ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| wordcraft_gnome::equations::target(&d.session).is_some())
    {
        context_item(
            &root,
            &group,
            &pop,
            ui,
            wordcraft_gnome::t!("Edit Equation…"),
            "edit-equation",
            true,
            |ui| {
                if let Some(host) = ui.host.upgrade() {
                    show_equation(ui, &host.window);
                }
            },
        );
    }
    let (selected, merge) = ui
        .current
        .borrow()
        .as_ref()
        .map(|d| {
            (
                !d.session.sel.is_collapsed(),
                wordcraft_gnome::tables::selected_cells(&d.session) > 1,
            )
        })
        .unwrap_or_default();
    if kind != DocumentContext::Picture {
        let issue = {
            let mut v = ui.current.borrow_mut();
            v.as_mut().and_then(|d| {
                let pos = d.session.sel.focus.clone();
                proofing::at(&mut d.session, &pos)
            })
        };
        if let Some(issue) = issue {
            append_proofing_actions(&root, &group, &pop, ui, issue);
        }
    }
    let edit = gio::Menu::new();
    root.append_section(None, &edit);
    let link_position = pos_at_canvas(ui, x as f64, y as f64);
    let link_identity = document_identity(ui);
    context_item(
        &edit,
        &group,
        &pop,
        ui,
        wordcraft_gnome::t!("Link"),
        "link",
        selected || kind != DocumentContext::Picture,
        move |u| {
            if document_identity(u) == link_identity {
                if let Some(pos) = &link_position {
                    show_link_at(u, pos);
                } else {
                    show_link(u);
                }
            }
        },
    );
    context_item(
        &edit,
        &group,
        &pop,
        ui,
        wordcraft_gnome::t!("Cut"),
        "cut",
        selected,
        |u| {
            if context_copy(u) {
                apply_user_edit(u, Edit::Delete);
            }
        },
    );
    context_clipboard::append(&edit, &group, &pop, ui, true, selected);
    context_clipboard::append(&edit, &group, &pop, ui, false, selected);
    context_item(
        &edit,
        &group,
        &pop,
        ui,
        wordcraft_gnome::t!("Select All"),
        "select-all",
        true,
        |u| selected_command(u, "select-all", false),
    );
    if let Some(host) = ui.host.upgrade() {
        refresh_main_actions(&host);
    }
    let specific = gio::Menu::new();
    match kind {
        DocumentContext::Text => {
            for (label, id) in [
                (wordcraft_gnome::t!("Style"), "context-style"),
                (wordcraft_gnome::t!("Paragraph"), "context-paragraph"),
                (wordcraft_gnome::t!("Text"), "context-text"),
            ] {
                let page = gio::Menu::new();
                let item = gio::MenuItem::new(None, None);
                item.set_attribute_value("custom", Some(&id.to_variant()));
                page.append_item(&item);
                specific.append_submenu(Some(label), &page);
            }
        }
        DocumentContext::Picture => {
            context_item(
                &specific,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Image Size…"),
                "picture-size",
                true,
                |u| context_picture_editor(u),
            );
            context_item(
                &specific,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Restore Original Size"),
                "picture-original",
                true,
                |u| {
                    tools_change(u, pictures::original);
                },
            );
            let p = parent.downgrade();
            context_item(
                &specific,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Replace Image…"),
                "picture-replace",
                true,
                move |u| {
                    if let Some(p) = p.upgrade() {
                        choose_picture(u, &p, true);
                    }
                },
            );
            context_item(
                &specific,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Delete Image"),
                "picture-delete",
                true,
                |u| {
                    tools_change(u, pictures::delete);
                },
            );
        }
        DocumentContext::Table => {
            for (title, prefix, items) in [
                (
                    wordcraft_gnome::t!("Select"),
                    "select",
                    vec![
                        (wordcraft_gnome::t!("Cell"), "table.selectCell"),
                        (wordcraft_gnome::t!("Row"), "table.selectRow"),
                        (wordcraft_gnome::t!("Column"), "table.selectColumn"),
                        (wordcraft_gnome::t!("Table"), "table.selectTable"),
                    ],
                ),
                (
                    wordcraft_gnome::t!("Insert"),
                    "insert",
                    vec![
                        (wordcraft_gnome::t!("Row Above"), "table.insertRowAbove"),
                        (wordcraft_gnome::t!("Row Below"), "table.insertRowBelow"),
                        (
                            wordcraft_gnome::t!("Column to the Left"),
                            "table.insertColumnLeft",
                        ),
                        (
                            wordcraft_gnome::t!("Column to the Right"),
                            "table.insertColumnRight",
                        ),
                    ],
                ),
                (
                    wordcraft_gnome::t!("Delete"),
                    "delete",
                    vec![
                        (wordcraft_gnome::t!("Row"), "table.deleteRow"),
                        (wordcraft_gnome::t!("Column"), "table.deleteColumn"),
                        (wordcraft_gnome::t!("Table"), "table.deleteTable"),
                    ],
                ),
            ] {
                let sub = gio::Menu::new();
                for (i, (label, cmd)) in items.into_iter().enumerate() {
                    context_item(
                        &sub,
                        &group,
                        &pop,
                        ui,
                        label,
                        &format!("{prefix}-{i}"),
                        true,
                        move |u| table_command(u, cmd, serde_json::json!({})),
                    );
                }
                specific.append_submenu(Some(title), &sub);
            }
            context_item(
                &specific,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Merge Cells"),
                "merge",
                merge,
                |u| table_command(u, "table.merge", serde_json::json!({})),
            );
            let split = gio::Menu::new();
            let p = parent.downgrade();
            context_item(
                &split,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Split Cells"),
                "split-cells",
                true,
                move |u| {
                    if let Some(p) = p.upgrade() {
                        show_split_cells(u, &p);
                    }
                },
            );
            context_item(
                &split,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Split Table"),
                "split-table",
                true,
                |u| {
                    table_command(u, "table.splitTable", serde_json::json!({}));
                },
            );
            specific.append_submenu(Some(wordcraft_gnome::t!("Split")), &split);
            let p = parent.downgrade();
            context_item(
                &specific,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Table Properties"),
                "table-properties",
                true,
                move |u| {
                    if let Some(p) = p.upgrade() {
                        show_table_properties(u, &p);
                    }
                },
            );
            context_item(
                &specific,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("More Table Options…"),
                "table-tools",
                true,
                context_table_editor,
            );
        }
    }
    append_insert_menu(&specific);
    root.append_section(None, &specific);
    let lease = if kind == DocumentContext::Text {
        Some(mount_context_formats(ui, &pop))
    } else {
        None
    };
    ui.context_popovers
        .borrow_mut()
        .retain(|(p, _)| p.upgrade().is_some());
    ui.context_popovers
        .borrow_mut()
        .push((pop.downgrade(), lease));
    // GtkModelButton closes before activating its action helper. Keep the
    // hierarchy and action muxer alive until that signal has fully returned.
    let weak = Rc::downgrade(ui);
    pop.connect_closed(move |p| {
        if let Some(ui) = weak.upgrade() {
            if !ui.closed.get() {
                ui.document_box.grab_focus();
            }
            ui.context_popovers
                .borrow_mut()
                .retain(|(weak, _)| weak.upgrade().as_ref() != Some(p));
        }
        let p = p.clone();
        glib::idle_add_local_once(move || {
            if !p.is_visible() && p.parent().is_some() {
                p.unparent();
            }
        });
    });
    pop.popup();
    if kind == DocumentContext::Text {
        pop.child_focus(gtk::DirectionType::TabForward);
    }
    pop
}
pub(super) fn show_table_context_at(ui: &Rc<Ui>, parent: &ApplicationWindow, x: i32, y: i32) {
    let kind = ui.current.borrow().as_ref().map(|d| {
        if pictures::selected(&d.session).is_some() {
            DocumentContext::Picture
        } else if wordcraft_gnome::tables::in_table(&d.session) {
            DocumentContext::Table
        } else {
            DocumentContext::Text
        }
    });
    if let Some(kind) = kind {
        show_document_context(ui, parent, x, y, kind);
    }
}
pub(super) fn install_table_context_menu(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let click = gtk::GestureClick::new();
    click.set_button(gdk::BUTTON_SECONDARY);
    let weak = Rc::downgrade(ui);
    let parent = parent.downgrade();
    click.connect_pressed(move |gesture, _, x, y| {
        if let (Some(ui), Some(parent)) = (weak.upgrade(), parent.upgrade()) {
            if let Some(kind) = context_prepare(&ui, x, y) {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                ui.document_box.grab_focus();
                show_document_context(&ui, &parent, x as i32, y as i32, kind);
            }
        }
    });
    ui.document_box.add_controller(click);
}

pub(super) fn context_model_row(root: &impl IsA<gtk::Widget>, label: &str) -> Option<gtk::Widget> {
    let root = root.as_ref();
    for (text, name) in [
        (wordcraft_gnome::t!("Copy"), "context-copy-primary"),
        (wordcraft_gnome::t!("Paste"), "context-paste-primary"),
    ] {
        if label == text {
            if let Some(button) = find_widget(root, name) {
                return Some(button);
            }
        }
    }
    // GtkPopoverMenu keeps identically named rows in hidden GtkStack pages.
    // Search only the active page, as a pointer/keyboard user would.
    if let Some(stack) = root.downcast_ref::<gtk::Stack>() {
        return stack
            .visible_child()
            .and_then(|page| context_model_row(&page, label));
    }
    if root.type_().name() == "GtkModelButton" {
        fn has_label(w: &gtk::Widget, label: &str) -> bool {
            if w.downcast_ref::<gtk::Label>()
                .is_some_and(|l| l.text() == label)
            {
                return true;
            }
            let mut c = w.first_child();
            while let Some(child) = c {
                if has_label(&child, label) {
                    return true;
                }
                c = child.next_sibling();
            }
            false
        }
        if has_label(root, label) {
            return Some(root.clone());
        }
    }
    let mut c = root.first_child();
    while let Some(child) = c {
        if let Some(found) = context_model_row(&child, label) {
            return Some(found);
        }
        c = child.next_sibling();
    }
    None
}
pub(super) fn context_activate_row(pop: &impl IsA<gtk::Widget>, label: &str) {
    let row = context_model_row(pop, label).unwrap_or_else(|| panic!("missing native row {label}"));
    assert!(row.is_sensitive());
    if let Some(button) = row.downcast_ref::<gtk::Button>() {
        // GtkButton::activate animates and emits clicked later. This helper's
        // callers assert synchronous results; drive the same native click signal.
        button.emit_clicked();
    } else {
        assert!(row.activate(), "native menu row must activate");
    }
}
pub(super) fn context_picture_editor(ui: &Rc<Ui>) {
    if let Some(panel) = ui.elements.borrow().as_ref() {
        panel.show("image");
        return;
    }
    if let Some(s) = ui.picture.borrow().as_ref() {
        s.refresh();
        s.pop.popup();
    }
}
pub(super) fn context_table_editor(ui: &Rc<Ui>) {
    if let Some(panel) = ui.elements.borrow().as_ref() {
        panel.show("table");
        return;
    }
    if let Some(p) = ui.table_popover.borrow().as_ref() {
        refresh_table_controls(ui, p);
        p.popup();
    }
}
pub(super) fn tool_diagnostics() -> bool {
    diagnostics::any_flag(|a| {
        matches!(
            a,
            "--pin-ui-test"
                | "--table-menu-demo"
                | "--table-properties-demo"
                | "--table-insert-demo"
                | "--picture-demo"
                | "--document-properties-demo"
                | "--document-tools-test"
        )
    })
}

pub(super) fn append_proofing_actions(
    root: &gio::Menu,
    group: &gio::SimpleActionGroup,
    pop: &gtk::PopoverMenu,
    ui: &Rc<Ui>,
    issue: proofing::Issue,
) {
    let identity = document_identity(ui);
    let spelling = gio::Menu::new();
    let suggestions = {
        let mut v = ui.current.borrow_mut();
        proofing::suggestions(&mut v.as_mut().unwrap().session, &issue)
    };
    for (n, text) in suggestions.into_iter().enumerate() {
        let i = issue.clone();
        let replacement = text.clone();
        context_item(
            &spelling,
            &group,
            &pop,
            ui,
            &text,
            &format!("spelling-{n}"),
            true,
            move |u| {
                if document_identity(u) == identity {
                    proof_replace(u, &i, &replacement);
                } else {
                    u.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                        "The text has changed. Check it again."
                    )));
                }
            },
        );
    }
    if issue.spelling {
        for (name, label, add_word) in [
            ("ignore-word", wordcraft_gnome::t!("Ignore All"), false),
            ("add-word", wordcraft_gnome::t!("Add to Dictionary"), true),
        ] {
            let issue = issue.clone();
            context_item(&spelling, &group, &pop, ui, label, name, true, move |u| {
                if document_identity(u) != identity {
                    return;
                }
                let result = {
                    let mut v = u.current.borrow_mut();
                    let s = &mut v.as_mut().unwrap().session;
                    if add_word {
                        proofing::add(s, &issue)
                    } else {
                        proofing::ignore(s, &issue)
                    }
                };
                if let Err(e) = result {
                    u.toasts.add_toast(Toast::new(&e));
                }
                refresh_all_proofing(u);
            });
        }
    }
    root.append_section(None, &spelling);
}
