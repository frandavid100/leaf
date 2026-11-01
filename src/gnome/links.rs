use super::*;
use wordcraft_gnome::links::{self, LinkState};

pub(super) struct LinkDialog {
    pub(super) window: adw::Window,
    pub(super) destination: gtk::Entry,
    pub(super) apply: gtk::Button,
}
pub(super) fn show_link(ui: &Rc<Ui>) -> Option<LinkDialog> {
    let host = ui.host.upgrade()?;
    let identity = document_identity(ui);
    let (selection, text, state) = {
        let mut v = ui.current.borrow_mut();
        let d = v.as_mut()?;
        if d.session.sel.is_collapsed() {
            return None;
        }
        (
            d.session.sel.clone(),
            selection::copy_text(&mut d.session).ok()?,
            links::state(&d.session),
        )
    };
    let old = ui.link_dialog.borrow_mut().take();
    if let Some(old) = old {
        old.close();
    }
    let window = adw::Window::builder()
        .title(wordcraft_gnome::t!("Link"))
        .transient_for(&host.window)
        .modal(true)
        .default_width(460)
        .default_height(260)
        .build();
    let toolbar = ToolbarView::new();
    let header = HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    header.pack_start(&cancel);
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
    apply.add_css_class("suggested-action");
    header.pack_end(&apply);
    toolbar.add_top_bar(&header);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
    body.set_margin_start(18);
    body.set_margin_end(18);
    body.set_margin_top(18);
    body.set_margin_bottom(18);
    let label = gtk::Label::new(Some(&text));
    label.set_wrap(true);
    label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    label.set_max_width_chars(40);
    label.set_xalign(0.0);
    body.append(&label);
    if state == LinkState::Mixed {
        let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
            "This selection contains different links. Enter a destination to replace them all."
        )));
        hint.set_wrap(true);
        hint.set_max_width_chars(40);
        body.append(&hint);
    }
    let destination = gtk::Entry::new();
    destination.set_widget_name("link-destination");
    destination.set_placeholder_text(Some(wordcraft_gnome::t!("Destination")));
    accessible_name(&destination, wordcraft_gnome::t!("Destination"));
    if let LinkState::Uniform(url) = state {
        destination.set_text(&url);
    }
    body.append(&destination);
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    error.add_css_class("error");
    body.append(&error);
    toolbar.set_content(Some(&body));
    window.set_content(Some(&toolbar));
    let weak = apply.downgrade();
    destination.connect_changed(move |e| {
        if let Some(b) = weak.upgrade() {
            b.set_sensitive(!e.text().trim().is_empty() && e.text().len() <= 4096)
        }
    });
    apply.set_sensitive(!destination.text().is_empty());
    let weak = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    let weak = Rc::downgrade(ui);
    let weak_window = window.downgrade();
    let entry = destination.clone();
    apply.connect_clicked(move |_| {
        let (Some(ui), Some(window)) = (weak.upgrade(), weak_window.upgrade()) else {
            return;
        };
        if ui.closed.get()
            || document_identity(&ui) != identity
            || ui.current.borrow().as_ref().map(|d| d.session.sel.clone())
                != Some(selection.clone())
        {
            error.set_text(wordcraft_gnome::t!(
                "The document has changed; open this dialog again."
            ));
            return;
        }
        let result = {
            let mut v = ui.current.borrow_mut();
            links::apply(&mut v.as_mut().unwrap().session, &entry.text())
        };
        match result {
            Ok(changed) => {
                window.close();
                if changed {
                    after_content_change(&ui);
                }
                ui.document_box.grab_focus();
            }
            Err(e) => error.set_text(&e),
        }
    });
    let weak = Rc::downgrade(ui);
    window.connect_close_request(move |_| {
        if let Some(ui) = weak.upgrade() {
            ui.link_dialog.borrow_mut().take();
        }
        glib::Propagation::Proceed
    });
    ui.link_dialog.replace(Some(window.clone()));
    window.present();
    Some(LinkDialog {
        window,
        destination,
        apply,
    })
}
