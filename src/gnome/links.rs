use super::*;
use wordcraft_gnome::links::{self, LinkState};

pub(super) struct LinkDialog {
    pub(super) window: adw::Window,
    pub(super) destination: gtk::Entry,
    pub(super) apply: gtk::Button,
}
pub(super) fn show_link(ui: &Rc<Ui>) -> Option<LinkDialog> {
    show_link_for(ui, None)
}
pub(super) fn show_link_at(ui: &Rc<Ui>, pos: &Pos) -> Option<LinkDialog> {
    show_link_for(ui, Some(pos))
}
fn show_link_for(ui: &Rc<Ui>, pointer: Option<&Pos>) -> Option<LinkDialog> {
    let host = ui.host.upgrade()?;
    if !selected_ui(&host).is_some_and(|active| Rc::ptr_eq(&active, ui)) {
        return None;
    }
    let identity = document_identity(ui);
    let (selection, target, text, state) = {
        let mut v = ui.current.borrow_mut();
        let d = v.as_mut()?;
        let original = d.session.sel.clone();
        let (a, b) = original.ordered();
        let range = links::range_at(&d.session, pointer.unwrap_or(&a)).filter(|(start, end, _)| {
            pointer.is_some() || original.is_collapsed() || (start <= &a && &b <= end)
        });
        let (target, text, state) = if let Some((a, b, url)) = range {
            let text = d
                .session
                .doc
                .para_at(&a)?
                .text
                .get(a.off..b.off)?
                .to_string();
            (
                wordcraft_engine::Selection {
                    anchor: a,
                    focus: b,
                },
                text,
                LinkState::Uniform(url),
            )
        } else {
            (
                original.clone(),
                selection::copy_text(&mut d.session).ok()?,
                links::state(&d.session),
            )
        };
        (original, target, text, state)
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
    cancel.set_widget_name("link-cancel");
    header.pack_start(&cancel);
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
    apply.set_widget_name("link-apply");
    apply.add_css_class("suggested-action");
    header.pack_end(&apply);
    toolbar.add_top_bar(&header);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
    body.set_margin_start(18);
    body.set_margin_end(18);
    body.set_margin_top(18);
    body.set_margin_bottom(18);
    let display_text = if target.is_collapsed() {
        let entry = gtk::Entry::new();
        entry.set_widget_name("link-display-text");
        entry.set_placeholder_text(Some(wordcraft_gnome::t!("Link Text")));
        accessible_name(&entry, wordcraft_gnome::t!("Link Text"));
        body.append(&entry);
        Some(entry)
    } else {
        let label = gtk::Label::new(Some(&text));
        label.set_wrap(true);
        label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        label.set_max_width_chars(40);
        label.set_xalign(0.0);
        body.append(&label);
        None
    };
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
    let text = display_text.as_ref().map(|e| e.downgrade());
    destination.connect_changed(move |e| {
        if let Some(b) = weak.upgrade() {
            b.set_sensitive(
                !e.text().trim().is_empty()
                    && e.text().len() <= 4096
                    && text
                        .as_ref()
                        .is_none_or(|e| e.upgrade().is_some_and(|e| !e.text().trim().is_empty())),
            )
        }
    });
    if let Some(text) = &display_text {
        let weak = apply.downgrade();
        let dest = destination.downgrade();
        text.connect_changed(move |text| {
            if let (Some(button), Some(dest)) = (weak.upgrade(), dest.upgrade()) {
                button.set_sensitive(
                    !text.text().trim().is_empty()
                        && !dest.text().trim().is_empty()
                        && dest.text().len() <= 4096,
                );
            }
        });
    }
    apply.set_sensitive(!destination.text().is_empty() && display_text.is_none());
    let weak = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    let weak = Rc::downgrade(ui);
    let weak_window = window.downgrade();
    let entry = destination.clone();
    let inserted_text = display_text.clone();
    apply.connect_clicked(move |_| {
        let (Some(ui), Some(window)) = (weak.upgrade(), weak_window.upgrade()) else {
            return;
        };
        if ui.closed.get()
            || !ui
                .host
                .upgrade()
                .and_then(|h| selected_ui(&h))
                .is_some_and(|active| Rc::ptr_eq(&active, &ui))
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
            links::apply_to(
                &mut v.as_mut().unwrap().session,
                &target,
                &entry.text(),
                &inserted_text
                    .as_ref()
                    .map(|e| e.text().to_string())
                    .unwrap_or_default(),
            )
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

pub(super) fn launch_document_link(ui: &Rc<Ui>, url: &str) {
    let Some(host) = ui.host.upgrade() else {
        return;
    };
    if std::env::args().any(|a| a == "--interaction-pointer-test") {
        eprintln!("LINK_ACTIVATED {url}");
        return;
    }
    let launcher = gtk::UriLauncher::new(url);
    let weak = Rc::downgrade(ui);
    launcher.launch(Some(&host.window), gio::Cancellable::NONE, move |result| {
        if let (Some(ui), Err(e)) = (weak.upgrade(), result) {
            if !ui.closed.get() {
                ui.toasts.add_toast(Toast::new(&e.to_string()));
            }
        }
    });
}

/// Query the current document at tooltip time, avoiding stale hover targets.
pub(super) fn install_link_tooltip(ui: &Rc<Ui>) {
    let weak = Rc::downgrade(ui);
    ui.document_box
        .connect_query_tooltip(move |_, x, y, keyboard, tooltip| {
            let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                return false;
            };
            let url = if keyboard {
                ui.current
                    .borrow()
                    .as_ref()
                    .and_then(|d| links::at(&d.session, &d.session.sel.focus))
            } else {
                document_link_at_point(&ui, x as f64, y as f64)
            };
            let Some(url) = url else {
                return false;
            };
            if std::env::args().any(|a| a == "--followup19-link") {
                eprintln!("ISSUE19_TOOLTIP {url}");
            }
            let label = gtk::Label::new(Some(&format!(
                "{}\n{}",
                url,
                wordcraft_gnome::t!("Ctrl+click to open a link")
            )));
            label.set_wrap(true);
            label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
            label.set_max_width_chars(72);
            tooltip.set_custom(Some(&label));
            true
        });
}

/// Verify real link ink bounds, rather than treating the nearest caret in a
/// blank page area as a link hit. Works across wrapped lines and zoom levels.
pub(super) fn document_link_at_point(ui: &Rc<Ui>, x: f64, y: f64) -> Option<String> {
    let (hover_page, pos) = inspect_canvas_point(ui, x, y)?;
    let mut v = ui.current.try_borrow_mut().ok()?;
    let d = v.as_mut()?;
    let (a, b, url) = links::range_at(&d.session, &pos)?;
    let layout = d.session.layout();
    for (page, rect) in layout.selection_rects(&d.session.doc, &a, &b, hover_page) {
        let frame = d.page_frames.get(page)?.compute_bounds(&ui.document_box)?;
        let left = frame.x() as f64 + rect.x as f64 * d.page_zoom(page) as f64;
        let top = frame.y() as f64 + rect.y as f64 * d.page_zoom(page) as f64;
        if x >= left
            && x <= left + rect.w as f64 * d.page_zoom(page) as f64
            && y >= top
            && y <= top + rect.h as f64 * d.page_zoom(page) as f64
        {
            return Some(url);
        }
    }
    None
}
