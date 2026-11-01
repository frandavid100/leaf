use super::*;
use wordcraft_gnome::annotations as model;

#[derive(Clone, Copy)]
pub(super) enum AnnotationEdit {
    Bookmark,
    NewComment,
    Edit(u32),
    Reply(u32),
}

pub(super) struct AnnotationDialog {
    pub(super) window: adw::Window,
    pub(super) name: gtk::Entry,
    pub(super) text: gtk::TextView,
    pub(super) submit: gtk::Button,
    pub(super) error: gtk::Label,
}

pub(super) fn show_annotation_dialog(
    ui: &Rc<Ui>,
    edit: AnnotationEdit,
) -> Option<AnnotationDialog> {
    let host = ui.host.upgrade()?;
    let identity = document_identity(ui)?;
    let (selection, existing) = {
        let value = ui.current.borrow();
        let d = value.as_ref()?;
        let allowed = match edit {
            AnnotationEdit::Bookmark => model::can_bookmark(&d.session),
            _ => model::can_comment(&d.session),
        };
        if !allowed {
            ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                "This annotation cannot be added or changed here."
            )));
            return None;
        }
        let text = match edit {
            AnnotationEdit::Edit(id) => {
                model::comments(&d.session)
                    .into_iter()
                    .find(|c| c.id == id)?
                    .text
            }
            _ => String::new(),
        };
        (d.session.sel.clone(), text)
    };
    let prior = ui.annotation_dialog.borrow_mut().take();
    if let Some(prior) = prior {
        prior.close();
    }
    let title = match edit {
        AnnotationEdit::Bookmark => wordcraft_gnome::t!("Add Bookmark"),
        AnnotationEdit::NewComment => wordcraft_gnome::t!("Add Comment"),
        AnnotationEdit::Edit(_) => wordcraft_gnome::t!("Edit Comment"),
        AnnotationEdit::Reply(_) => wordcraft_gnome::t!("Reply to Comment"),
    };
    let window = adw::Window::builder()
        .title(title)
        .transient_for(&host.window)
        .modal(true)
        .default_width(440)
        .default_height(if matches!(edit, AnnotationEdit::Bookmark) {
            240
        } else {
            360
        })
        .build();
    window.set_widget_name("annotation-dialog");
    let toolbar = ToolbarView::new();
    let header = HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    let submit = gtk::Button::with_label(if matches!(edit, AnnotationEdit::Edit(_)) {
        wordcraft_gnome::t!("Save")
    } else {
        wordcraft_gnome::t!("Add")
    });
    submit.add_css_class("suggested-action");
    submit.set_widget_name("annotation-submit");
    header.pack_start(&cancel);
    header.pack_end(&submit);
    toolbar.add_top_bar(&header);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
    body.set_margin_start(18);
    body.set_margin_end(18);
    body.set_margin_top(18);
    body.set_margin_bottom(18);
    let name = gtk::Entry::new();
    name.set_widget_name("bookmark-name");
    name.set_max_length(40);
    name.set_activates_default(true);
    accessible_name(&name, wordcraft_gnome::t!("Bookmark Name"));
    let text = gtk::TextView::new();
    text.set_widget_name("comment-text");
    text.set_wrap_mode(gtk::WrapMode::WordChar);
    text.set_top_margin(10);
    text.set_bottom_margin(10);
    text.set_left_margin(10);
    text.set_right_margin(10);
    accessible_name(&text, wordcraft_gnome::t!("Comment Text"));
    text.buffer().set_text(&existing);
    if matches!(edit, AnnotationEdit::Bookmark) {
        body.append(&gtk::Label::new(Some(wordcraft_gnome::t!("Bookmark Name"))));
        body.append(&name);
        let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
            "Start with a letter. Use letters, numbers or underscores; no spaces."
        )));
        hint.set_wrap(true);
        hint.add_css_class("dim-label");
        body.append(&hint);
        let weak = submit.downgrade();
        name.connect_changed(move |entry| {
            if let Some(button) = weak.upgrade() {
                button.set_sensitive(!entry.text().trim().is_empty());
            }
        });
    } else {
        let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
            "Save comments as DOCX. ODT does not preserve them."
        )));
        hint.set_wrap(true);
        hint.add_css_class("dim-label");
        body.append(&hint);
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scroll.set_vexpand(true);
        scroll.add_css_class("view");
        scroll.add_css_class("frame");
        scroll.set_child(Some(&text));
        body.append(&scroll);
        let weak = submit.downgrade();
        text.buffer().connect_changed(move |buffer| {
            if let Some(button) = weak.upgrade() {
                button.set_sensitive(
                    !buffer
                        .text(&buffer.start_iter(), &buffer.end_iter(), true)
                        .trim()
                        .is_empty(),
                );
            }
        });
    }
    submit.set_sensitive(!existing.trim().is_empty());
    let error = gtk::Label::new(None);
    error.set_widget_name("annotation-error");
    error.set_wrap(true);
    error.add_css_class("error");
    body.append(&error);
    toolbar.set_content(Some(&body));
    window.set_content(Some(&toolbar));
    window.set_default_widget(Some(&submit));
    let weak_window = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = weak_window.upgrade() {
            w.close();
        }
    });
    let weak_ui = Rc::downgrade(ui);
    let weak_window = window.downgrade();
    let field = name.clone();
    let buffer = text.buffer();
    let error_label = error.clone();
    submit.connect_clicked(move |_| {
        let (Some(ui), Some(window)) = (weak_ui.upgrade(), weak_window.upgrade()) else {
            return;
        };
        if ui.closed.get() || document_identity(&ui) != Some(identity) {
            error_label.set_text(wordcraft_gnome::t!(
                "The document has changed; open this dialog again."
            ));
            return;
        }
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        let result = {
            let mut value = ui.current.borrow_mut();
            let Some(d) = value.as_mut() else { return };
            // A modal editor cannot silently attach to a newly navigated selection.
            if d.session.sel != selection
                && matches!(edit, AnnotationEdit::Bookmark | AnnotationEdit::NewComment)
            {
                error_label.set_text(wordcraft_gnome::t!(
                    "The selection has changed; open this dialog again."
                ));
                return;
            }
            let rev = d.session.rev();
            let result = match edit {
                AnnotationEdit::Bookmark => model::add_bookmark(&mut d.session, &field.text()),
                AnnotationEdit::NewComment => model::add_comment(&mut d.session, &text),
                AnnotationEdit::Edit(id) => model::edit_comment(&mut d.session, id, &text),
                AnnotationEdit::Reply(id) => model::reply(&mut d.session, id, &text),
            };
            result.map(|_| d.session.rev() != rev)
        };
        match result {
            Err(e) => error_label.set_text(&e),
            Ok(changed) => {
                window.close();
                if changed {
                    after_content_change(&ui);
                }
                if let Some(panel) = ui.annotations.borrow().as_ref() {
                    panel.show();
                }
                ui.document_box.grab_focus();
            }
        }
    });
    let weak = Rc::downgrade(ui);
    window.connect_close_request(move |_| {
        if let Some(ui) = weak.upgrade() {
            ui.annotation_dialog.borrow_mut().take();
        }
        glib::Propagation::Proceed
    });
    // Escape cancels; Ctrl+Enter submits multiline comments without inserting a line.
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let w = window.downgrade();
    let button = submit.downgrade();
    keys.connect_key_pressed(move |_, key, _, mods| {
        if key == gdk::Key::Escape {
            if let Some(w) = w.upgrade() {
                w.close();
            }
            return glib::Propagation::Stop;
        }
        if key == gdk::Key::Return && mods.contains(gdk::ModifierType::CONTROL_MASK) {
            if let Some(button) = button.upgrade().filter(|b| b.is_sensitive()) {
                button.emit_clicked();
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(keys);
    ui.annotation_dialog.replace(Some(window.clone()));
    window.present();
    if matches!(edit, AnnotationEdit::Bookmark) {
        name.grab_focus();
    } else {
        text.grab_focus();
    }
    Some(AnnotationDialog {
        window,
        name,
        text,
        submit,
        error,
    })
}
