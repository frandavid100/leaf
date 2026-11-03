use super::*;
use wordcraft_gnome::annotations as model;

/// The draft holds widgets only. The submit callback leases the document revision
/// and the original selection so moving the caret cannot retarget a new comment.
pub(super) struct CommentComposer {
    pub(super) parent: Option<u32>,
    pub(super) root: gtk::Box,
    pub(super) text: gtk::TextView,
}

impl CommentComposer {
    pub(super) fn new(
        panel: &Rc<AnnotationPanel>,
        ui: &Rc<Ui>,
        parent: Option<u32>,
    ) -> Option<Self> {
        let identity = document_identity(ui)?;
        let (selection, page_hint) = {
            let current = ui.current.borrow();
            let session = &current.as_ref()?.session;
            if !model::can_comment(session) {
                ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                    "This annotation cannot be added or changed here."
                )));
                return None;
            }
            (session.sel.clone(), session.page_hint)
        };
        let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
        root.set_widget_name("comment-composer");
        root.set_margin_start(8);
        root.set_margin_end(8);
        root.set_margin_top(8);
        root.set_margin_bottom(8);
        let title = gtk::Label::new(Some(if parent.is_some() {
            wordcraft_gnome::t!("Reply to Comment")
        } else {
            wordcraft_gnome::t!("Add Comment")
        }));
        title.set_xalign(0.0);
        title.add_css_class("heading");
        root.append(&title);

        let text = gtk::TextView::new();
        text.set_widget_name("comment-text");
        text.set_wrap_mode(gtk::WrapMode::WordChar);
        accessible_name(&text, wordcraft_gnome::t!("Comment Text"));
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_min_content_height(72);
        scroll.set_child(Some(&text));
        scroll.add_css_class("frame");
        root.append(&scroll);

        let error = gtk::Label::new(None);
        error.set_widget_name("annotation-error");
        error.set_wrap(true);
        error.set_max_width_chars(24);
        error.add_css_class("error");
        error.set_visible(false);
        root.append(&error);
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        actions.set_halign(gtk::Align::End);
        let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
        cancel.set_widget_name("annotation-cancel");
        let submit = gtk::Button::with_label(wordcraft_gnome::t!("Add"));
        submit.set_widget_name("annotation-submit");
        submit.add_css_class("suggested-action");
        submit.set_sensitive(false);
        actions.append(&cancel);
        actions.append(&submit);
        root.append(&actions);

        let button = submit.downgrade();
        text.buffer().connect_changed(move |buffer| {
            if let Some(button) = button.upgrade() {
                let body = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
                button.set_sensitive(!body.trim().is_empty());
            }
        });
        let weak_panel = Rc::downgrade(panel);
        let weak_root = root.downgrade();
        let weak_ui = Rc::downgrade(ui);
        cancel.connect_clicked(move |_| {
            if let (Some(panel), Some(root), Some(ui)) =
                (weak_panel.upgrade(), weak_root.upgrade(), weak_ui.upgrade())
            {
                if panel.dismiss_composer(&root) {
                    ui.document_box.grab_focus();
                }
            }
        });
        let weak_panel = Rc::downgrade(panel);
        let weak_root = root.downgrade();
        let weak_ui = Rc::downgrade(ui);
        let buffer = text.buffer();
        submit.connect_clicked(move |_| {
            let (Some(panel), Some(root), Some(ui)) =
                (weak_panel.upgrade(), weak_root.upgrade(), weak_ui.upgrade())
            else {
                return;
            };
            if root.parent().is_none() || ui.closed.get() {
                return;
            }
            if document_identity(&ui) != Some(identity) {
                error.set_text(wordcraft_gnome::t!("The document has changed; try again."));
                error.set_visible(true);
                return;
            }
            let body = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
            if body.trim().is_empty() {
                return;
            }
            let result = {
                let mut current = ui.current.borrow_mut();
                let Some(document) = current.as_mut() else {
                    return;
                };
                let session = &mut document.session;
                match parent {
                    Some(id) => model::reply(session, id, &body),
                    None => {
                        let previous = session.sel.clone();
                        let previous_page = session.page_hint;
                        let result =
                            selection::select_range(session, &selection.anchor, &selection.focus)
                                .and_then(|()| {
                                    session.page_hint = page_hint;
                                    model::add_comment(session, &body)
                                });
                        if result.is_err() {
                            let _ =
                                selection::select_range(session, &previous.anchor, &previous.focus);
                            session.page_hint = previous_page;
                        }
                        result
                    }
                }
            };
            match result {
                Ok(()) => {
                    panel.dismiss_composer(&root);
                    after_content_change(&ui);
                    panel.refresh();
                    ui.document_box.grab_focus();
                }
                Err(message) => {
                    error.set_text(&message);
                    error.set_visible(true);
                }
            }
        });
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let submit = submit.downgrade();
        let cancel = cancel.downgrade();
        keys.connect_key_pressed(move |_, key, _, mods| {
            if key == gdk::Key::Escape {
                if let Some(cancel) = cancel.upgrade() {
                    cancel.emit_clicked();
                }
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::Return && mods.contains(gdk::ModifierType::CONTROL_MASK) {
                if let Some(submit) = submit.upgrade().filter(|button| button.is_sensitive()) {
                    submit.emit_clicked();
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        root.add_controller(keys);
        Some(Self { parent, root, text })
    }

    pub(super) fn detach(&self) {
        if let Some(container) = self.root.parent().and_downcast::<gtk::Box>() {
            container.remove(&self.root);
        }
    }
}
