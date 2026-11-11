use super::*;

// Immediate confirmed section layout; only public engine commands edit the model.
use wordcraft_gnome::document_properties::{ColumnKind, ColumnScope, ColumnSetup};
pub(super) struct ColumnControls {
    pub(super) mode: adw::ComboRow,
    pub(super) count: gtk::SpinButton,
    pub(super) gap: gtk::SpinButton,
    pub(super) separator: gtk::CheckButton,
    pub(super) mixed: Cell<bool>,
    pub(super) custom: adw::PreferencesGroup,
    pub(super) widths: Vec<(gtk::SpinButton, gtk::SpinButton)>,
    pub(super) rows: Vec<adw::ActionRow>,
    pub(super) format_note: gtk::Label,
    pub(super) scope: adw::ComboRow,
    pub(super) scopes: Vec<ColumnScope>,
    pub(super) preview: gtk::DrawingArea,
    pub(super) text_width: Cell<f32>,
    section: RefCell<wordcraft_doc::section::SectionProps>,
    pub(super) last_count: Cell<u32>,
    pub(super) last_mode: Cell<u32>,
}
impl ColumnControls {
    pub(super) fn setup(&self) -> ColumnSetup {
        let kind = match self.mode.selected() {
            1 => ColumnKind::Left,
            2 => ColumnKind::Right,
            3 => ColumnKind::Custom,
            _ => ColumnKind::Equal,
        };
        let count = if matches!(kind, ColumnKind::Left | ColumnKind::Right) {
            2
        } else {
            self.count.value_as_int() as u32
        };
        ColumnSetup {
            count,
            kind,
            space: self.gap.value() as f32 * PT_PER_MM as f32,
            separator: self.separator.is_active(),
            widths: self
                .widths
                .iter()
                .take(count as usize)
                .map(|(w, g)| {
                    (
                        w.value() as f32 * PT_PER_MM as f32,
                        g.value() as f32 * PT_PER_MM as f32,
                    )
                })
                .collect(),
        }
    }
    pub(super) fn refresh(&self) {
        let preset = matches!(self.mode.selected(), 1 | 2);
        if preset {
            self.count.set_value(2.0);
        }
        let custom = self.mode.selected() == 3;
        let count = self.count.value_as_int() as usize;
        let changed_count = self.last_count.replace(count as u32) != count as u32;
        let changed_mode = self.last_mode.replace(self.mode.selected()) != self.mode.selected();
        if custom && (changed_count || changed_mode) {
            let gap = self.gap.value() * PT_PER_MM;
            let width = ((self.text_width.get() as f64 - gap * (count.saturating_sub(1)) as f64)
                / count.max(1) as f64)
                .max(18.0);
            for (w, g) in self.widths.iter().take(count) {
                w.set_value(width / PT_PER_MM);
                g.set_value(gap / PT_PER_MM);
            }
        }
        self.count.set_sensitive(!preset && !self.mixed.get());
        self.gap
            .set_sensitive(count > 1 && !custom && !self.mixed.get());
        self.separator.set_sensitive(count > 1 && !self.mixed.get());
        self.custom.set_visible(custom);
        self.format_note.set_visible(self.mode.selected() != 0);
        for (i, row) in self.rows.iter().enumerate() {
            row.set_visible(i < count);
            self.widths[i].1.set_sensitive(i + 1 < count);
        }
        self.preview.queue_draw();
    }
}
pub(super) struct ColumnEditor {
    pub(super) root: gtk::Box,
    pub(super) controls: Rc<ColumnControls>,
    pub(super) apply: gtk::Button,
    pub(super) scope_header: adw::PreferencesGroup,
}
pub(super) struct ColumnsDialog {
    pub(super) window: adw::Dialog,
    pub(super) controls: Rc<ColumnControls>,
}
pub(super) fn columns_editor(ui: &Rc<Ui>, on_applied: impl Fn() + 'static) -> ColumnEditor {
    let (initial, text_width, section, can_select, can_forward) = {
        let state = ui.current.borrow();
        let s = &state.as_ref().unwrap().session;
        let scope = if wordcraft_engine::cmd::page::selection_sectionable(s) {
            ColumnScope::Selection
        } else {
            ColumnScope::Section
        };
        let section = document_properties::column_section(s, scope);
        (
            ColumnSetup::for_scope(s, scope),
            section.text_width(),
            section,
            wordcraft_engine::cmd::page::selection_sectionable(s),
            s.sel.focus.story == wordcraft_doc::StoryRef::Body && s.sel.focus.path.depth() == 0,
        )
    };
    let identity = Rc::new(Cell::new(document_identity(ui)));
    let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    page.set_margin_start(4);
    page.set_margin_end(4);
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
    // Internal commit target shared by presets and confirmed field edits.
    apply.set_visible(false);
    let keep_open = Rc::new(Cell::new(false));
    apply.add_css_class("suggested-action");
    apply.set_widget_name("columns-apply");
    let group = adw::PreferencesGroup::new();
    group.set_title(wordcraft_gnome::t!("Layout"));
    page.append(&group);
    let preview = gtk::DrawingArea::new();
    preview.set_content_width(320);
    preview.set_content_height(190);
    preview.set_halign(gtk::Align::Fill);
    // Preview follows detailed controls; presets remain at the beginning.

    let mode = adw::ComboRow::new();
    mode.set_title(wordcraft_gnome::t!("Widths"));
    mode.set_model(Some(&gtk::StringList::new(&[
        wordcraft_gnome::t!("Equal"),
        wordcraft_gnome::t!("Narrow Left Column"),
        wordcraft_gnome::t!("Narrow Right Column"),
        wordcraft_gnome::t!("Custom"),
    ])));
    let initial_mode = match initial
        .preset(text_width)
        .map(|(_, kind)| kind)
        .unwrap_or(initial.kind)
    {
        ColumnKind::Left => 1,
        ColumnKind::Right => 2,
        ColumnKind::Custom => 3,
        _ => 0,
    };
    mode.set_selected(initial_mode);
    let count = gtk::SpinButton::with_range(1.0, 12.0, 1.0);
    count.set_value(initial.count.clamp(1, 12) as f64);
    count.set_valign(gtk::Align::Center);
    count.set_width_chars(3);
    count.set_widget_name("columns-count");
    count.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
    let row = adw::ActionRow::new();
    row.set_title(wordcraft_gnome::t!("Number of Columns"));
    row.add_suffix(&count);
    row.set_activatable_widget(Some(&count));
    accessible_name(&count, wordcraft_gnome::t!("Number of Columns"));
    group.add(&row);
    group.add(&mode);
    let spacing_group = adw::PreferencesGroup::new();
    page.append(&spacing_group);
    let gap = properties_number(
        &spacing_group,
        wordcraft_gnome::t!("Spacing (mm)"),
        initial.space,
    );
    gap.set_range(0.0, 300.0 / PT_PER_MM);
    gap.set_widget_name("columns-gap");
    let separator = gtk::CheckButton::new();
    separator.set_widget_name("columns-separator");
    accessible_name(&separator, wordcraft_gnome::t!("Line Between Columns"));
    separator.set_active(initial.separator);
    separator.set_valign(gtk::Align::Center);
    let row = adw::ActionRow::new();
    row.set_title(wordcraft_gnome::t!("Line Between Columns"));
    row.set_activatable_widget(Some(&separator));
    row.add_suffix(&separator);
    spacing_group.add(&row);
    let format_note = gtk::Label::new(Some(wordcraft_gnome::t!(
        "Unequal widths are preserved in DOCX. In ODT, the engine saves them as equal-width columns. The preview shows column geometry; unequal-column text flow is limited by the current engine."
    )));
    format_note.set_wrap(true);
    format_note.set_xalign(0.0);
    format_note.add_css_class("dim-label");
    format_note.set_margin_start(12);
    format_note.set_margin_end(12);
    spacing_group.add(&format_note);
    let custom = adw::PreferencesGroup::new();
    custom.set_title(wordcraft_gnome::t!("Custom Widths"));
    custom.set_description(Some(wordcraft_gnome::t!(
        "Width and following spacing in millimeters. The last column has no following spacing."
    )));
    page.insert_child_after(&custom, Some(&group));
    let mut widths = Vec::new();
    let mut rows = Vec::new();
    for i in 0..12 {
        let (w, g) = initial.widths.get(i).copied().unwrap_or((
            (text_width - initial.space * (initial.count.saturating_sub(1)) as f32)
                / initial.count.max(1) as f32,
            initial.space,
        ));
        let row = adw::ActionRow::new();
        row.set_title(&wordcraft_gnome::i18n::interpolate(
            wordcraft_gnome::t!("Column {n}"),
            &[("n", (i + 1).to_string())],
        ));
        let width = mm_input(w as f64 / PT_PER_MM, 18.0 / PT_PER_MM, 1584.0 / PT_PER_MM);
        width.set_width_chars(5);
        width.set_tooltip_text(Some(wordcraft_gnome::t!("Width (mm)")));
        width.update_property(&[gtk::accessible::Property::Label(
            &wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Width of Column {n} (mm)"),
                &[("n", (i + 1).to_string())],
            ),
        )]);
        let after = mm_input(g as f64 / PT_PER_MM, 0.0, 300.0 / PT_PER_MM);
        after.set_width_chars(5);
        after.set_tooltip_text(Some(wordcraft_gnome::t!("Following Spacing (mm)")));
        after.update_property(&[gtk::accessible::Property::Label(
            &wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Spacing After Column {} (mm)"),
                &[("0", format!("{}", i + 1))],
            ),
        )]);
        row.add_suffix(&width);
        row.add_suffix(&after);
        custom.add(&row);
        rows.push(row);
        widths.push((width, after));
    }
    let scopes_group = adw::PreferencesGroup::new();
    page.prepend(&scopes_group);
    let scope = adw::ComboRow::new();
    scope.set_title(wordcraft_gnome::t!("Apply To"));
    let mut labels = vec![
        wordcraft_gnome::t!("Current Section"),
        wordcraft_gnome::t!("Whole Document"),
    ];
    let mut scopes = vec![ColumnScope::Section, ColumnScope::Document];
    if can_select {
        labels.push(wordcraft_gnome::t!("Selected Text"));
        scopes.push(ColumnScope::Selection);
    }
    if can_forward {
        labels.push(wordcraft_gnome::t!("From This Point Onwards"));
        scopes.push(ColumnScope::Forward);
    }
    scope.set_model(Some(&gtk::StringList::new(&labels)));
    if can_select {
        scope.set_selected(2);
    }
    scopes_group.add(&scope);
    let hint = adw::ActionRow::new();
    hint.set_title(wordcraft_gnome::t!("Selected Text"));
    hint.set_subtitle(wordcraft_gnome::t!("Creates continuous sections around the selected text, leaving the rest of the document unchanged."));
    hint.set_visible(can_select);
    scopes_group.add(&hint);
    let error = gtk::Label::new(None);
    error.set_widget_name("columns-error");
    error.set_wrap(true);
    error.add_css_class("error");
    error.set_visible(false);
    let errors = adw::PreferencesGroup::new();
    errors.add(&error);
    page.append(&errors);
    page.append(&preview);
    let scope_header = scopes_group.clone();
    let controls = Rc::new(ColumnControls {
        mixed: Cell::new(false),
        mode,
        count,
        gap,
        separator,
        custom,
        widths,
        rows,
        format_note,
        scope,
        scopes,
        preview,
        text_width: Cell::new(text_width),
        section: RefCell::new(section),
        last_count: Cell::new(initial.count.clamp(1, 12)),
        last_mode: Cell::new(initial_mode),
    });
    {
        let weak = Rc::downgrade(&controls);
        let ui = Rc::downgrade(ui);
        controls.mode.connect_selected_notify(move |_| {
            if let (Some(c), Some(ui)) = (weak.upgrade(), ui.upgrade()) {
                let syncing = ui.format_syncing.replace(true);
                if !syncing {
                    c.mixed.set(false);
                    c.count.set_text(&c.count.value_as_int().to_string());
                    c.gap.set_text(&format!("{:.2}", c.gap.value()));
                    c.separator.set_inconsistent(false);
                }
                c.refresh();
                ui.format_syncing.set(syncing);
            }
        });
    }
    {
        let weak = Rc::downgrade(&controls);
        let ui = Rc::downgrade(ui);
        controls.count.connect_value_changed(move |_| {
            if let (Some(c), Some(ui)) = (weak.upgrade(), ui.upgrade()) {
                let syncing = ui.format_syncing.replace(true);
                c.refresh();
                ui.format_syncing.set(syncing);
            }
        });
    }
    for n in std::iter::once(&controls.gap).chain(controls.widths.iter().flat_map(|(w, g)| [w, g]))
    {
        let weak = Rc::downgrade(&controls);
        n.connect_value_changed(move |_| {
            if let Some(c) = weak.upgrade() {
                c.preview.queue_draw();
            }
        });
    }
    {
        let weak = Rc::downgrade(&controls);
        controls.separator.connect_toggled(move |_| {
            if let Some(c) = weak.upgrade() {
                c.preview.queue_draw();
            }
        });
    }
    {
        let weak = Rc::downgrade(&controls);
        controls.preview.set_draw_func(move |area, cr, w, h| {
            let Some(c) = weak.upgrade() else {
                return;
            };
            if c.mixed.get() {
                return;
            }
            let Ok(cols) = c.setup().resolved(c.text_width.get()) else {
                return;
            };
            let color = area.color();
            let section = c.section.borrow();
            let paper_w = section.page_w as f64;
            let paper_h = section.page_h as f64;
            let scale = ((w as f64 - 28.) / paper_w).min((h as f64 - 24.) / paper_h);
            let margin = (w as f64 - paper_w * scale) / 2.;
            cr.set_source_rgba(
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
                0.15,
            );
            cr.rectangle(margin, 12., paper_w * scale, paper_h * scale);
            let _ = cr.stroke();
            let y = 12. + section.margin_top as f64 * scale;
            let height = section.text_height() as f64 * scale;
            let mut x = margin + (section.margin_left + section.gutter) as f64 * scale;
            for i in 0..cols.count as usize {
                let (cw, g) = cols.widths.get(i).copied().unwrap_or((
                    (c.text_width.get() - cols.space * (cols.count - 1) as f32) / cols.count as f32,
                    cols.space,
                ));
                let width = cw as f64 * scale;
                cr.set_source_rgba(
                    color.red() as f64,
                    color.green() as f64,
                    color.blue() as f64,
                    0.6,
                );
                for line in 0..8 {
                    let width = if line == 7 { width * 0.64 } else { width };
                    cr.rectangle(
                        x + 3.0,
                        y + 4.0 + line as f64 * ((height - 12.) / 8.).max(2.),
                        (width - 6.0).max(1.0),
                        2.0,
                    );
                }
                let _ = cr.fill();
                if cols.separator && i + 1 < cols.count as usize {
                    cr.move_to(x + width + g as f64 * scale / 2.0, y + 6.0);
                    cr.line_to(x + width + g as f64 * scale / 2.0, y + height - 6.0);
                    let _ = cr.stroke();
                }
                x += width + g as f64 * scale;
            }
        });
    }
    {
        let c = controls.clone();
        page.connect_destroy(move |_| {
            let _ = &c;
        });
    }
    {
        let weak = Rc::downgrade(&controls);
        let ui = Rc::downgrade(ui);
        let keep_open = keep_open.clone();
        apply.connect_clicked(move |_| {
            let (Some(c), Some(ui)) = (weak.upgrade(), ui.upgrade()) else {
                return;
            };
            if ui.format_syncing.get() || c.mixed.get() {
                return;
            }
            if numeric_value(&c.count).is_none_or(|v| v.fract() != 0.) {
                error.set_text(wordcraft_gnome::t!(
                    "Enter a whole number of columns from 1 to 12."
                ));
                error.set_visible(true);
                return;
            }
            if ui.closed.get() || document_identity(&ui) != identity.get() {
                error.set_text(wordcraft_gnome::t!(
                    "The document has changed. Close and reopen Columns."
                ));
                error.set_visible(true);
                return;
            }
            let mut numbers = vec![&c.count];
            if c.count.value_as_int() > 1 {
                if c.mode.selected() == 3 {
                    numbers.extend(
                        c.widths
                            .iter()
                            .take(c.count.value_as_int() as usize)
                            .flat_map(|(w, g)| [w, g]),
                    );
                } else {
                    numbers.push(&c.gap);
                }
            }
            if numbers.iter().any(|n| numeric_value(n).is_none()) {
                error.set_text(wordcraft_gnome::t!(
                    "Complete the column values before applying them."
                ));
                error.set_visible(true);
                return;
            }
            let syncing = ui.format_syncing.replace(true);
            for n in numbers {
                n.update();
            }
            ui.format_syncing.set(syncing);
            let setup = c.setup();
            let scope = c.scopes[c.scope.selected() as usize];
            let result = {
                let mut d = ui.current.borrow_mut();
                document_properties::apply_columns(&mut d.as_mut().unwrap().session, &setup, scope)
            };
            match result {
                Ok(changed) => {
                    identity.set(document_identity(&ui));
                    if let Some(p) = ui.paragraph.borrow().as_ref() {
                        p.last_columns.replace(Some((setup.clone(), scope)));
                    }
                    if changed {
                        after_content_change(&ui);
                    }
                    error.set_visible(false);
                    if !keep_open.get() {
                        on_applied();
                    }
                }
                Err(e) => {
                    error.set_text(&e);
                    error.set_visible(true);
                }
            }
        });
    }
    let presets = gtk::FlowBox::new();
    presets.set_selection_mode(gtk::SelectionMode::None);
    presets.set_min_children_per_line(3);
    presets.set_max_children_per_line(3);
    presets.set_homogeneous(true);
    presets.set_row_spacing(4);
    presets.set_column_spacing(4);
    let mut choices = Vec::new();
    for (label, count, mode, id) in [
        (wordcraft_gnome::t!("One"), 1, 0, "one"),
        (wordcraft_gnome::t!("Two"), 2, 0, "two"),
        (wordcraft_gnome::t!("Three"), 3, 0, "three"),
        (wordcraft_gnome::t!("Narrow Left Column"), 2, 1, "left"),
        (wordcraft_gnome::t!("Narrow Right Column"), 2, 2, "right"),
    ] {
        let b = gtk::Button::new();
        b.set_widget_name(&format!("columns-preset-{id}"));
        b.set_tooltip_text(Some(label));
        accessible_name(&b, label);
        let sample = gtk::DrawingArea::new();
        sample.set_content_width(44);
        sample.set_content_height(44);
        sample.set_draw_func(move |w, c, _, _| {
            let fg = w.color();
            c.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 0.8);
            let ratios: Vec<f64> = if mode == 1 {
                vec![0.3, 0.7]
            } else if mode == 2 {
                vec![0.7, 0.3]
            } else {
                vec![1. / count as f64; count]
            };
            let mut x = 3.;
            for ratio in ratios {
                let width = 38. * ratio - 3.;
                for y in [6., 13., 20., 27., 34.] {
                    c.move_to(x, y);
                    c.line_to(x + width, y);
                }
                x += width + 3.;
            }
            let _ = c.stroke();
        });
        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&sample));
        let mark = gtk::Image::from_icon_name("object-select-symbolic");
        mark.set_widget_name("columns-current-marker");
        mark.set_halign(gtk::Align::End);
        mark.set_valign(gtk::Align::End);
        mark.set_visible(false);
        overlay.add_overlay(&mark);
        b.set_child(Some(&overlay));
        presets.insert(&b, -1);
        choices.push((b.clone(), count, mode));
        let weak = Rc::downgrade(&controls);
        let commit = apply.downgrade();
        let ui = Rc::downgrade(ui);
        b.connect_clicked(move |_| {
            if let (Some(c), Some(apply), Some(ui)) =
                (weak.upgrade(), commit.upgrade(), ui.upgrade())
            {
                let syncing = ui.format_syncing.replace(true);
                c.mixed.set(false);
                c.count.set_value(count as f64);
                c.mode.set_selected(mode);
                c.count.set_text(&count.to_string());
                c.gap.set_text(&format!("{:.2}", c.gap.value()));
                c.separator.set_inconsistent(false);
                c.refresh();
                ui.format_syncing.set(syncing);
                apply.emit_clicked();
            }
        });
    }
    let matching = gtk::Label::new(None);
    matching.add_css_class("dim-label");
    matching.set_wrap(true);
    matching.set_max_width_chars(34);
    matching.set_xalign(0.);
    matching.set_widget_name("columns-preset-state");
    let update: Rc<dyn Fn()> = {
        let weak = Rc::downgrade(&controls);
        let matching = matching.clone();
        Rc::new(move || {
            if let Some(c) = weak.upgrade() {
                let setup = c.setup();
                let mut found = false;
                for (b, count, mode) in &choices {
                    let on = !c.mixed.get()
                        && setup.count == *count as u32
                        && c.mode.selected() == *mode;
                    if on {
                        b.add_css_class("current-style");
                        found = true;
                    } else {
                        b.remove_css_class("current-style");
                    }
                    if let Some(mark) = find_widget(b, "columns-current-marker") {
                        mark.set_visible(on);
                    }
                    b.update_state(&[gtk::accessible::State::Pressed(if c.mixed.get() {
                        gtk::AccessibleTristate::Mixed
                    } else if on {
                        gtk::AccessibleTristate::True
                    } else {
                        gtk::AccessibleTristate::False
                    })]);
                }
                matching.set_text(if c.mixed.get() {
                    wordcraft_gnome::t!("Mixed — choose a layout to edit these sections together.")
                } else if found {
                    ""
                } else {
                    wordcraft_gnome::t!("Custom")
                });
                matching.set_visible(c.mixed.get() || !found);
            }
        })
    };
    let f = update.clone();
    controls.mode.connect_selected_notify(move |_| f());
    let f = update.clone();
    controls.count.connect_value_changed(move |_| f());
    update();
    page.insert_child_after(&presets, Some(&scope_header));
    page.insert_child_after(&matching, Some(&presets));
    controls.refresh();
    let commit: Rc<dyn Fn()> = {
        let button = apply.downgrade();
        let ui = Rc::downgrade(ui);
        let keep = keep_open.clone();
        Rc::new(move || {
            let (Some(button), Some(ui)) = (button.upgrade(), ui.upgrade()) else {
                return;
            };
            if ui.format_syncing.get() {
                return;
            }
            keep.set(true);
            button.emit_clicked();
            keep.set(false);
        })
    };
    for n in std::iter::once(&controls.count)
        .chain(std::iter::once(&controls.gap))
        .chain(controls.widths.iter().flat_map(|(w, g)| [w, g]))
    {
        n.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
        let commit = commit.clone();
        bind_numeric(ui, n, move |_| {
            commit();
            Ok(())
        });
    }
    let commit_mode = commit.clone();
    controls
        .mode
        .connect_selected_notify(move |_| commit_mode());
    let commit_separator = commit.clone();
    controls
        .separator
        .connect_toggled(move |_| commit_separator());
    let reload: Rc<dyn Fn()> = {
        let weak = Rc::downgrade(&controls);
        let ui = Rc::downgrade(ui);
        let update = update.clone();
        Rc::new(move || {
            let (Some(c), Some(ui)) = (weak.upgrade(), ui.upgrade()) else {
                return;
            };
            let syncing = ui.format_syncing.replace(true);
            let current = ui.current.borrow();
            let Some(d) = current.as_ref() else {
                ui.format_syncing.set(syncing);
                return;
            };
            let scope = c.scopes[c.scope.selected() as usize];
            let section = document_properties::column_section(&d.session, scope);
            let setup = ColumnSetup::for_scope(&d.session, scope);
            c.text_width.set(section.text_width());
            c.section.replace(section);
            let mixed = document_properties::columns_mixed(&d.session, scope);
            drop(current);
            c.mixed.set(mixed);
            c.count.set_value(setup.count as f64);
            c.count.set_text(&setup.count.to_string());
            c.gap.set_value(setup.space as f64 / PT_PER_MM);
            c.gap.set_text(&format!("{:.2}", c.gap.value()));
            let mode = match setup
                .preset(c.text_width.get())
                .map(|(_, k)| k)
                .unwrap_or(setup.kind)
            {
                ColumnKind::Left => 1,
                ColumnKind::Right => 2,
                ColumnKind::Custom => 3,
                _ => 0,
            };
            c.mode.set_selected(if mixed {
                gtk::INVALID_LIST_POSITION
            } else {
                mode
            });
            c.last_mode.set(c.mode.selected());
            c.last_count.set(setup.count);
            for ((w, g), (width, gap)) in c.widths.iter().zip(&setup.widths) {
                w.set_value(*width as f64 / PT_PER_MM);
                g.set_value(*gap as f64 / PT_PER_MM);
                w.set_text(&format!("{:.2}", w.value()));
                g.set_text(&format!("{:.2}", g.value()));
            }
            c.separator.set_active(setup.separator);
            c.separator.set_inconsistent(mixed);
            c.refresh();
            if mixed {
                c.count.set_text("");
                c.gap.set_text("");
            }
            hint.set_visible(scope == ColumnScope::Selection);
            update();
            ui.format_syncing.set(syncing);
        })
    };
    let f = reload.clone();
    controls.scope.connect_selected_notify(move |_| f());
    reload();
    page.append(&apply); // Keep the hidden shared commit alive with its editor.
    ColumnEditor {
        root: page,
        scope_header,
        controls,
        apply,
    }
}
pub(super) fn show_columns(ui: &Rc<Ui>, parent: &ApplicationWindow) -> ColumnsDialog {
    let window = adw::Dialog::builder()
        .title(wordcraft_gnome::t!("Columns"))
        .content_width(480)
        .content_height(620)
        .build();
    let weak = window.downgrade();
    let editor = columns_editor(ui, move || {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    let toolbar = ToolbarView::new();
    let header = HeaderBar::new();
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Close"));
    header.pack_start(&cancel);
    // Numeric edits and presets apply directly; closing is not an extra Apply.
    toolbar.add_top_bar(&header);
    editor.root.remove(&editor.scope_header);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 12);
    body.append(&editor.scope_header);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_child(Some(&editor.root));
    body.append(&scroll);
    toolbar.set_content(Some(&body));
    window.set_child(Some(&toolbar));
    let weak = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    window.present(Some(parent));
    ColumnsDialog {
        window,
        controls: editor.controls,
    }
}
