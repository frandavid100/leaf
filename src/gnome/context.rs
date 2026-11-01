use super::*;

// Native context menus. Selection and editing remain public engine commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DocumentContext {
    Text,
    Table,
    Picture,
}

// Share the engine fragment between tabs only while GTK still owns this exact
// clipboard provider. Never paste an old fragment after another app copies.
thread_local! {
    static CLIPBOARD_WATCHED:Cell<bool>=const{Cell::new(false)};
    pub(super) static RICH_CLIPBOARD: RefCell<Option<(gdk::ContentProvider,wordcraft_doc::edit::Fragment)>> = const { RefCell::new(None) };
}
pub(super) fn context_copy(ui: &Rc<Ui>) -> bool {
    if !CLIPBOARD_WATCHED.with(|v| v.replace(true)) {
        ui.document_box.clipboard().connect_changed(|clipboard| {
            RICH_CLIPBOARD.with(|c| {
                let mut c = c.borrow_mut();
                if c.as_ref()
                    .is_some_and(|(provider, _)| clipboard.content().as_ref() != Some(provider))
                {
                    c.take();
                }
            });
        });
    }
    let result = {
        let mut value = ui.current.borrow_mut();
        let Some(d) = value.as_mut() else {
            return false;
        };
        if d.session.sel.is_collapsed() {
            return false;
        }
        d.session
            .run("edit.copy", &serde_json::json!({}))
            .map(|v| (v, d.session.clipboard.clone()))
    };
    let (value, fragment) = match result {
        Ok(v) => v,
        Err(e) => {
            ui.toasts.add_toast(Toast::new(&e.to_string()));
            return false;
        }
    };
    let text = value["text"].as_str().unwrap_or_default();
    let mut providers = vec![gdk::ContentProvider::for_value(&text.to_value())];
    if let Some(html) = value["html"].as_str().filter(|h| !h.is_empty()) {
        providers.push(gdk::ContentProvider::for_bytes(
            "text/html",
            &glib::Bytes::from_owned(html.as_bytes().to_vec()),
        ));
    }
    let provider = gdk::ContentProvider::new_union(&providers);
    if let Err(e) = ui.document_box.clipboard().set_content(Some(&provider)) {
        ui.toasts.add_toast(Toast::new(&e.to_string()));
        return false;
    }
    RICH_CLIPBOARD.with(|c| c.replace(fragment.map(|f| (provider, f))));
    true
}
pub(super) fn context_paste(ui: &Rc<Ui>) {
    let clipboard = ui.document_box.clipboard();
    let fragment = RICH_CLIPBOARD.with(|c| {
        c.borrow()
            .as_ref()
            .filter(|(p, _)| clipboard.content().as_ref() == Some(p))
            .map(|(_, f)| f.clone())
    });
    if let Some(fragment) = fragment {
        tools_change(ui, move |s| {
            s.clipboard_text = fragment.clipboard_text();
            s.clipboard = Some(fragment);
            s.run("edit.paste", &serde_json::json!({}))
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        ui.document_box.grab_focus();
        caret_ensure_visible(ui);
        return;
    }
    let identity = document_identity(ui);
    let selection = ui.current.borrow().as_ref().map(|d| d.session.sel.clone());
    let weak = Rc::downgrade(ui);
    clipboard.read_text_async(None::<&gio::Cancellable>, move |result| {
        let Some(ui) = weak.upgrade() else {
            return;
        };
        if ui.closed.get()
            || document_identity(&ui) != identity
            || ui.current.borrow().as_ref().map(|d| d.session.sel.clone()) != selection
        {
            return;
        }
        match result {
            Ok(Some(text)) => apply_user_edit(&ui, Edit::Paste(text.to_string())),
            Ok(None) => {}
            Err(e) => ui.toasts.add_toast(Toast::new(&e.to_string())),
        }
        ui.document_box.grab_focus();
    });
}
pub(super) fn context_prepare(ui: &Rc<Ui>, x: f64, y: f64) -> Option<DocumentContext> {
    let pos = pos_at_canvas(ui, x, y)?;
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
        let Some(origin) = format_button(ui, name).and_then(|m| m.popover()) else {
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
        "cut" => Some("<Control>x"),
        "copy" => Some("<Control>c"),
        "paste" => Some("<Control>v"),
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
    }
    let edit = gio::Menu::new();
    context_item(
        &edit,
        &group,
        &pop,
        ui,
        wordcraft_gnome::t!("Link"),
        "link",
        selected,
        |u| {
            show_link(u);
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
    context_item(
        &edit,
        &group,
        &pop,
        ui,
        wordcraft_gnome::t!("Copy"),
        "copy",
        selected,
        |u| {
            context_copy(u);
        },
    );
    let formats = ui.document_box.clipboard().formats();
    let can_paste = formats.contains_type(String::static_type())
        || formats.contain_mime_type("text/plain")
        || formats.contain_mime_type("text/plain;charset=utf-8");
    context_item(
        &edit,
        &group,
        &pop,
        ui,
        wordcraft_gnome::t!("Paste"),
        "paste",
        can_paste,
        context_paste,
    );
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
    root.append_section(None, &edit);
    let annotations = gio::Menu::new();
    let (can_bookmark, can_comment) = ui
        .current
        .borrow()
        .as_ref()
        .map(|d| {
            (
                wordcraft_gnome::annotations::can_bookmark(&d.session),
                wordcraft_gnome::annotations::can_comment(&d.session),
            )
        })
        .unwrap_or_default();
    context_item(
        &annotations,
        &group,
        &pop,
        ui,
        wordcraft_gnome::t!("Add Bookmark…"),
        "add-bookmark",
        can_bookmark,
        |u| {
            show_annotation_dialog(u, AnnotationEdit::Bookmark);
        },
    );
    context_item(
        &annotations,
        &group,
        &pop,
        ui,
        wordcraft_gnome::t!("Add Comment…"),
        "add-comment",
        can_comment,
        |u| {
            show_annotation_dialog(u, AnnotationEdit::NewComment);
        },
    );
    root.append_section(None, &annotations);
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
                    wordcraft_gnome::t!("Add"),
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
                wordcraft_gnome::t!("More Table Options…"),
                "table-tools",
                true,
                context_table_editor,
            );
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
            let p = parent.downgrade();
            context_item(
                &specific,
                &group,
                &pop,
                ui,
                wordcraft_gnome::t!("Properties and Split Cells…"),
                "table-properties",
                true,
                move |u| {
                    if let Some(p) = p.upgrade() {
                        show_table_properties(u, &p);
                    }
                },
            );
        }
    }
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
pub(super) fn context_activate_row(pop: &gtk::PopoverMenu, label: &str) {
    let row = context_model_row(pop, label).unwrap_or_else(|| panic!("missing native row {label}"));
    assert!(row.is_sensitive());
    assert!(row.activate(), "native menu row must activate");
}
pub(super) fn context_picture_editor(ui: &Rc<Ui>) {
    if let Some(s) = ui.picture.borrow().as_ref() {
        s.refresh();
        s.pop.popup();
    }
}
pub(super) fn context_table_editor(ui: &Rc<Ui>) {
    if let Some(p) = ui.table_popover.borrow().as_ref() {
        refresh_table_controls(ui, p);
        p.popup();
    }
}
pub(super) fn tool_diagnostics() -> bool {
    std::env::args().any(|a| {
        matches!(
            a.as_str(),
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
