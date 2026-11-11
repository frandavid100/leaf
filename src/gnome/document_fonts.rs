//! Font inventory is read-only until an explicit per-font Replace action.
use super::*;
pub(super) fn refresh_open(ui: &Rc<Ui>) {
    let Some(host) = ui.host.upgrade() else {
        return;
    };
    let Some(group) = find_widget(&host.window, "properties-document-fonts")
        .and_downcast::<adw::PreferencesGroup>()
    else {
        return;
    };
    if let Some(list) = find_widget(&group, "document-font-list") {
        group.remove(&list);
    }
    refresh(ui, &group, Rc::new(Cell::new(document_identity(ui))));
}
pub(super) fn section(
    ui: &Rc<Ui>,
    identity: Rc<Cell<Option<(u64, u64)>>>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::new();
    group.set_widget_name("properties-document-fonts");
    group.set_title(wordcraft_gnome::t!("Fonts"));
    group.set_description(Some(wordcraft_gnome::t!("Opening a document preserves its font names. Replace changes the document and can be undone.")));
    let install = gtk::Button::with_label(wordcraft_gnome::t!("Install Missing Fonts…"));
    install.set_widget_name("properties-install-fonts");
    let weak = Rc::downgrade(ui);
    install.connect_clicked(move |_| {
        if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
            font_install::show(&ui);
        }
    });
    group.add(&install);
    refresh(ui, &group, identity);
    group
}
fn refresh(ui: &Rc<Ui>, group: &adw::PreferencesGroup, identity: Rc<Cell<Option<(u64, u64)>>>) {
    let (fonts, missing, families) = {
        let mut value = ui.current.borrow_mut();
        let Some(d) = value.as_mut() else { return };
        let fonts = wordcraft_gnome::document_fonts::inventory(&d.session);
        let mut missing = d
            .session
            .run("font.missing", &serde_json::json!({}))
            .unwrap_or_default();
        for font in &fonts {
            let known = missing["missing"].as_array().is_some_and(|values| {
                values
                    .iter()
                    .any(|v| v["font"].as_str() == Some(&font.name))
            });
            if !known {
                if let Some(fallback) =
                    wordcraft_engine::cmd::font::shown_as(&d.session.doc, &font.name)
                {
                    if let Some(values) = missing["missing"].as_array_mut() {
                        values.push(serde_json::json!({"font":font.name,"shownAs":[fallback]}));
                    }
                }
            }
        }
        let families = d
            .session
            .run("font.list", &serde_json::json!({"all":true}))
            .unwrap_or_default();
        (fonts, missing, families)
    };
    // The surrounding PreferencesPage owns scrolling; no secondary window.
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    list.set_widget_name("document-font-list");
    group.add(&list);
    let choices = families["families"]
        .as_array()
        .map(|v| {
            v.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for font in fonts {
        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 4);
        box_.set_widget_name(&format!("document-font-{}", font.name));
        box_.set_margin_start(12);
        box_.set_margin_end(12);
        let title = gtk::Label::new(Some(&font.name));
        title.set_xalign(0.);
        title.set_wrap(true);
        title.add_css_class("heading");
        box_.append(&title);
        if let Some(missing) = missing["missing"].as_array().and_then(|values| {
            values
                .iter()
                .find(|v| v["font"].as_str() == Some(&font.name))
        }) {
            let shown = missing["shownAs"]
                .as_array()
                .map(|v| {
                    v.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let status = if shown.is_empty() {
                wordcraft_gnome::t!("Missing; fallback information unavailable").to_owned()
            } else {
                format!(
                    "{}: {}",
                    wordcraft_gnome::t!("Missing; displayed as"),
                    shown
                )
            };
            let label = gtk::Label::new(Some(&status));
            label.set_xalign(0.);
            label.set_wrap(true);
            box_.append(&label);
        } else if choices
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&font.name))
        {
            let label = gtk::Label::new(Some(wordcraft_gnome::t!("Installed")));
            label.set_xalign(0.);
            label.add_css_class("dim-label");
            box_.append(&label);
        }
        let actions = gtk::Box::new(gtk::Orientation::Vertical, 6);
        let substitution = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let next = gtk::Button::with_label(wordcraft_gnome::t!("Next Use"));
        next.set_widget_name("font-next-use");
        next.set_sensitive(!font.uses.is_empty());
        actions.append(&next);
        let weak = Rc::downgrade(ui);
        let use_identity = identity.clone();
        let uses = font.uses;
        let index = Cell::new(0usize);
        next.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                if document_identity(&ui) != use_identity.get() {
                    return;
                }
                if let Some((a, b)) = uses.get(index.get() % uses.len().max(1)) {
                    if let Some(d) = ui.current.borrow_mut().as_mut() {
                        let _ = selection::select_range(&mut d.session, a, b);
                    }
                    index.set(index.get() + 1);
                    refresh_selection(&ui);
                    caret_ensure_visible(&ui);
                }
            }
        });
        let replacement =
            gtk::DropDown::from_strings(&choices.iter().map(String::as_str).collect::<Vec<_>>());
        replacement.set_widget_name("font-replacement");
        replacement.set_enable_search(true);
        replacement.set_hexpand(true);
        if let Some(label) = replacement
            .first_child()
            .and_then(|w| w.first_child())
            .and_then(|w| w.downcast::<gtk::Label>().ok())
        {
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(18);
        }
        accessible_name(&replacement, wordcraft_gnome::t!("Replacement Font"));
        substitution.append(&replacement);
        let replace = gtk::Button::with_label(wordcraft_gnome::t!("Replace"));
        replace.set_widget_name("font-replace");
        replace.set_sensitive(!choices.is_empty());
        substitution.append(&replace);
        actions.append(&substitution);
        let choices = choices.clone();
        let weak = Rc::downgrade(ui);
        let group = group.downgrade();
        let list_ref = list.downgrade();
        let identity = identity.clone();
        replace.connect_clicked(move |_| {
            let (Some(ui), Some(group), Some(list)) =
                (weak.upgrade(), group.upgrade(), list_ref.upgrade())
            else {
                return;
            };
            if document_identity(&ui) != identity.get() {
                ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                    "The document changed. Reopen the font list."
                )));
                return;
            }
            if let Some(to) = choices.get(replacement.selected() as usize) {
                if tools_change(&ui, |s| {
                    wordcraft_gnome::document_fonts::substitute(s, &font.name, to)
                }) {
                    identity.set(document_identity(&ui));
                    group.remove(&list);
                    refresh(&ui, &group, identity.clone());
                }
            }
        });
        box_.append(&actions);
        list.append(&box_);
        list.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    }
}
