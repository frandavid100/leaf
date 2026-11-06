use super::*;
use wordcraft_gnome::annotations as model;

/// Derived sidebar content; WordCraft owns annotation data and undo history.
pub(super) struct AnnotationPanel {
    ui: std::rc::Weak<Ui>,
    toggle: gtk::ToggleButton,
    pub(super) content: gtk::Box,
    pub(super) scroll: gtk::ScrolledWindow,
    pub(super) active: RefCell<HashSet<u32>>,
    document: Cell<Option<u64>>,
    pending: Cell<bool>,
    draft: RefCell<Option<CommentComposer>>,
    rendered: RefCell<
        Option<(
            u64,
            Vec<model::Comment>,
            bool,
            HashSet<u32>,
            Option<gtk::Widget>,
        )>,
    >,
}

impl AnnotationPanel {
    pub(super) fn reveal(self: &Rc<Self>, ids: Vec<u32>) {
        let Some(ui) = self.ui.upgrade() else { return };
        self.document.set(
            ui.current
                .borrow()
                .as_ref()
                .map(|d| d.session.document_id()),
        );
        self.active.replace(ids.iter().copied().collect());
        self.show();
        let identity = document_identity(&ui);
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(80), move || {
            let Some(panel) = weak.upgrade() else { return };
            let Some(ui) = panel.ui.upgrade() else { return };
            if document_identity(&ui) != identity {
                return;
            }
            if let Some(card) = ids
                .first()
                .and_then(|id| find_widget(&panel.content, &format!("comment-thread-{id}")))
            {
                if let Some(bounds) = card.compute_bounds(&panel.content) {
                    panel.scroll.vadjustment().set_value(bounds.y() as f64);
                }
            }
        });
    }
    pub(super) fn show(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if let Some(review) = ui.review.borrow().as_ref() {
            review.show(Some(ReviewKind::Comment));
            return;
        }
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
        if let Some(ui) = self.ui.upgrade() {
            if let Some(review) = ui.review.borrow().as_ref() {
                review.schedule();
                return;
            }
        }
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
        if let Some(review) = ui.review.borrow().as_ref() {
            if !review.rebuilding.get() {
                review.refresh();
                return;
            }
        }
        if !self.toggle.is_active() {
            return;
        }
        let (comments, writable, id) = {
            let value = ui.current.borrow();
            let Some(d) = value.as_ref() else { return };
            (
                model::comments(&d.session),
                model::can_comment(&d.session),
                d.session.document_id(),
            )
        };
        if self.document.replace(Some(id)) != Some(id) {
            self.active.borrow_mut().clear();
            self.draft.borrow_mut().take();
        }
        self.active
            .borrow_mut()
            .retain(|id| comments.iter().any(|c| c.id == *id));
        let signature = (
            id,
            comments.clone(),
            writable,
            self.active.borrow().clone(),
            self.draft
                .borrow()
                .as_ref()
                .map(|d| d.root.clone().upcast::<gtk::Widget>()),
        );
        let roots_present = comments.is_empty()
            || comments
                .iter()
                .any(|c| find_widget(&self.content, &format!("comment-thread-{}", c.id)).is_some());
        if roots_present && self.rendered.borrow().as_ref() == Some(&signature) {
            return;
        }
        self.rendered.replace(Some(signature));
        let draft = self.draft.borrow();
        let had_focus = draft.as_ref().is_some_and(|draft| draft.text.has_focus());
        if let Some(draft) = draft.as_ref() {
            draft.detach();
        }
        clear_box(&self.content);
        if let Some(draft) = draft.as_ref().filter(|draft| {
            draft
                .parent
                .is_none_or(|id| !comments.iter().any(|comment| comment.id == id))
        }) {
            self.content.append(&draft.root);
        }
        drop(draft);
        accessible_name(
            &self.content,
            if ui.review.borrow().is_some() {
                wordcraft_gnome::t!("Review")
            } else {
                wordcraft_gnome::t!("Comments")
            },
        );
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
                self.append_thread(
                    &ui,
                    &comments,
                    model::thread_root(&comments, c.id),
                    writable,
                    &mut seen,
                );
            }
        }
        if had_focus {
            if let Some(draft) = self.draft.borrow().as_ref() {
                draft.text.grab_focus();
            }
        }
    }

    pub(super) fn compose(self: &Rc<Self>, parent: Option<u32>) {
        let Some(ui) = self.ui.upgrade().filter(|ui| !ui.closed.get()) else {
            return;
        };
        self.show();
        if self.draft.borrow().is_none() {
            let Some(draft) = CommentComposer::new(self, &ui, parent) else {
                return;
            };
            self.draft.replace(Some(draft));
            self.refresh();
        }
        if let Some(draft) = self.draft.borrow().as_ref() {
            draft.text.grab_focus();
        }
    }

    pub(super) fn dismiss_composer(&self, root: &gtk::Box) -> bool {
        let mut draft = self.draft.borrow_mut();
        if draft.as_ref().is_none_or(|draft| draft.root != *root) {
            return false;
        }
        let draft = draft.take().unwrap();
        draft.detach();
        true
    }

    pub(super) fn sync_caret(&self) {
        if !self.toggle.is_active() {
            return;
        }
        let Some(ui) = self.ui.upgrade() else { return };
        let root = {
            let value = ui.current.borrow();
            let Some(doc) = value.as_ref() else { return };
            let (start, _) = doc.session.sel.ordered();
            let roots = model::threads_at(&doc.session, &start);
            let active = self.active.borrow();
            roots
                .iter()
                .find(|root| active.contains(root))
                .copied()
                .or_else(|| roots.first().copied())
        };
        self.select_thread(root);
    }

    fn select_thread(&self, root: Option<u32>) {
        let active: HashSet<_> = root.into_iter().collect();
        if *self.active.borrow() == active {
            return;
        }
        self.active.replace(active);
        let mut child = self.content.first_child();
        while let Some(row) = child {
            child = row.next_sibling();
            let name = row.widget_name();
            let Some(id) = name
                .strip_prefix("comment-thread-")
                .and_then(|s| s.parse::<u32>().ok())
            else {
                continue;
            };
            let expanded = Some(id) == root;
            if expanded {
                row.add_css_class("selected");
            } else {
                row.remove_css_class("selected");
            }
            if let Some(label) = find_widget(&row, "comment-root-text")
                .and_then(|widget| widget.downcast::<gtk::Label>().ok())
            {
                label.set_lines(if expanded { -1 } else { 2 });
                label.set_ellipsize(if expanded {
                    gtk::pango::EllipsizeMode::None
                } else {
                    gtk::pango::EllipsizeMode::End
                });
            }
            if let Some(actions) = find_widget(&row, "comment-root-actions") {
                actions.set_visible(expanded);
            }
            if let Some(details) = find_widget(&row, "comment-replies")
                .and_then(|widget| widget.downcast::<gtk::Revealer>().ok())
            {
                details.set_reveal_child(expanded);
            }
            if let Some(count) = find_widget(&row, "comment-reply-count") {
                count.set_visible(!expanded);
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
        thread.add_css_class("comment-thread");
        if self.active.borrow().contains(&root) {
            thread.add_css_class("selected");
        }
        thread.set_overflow(gtk::Overflow::Hidden);
        let expanded = self.active.borrow().contains(&root);
        let replies = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let details = gtk::Revealer::new();
        details.set_widget_name("comment-replies");
        details.set_transition_type(gtk::RevealerTransitionType::SlideDown);
        details.set_transition_duration(160);
        details.set_child(Some(&replies));
        let mut count = 0;
        let mut stack = vec![(root, 0usize)];
        while let Some((id, depth)) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            let Some(c) = comments.iter().find(|c| c.id == id) else {
                continue;
            };
            let destination = if depth == 0 {
                &thread
            } else {
                count += 1;
                &replies
            };
            if depth > 0 {
                let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                separator.add_css_class("comment-divider");
                separator.set_margin_start(8);
                separator.set_margin_end(8);
                destination.append(&separator);
            }
            destination.append(&comment_message(
                ui,
                c,
                root,
                depth,
                writable,
                model::anchor_in(comments, c.id).is_some(),
                expanded,
            ));
            if let Some(draft) = self
                .draft
                .borrow()
                .as_ref()
                .filter(|draft| draft.parent == Some(id))
            {
                destination.append(&draft.root);
            }
            for reply in comments.iter().rev().filter(|c| c.parent == Some(id)) {
                stack.push((reply.id, depth + 1));
            }
        }
        if count > 0 {
            let count = annotation_label(
                &wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Replies: {count}"),
                    &[("count", count.to_string())],
                ),
                true,
            );
            count.set_widget_name("comment-reply-count");
            count.set_margin_start(14);
            count.set_margin_bottom(8);
            count.set_visible(!expanded);
            thread.append(&count);
        }
        details.set_reveal_child(expanded);
        thread.append(&details);
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
    text.add_css_class("bookmark-name");
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
    root: u32,
    depth: usize,
    writable: bool,
    has_anchor: bool,
    expanded: bool,
) -> gtk::Box {
    let message = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    message.set_widget_name(&format!("comment-{}", c.id));
    message.set_margin_start((depth.min(2) * 6) as i32);
    if depth > 0 {
        let branch = gtk::DrawingArea::new();
        branch.set_content_width(10);
        branch.set_draw_func(|widget, cr, _, height| {
            let color = widget.color();
            cr.set_source_rgba(
                color.red().into(),
                color.green().into(),
                color.blue().into(),
                0.38,
            );
            cr.set_line_width(1.);
            cr.move_to(2.5, 0.);
            cr.line_to(2.5, 13.5_f64.min(f64::from(height)));
            cr.curve_to(2.5, 16.5, 5., 16.5, 7.5, 16.5);
            let _ = cr.stroke();
            cr.arc(7.5, 16.5, 2., 0., std::f64::consts::TAU);
            let _ = cr.fill();
        });
        message.append(&branch);
    }
    let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    body.set_hexpand(true);
    body.set_margin_start(6);
    body.set_margin_end(6);
    body.set_margin_top(8);
    body.set_margin_bottom(8);
    let summary = gtk::Box::new(gtk::Orientation::Vertical, 4);
    // Plain labels deliberately avoid interpreting document content as markup.
    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let author = annotation_label(&c.author, false);
    author.set_hexpand(true);
    author.set_wrap(false);
    author.set_ellipsize(gtk::pango::EllipsizeMode::End);
    author.add_css_class("heading");
    if depth == 0 {
        heading.append(&review_type_icon(ReviewKind::Comment));
    }
    heading.append(&author);
    if let Ok(date) = glib::DateTime::from_iso8601(&c.date, None).and_then(|date| date.to_local()) {
        let age =
            glib::DateTime::now_local().map_or(0, |now| (now.to_unix() - date.to_unix()).max(0));
        let day = 86400;
        let (count, format) = if age >= 365 * day {
            (age / (365 * day), wordcraft_gnome::t!("{count}Y"))
        } else if age >= 30 * day {
            (age / (30 * day), wordcraft_gnome::t!("{count}M"))
        } else if age >= day {
            (age / day, wordcraft_gnome::t!("{count}D"))
        } else {
            (age / 3600, wordcraft_gnome::t!("{count}H"))
        };
        let value = wordcraft_gnome::i18n::interpolate(format, &[("count", count.to_string())]);
        let timestamp = gtk::Label::new(Some(&value));
        timestamp.add_css_class("caption");
        timestamp.add_css_class("dim-label");
        if let Ok(full) = date.format("%x %X") {
            timestamp.set_tooltip_text(Some(&full));
        }
        heading.append(&timestamp);
    }
    summary.append(&heading);
    if c.resolved {
        summary.append(&annotation_label(wordcraft_gnome::t!("Resolved"), true));
    }
    let text = annotation_label(&c.text, false);
    text.add_css_class("comment-body");
    if depth == 0 {
        text.set_widget_name("comment-root-text");
        if !expanded {
            text.set_lines(2);
            text.set_ellipsize(gtk::pango::EllipsizeMode::End);
        }
    }
    summary.append(&text);
    let go = gtk::Button::new();
    go.add_css_class("flat");
    go.set_child(Some(&summary));
    go.set_widget_name(&format!("comment-go-{}", c.id));
    go.set_tooltip_text(Some(wordcraft_gnome::t!("Go to Comment")));
    go.set_sensitive(has_anchor);
    body.append(&go);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 2);
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
        let expected = c.clone();
        action.connect_activate(move |_, _| {
            let Some(ui) = comment_target(&weak, identity, &expected) else {
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
    if depth == 0 {
        actions.set_widget_name("comment-root-actions");
        actions.set_visible(expanded);
    }
    body.append(&actions);
    message.append(&body);
    let weak = Rc::downgrade(ui);
    let expected = c.clone();
    go.connect_clicked(move |_| {
        if let Some(ui) = comment_target(&weak, identity, &expected) {
            if let Some(panel) = ui.annotations.borrow().as_ref() {
                panel.select_thread(Some(root));
            }
            annotation_navigate(&ui, |s| model::goto_comment(s, id));
        }
    });
    let weak = Rc::downgrade(ui);
    let expected = c.clone();
    reply.connect_clicked(move |_| {
        if let Some(ui) = comment_target(&weak, identity, &expected) {
            compose_comment(&ui, Some(id));
        }
    });
    message
}

// A retained comment card can outlive unrelated edits. Resolve its id against
// current canonical comments, without weakening revision leases on dialogs.
fn comment_target(
    weak: &std::rc::Weak<Ui>,
    identity: Option<(u64, u64)>,
    expected: &model::Comment,
) -> Option<Rc<Ui>> {
    let ui = weak.upgrade().filter(|u| !u.closed.get())?;
    let valid = ui.current.borrow().as_ref().is_some_and(|d| {
        Some(d.session.document_id()) == identity.map(|i| i.0)
            && model::comments(&d.session).contains(expected)
    });
    valid.then_some(ui)
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
    content.set_margin_start(6);
    content.set_margin_end(6);
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
        scroll,
        active: RefCell::new(HashSet::new()),
        document: Cell::new(None),
        rendered: RefCell::new(None),
        pending: Cell::new(false),
        draft: RefCell::new(None),
    });
    panel
}

pub(super) fn compose_comment(ui: &Rc<Ui>, parent: Option<u32>) {
    let panel = ui.annotations.borrow().as_ref().cloned();
    if let Some(panel) = panel {
        panel.compose(parent);
    }
}
