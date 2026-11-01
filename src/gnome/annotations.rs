use super::*;
use wordcraft_gnome::annotations as model;

/// Derived sidebar content; WordCraft owns annotation data and undo history.
pub(super) struct AnnotationPanel {
    ui: std::rc::Weak<Ui>,
    toggle: gtk::ToggleButton,
    content: gtk::Box,
    pending: Cell<bool>,
}

impl AnnotationPanel {
    pub(super) fn show(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if let Some(search) = ui.search.borrow().as_ref() {
            search.toggle.set_active(false);
        }
        self.toggle.set_active(true);
        ui.sidebar_pages.set_visible_child_name("navigation");
        ui.navigation_pages.set_visible_child_name("annotations");
        ui.sidebar_button.set_active(true);
        self.refresh();
    }

    pub(super) fn schedule(self: &Rc<Self>) {
        if !self.toggle.is_active() || self.pending.replace(true) {
            return;
        }
        let weak = Rc::downgrade(self);
        glib::idle_add_local_once(move || {
            if let Some(panel) = weak.upgrade() {
                panel.pending.set(false);
                panel.refresh();
            }
        });
    }

    pub(super) fn refresh(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if !self.toggle.is_active() {
            return;
        }
        let (comments, writable) = {
            let value = ui.current.borrow();
            let Some(d) = value.as_ref() else { return };
            (model::comments(&d.session), model::can_comment(&d.session))
        };
        clear_box(&self.content);
        self.content
            .append(&section_label(wordcraft_gnome::t!("Comments")));
        if comments.is_empty() {
            self.content
                .append(&annotation_label(wordcraft_gnome::t!("No Comments"), true));
        }
        // Reading-order roots, followed by their replies. Orphans/cycles remain visible.
        let mut seen = HashSet::new();
        for c in comments
            .iter()
            .filter(|c| c.parent.is_none() || !comments.iter().any(|p| Some(p.id) == c.parent))
        {
            self.append_thread(&ui, &comments, c.id, writable, &mut seen);
        }
        for c in &comments {
            if !seen.contains(&c.id) {
                self.append_thread(&ui, &comments, c.id, writable, &mut seen);
            }
        }
    }

    fn append_thread(
        &self,
        ui: &Rc<Ui>,
        comments: &[model::Comment],
        root: u32,
        writable: bool,
        seen: &mut HashSet<u32>,
    ) {
        let thread = gtk::Box::new(gtk::Orientation::Vertical, 0);
        thread.set_widget_name(&format!("comment-thread-{root}"));
        thread.add_css_class("card");
        thread.set_overflow(gtk::Overflow::Hidden);
        let mut stack = vec![(root, 0usize)];
        while let Some((id, depth)) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(c) = comments.iter().find(|c| c.id == id) else {
                continue;
            };
            if thread.first_child().is_some() {
                thread.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            }
            thread.append(&comment_message(
                ui,
                c,
                depth,
                writable,
                model::anchor_in(comments, c.id).is_some(),
            ));
            for reply in comments.iter().rev().filter(|c| c.parent == Some(id)) {
                stack.push((reply.id, depth + 1));
            }
        }
        self.content.append(&thread);
    }
}

fn annotation_label(text: &str, dim: bool) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label.set_max_width_chars(1);
    if dim {
        label.add_css_class("dim-label");
    }
    label
}

fn section_label(text: &str) -> gtk::Label {
    let label = annotation_label(text, false);
    label.add_css_class("heading");
    label.set_margin_top(12);
    label.set_margin_bottom(4);
    label
}

fn annotation_button(icon: &str, label: &str) -> gtk::Button {
    let button = gtk::Button::from_icon_name(icon);
    button.add_css_class("flat");
    button.set_tooltip_text(Some(label));
    accessible_name(&button, label);
    button
}

fn annotation_navigate(ui: &Rc<Ui>, f: impl FnOnce(&mut Session) -> Result<(), String>) {
    let result = {
        let mut value = ui.current.borrow_mut();
        let Some(d) = value.as_mut() else { return };
        f(&mut d.session)
    };
    if let Err(e) = result {
        ui.toasts.add_toast(Toast::new(&e));
        return;
    }
    refresh_selection(ui);
    caret_ensure_visible(ui);
    if ui.sidebar.is_collapsed() {
        ui.sidebar_button.set_active(false);
    }
    ui.document_box.grab_focus();
}

