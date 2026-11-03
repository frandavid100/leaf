//! Document metadata, page geometry and section options in one apply transaction.
use super::*;
use wordcraft_gnome::document_properties::DocumentInfo;

pub(super) struct DocumentProperties {
    pub(super) window: adw::Window,
    pub(super) metadata: [adw::EntryRow; 5],
    pub(super) width: gtk::SpinButton,
    pub(super) height: gtk::SpinButton,
    pub(super) top: gtk::SpinButton,
    pub(super) bottom: gtk::SpinButton,
    pub(super) left: gtk::SpinButton,
    pub(super) right: gtk::SpinButton,
    pub(super) apply: gtk::Button,
    pub(super) headers: Rc<HeaderControls>,
}
pub(super) fn properties_number(
    group: &adw::PreferencesGroup,
    label: &str,
    value: f32,
) -> gtk::SpinButton {
    let row = adw::ActionRow::new();
    row.set_title(label);
    let n = mm_input(value as f64 / PT_PER_MM, 0.0, 1584.0 / PT_PER_MM);
    row.add_suffix(&n);
    row.set_activatable_widget(Some(&n));
    accessible_name(&n, label);
    group.add(&row);
    n
}
pub(super) fn show_document_properties(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
) -> DocumentProperties {
    let (state, section, info) = {
        let mut value = ui.current.borrow_mut();
        let s = &mut value.as_mut().unwrap().session;
        let i = page_options::current_section(s);
        let p = s.doc.sections()[i].1;
        (
            PageSetup {
                width: p.page_w,
                height: p.page_h,
                top: p.margin_top,
                bottom: p.margin_bottom,
                left: p.margin_left,
                right: p.margin_right,
            },
            i,
            DocumentInfo::current(s),
        )
    };
    let identity = document_identity(ui);
    let window = adw::Window::builder()
        .title(wordcraft_gnome::t!("Document Properties"))
        .transient_for(parent)
        .modal(true)
        .default_width(480)
        .default_height(650)
        .build();
    let toolbar = ToolbarView::new();
    let header = HeaderBar::new();
    header.set_show_end_title_buttons(false);
    header.set_show_start_title_buttons(false);
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    header.pack_start(&cancel);
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
    apply.set_widget_name("document-properties-apply");
    apply.add_css_class("suggested-action");
    header.pack_end(&apply);
    toolbar.add_top_bar(&header);
    let page = adw::PreferencesPage::new();
    let paper = adw::PreferencesGroup::new();
    paper.set_title(wordcraft_gnome::t!("Page"));
    page.add(&paper);
    let sizes = adw::ComboRow::new();
    sizes.set_title(wordcraft_gnome::t!("Paper Size"));
    sizes.set_model(Some(&gtk::StringList::new(&[
        wordcraft_gnome::t!("Custom"),
        "A4",
        "A5",
        "A3",
        wordcraft_gnome::t!("Letter"),
        "Legal",
    ])));
    paper.add(&sizes);
    let orientation = adw::ComboRow::new();
    orientation.set_title(wordcraft_gnome::t!("Orientation"));
    orientation.set_model(Some(&gtk::StringList::new(&[
        wordcraft_gnome::t!("Portrait"),
        wordcraft_gnome::t!("Landscape"),
    ])));
    orientation.set_selected(if state.width > state.height { 1 } else { 0 });
    paper.add(&orientation);
    let width = properties_number(&paper, wordcraft_gnome::t!("Width (mm)"), state.width);
    let height = properties_number(&paper, wordcraft_gnome::t!("Height (mm)"), state.height);
    width.set_widget_name("paper-width");
    height.set_widget_name("paper-height");
    sizes.set_widget_name("paper-size");
    orientation.set_widget_name("paper-orientation");
    let presets = [
        (210.0, 297.0),
        (148.0, 210.0),
        (297.0, 420.0),
        (215.9, 279.4),
        (215.9, 355.6),
    ];
    for (i, (w, h)) in presets.iter().enumerate() {
        let (a, b) = if state.width > state.height {
            (*h, *w)
        } else {
            (*w, *h)
        };
        if (width.value() - a).abs() < 0.1 && (height.value() - b).abs() < 0.1 {
            sizes.set_selected(i as u32 + 1);
        }
    }
    width.set_sensitive(sizes.selected() == 0);
    height.set_sensitive(sizes.selected() == 0);
    let margins = adw::PreferencesGroup::new();
    margins.set_title(wordcraft_gnome::t!("Margins"));
    page.add(&margins);
    let top = properties_number(&margins, wordcraft_gnome::t!("Top (mm)"), state.top);
    let bottom = properties_number(&margins, wordcraft_gnome::t!("Bottom (mm)"), state.bottom);
    let left = properties_number(&margins, wordcraft_gnome::t!("Left (mm)"), state.left);
    let right = properties_number(&margins, wordcraft_gnome::t!("Right (mm)"), state.right);
    let scope = adw::ComboRow::new();
    scope.set_title(wordcraft_gnome::t!("Apply To"));
    scope.set_model(Some(&gtk::StringList::new(&[
        wordcraft_gnome::t!("Whole Document"),
        wordcraft_gnome::t!("Current Section"),
    ])));
    let multi = ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| d.session.doc.sections().len() > 1);
    scope.set_selected(if multi { 1 } else { 0 });
    scope.set_visible(multi);
    margins.add(&scope);
    let message = gtk::Label::new(None);
    message.set_wrap(true);
    message.add_css_class("error");
    message.set_margin_start(12);
    message.set_margin_end(12);
    message.set_margin_bottom(12);
    message.set_visible(false);
    let stack = adw::ViewStack::new();
    stack.set_widget_name("document-properties-pages");
    stack.set_vexpand(true);
    let information = adw::PreferencesPage::new();
    let properties = adw::PreferencesGroup::new();
    properties.set_description(Some(wordcraft_gnome::t!(
        "Save as DOCX to preserve the category."
    )));
    information.add(&properties);
    let metadata = [
        ("title", wordcraft_gnome::t!("Title"), info.title),
        ("subject", wordcraft_gnome::t!("Subject"), info.subject),
        ("author", wordcraft_gnome::t!("Author"), info.author),
        ("keywords", wordcraft_gnome::t!("Keywords"), info.keywords),
        ("category", wordcraft_gnome::t!("Category"), info.category),
    ]
    .map(|(id, title, text)| {
        let row = adw::EntryRow::new();
        row.set_widget_name(&format!("document-{id}"));
        row.set_title(title);
        row.set_text(&text);
        properties.add(&row);
        row
    });
    let review_group = adw::PreferencesGroup::new();
    review_group.set_title(wordcraft_gnome::t!("Language and Review"));
    let language = adw::EntryRow::new();
    language.set_title(wordcraft_gnome::t!("Language"));
    language.set_widget_name("document-language");
    let review = option_switch(&review_group, wordcraft_gnome::t!("Review"));
    review.set_widget_name("document-review");
    {
        let current = ui.current.borrow();
        let s = &current.as_ref().unwrap().session;
        language.set_text(&wordcraft_gnome::proofing_defaults::language(s));
        review.set_active(wordcraft_gnome::proofing_defaults::review(s));
    }
    review_group.add(&language);
    information.add(&review_group);
    stack.add_titled(
        &information,
        Some("information"),
        wordcraft_gnome::t!("Information"),
    );
    stack.add_titled(&page, Some("page"), wordcraft_gnome::t!("Page"));
    let header_page = adw::PreferencesPage::new();
    let headers = HeaderControls::new(ui, &header_page, section);
    stack.add_titled(
        &header_page,
        Some("headers"),
        wordcraft_gnome::t!("Headers and Footers"),
    );
    for (child, icon) in [
        (
            information.upcast_ref::<gtk::Widget>(),
            "dialog-information-symbolic",
        ),
        (
            page.upcast_ref::<gtk::Widget>(),
            "document-properties-symbolic",
        ),
        (
            header_page.upcast_ref::<gtk::Widget>(),
            "view-paged-symbolic",
        ),
    ] {
        stack.page(child).set_icon_name(Some(icon));
    }
    // GtkBuilder resolves the native widget on newer libadwaita runtimes.
    // Keep the existing switcher for supported hosts with libadwaita 1.4–1.6.
    let builder = gtk::Builder::new();
    let switcher = if adw::minor_version() >= 7
        && builder
            .add_from_string(
                r#"<interface>
                <object class="AdwInlineViewSwitcher" id="properties-switcher">
                    <property name="display-mode">labels</property>
                    <property name="homogeneous">true</property>
                </object>
            </interface>"#,
            )
            .is_ok()
    {
        let switcher = builder
            .object::<gtk::Widget>("properties-switcher")
            .unwrap();
        switcher.set_property("stack", &stack);
        switcher
    } else {
        let switcher = adw::ViewSwitcher::new();
        switcher.set_stack(Some(&stack));
        switcher.set_policy(adw::ViewSwitcherPolicy::Narrow);
        switcher.upcast()
    };
    switcher.set_widget_name("document-properties-switcher");
    switcher.set_halign(gtk::Align::Center);
    switcher.set_margin_top(6);
    switcher.set_margin_bottom(6);
    toolbar.add_top_bar(&switcher);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.append(&stack);
    content.append(&message);
    toolbar.set_content(Some(&content));
    window.set_content(Some(&toolbar));
    let syncing = Rc::new(Cell::new(false));
    {
        let width = width.downgrade();
        let height = height.downgrade();
        let orientation = orientation.downgrade();
        let syncing = syncing.clone();
        sizes.connect_selected_notify(move |r| {
            if let (Some(width), Some(height)) = (width.upgrade(), height.upgrade()) {
                width.set_sensitive(r.selected() == 0);
                height.set_sensitive(r.selected() == 0);
            }
            let Some(i) = r.selected().checked_sub(1) else {
                return;
            };
            let Some(&(a, b)) = presets.get(i as usize) else {
                return;
            };
            let (Some(width), Some(height), Some(orientation)) =
                (width.upgrade(), height.upgrade(), orientation.upgrade())
            else {
                return;
            };
            syncing.set(true);
            let (a, b) = if orientation.selected() == 1 {
                (b, a)
            } else {
                (a, b)
            };
            width.set_value(a);
            height.set_value(b);
            syncing.set(false);
        });
    }
    {
        let width = width.downgrade();
        let height = height.downgrade();
        let syncing = syncing.clone();
        orientation.connect_selected_notify(move |r| {
            let (Some(width), Some(height)) = (width.upgrade(), height.upgrade()) else {
                return;
            };
            let (w, h) = (width.value(), height.value());
            if (r.selected() == 1 && w < h) || (r.selected() == 0 && w > h) {
                syncing.set(true);
                width.set_value(h);
                height.set_value(w);
                syncing.set(false);
            }
        });
    }
    for n in [&width, &height] {
        let sizes = sizes.downgrade();
        let orientation = orientation.downgrade();
        let width = width.downgrade();
        let height = height.downgrade();
        let syncing = syncing.clone();
        n.connect_value_changed(move |_| {
            if !syncing.get() {
                let (Some(sizes), Some(orientation), Some(width), Some(height)) = (
                    sizes.upgrade(),
                    orientation.upgrade(),
                    width.upgrade(),
                    height.upgrade(),
                ) else {
                    return;
                };
                sizes.set_selected(0);
                orientation.set_selected(if width.value() > height.value() { 1 } else { 0 });
            }
        });
    }
    {
        let weak = window.downgrade();
        cancel.connect_clicked(move |_| {
            if let Some(w) = weak.upgrade() {
                w.close();
            }
        });
    }
    {
        let ui = ui.clone();
        let window = window.downgrade();
        let fields = [
            width.clone(),
            height.clone(),
            top.clone(),
            bottom.clone(),
            left.clone(),
            right.clone(),
        ];
        let headers = headers.clone();
        let metadata = metadata.clone();
        let language = language.clone();
        let review = review.clone();
        let initial_language = language.text().to_string();
        let initial_review = review.is_active();
        apply.connect_clicked(glib::clone!(
            #[weak]
            ui,
            move |_| {
                if ui.closed.get() || document_identity(&ui) != identity {
                    message.set_text(wordcraft_gnome::t!(
                        "The document has changed. Close and reopen its properties."
                    ));
                    message.set_visible(true);
                    return;
                }
                if wordcraft_engine::proof::lang::normalize_tag(&language.text()).is_none() {
                    language.add_css_class("error");
                    return;
                }
                for n in &fields {
                    n.update();
                }
                let values = fields
                    .each_ref()
                    .map(|n| n.value() as f32 * PT_PER_MM as f32);
                let setup = PageSetup {
                    width: values[0],
                    height: values[1],
                    top: values[2],
                    bottom: values[3],
                    left: values[4],
                    right: values[5],
                };
                let result = {
                    let mut value = ui.current.borrow_mut();
                    let [title, subject, author, keywords, category] =
                        metadata.each_ref().map(|r| r.text().to_string());
                    let session = &mut value.as_mut().unwrap().session;
                    let snapshot = session.edit_snapshot();
                    let pending = session.pending.clone();
                    let mark = session.undo_mark();
                    let result = apply_properties(
                        session,
                        setup,
                        scope.selected() == 0,
                        section,
                        &headers.values(),
                        &DocumentInfo {
                            title,
                            subject,
                            author,
                            keywords,
                            category,
                        },
                    )
                    .and_then(|changed| {
                        if language.text().as_str() == initial_language
                            && review.is_active() == initial_review
                        {
                            Ok(changed)
                        } else {
                            wordcraft_gnome::proofing_defaults::apply(
                                session,
                                &language.text(),
                                review.is_active(),
                            )
                            .map(|defaults| changed || defaults)
                        }
                    });
                    if result.is_err() {
                        session.restore(snapshot);
                        session.pending = pending;
                    } else {
                        session.merge_undo_since(mark);
                    }
                    result
                };
                match result {
                    Ok(changed) => {
                        if changed {
                            after_content_change(&ui);
                        }
                        if let Some(w) = window.upgrade() {
                            w.close();
                        }
                    }
                    Err(e) => {
                        message.set_text(&e);
                        message.set_visible(true);
                    }
                }
            }
        ));
    }
    window.present();
    DocumentProperties {
        window,
        metadata,
        width,
        height,
        top,
        bottom,
        left,
        right,
        apply,
        headers,
    }
}
