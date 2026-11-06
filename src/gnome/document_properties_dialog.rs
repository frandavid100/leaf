//! Document metadata, page geometry and section options in one apply transaction.
use super::*;
use wordcraft_doc::{
    props::TextDirection,
    section::{LineNumberRestart, LineNumbering, PageVAlign, SectionStart},
};
use wordcraft_gnome::document_properties::{ColumnKind, ColumnSetup, DocumentInfo};

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
    show_properties(ui, parent, None)
}

pub(super) fn show_section_properties(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    section: usize,
) -> DocumentProperties {
    show_properties(ui, parent, Some(section))
}

fn show_properties(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    target: Option<usize>,
) -> DocumentProperties {
    let (state, section, info) = {
        let mut value = ui.current.borrow_mut();
        let s = &mut value.as_mut().unwrap().session;
        let i = target.unwrap_or_else(|| page_options::current_section(s));
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
        .title(if target.is_some() {
            wordcraft_gnome::t!("Section Properties")
        } else {
            wordcraft_gnome::t!("Document Properties")
        })
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
    let original = ui.current.borrow().as_ref().unwrap().session.doc.sections()[section]
        .1
        .clone();
    let flow = adw::PreferencesGroup::new();
    flow.set_title(wordcraft_gnome::t!("Section Layout"));
    page.add(&flow);
    let starts = [
        SectionStart::NextPage,
        SectionStart::Continuous,
        SectionStart::EvenPage,
        SectionStart::OddPage,
        SectionStart::NextColumn,
    ];
    let start = option_combo(
        &flow,
        wordcraft_gnome::t!("Section Start"),
        &[
            wordcraft_gnome::t!("Next Page"),
            wordcraft_gnome::t!("Continuous"),
            wordcraft_gnome::t!("Even Page"),
            wordcraft_gnome::t!("Odd Page"),
            wordcraft_gnome::t!("Next Column"),
        ],
    );
    start.set_selected(
        starts
            .iter()
            .position(|value| *value == original.start)
            .unwrap() as u32,
    );
    start.set_sensitive(section != 0);
    let gutter = properties_number(
        &margins,
        wordcraft_gnome::t!("Gutter (mm)"),
        original.gutter,
    );
    let count = properties_number(
        &flow,
        wordcraft_gnome::t!("Number of Columns"),
        original.columns.count as f32 * PT_PER_MM as f32,
    );
    count.set_range(1., 12.);
    count.set_digits(0);
    let gap = properties_number(
        &flow,
        wordcraft_gnome::t!("Column Spacing (mm)"),
        original.columns.space,
    );
    let separator = option_switch(&flow, wordcraft_gnome::t!("Column Separator"));
    separator.set_active(original.columns.separator);
    let custom = option_switch(&flow, wordcraft_gnome::t!("Custom Column Widths"));
    custom.set_active(!original.columns.widths.is_empty());
    let mut widths = Vec::new();
    for index in 0..12 {
        let (width, spacing) = original.columns.widths.get(index).copied().unwrap_or((
            original.text_width() / original.columns.count.max(1) as f32,
            original.columns.space,
        ));
        let group = adw::PreferencesGroup::new();
        group.set_title(&wordcraft_gnome::i18n::interpolate(
            wordcraft_gnome::t!("Column {number}"),
            &[("number", (index + 1).to_string())],
        ));
        page.add(&group);
        let width = properties_number(&group, wordcraft_gnome::t!("Width (mm)"), width);
        let spacing =
            properties_number(&group, wordcraft_gnome::t!("Column Spacing (mm)"), spacing);
        group.set_visible(custom.is_active() && index < count.value_as_int() as usize);
        let group_ref = group.downgrade();
        let custom_ref = custom.downgrade();
        count.connect_value_changed(move |count| {
            if let (Some(group), Some(custom)) = (group_ref.upgrade(), custom_ref.upgrade()) {
                group.set_visible(custom.is_active() && index < count.value_as_int() as usize);
            }
        });
        let group_ref = group.downgrade();
        let count_ref = count.downgrade();
        custom.connect_active_notify(move |custom| {
            if let (Some(group), Some(count)) = (group_ref.upgrade(), count_ref.upgrade()) {
                group.set_visible(custom.is_active() && index < count.value_as_int() as usize);
            }
        });
        widths.push((width, spacing));
    }
    let vertical_modes = [
        PageVAlign::Top,
        PageVAlign::Center,
        PageVAlign::Justify,
        PageVAlign::Bottom,
    ];
    let vertical = option_combo(
        &flow,
        wordcraft_gnome::t!("Vertical Alignment"),
        &[
            wordcraft_gnome::t!("Top"),
            wordcraft_gnome::t!("Center"),
            wordcraft_gnome::t!("Justify"),
            wordcraft_gnome::t!("Bottom"),
        ],
    );
    vertical.set_selected(
        vertical_modes
            .iter()
            .position(|mode| *mode == original.valign)
            .unwrap() as u32,
    );
    let directions = [
        TextDirection::Horizontal,
        TextDirection::Down,
        TextDirection::Up,
    ];
    let direction = option_combo(
        &flow,
        wordcraft_gnome::t!("Section Text Direction"),
        &[
            wordcraft_gnome::t!("Horizontal"),
            wordcraft_gnome::t!("Top to Bottom"),
            wordcraft_gnome::t!("Bottom to Top"),
        ],
    );
    direction.set_selected(
        directions
            .iter()
            .position(|mode| *mode == original.text_direction)
            .unwrap() as u32,
    );
    let rtl = option_switch(&flow, wordcraft_gnome::t!("Right-to-Left Section"));
    rtl.set_active(original.rtl);
    let line_group = adw::PreferencesGroup::new();
    line_group.set_title(wordcraft_gnome::t!("Line Numbering"));
    page.add(&line_group);
    let line_numbers = option_switch(&line_group, wordcraft_gnome::t!("Show Line Numbers"));
    line_numbers.set_active(original.line_numbers.is_some());
    let line = original.line_numbers.clone().unwrap_or_default();
    let count_by = properties_number(
        &line_group,
        wordcraft_gnome::t!("Count By"),
        line.count_by as f32 * PT_PER_MM as f32,
    );
    count_by.set_range(1., 10000.);
    count_by.set_digits(0);
    let first_line = properties_number(
        &line_group,
        wordcraft_gnome::t!("Starting Line Number"),
        line.start as f32 * PT_PER_MM as f32,
    );
    first_line.set_range(0., 100000.);
    first_line.set_digits(0);
    let line_distance = properties_number(
        &line_group,
        wordcraft_gnome::t!("Distance from Text (mm)"),
        line.distance,
    );
    let restarts = [
        LineNumberRestart::Page,
        LineNumberRestart::Section,
        LineNumberRestart::Continuous,
    ];
    let restart = option_combo(
        &line_group,
        wordcraft_gnome::t!("Restart Line Numbers"),
        &[
            wordcraft_gnome::t!("Each Page"),
            wordcraft_gnome::t!("Each Section"),
            wordcraft_gnome::t!("Continuous"),
        ],
    );
    restart.set_selected(
        restarts
            .iter()
            .position(|value| *value == line.restart)
            .unwrap() as u32,
    );
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
    let odd_even = option_switch(
        &review_group,
        wordcraft_gnome::t!("Different Odd and Even Pages"),
    );
    let headers = HeaderControls::new(ui, &header_page, section, odd_even);
    let note_group = adw::PreferencesGroup::new();
    note_group.set_title(wordcraft_gnome::t!("Footnotes"));
    let notes = {
        let current = ui.current.borrow();
        NoteControls::new(
            &note_group,
            &current.as_ref().unwrap().session,
            section,
            target.is_none(),
        )
    };
    if target.is_some() {
        page.add(&note_group)
    } else {
        information.add(&note_group)
    }

    if target.is_some() {
        headers.section.set_sensitive(false);
        stack.page(&information).set_visible(false);
        stack.set_visible_child_name("page");
    }
    stack.add_titled(
        &header_page,
        Some("headers"),
        wordcraft_gnome::t!("Headers and Footers"),
    );
    if target.is_none() {
        stack.page(&page).set_visible(false);
        stack.page(&header_page).set_visible(false);
    }
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
    if target.is_some() {
        toolbar.add_top_bar(&switcher);
    }
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
                    let metadata = DocumentInfo {
                        title,
                        subject,
                        author,
                        keywords,
                        category,
                    };
                    let result = if target.is_some() {
                        document_properties::apply_section_setup(session, false, section, |old| {
                            let mut next = setup.section(old.clone())?;
                            next.start = starts[start.selected() as usize];
                            next.valign = vertical_modes[vertical.selected() as usize];
                            next.rtl = rtl.is_active();
                            let text_direction = directions[direction.selected() as usize];
                            next.text_direction_set |= next.text_direction != text_direction;
                            next.text_direction = text_direction;
                            next.line_numbers = line_numbers.is_active().then(|| LineNumbering {
                                count_by: count_by.value_as_int() as u32,
                                start: first_line.value_as_int() as u32,
                                distance: line_distance.value() as f32 * PT_PER_MM as f32,
                                restart: restarts[restart.selected() as usize],
                            });
                            let new_gutter = gutter.value() as f32 * PT_PER_MM as f32;
                            if (new_gutter - old.gutter).abs() >= 0.02 {
                                next.gutter = new_gutter;
                            }
                            let new_count = count.value_as_int() as u32;
                            let new_gap = gap.value() as f32 * PT_PER_MM as f32;
                            let new_widths: Vec<_> = if custom.is_active() {
                                widths
                                    .iter()
                                    .take(new_count as usize)
                                    .map(|(width, spacing)| {
                                        (
                                            width.value() as f32 * PT_PER_MM as f32,
                                            spacing.value() as f32 * PT_PER_MM as f32,
                                        )
                                    })
                                    .collect()
                            } else {
                                Vec::new()
                            };
                            let near = |a: f32, b: f32| (a - b).abs() < 0.02;
                            let widths_changed = new_widths.len() != old.columns.widths.len()
                                || new_widths
                                    .iter()
                                    .zip(&old.columns.widths)
                                    .any(|((w, g), (x, y))| !near(*w, *x) || !near(*g, *y));
                            if new_count != old.columns.count
                                || !near(new_gap, old.columns.space)
                                || widths_changed
                            {
                                next.columns = ColumnSetup {
                                    count: new_count,
                                    space: new_gap,
                                    separator: separator.is_active(),
                                    kind: if custom.is_active() {
                                        ColumnKind::Custom
                                    } else {
                                        ColumnKind::Equal
                                    },
                                    widths: new_widths,
                                }
                                .resolved(next.text_width())?;
                            } else {
                                next.columns.separator = separator.is_active();
                            }
                            Ok(next)
                        })
                        .and_then(|changed| {
                            page_options::apply(session, &headers.values())
                                .map(|headers| changed || headers)
                        })
                    } else {
                        let mut options =
                            wordcraft_gnome::page_options::Options::current(session, section);
                        options.odd_even = headers.odd_even.is_active();
                        metadata.apply(session).and_then(|changed| {
                            page_options::apply(session, &options).map(|options| changed || options)
                        })
                    }
                    .and_then(|changed| notes.apply(session).map(|n| changed || n))
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

struct NoteControls {
    format: adw::ComboRow,
    restart: adw::ComboRow,
    start: gtk::SpinButton,
    formats: Vec<String>,
    initial: (u32, u32, i32),
    section: usize,
    whole: bool,
}
impl NoteControls {
    fn new(group: &adw::PreferencesGroup, s: &Session, section: usize, whole: bool) -> Self {
        let mut sect = s.doc.sections()[section].1.clone();
        if whole {
            sect.footnote_pr = Default::default();
        }
        let options = s.doc.note_options(&sect, false);
        let mut formats = vec![
            "decimal".to_string(),
            "lowerRoman".into(),
            "upperRoman".into(),
            "lowerLetter".into(),
            "upperLetter".into(),
        ];
        let mut labels = vec![
            "1, 2, 3".to_string(),
            "i, ii, iii".into(),
            "I, II, III".into(),
            "a, b, c".into(),
            "A, B, C".into(),
        ];
        if !formats.iter().any(|f| f == options.num_fmt.ooxml()) {
            formats.push(options.num_fmt.ooxml().into());
            labels.push(options.num_fmt.ooxml().into());
        }
        let label_refs: Vec<_> = labels.iter().map(String::as_str).collect();
        let format = option_combo(group, wordcraft_gnome::t!("Number Format"), &label_refs);
        format.set_widget_name("properties-footnote-format");
        format.set_selected(
            formats
                .iter()
                .position(|f| f == options.num_fmt.ooxml())
                .unwrap() as u32,
        );
        let restart = option_combo(
            group,
            wordcraft_gnome::t!("Restart Numbering"),
            &[
                wordcraft_gnome::t!("Continuous"),
                wordcraft_gnome::t!("Each Section"),
                wordcraft_gnome::t!("Each Page"),
            ],
        );
        restart.set_selected(match options.num_restart {
            wordcraft_doc::section::NoteRestart::EachSect => 1,
            wordcraft_doc::section::NoteRestart::EachPage => 2,
            _ => 0,
        });
        let row = adw::ActionRow::new();
        row.set_title(wordcraft_gnome::t!("Start At"));
        let start = gtk::SpinButton::with_range(1., 32767., 1.);
        start.set_value(options.num_start as f64);
        start.set_valign(gtk::Align::Center);
        start.set_widget_name("properties-footnote-start");
        accessible_name(&start, wordcraft_gnome::t!("Start At"));
        row.add_suffix(&start);
        row.set_activatable_widget(Some(&start));
        group.add(&row);
        let initial = (format.selected(), restart.selected(), start.value_as_int());
        Self {
            format,
            restart,
            start,
            formats,
            initial,
            section,
            whole,
        }
    }
    fn apply(&self, s: &mut Session) -> Result<bool, String> {
        self.start.update();
        let now = (
            self.format.selected(),
            self.restart.selected(),
            self.start.value_as_int(),
        );
        if now == self.initial {
            return Ok(false);
        }
        let mut args =
            serde_json::json!({"kind":"footnote","scope":if self.whole{"document"}else{"section"}});
        if now.0 != self.initial.0 {
            args["numFmt"] = serde_json::json!(self.formats[now.0 as usize]);
        }
        if now.1 != self.initial.1 {
            args["numRestart"] =
                serde_json::json!(["continuous", "eachSect", "eachPage"][now.1 as usize]);
        }
        if now.2 != self.initial.2 {
            args["numStart"] = serde_json::json!(now.2);
        }
        let selection = s.sel.clone();
        let hint = s.page_hint;
        let pending = s.pending.clone();
        if !self.whole {
            let ends = s.doc.sections();
            let start = if self.section == 0 {
                0
            } else {
                ends[self.section - 1].0 + 1
            };
            let end = ends[self.section].0;
            let path = s
                .doc
                .para_paths(wordcraft_doc::StoryRef::Body)
                .into_iter()
                .find(|p| {
                    p.0.first()
                        .is_some_and(|i| (*i as usize) >= start && (*i as usize) <= end)
                })
                .ok_or(wordcraft_gnome::t!(
                    "The section does not contain an editable paragraph"
                ))?;
            selection::set_caret(
                s,
                &Pos {
                    story: wordcraft_doc::StoryRef::Body,
                    path,
                    off: 0,
                },
                false,
            )?;
        }
        let result = s
            .run("references.noteOptions", &args)
            .map_err(|e| e.to_string());
        let _ = selection::select_range(s, &selection.anchor, &selection.focus);
        s.page_hint = hint;
        s.pending = pending;
        result.map(|_| true)
    }
}
