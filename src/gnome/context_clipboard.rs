//! Joined primary actions keep Copy/Paste direct while variants stay grouped.
use super::*;

pub(super) fn append(
    menu: &gio::Menu,
    actions: &gio::SimpleActionGroup,
    host: &gtk::PopoverMenu,
    ui: &Rc<Ui>,
    copy: bool,
    selected: bool,
) {
    let (name, label, shortcut, help) = if copy {
        (
            "copy",
            wordcraft_gnome::t!("Copy"),
            "Ctrl+C",
            wordcraft_gnome::t!("Copy Options"),
        )
    } else {
        (
            "paste",
            wordcraft_gnome::t!("Paste"),
            "Ctrl+V",
            wordcraft_gnome::t!("Paste Options"),
        )
    };
    let split = adw::SplitButton::new();
    split.set_widget_name(&format!("context-{name}-split"));
    split.add_css_class("flat");
    split.set_hexpand(true);
    split.set_dropdown_tooltip(help);
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    let title = gtk::Label::new(Some(label));
    title.set_xalign(0.);
    title.set_hexpand(true);
    content.append(&title);
    let accel = gtk::Label::new(Some(shortcut));
    accel.add_css_class("dim-label");
    content.append(&accel);
    split.set_child(Some(&content));
    accessible_name(&split, label);
    if let Some(primary) = split.first_child() {
        primary.set_widget_name(&format!("context-{name}-primary"));
        primary.set_sensitive(if copy {
            selected
        } else {
            clipboard_can_paste(ui)
        });
        accessible_name(&primary, label);
    }
    let weak_ui = Rc::downgrade(ui);
    let weak_host = host.downgrade();
    split.connect_clicked(move |_| {
        if let Some(host) = weak_host.upgrade() {
            host.popdown();
        }
        if let Some(ui) = weak_ui.upgrade().filter(|u| !u.closed.get()) {
            if copy {
                context_copy(&ui);
            } else {
                context_paste(&ui);
            }
        }
    });
    let model = gio::Menu::new();
    let variants = gtk::PopoverMenu::from_model(Some(&model));
    let weak_variants = variants.downgrade();
    host.connect_closed(move |_| {
        if let Some(variants) = weak_variants.upgrade() {
            variants.popdown();
        }
    });
    let back_actions = gio::SimpleActionGroup::new();
    let back = gio::SimpleAction::new("back", None);
    let weak = variants.downgrade();
    back.connect_activate(move |_, _| {
        if let Some(pop) = weak.upgrade() {
            pop.popdown();
        }
    });
    back_actions.add_action(&back);
    variants.insert_action_group("variants", Some(&back_actions));
    model.append(Some(wordcraft_gnome::t!("Back")), Some("variants.back"));
    let choices = gio::Menu::new();
    if copy {
        context_item(
            &choices,
            actions,
            host,
            ui,
            wordcraft_gnome::t!("Copy Formatting"),
            "copy-format",
            true,
            |u| format_painter::start(u, false),
        );
        context_item(
            &choices,
            actions,
            host,
            ui,
            wordcraft_gnome::t!("Keep Copying Formatting"),
            "copy-format-sticky",
            true,
            |u| format_painter::start(u, true),
        );
    } else {
        let formats = ui.document_box.clipboard().formats();
        let plain = formats.contains_type(String::static_type())
            || formats.contain_mime_type("text/plain")
            || formats.contain_mime_type("text/plain;charset=utf-8");
        context_item(
            &choices,
            actions,
            host,
            ui,
            wordcraft_gnome::t!("Paste Plain Text"),
            "paste-plain",
            plain,
            clipboard::paste_plain,
        );
    }
    model.append_section(None, &choices);
    split.set_popover(Some(&variants));
    let item = gio::MenuItem::new(None, None);
    let id = format!("context-{name}-row");
    item.set_attribute_value("custom", Some(&id.to_variant()));
    menu.append_item(&item);
    assert!(host.add_child(&split, &id));
}