pub(super) fn bookmark_row(ui: &Rc<Ui>, name: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let go = gtk::Button::new();
    let label = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    label.append(&gtk::Image::from_icon_name("bookmark-new-symbolic"));
    let text = annotation_label(name, false);
    text.set_hexpand(true);
    label.append(&text);
    go.set_child(Some(&label));
    go.set_hexpand(true);
    go.add_css_class("flat");
    go.set_widget_name(&format!("bookmark-go-{name}"));
    row.append(&go);
    let remove = annotation_button(
        "user-trash-symbolic",
        wordcraft_gnome::t!("Delete Bookmark"),
    );
    remove.set_widget_name(&format!("bookmark-delete-{name}"));
    remove.set_sensitive(
        ui.current
            .borrow()
            .as_ref()
            .is_some_and(|d| model::can_bookmark(&d.session)),
    );
    row.append(&remove);
    let identity = document_identity(ui);
    let weak = Rc::downgrade(ui);
    let n = name.to_string();
    go.connect_clicked(move |_| {
        if let Some(ui) = annotation_target(&weak, identity) {
            annotation_navigate(&ui, |s| model::goto_bookmark(s, &n));
        }
    });
    let weak = Rc::downgrade(ui);
    let n = name.to_string();
    remove.connect_clicked(move |_| {
        if let Some(ui) = annotation_target(&weak, identity) {
            tools_change(&ui, |s| model::delete_bookmark(s, &n));
        }
    });
    row
}

fn comment_message(
    ui: &Rc<Ui>,
    c: &model::Comment,
    depth: usize,
    writable: bool,
    has_anchor: bool,
) -> gtk::Box {
    let message = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    message.set_widget_name(&format!("comment-{}", c.id));
    message.set_margin_start((depth.min(2) * 10) as i32);
    if depth > 0 {
        let branch = gtk::Separator::new(gtk::Orientation::Vertical);
        branch.set_margin_top(10);
        branch.set_margin_bottom(10);
        message.append(&branch);
    }
    let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    body.set_hexpand(true);
    body.set_margin_start(10);
    body.set_margin_end(10);
    body.set_margin_top(10);
    body.set_margin_bottom(10);
    // Plain labels deliberately avoid interpreting document content as markup.
    let author = annotation_label(&c.author, false);
    author.add_css_class("heading");
    body.append(&author);
    if !c.date.is_empty() {
        let formatted = glib::DateTime::from_iso8601(&c.date, None)
            .and_then(|d| d.to_local())
            .and_then(|d| d.format("%x %H:%M"))
            .map(|s| s.to_string())
            .unwrap_or_else(|_| c.date.clone());
        let date = annotation_label(&formatted, true);
        date.add_css_class("caption");
        body.append(&date);
    }
    if c.resolved {
        body.append(&annotation_label(wordcraft_gnome::t!("Resolved"), true));
    }
    body.append(&annotation_label(&c.text, false));
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let go = annotation_button("go-jump-symbolic", wordcraft_gnome::t!("Go to Comment"));
    go.set_widget_name(&format!("comment-go-{}", c.id));
    go.set_sensitive(has_anchor);
    actions.append(&go);
    let reply = gtk::Button::with_label(wordcraft_gnome::t!("Reply"));
    reply.add_css_class("flat");
    reply.set_sensitive(writable);
    reply.set_widget_name(&format!("comment-reply-{}", c.id));
    actions.append(&reply);
    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    actions.append(&spacer);
    let menu = gtk::MenuButton::new();
    menu.set_widget_name(&format!("comment-options-{}", c.id));
    menu.set_icon_name("view-more-symbolic");
    menu.add_css_class("flat");
    accessible_name(&menu, wordcraft_gnome::t!("Comment Options"));
    let items = gio::Menu::new();
    let group = gio::SimpleActionGroup::new();
    let identity = document_identity(ui);
    let id = c.id;
    for (name, label) in [
        ("edit", wordcraft_gnome::t!("Edit Comment…")),
        (
            "resolve",
            if c.resolved {
                wordcraft_gnome::t!("Reopen Comment")
            } else {
                wordcraft_gnome::t!("Resolve Comment")
            },
        ),
        ("delete", wordcraft_gnome::t!("Delete Comment…")),
    ] {
        if name == "resolve" && c.parent.is_some() {
            continue;
        }
        let action = gio::SimpleAction::new(name, None);
        action.set_enabled(writable);
        let weak = Rc::downgrade(ui);
        let resolved = c.resolved;
        action.connect_activate(move |_, _| {
            let Some(ui) = annotation_target(&weak, identity) else {
                return;
            };
            match name {
                "edit" => {
                    show_annotation_dialog(&ui, AnnotationEdit::Edit(id));
                }
                "resolve" => {
                    tools_change(&ui, |s| model::resolve(s, id, !resolved));
                }
                _ => confirm_delete_comment(&ui, id),
            }
        });
        group.add_action(&action);
        items.append(Some(label), Some(&format!("annotation.{name}")));
    }
    menu.insert_action_group("annotation", Some(&group));
    menu.set_menu_model(Some(&items));
    actions.append(&menu);
    body.append(&actions);
    message.append(&body);
    let weak = Rc::downgrade(ui);
    go.connect_clicked(move |_| {
        if let Some(ui) = annotation_target(&weak, identity) {
            annotation_navigate(&ui, |s| model::goto_comment(s, id));
        }
    });
    let weak = Rc::downgrade(ui);
    reply.connect_clicked(move |_| {
        if let Some(ui) = annotation_target(&weak, identity) {
            show_annotation_dialog(&ui, AnnotationEdit::Reply(id));
        }
    });
    message
}

