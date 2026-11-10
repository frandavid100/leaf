use super::*;

// Staged section layout; only the public engine command changes the document.
use wordcraft_gnome::document_properties::{ColumnKind, ColumnScope, ColumnSetup};
pub(super) struct ColumnControls {
    pub(super) mode: adw::ComboRow,
    pub(super) count: gtk::SpinButton,
    pub(super) gap: gtk::SpinButton,
    pub(super) separator: gtk::Switch,
    pub(super) custom: adw::PreferencesGroup,
    pub(super) widths: Vec<(gtk::SpinButton, gtk::SpinButton)>,
    pub(super) rows: Vec<adw::ActionRow>,
    pub(super) format_note: gtk::Label,
    pub(super) scope: adw::ComboRow,
    pub(super) scopes: Vec<ColumnScope>,
    pub(super) preview: gtk::DrawingArea,
    pub(super) text_width: f32,
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
            let width = ((self.text_width as f64 - gap * (count.saturating_sub(1)) as f64)
                / count.max(1) as f64)
                .max(18.0);
            for (w, g) in self.widths.iter().take(count) {
                w.set_value(width / PT_PER_MM);
                g.set_value(gap / PT_PER_MM);
            }
        }
        self.count.set_sensitive(!preset);
        self.gap.set_sensitive(count > 1 && !custom);
        self.separator.set_sensitive(count > 1);
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
    pub(super) apply: gtk::Button,
}
pub(super) fn columns_editor(ui: &Rc<Ui>, on_applied: impl Fn() + 'static) -> ColumnEditor {
    let (initial, text_width, can_select, can_forward) = {
        let state = ui.current.borrow();
        let s = &state.as_ref().unwrap().session;
        (
            ColumnSetup::current(s),
            wordcraft_engine::cmd::page::sect(s).text_width(),
            wordcraft_engine::cmd::page::selection_sectionable(s),
            s.sel.focus.story == wordcraft_doc::StoryRef::Body && s.sel.focus.path.depth() == 0,
        )
    };
    let identity = Rc::new(Cell::new(document_identity(ui)));
    let page = gtk::Box::new(gtk::Orientation::Vertical, 12);
    page.set_margin_start(4);
    page.set_margin_end(4);
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
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
    mode.set_selected(if initial.kind == ColumnKind::Custom {
        3
    } else {
        0
    });
    group.add(&mode);
    let count = gtk::SpinButton::with_range(1.0, 12.0, 1.0);
    count.set_value(initial.count.clamp(1, 12) as f64);
    count.set_valign(gtk::Align::Center);
    count.set_width_chars(3);
    count.set_widget_name("columns-count");
    let row = adw::ActionRow::new();
    row.set_title(wordcraft_gnome::t!("Number of Columns"));
    row.add_suffix(&count);
    row.set_activatable_widget(Some(&count));
    accessible_name(&count, wordcraft_gnome::t!("Number of Columns"));
    group.add(&row);
    let gap = properties_number(&group, wordcraft_gnome::t!("Spacing (mm)"), initial.space);
    gap.set_range(0.0, 300.0 / PT_PER_MM);
    gap.set_widget_name("columns-gap");
    let separator = gtk::Switch::new();
    separator.set_active(initial.separator);
    separator.set_valign(gtk::Align::Center);
    let row = adw::ActionRow::new();
    row.set_title(wordcraft_gnome::t!("Line Between Columns"));
    row.set_activatable_widget(Some(&separator));
    row.add_suffix(&separator);
    group.add(&row);
    let format_note = gtk::Label::new(Some(wordcraft_gnome::t!(
        "Unequal widths are preserved in DOCX. In ODT, the engine saves them as equal-width columns."
    )));
    format_note.set_wrap(true);
    format_note.set_xalign(0.0);
    format_note.add_css_class("dim-label");
    format_note.set_margin_start(12);
    format_note.set_margin_end(12);
    group.add(&format_note);
    let custom = adw::PreferencesGroup::new();
    custom.set_title(wordcraft_gnome::t!("Custom Widths"));
    custom.set_description(Some(wordcraft_gnome::t!(
        "Width and following spacing in millimeters. The last column has no following spacing."
    )));
    page.append(&custom);
    let mut widths = Vec::new();
    let mut rows = Vec::new();
    for i in 0..12 {
        let (w, g) = initial.widths.get(i).copied().unwrap_or((
            (text_width - initial.space * (initial.count.saturating_sub(1)) as f32)
                / initial.count.max(1) as f32,
            initial.space,
        ));
        let row = adw::ActionRow::new();
        row.set_title(&format!("Columna {}", i + 1));
        let width = mm_input(w as f64 / PT_PER_MM, 18.0 / PT_PER_MM, 1584.0 / PT_PER_MM);
        width.set_width_chars(5);
        width.set_tooltip_text(Some(wordcraft_gnome::t!("Width (mm)")));
        width.update_property(&[gtk::accessible::Property::Label(&format!(
            "Anchura de columna {} (mm)",
            i + 1
        ))]);
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
    error.set_wrap(true);
    error.add_css_class("error");
    error.set_visible(false);
    let errors = adw::PreferencesGroup::new();
    errors.add(&error);
    page.append(&errors);
    page.append(&preview);
    let scope_header = scopes_group.clone();
    let controls = Rc::new(ColumnControls {
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
        text_width,
        last_count: Cell::new(initial.count.clamp(1, 12)),
        last_mode: Cell::new(if initial.kind == ColumnKind::Custom {
            3
        } else {
            0
        }),
    });
    {
        let weak = Rc::downgrade(&controls);
        controls.mode.connect_selected_notify(move |_| {
            if let Some(c) = weak.upgrade() {
                c.refresh();
            }
        });
    }
    {
        let weak = Rc::downgrade(&controls);
        controls.count.connect_value_changed(move |_| {
            if let Some(c) = weak.upgrade() {
                c.refresh();
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
        controls.separator.connect_active_notify(move |_| {
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
            let Ok(cols) = c.setup().resolved(c.text_width) else {
                return;
            };
            let color = area.color();
            let total = c.text_width as f64;
            let height = h as f64 - 24.;
            let scale = ((w as f64 - 28.) / total).min(height / (total * 1.414));
            let frame_width = total * scale;
            let margin = (w as f64 - frame_width) / 2.;
            let y = 12.;
            cr.set_source_rgba(
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
                0.15,
            );
            cr.rectangle(margin, y, total * scale, height);
            let _ = cr.stroke();
            let mut x = margin;
            for i in 0..cols.count as usize {
                let (cw, g) = cols.widths.get(i).copied().unwrap_or((
                    (c.text_width - cols.space * (cols.count - 1) as f32) / cols.count as f32,
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
                        y + 8.0 + line as f64 * 8.0,
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
        apply.connect_clicked(move |_| {
            let (Some(c), Some(ui)) = (weak.upgrade(), ui.upgrade()) else {
                return;
            };
            if ui.closed.get() || document_identity(&ui) != identity.get() {
                error.set_text(wordcraft_gnome::t!(
                    "The document has changed. Close and reopen Columns."
                ));
                error.set_visible(true);
                return;
            }
            c.count.update();
            c.gap.update();
            for (w, g) in &c.widths {
                w.update();
                g.update();
            }
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
                    on_applied();
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
    presets.set_max_children_per_line(5);
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
        b.set_child(Some(&sample));
        presets.insert(&b, -1);
        choices.push((b.clone(), count, mode));
        let weak = Rc::downgrade(&controls);
        let commit = apply.downgrade();
        b.connect_clicked(move |_| {
            if let (Some(c), Some(apply)) = (weak.upgrade(), commit.upgrade()) {
                c.count.set_value(count as f64);
                c.mode.set_selected(mode);
                apply.emit_clicked();
            }
        });
    }
    let matching = gtk::Label::new(None);
    matching.add_css_class("dim-label");
    matching.set_widget_name("columns-preset-state");
    let update: Rc<dyn Fn()> = {
        let weak = Rc::downgrade(&controls);
        let matching = matching.clone();
        Rc::new(move || {
            if let Some(c) = weak.upgrade() {
                let setup = c.setup();
                let mut found = false;
                for (b, count, mode) in &choices {
                    let on = setup.count == *count as u32 && c.mode.selected() == *mode;
                    if on {
                        b.add_css_class("current-style");
                        found = true;
                    } else {
                        b.remove_css_class("current-style");
                    }
                    b.update_state(&[gtk::accessible::State::Pressed(if on {
                        gtk::AccessibleTristate::True
                    } else {
                        gtk::AccessibleTristate::False
                    })]);
                }
                matching.set_text(if found {
                    ""
                } else {
                    wordcraft_gnome::t!("Custom")
                });
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
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    header.pack_start(&cancel);
    header.pack_end(&editor.apply);
    toolbar.add_top_bar(&header);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_child(Some(&editor.root));
    toolbar.set_content(Some(&scroll));
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
        apply: editor.apply,
    }
}
