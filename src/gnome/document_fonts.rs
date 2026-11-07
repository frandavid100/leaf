//! Font inventory is read-only until an explicit per-font Replace action.
use super::*;
pub(super) fn show(ui: &Rc<Ui>, parent: &impl IsA<gtk::Window>) {
    let parent = parent.as_ref();
    let (fonts, missing, families, identity) = {
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
        (
            fonts,
            missing,
            families,
            Some((d.session.document_id(), d.session.rev())),
        )
    };
    let window = adw::Window::builder()
        .title(wordcraft_gnome::t!("Document Fonts"))
        .destroy_with_parent(true)
        .transient_for(parent)
        .default_width(520)
        .default_height(560)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
    content.append(&adw::HeaderBar::new());
    let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
        "Opening a document preserves its font names. Replace changes the document and can be undone."
    )));
    hint.set_wrap(true);
    content.append(&hint);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_child(Some(&list));
    content.append(&scroll);
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
        box_.set_margin_start(12);
        box_.set_margin_end(12);
        let title = gtk::Label::new(Some(&font.name));
        title.set_xalign(0.);
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
            let label = gtk::Label::new(Some(&format!(
                "{}: {}",
                wordcraft_gnome::t!("Missing; displayed as"),
                shown
            )));
            label.set_xalign(0.);
            label.set_wrap(true);
            box_.append(&label);
        }
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let next = gtk::Button::with_label(wordcraft_gnome::t!("Next Use"));
        next.set_sensitive(!font.uses.is_empty());
        actions.append(&next);
        let weak = Rc::downgrade(ui);
        let uses = font.uses;
        let index = Cell::new(0usize);
        next.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                if document_identity(&ui) != identity {
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
        replacement.set_enable_search(true);
        replacement.set_hexpand(true);
        accessible_name(&replacement, wordcraft_gnome::t!("Replacement Font"));
        actions.append(&replacement);
        let replace = gtk::Button::with_label(wordcraft_gnome::t!("Replace"));
        replace.set_sensitive(!choices.is_empty());
        actions.append(&replace);
        let choices = choices.clone();
        let weak = Rc::downgrade(ui);
        let parent = parent.downgrade();
        let dialog = window.downgrade();
        replace.connect_clicked(move |_| {
            let (Some(ui), Some(parent), Some(dialog)) =
                (weak.upgrade(), parent.upgrade(), dialog.upgrade())
            else {
                return;
            };
            if document_identity(&ui) != identity {
                ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                    "The document changed. Reopen the font list."
                )));
                return;
            }
            if let Some(to) = choices.get(replacement.selected() as usize) {
                if tools_change(&ui, |s| {
                    wordcraft_gnome::document_fonts::substitute(s, &font.name, to)
                }) {
                    dialog.close();
                    show(&ui, &parent);
                }
            }
        });
        box_.append(&actions);
        list.append(&box_);
        list.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    }
    window.set_content(Some(&content));
    window.present();
}