pub(super) fn annotation_target(
    weak: &std::rc::Weak<Ui>,
    identity: Option<(u64, u64)>,
) -> Option<Rc<Ui>> {
    let ui = weak.upgrade().filter(|u| !u.closed.get())?;
    if document_identity(&ui) != identity {
        ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
            "The document has changed; try again."
        )));
        if let Some(panel) = ui.annotations.borrow().as_ref() {
            panel.schedule();
        }
        if let Some(panel) = ui.outline.borrow().as_ref() {
            panel.schedule();
        }
        return None;
    }
    Some(ui)
}

fn confirm_delete_comment(ui: &Rc<Ui>, id: u32) {
    let Some(host) = ui.host.upgrade() else {
        return;
    };
    let dialog = adw::MessageDialog::new(
        Some(&host.window),
        Some(wordcraft_gnome::t!("Delete Comment?")),
        Some(wordcraft_gnome::t!(
            "This also deletes its replies. You can undo this change."
        )),
    );
    dialog.add_response("cancel", wordcraft_gnome::t!("Cancel"));
    dialog.add_response("delete", wordcraft_gnome::t!("Delete"));
    dialog.set_default_response(Some("cancel"));
    dialog.set_close_response("cancel");
    dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
    let weak = Rc::downgrade(ui);
    let identity = document_identity(ui);
    dialog.connect_response(None, move |_, response| {
        if response == "delete" {
            if let Some(ui) = annotation_target(&weak, identity) {
                tools_change(&ui, |s| model::delete_comment(s, id));
            }
        }
    });
    dialog.present();
}

pub(super) fn install_annotations(ui: &Rc<Ui>, toggle: &gtk::ToggleButton) -> Rc<AnnotationPanel> {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.set_margin_start(10);
    content.set_margin_end(10);
    content.set_margin_bottom(12);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_widget_name("comments-panel");
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_child(Some(&content));
    ui.navigation_pages.add_named(&scroll, Some("annotations"));
    let panel = Rc::new(AnnotationPanel {
        ui: Rc::downgrade(ui),
        toggle: toggle.clone(),
        content,
        pending: Cell::new(false),
    });
    let weak = Rc::downgrade(&panel);
    toggle.connect_toggled(move |b| {
        if b.is_active() {
            if let Some(panel) = weak.upgrade() {
                panel.show();
            }
        }
    });
    panel
}
