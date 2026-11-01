use super::*;

// Header/footer contents are engine stories edited on the same document canvas.
use wordcraft_gnome::header_footer::{self, NumberPosition, Region};

pub(super) fn header_change(ui: &Rc<Ui>, f: impl FnOnce(&mut Session) -> Result<bool, String>) {
    let result = {
        let mut value = ui.current.borrow_mut();
        let Some(d) = value.as_mut() else {
            return;
        };
        f(&mut d.session)
    };
    match result {
        Ok(true) => after_content_change(ui),
        Ok(false) => refresh_selection(ui),
        Err(e) => ui.toasts.add_toast(Toast::new(&e)),
    }
    ui.document_box.grab_focus();
    caret_ensure_visible(ui);
}
pub(super) fn enter_header(ui: &Rc<Ui>, region: Region) {
    let page = ui.current.borrow().as_ref().map(|d| d.active).unwrap_or(0);
    header_change(ui, |s| header_footer::enter(s, page, region));
}

/// A canvas click leaves the active page story when it is outside that region.
/// The result distinguishes blank canvas from a page, so a gap cannot re-enter
/// a header through the ordinary clamped page hit test.
pub(super) fn leave_header_outside_at(ui: &Rc<Ui>, x: f64, y: f64) -> Option<bool> {
    let inside_page = {
        let mut value = ui.current.borrow_mut();
        let d = value.as_mut()?;
        let active = header_footer::active(&d.session)?;
        let mut in_page = false;
        for (i, frame) in d.page_frames.iter().enumerate() {
            let Some(r) = frame.compute_bounds(&ui.document_box) else {
                continue;
            };
            if x < r.x() as f64
                || x > (r.x() + r.width()) as f64
                || y < r.y() as f64
                || y > (r.y() + r.height()) as f64
            {
                continue;
            }
            in_page = true;
            let py = ((y - r.y() as f64) / d.zoom as f64) as f32;
            if let Some((_, header)) = d.session.layout().header_footer_zone(i, py) {
                if (header && active == Region::Header) || (!header && active == Region::Footer) {
                    return None;
                }
            }
            break;
        }
        if let Err(e) = header_footer::close(&mut d.session) {
            ui.toasts.add_toast(Toast::new(&e));
            return None;
        }
        in_page
    };
    Some(inside_page)
}
// Double-clicking a previously empty page margin creates its editable story.
pub(super) fn enter_empty_header_at(ui: &Rc<Ui>, x: f64, y: f64) -> bool {
    let target = {
        let mut value = ui.current.borrow_mut();
        let Some(d) = value.as_mut() else {
            return false;
        };
        d.page_frames.iter().enumerate().find_map(|(i, frame)| {
            let r = frame.compute_bounds(&ui.document_box)?;
            if x < r.x() as f64
                || x > (r.x() + r.width()) as f64
                || y < r.y() as f64
                || y > (r.y() + r.height()) as f64
            {
                return None;
            }
            let py = ((y - r.y() as f64) / d.zoom as f64) as f32;
            let (story, header) = d.session.layout().header_footer_zone(i, py)?;
            if story.is_some() {
                return None;
            }
            Some((
                i,
                if header {
                    Region::Header
                } else {
                    Region::Footer
                },
            ))
        })
    };
    if let Some((page, region)) = target {
        header_change(ui, |s| header_footer::enter(s, page, region));
        true
    } else {
        false
    }
}

pub(super) struct PageNumberDialog {
    pub(super) window: adw::Window,
    pub(super) total: gtk::Switch,
    pub(super) insert: gtk::Button,
}
pub(super) fn show_page_number(ui: &Rc<Ui>, parent: &ApplicationWindow) -> PageNumberDialog {
    let identity = document_identity(ui);
    let (page, selection) = {
        let value = ui.current.borrow();
        let d = value.as_ref().unwrap();
        (d.active, d.session.sel.clone())
    };
    let window = adw::Window::builder()
        .title(wordcraft_gnome::t!("Page Number"))
        .transient_for(parent)
        .modal(true)
        .default_width(430)
        .default_height(350)
        .build();
    let toolbar = ToolbarView::new();
    let header = HeaderBar::new();
    header.set_show_start_title_buttons(false);
    header.set_show_end_title_buttons(false);
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    header.pack_start(&cancel);
    let insert = gtk::Button::with_label(wordcraft_gnome::t!("Insert"));
    insert.add_css_class("suggested-action");
    header.pack_end(&insert);
    toolbar.add_top_bar(&header);
    let body = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();
    group.set_description(Some(wordcraft_gnome::t!(
        "Page numbers update automatically. Existing header and footer content is kept."
    )));
    body.add(&group);
    let row = adw::ActionRow::new();
    row.set_title(wordcraft_gnome::t!("Include Total Pages"));
    let total = gtk::Switch::new();
    total.set_valign(gtk::Align::Center);
    row.add_suffix(&total);
    row.set_activatable_widget(Some(&total));
    group.add(&row);
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    error.add_css_class("error");
    body.add(&{
        let g = adw::PreferencesGroup::new();
        g.add(&error);
        g
    });
    toolbar.set_content(Some(&body));
    window.set_content(Some(&toolbar));
    {
        let w = window.downgrade();
        cancel.connect_clicked(move |_| {
            if let Some(w) = w.upgrade() {
                w.close();
            }
        });
    }
    {
        let weak = Rc::downgrade(ui);
        let w = window.downgrade();
        let total = total.clone();
        insert.connect_clicked(move |_| {
            let (Some(ui), Some(w)) = (weak.upgrade(), w.upgrade()) else {
                return;
            };
            if ui.closed.get() || document_identity(&ui) != identity {
                error.set_text(wordcraft_gnome::t!(
                    "The document has changed; open this dialog again."
                ));
                return;
            }
            let result = {
                let mut value = ui.current.borrow_mut();
                let Some(d) = value.as_mut() else {
                    return;
                };
                if d.session.sel != selection {
                    error.set_text(wordcraft_gnome::t!(
                        "The selection has changed; open this dialog again."
                    ));
                    return;
                }
                header_footer::insert_number(
                    &mut d.session,
                    page,
                    NumberPosition::Current,
                    "left",
                    total.is_active(),
                )
            };
            match result {
                Ok(changed) => {
                    w.close();
                    if changed {
                        after_content_change(&ui);
                    } else {
                        refresh_selection(&ui);
                    }
                    ui.document_box.grab_focus();
                    caret_ensure_visible(&ui);
                }
                Err(e) => error.set_text(&e),
            }
        });
    }
    window.present();
    PageNumberDialog {
        window,
        total,
        insert,
    }
}
