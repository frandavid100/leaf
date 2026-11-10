use super::*;

// Native table controls. Document changes remain upstream commands.
fn use_table_target(s: &mut Session) -> Result<(), String> {
    if wordcraft_gnome::tables::in_table(s) {
        return Ok(());
    }
    let target = wordcraft_gnome::captions::table_target(s)
        .ok_or(wordcraft_gnome::t!("Place the caret in a table."))?;
    let wordcraft_gnome::captions::Target::Table { path, .. } = target else {
        return Err(wordcraft_gnome::t!("Place the caret in a table.").into());
    };
    let element = wordcraft_gnome::navigation::elements(&s.doc)
        .into_iter()
        .find(|e| e.table.as_ref() == Some(&path))
        .ok_or(wordcraft_gnome::t!("Place the caret in a table."))?;
    wordcraft_gnome::navigation::activate_element(s, &element)
}
pub(super) fn table_command(ui: &Rc<Ui>, command: &str, args: serde_json::Value) {
    tools_change(ui, |s| {
        // Creation remains available without an existing table. Only commands
        // that operate on an existing table resolve a caption's owner first.
        if command.starts_with("table.") {
            use_table_target(s)?;
        }
        s.run(command, &args).map_err(|e| e.to_string())?;
        Ok(())
    });
    if !ui.format_group.pinned.get() {
        caret_ensure_visible(ui);
        ui.document_box.grab_focus();
    }
    if let Some(menu) = find_widget(&ui.format_toolbar, "table-menu")
        .and_then(|w| w.downcast::<gtk::MenuButton>().ok())
    {
        if let Some(pop) = menu.popover() {
            refresh_table_controls(ui, &pop);
        }
    }
}

pub(super) fn table_navigate(ui: &Rc<Ui>, backwards: bool, literal: bool) {
    let result = {
        let mut state = ui.current.borrow_mut();
        state
            .as_mut()
            .map(|d| wordcraft_gnome::tables::navigate(&mut d.session, backwards, literal))
    };
    match result {
        Some(Ok(true)) => after_content_change(ui),
        Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
        _ => refresh_selection(ui),
    }
    caret_ensure_visible(ui);
}

pub(super) fn show_insert_table(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let w = preferences_dialog(wordcraft_gnome::t!("Insert Table"));
    w.set_content_width(440);
    w.set_content_height(640);
    w.set_search_enabled(false);
    let p = panel_page(&w, wordcraft_gnome::t!("Table"));
    let g = panel_group(&p, wordcraft_gnome::t!("Table Size"));
    let grid = gtk::Grid::new();
    grid.set_widget_name("insert-table-grid");
    grid.set_row_spacing(3);
    grid.set_column_spacing(3);
    grid.set_halign(gtk::Align::Center);
    let label = gtk::Label::new(Some(wordcraft_gnome::t!("Choose Columns × Rows")));
    g.add(&label);
    for row in 1..=5 {
        for col in 1..=6 {
            let b = gtk::Button::with_label("□");
            b.set_tooltip_text(Some(&format!("{col} columnas × {row} filas")));
            accessible_name(
                &b,
                &wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Insert a table with {col} columns and {row} rows"),
                    &[("col", format!("{}", col)), ("row", format!("{}", row))],
                ),
            );
            b.set_size_request(30, 28);
            grid.attach(&b, col - 1, row - 1, 1, 1);
            let motion = gtk::EventControllerMotion::new();
            let l = label.clone();
            motion
                .connect_enter(move |_, _, _| l.set_text(&format!("{col} columnas × {row} filas")));
            b.add_controller(motion);
            let ui = ui.clone();
            let win = w.clone();
            let valid = panel_target(&ui, &win);
            b.connect_clicked(glib::clone!(
                #[weak]
                ui,
                #[weak]
                win,
                move |_| {
                    if !valid() {
                        return;
                    }
                    if let Err(e) = format_execute(
                        &ui,
                        "insert.table",
                        serde_json::json!({"rows":row,"cols":col}),
                    ) {
                        win.add_toast(Toast::new(&e));
                    } else {
                        win.close();
                        ui.document_box.grab_focus();
                        caret_ensure_visible(&ui);
                    }
                }
            ));
        }
    }
    g.add(&grid);
    let g = panel_group(&p, wordcraft_gnome::t!("Custom Size"));
    let rows = panel_number(&g, wordcraft_gnome::t!("Rows"), 3.0, 1.0, 1000.0);
    rows.set_digits(0);
    rows.set_increments(1.0, 5.0);
    let cols = panel_number(&g, wordcraft_gnome::t!("Columns"), 3.0, 1.0, 63.0);
    cols.set_digits(0);
    cols.set_increments(1.0, 5.0);
    let b = gtk::Button::with_label(wordcraft_gnome::t!("Insert Table"));
    b.add_css_class("suggested-action");
    g.add(&b);
    let ui = ui.clone();
    let win = w.clone();
    let valid = panel_target(&ui, &win);
    b.connect_clicked(glib::clone!(
        #[weak]
        ui,
        #[weak]
        win,
        move |_| {
            if !valid() {
                return;
            }
            rows.update();
            cols.update();
            if let Err(e) = format_execute(
                &ui,
                "insert.table",
                serde_json::json!({"rows":rows.value_as_int(),"cols":cols.value_as_int()}),
            ) {
                win.add_toast(Toast::new(&e));
            } else {
                win.close();
                ui.document_box.grab_focus();
                caret_ensure_visible(&ui);
            }
        }
    ));
    w.present(Some(parent));
}

pub(super) fn table_property_toggle(
    g: &adw::PreferencesGroup,
    label: &str,
    on: bool,
    key: &'static str,
    ui: &Rc<Ui>,
    w: &adw::PreferencesDialog,
) {
    let row = adw::SwitchRow::new();
    row.set_widget_name(&format!("tool-control-table.properties-{key}"));
    row.set_title(label);
    row.set_active(on);
    g.add(&row);
    let ui = ui.clone();
    let w = w.clone();
    let valid = panel_target(&ui, &w);
    row.connect_active_notify(glib::clone!(
        #[weak]
        ui,
        #[weak]
        w,
        move |r| {
            if !valid() {
                return;
            }
            if let Err(e) = format_execute(
                &ui,
                "table.properties",
                serde_json::json!({key:r.is_active()}),
            ) {
                w.add_toast(Toast::new(&e));
            }
        }
    ));
}

pub(super) fn table_measurement(
    g: &adw::PreferencesGroup,
    label: &str,
    value: f64,
    min: f64,
    ui: &Rc<Ui>,
    w: &adw::PreferencesDialog,
    command: &'static str,
    key: &'static str,
    extra: serde_json::Value,
) {
    let row = adw::ActionRow::new();
    row.set_title(label);
    let n = gtk::SpinButton::with_range(min, 1584.0, 0.5);
    n.set_widget_name(&format!("tool-control-{command}-{key}"));
    n.set_digits(1);
    n.set_value(value);
    n.set_valign(gtk::Align::Center);
    accessible_name(&n, label);
    row.set_activatable_widget(Some(&n));
    row.add_suffix(&n);
    g.add(&row);
    let weak = Rc::downgrade(ui);
    let weak_window = w.downgrade();
    let valid = panel_target(ui, w);
    bind_numeric(ui, &n, move |value| {
        if !valid() {
            return Err(
                wordcraft_gnome::t!("The document has changed; open this dialog again.").into(),
            );
        }
        let ui = weak.upgrade().ok_or("Document closed")?;
        let mut args = extra.clone();
        args[key] = serde_json::json!(value);
        let result = immediate_format(&ui, command, args);
        if let Err(e) = &result {
            if let Some(w) = weak_window.upgrade() {
                w.add_toast(Toast::new(e));
            }
        }
        result
    });
}

pub(super) fn show_table_properties(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
) -> adw::PreferencesDialog {
    let state = {
        let mut state = ui.current.borrow_mut();
        let Some(doc) = state.as_mut() else {
            return preferences_dialog(wordcraft_gnome::t!("Table Properties"));
        };
        if let Err(e) = use_table_target(&mut doc.session) {
            ui.toasts.add_toast(Toast::new(&e));
            return preferences_dialog(wordcraft_gnome::t!("Table Properties"));
        }
        match doc.session.run("table.properties", &serde_json::json!({})) {
            Ok(v) => v,
            Err(e) => {
                ui.toasts.add_toast(Toast::new(&e.to_string()));
                return preferences_dialog(wordcraft_gnome::t!("Table Properties"));
            }
        }
    };
    let w = preferences_dialog(wordcraft_gnome::t!("Table Properties"));
    w.set_search_enabled(false);
    let p = panel_page(&w, wordcraft_gnome::t!("Size"));
    let g = panel_group(&p, wordcraft_gnome::t!("Table"));
    panel_choice(
        &g,
        wordcraft_gnome::t!("Table Alignment"),
        &[
            (wordcraft_gnome::t!("Left"), "left"),
            (wordcraft_gnome::t!("Centered"), "center"),
            (wordcraft_gnome::t!("Right"), "right"),
        ],
        ui,
        &w,
        "table.properties",
        "align",
    );
    panel_choice(
        &g,
        wordcraft_gnome::t!("Fit Width"),
        &[
            (wordcraft_gnome::t!("Fit Contents"), "contents"),
            (wordcraft_gnome::t!("Page Width"), "window"),
            (wordcraft_gnome::t!("Fixed Width"), "fixed"),
        ],
        ui,
        &w,
        "table.autofit",
        "mode",
    );
    panel_action(
        &g,
        wordcraft_gnome::t!("Distribute Columns Evenly"),
        ui,
        &w,
        "table.distributeColumns",
        serde_json::json!({}),
    );
    panel_action(
        &g,
        wordcraft_gnome::t!("Distribute Rows Evenly"),
        ui,
        &w,
        "table.distributeRows",
        serde_json::json!({}),
    );
    let g = panel_group(&p, wordcraft_gnome::t!("Selected Column and Row (Points)"));
    table_measurement(
        &g,
        wordcraft_gnome::t!("Column Width"),
        state["columnWidth"].as_f64().unwrap_or(72.0),
        6.0,
        ui,
        &w,
        "table.columnWidth",
        "width",
        serde_json::json!({}),
    );
    table_measurement(
        &g,
        wordcraft_gnome::t!("Minimum Row Height"),
        state["row"]["height"].as_f64().unwrap_or(18.0),
        1.0,
        ui,
        &w,
        "table.rowHeight",
        "height",
        serde_json::json!({"rule":"atLeast"}),
    );
    panel_action(
        &g,
        wordcraft_gnome::t!("Restore Automatic Height for Current Row"),
        ui,
        &w,
        "table.properties",
        serde_json::json!({"rowHeight":null}),
    );
    let g = panel_group(&p, wordcraft_gnome::t!("Current Row Pagination"));
    table_property_toggle(
        &g,
        wordcraft_gnome::t!("Repeat as Header"),
        state["row"]["header"].as_bool().unwrap_or(false),
        "headerRow",
        ui,
        &w,
    );
    table_property_toggle(
        &g,
        wordcraft_gnome::t!("Allow Row to Split Across Pages"),
        !state["row"]["cantSplit"].as_bool().unwrap_or(false),
        "allowBreak",
        ui,
        &w,
    );
    let p = panel_page(&w, wordcraft_gnome::t!("Cells"));
    let g = panel_group(&p, wordcraft_gnome::t!("Content Alignment"));
    panel_choice(
        &g,
        wordcraft_gnome::t!("Horizontal and Vertical Alignment"),
        &[
            (wordcraft_gnome::t!("Top · Left"), "topLeft"),
            (wordcraft_gnome::t!("Top · Center"), "topCenter"),
            (wordcraft_gnome::t!("Top · Right"), "topRight"),
            (wordcraft_gnome::t!("Center · Left"), "centerLeft"),
            (wordcraft_gnome::t!("Center"), "center"),
            (wordcraft_gnome::t!("Center · Right"), "centerRight"),
            (wordcraft_gnome::t!("Bottom · Left"), "bottomLeft"),
            (wordcraft_gnome::t!("Bottom · Center"), "bottomCenter"),
            (wordcraft_gnome::t!("Bottom · Right"), "bottomRight"),
        ],
        ui,
        &w,
        "table.cellAlign",
        "value",
    );
    panel_choice(
        &g,
        wordcraft_gnome::t!("Text Direction"),
        &[
            (wordcraft_gnome::t!("Horizontal"), "horizontal"),
            (wordcraft_gnome::t!("Vertical Downwards"), "down"),
            (wordcraft_gnome::t!("Vertical Upwards"), "up"),
        ],
        ui,
        &w,
        "table.textDirection",
        "value",
    );
    let p = panel_page(&w, wordcraft_gnome::t!("Appearance"));
    let g = panel_group(&p, wordcraft_gnome::t!("Selection Borders and Shading"));
    panel_color(
        &g,
        wordcraft_gnome::t!("Background Color"),
        ui,
        &w,
        "table.shading",
    );
    panel_action(
        &g,
        wordcraft_gnome::t!("No Background Color"),
        ui,
        &w,
        "table.shading",
        serde_json::json!({"color":null}),
    );
    panel_choice(
        &g,
        wordcraft_gnome::t!("Borders"),
        &[
            (wordcraft_gnome::t!("All"), "all"),
            (wordcraft_gnome::t!("Outside"), "outside"),
            (wordcraft_gnome::t!("Inside"), "inside"),
            (wordcraft_gnome::t!("None"), "none"),
            (wordcraft_gnome::t!("Top"), "top"),
            (wordcraft_gnome::t!("Bottom"), "bottom"),
            (wordcraft_gnome::t!("Left"), "left"),
            (wordcraft_gnome::t!("Right"), "right"),
        ],
        ui,
        &w,
        "table.borders",
        "kind",
    );
    let width = panel_number(
        &g,
        wordcraft_gnome::t!("Border Width (pt)"),
        0.5,
        0.25,
        12.0,
    );
    let color = panel_entry(&g, wordcraft_gnome::t!("Border Color (RRGGBB)"), "808080");
    let border_entry = color.downgrade();
    let picker = embedded_color_chooser("element-border-chooser", Some("#808080"), move |hex| {
        if let Some(entry) = border_entry.upgrade() {
            entry.set_text(hex);
            true
        } else {
            false
        }
    });
    g.add(&picker.widget);
    let b = gtk::Button::with_label(wordcraft_gnome::t!("Apply to All Borders"));
    g.add(&b);
    {
        let ui = ui.clone();
        let w = w.clone();
        let valid = panel_target(&ui, &w);
        b.connect_clicked(glib::clone!(
            #[weak]
            ui,
            #[weak]
            w,
            move |_| {
                if !valid() {
                    return;
                }
                width.update();
                let c = color.text().trim().to_string();
                if c.len() != 6 || !c.bytes().all(|b| b.is_ascii_hexdigit()) {
                    w.add_toast(Toast::new(wordcraft_gnome::t!(
                        "Enter Six Hexadecimal Characters, for Example 808080"
                    )));
                    return;
                }
                if let Err(e) = format_execute(
                    &ui,
                    "table.borders",
                    serde_json::json!({"kind":"all","width":width.value(),"color":c}),
                ) {
                    w.add_toast(Toast::new(&e));
                }
            }
        ));
    }
    w.present(Some(parent));
    w
}

pub(super) fn show_split_cells(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let w = preferences_dialog(wordcraft_gnome::t!("Split Cells"));
    w.set_search_enabled(false);
    let p = panel_page(&w, wordcraft_gnome::t!("Split Cells"));
    let g = panel_group(&p, wordcraft_gnome::t!("Split Cells"));
    let cols = panel_number(&g, wordcraft_gnome::t!("Columns"), 2.0, 1.0, 63.0);
    cols.set_digits(0);
    cols.set_increments(1.0, 1.0);
    let rows = panel_number(&g, wordcraft_gnome::t!("Rows"), 1.0, 1.0, 63.0);
    rows.set_digits(0);
    rows.set_increments(1.0, 1.0);
    let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
        "Multiple selected cells are merged before splitting."
    )));
    hint.set_wrap(true);
    hint.add_css_class("dim-label");
    g.add(&hint);
    let b = gtk::Button::with_label(wordcraft_gnome::t!("Split Selected Cells"));
    g.add(&b);
    {
        let ui = ui.clone();
        let w = w.clone();
        let valid = panel_target(&ui, &w);
        b.connect_clicked(glib::clone!(
            #[weak]
            ui,
            #[weak]
            w,
            move |_| {
                if !valid() {
                    return;
                }
                cols.update();
                rows.update();
                if let Err(e) = format_execute(
                    &ui,
                    "table.split",
                    serde_json::json!({"columns":cols.value_as_int(),"rows":rows.value_as_int()}),
                ) {
                    w.add_toast(Toast::new(&e));
                }
            }
        ));
    }
    w.present(Some(parent));
}

fn table_submenu(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    outer: &gtk::Popover,
    title: &str,
    name: &str,
    items: &[(&str, &'static str)],
) -> gtk::MenuButton {
    let menu = gtk::MenuButton::new();
    menu.set_label(title);
    menu.add_css_class("flat");
    menu.set_widget_name(name);
    menu.set_hexpand(true);
    menu.set_direction(gtk::ArrowType::Right);
    accessible_name(&menu, title);
    let pop = gtk::Popover::new();
    let body = gtk::Box::new(gtk::Orientation::Vertical, 3);
    for &(label, command) in items {
        let b = gtk::Button::with_label(label);
        b.add_css_class("flat");
        b.set_widget_name(&format!("table-action-{command}"));
        let weak = Rc::downgrade(ui);
        let outer = outer.downgrade();
        let inner = pop.downgrade();
        let parent = parent.downgrade();
        b.connect_clicked(move |_| {
            let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                return;
            };
            if let Some(p) = inner.upgrade() {
                p.popdown();
            }
            if let Some(p) = outer.upgrade() {
                close_format_popover(&p);
            }
            if command == "leaf.splitCells" {
                if tools_change(&ui, use_table_target) {
                    if let Some(parent) = parent.upgrade() {
                        show_split_cells(&ui, &parent);
                    }
                }
            } else {
                table_command(&ui, command, serde_json::json!({}));
            }
        });
        body.append(&b);
    }
    pop.set_child(Some(&body));
    menu.set_popover(Some(&pop));
    menu
}

pub(super) fn install_table_controls(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    bar: &gtk::Box,
) -> gtk::MenuButton {
    let menu = gtk::MenuButton::new();
    menu.set_widget_name("table-menu");
    format_menu_label(&menu, wordcraft_gnome::t!("Table"));
    menu.set_direction(gtk::ArrowType::Up);
    menu.set_tooltip_text(Some(wordcraft_gnome::t!("Insert and Edit Tables")));
    let pop = gtk::Popover::new();
    pop.add_css_class("format-popover");
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    format_popover_header(
        &pop,
        &content,
        wordcraft_gnome::t!("Table"),
        &ui.format_group,
    );
    let insert = gtk::Button::with_label(wordcraft_gnome::t!("Insert Table…"));
    insert.add_css_class("flat");
    insert.set_widget_name("table-insert");
    // Stable creation stays below the existing-object properties.
    {
        let ui = ui.clone();
        let p = parent.clone();
        let pop = pop.clone();
        insert.connect_clicked(glib::clone!(
            #[weak]
            ui,
            #[weak]
            p,
            #[weak]
            pop,
            move |_| {
                close_format_popover(&pop);
                show_insert_table(&ui, &p);
            }
        ));
    }
    let context = gtk::Box::new(gtk::Orientation::Vertical, 6);
    context.set_widget_name("table-context");
    content.append(&context);
    let heading = gtk::Label::new(Some(wordcraft_gnome::t!("Current Table")));
    heading.add_css_class("heading");
    context.append(&heading);
    let caption = gtk::CheckButton::with_label(wordcraft_gnome::t!("Show Caption"));
    caption.set_widget_name("table-show-caption");
    context.append(&caption);
    {
        let weak = Rc::downgrade(ui);
        let pop = pop.downgrade();
        caption.connect_toggled(move |button| {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            let show = button.is_active();
            let current = ui
                .current
                .borrow()
                .as_ref()
                .and_then(|d| {
                    wordcraft_gnome::captions::table_target(&d.session)
                        .and_then(|t| wordcraft_gnome::captions::for_target(&d.session.doc, &t))
                })
                .is_some_and(|c| c.visible);
            if current == show {
                return;
            }
            if tools_change(&ui, |s| {
                use_table_target(s)?;
                s.run("table.selectTable", &serde_json::json!({}))
                    .map_err(|e| e.to_string())?;
                wordcraft_gnome::captions::set_visible(s, show)
            }) {
                if let Some(pop) = pop.upgrade() {
                    close_format_popover(&pop);
                }
                ui.document_box.grab_focus();
                caret_ensure_visible(&ui);
            }
        });
    }
    context.remove(&caption);
    for (title, name, items) in [
        (
            wordcraft_gnome::t!("Select"),
            "table-select",
            vec![
                (wordcraft_gnome::t!("Cell"), "table.selectCell"),
                (wordcraft_gnome::t!("Row"), "table.selectRow"),
                (wordcraft_gnome::t!("Column"), "table.selectColumn"),
                (wordcraft_gnome::t!("Table"), "table.selectTable"),
            ],
        ),
        (
            wordcraft_gnome::t!("Insert"),
            "table-add",
            vec![
                (wordcraft_gnome::t!("Row Above"), "table.insertRowAbove"),
                (wordcraft_gnome::t!("Row Below"), "table.insertRowBelow"),
                (
                    wordcraft_gnome::t!("Column to the Left"),
                    "table.insertColumnLeft",
                ),
                (
                    wordcraft_gnome::t!("Column to the Right"),
                    "table.insertColumnRight",
                ),
            ],
        ),
        (
            wordcraft_gnome::t!("Delete"),
            "table-delete",
            vec![
                (wordcraft_gnome::t!("Row"), "table.deleteRow"),
                (wordcraft_gnome::t!("Column"), "table.deleteColumn"),
                (wordcraft_gnome::t!("Table"), "table.deleteTable"),
            ],
        ),
    ] {
        context.append(&table_submenu(ui, parent, &pop, title, name, &items));
    }
    let merge = gtk::Button::with_label(wordcraft_gnome::t!("Merge Cells"));
    merge.set_widget_name("table-merge");
    merge.add_css_class("flat");
    let weak = Rc::downgrade(ui);
    let outer = pop.downgrade();
    merge.connect_clicked(move |_| {
        if let Some(ui) = weak.upgrade() {
            if let Some(p) = outer.upgrade() {
                close_format_popover(&p);
            }
            table_command(&ui, "table.merge", serde_json::json!({}));
        }
    });
    context.append(&merge);
    context.append(&table_submenu(
        ui,
        parent,
        &pop,
        wordcraft_gnome::t!("Split"),
        "table-split",
        &[
            (wordcraft_gnome::t!("Split Cells"), "leaf.splitCells"),
            (wordcraft_gnome::t!("Split Table"), "table.splitTable"),
        ],
    ));
    let properties = gtk::Button::with_label(wordcraft_gnome::t!("Table Properties"));
    properties.set_widget_name("table-properties");
    properties.add_css_class("flat");
    context.append(&properties);
    {
        let ui = ui.clone();
        let parent = parent.clone();
        let pop = pop.clone();
        properties.connect_clicked(glib::clone!(
            #[weak]
            ui,
            #[weak]
            parent,
            #[weak]
            pop,
            move |_| {
                close_format_popover(&pop);
                show_table_properties(&ui, &parent);
            }
        ));
    }
    context.append(&caption);
    {
        let ui = ui.clone();
        pop.connect_show(glib::clone!(
            #[weak]
            ui,
            move |p| refresh_table_controls(&ui, p)
        ));
    }
    for id in [
        "table-select",
        "table-add",
        "table-delete",
        "table-merge",
        "table-split",
        "table-properties",
        "table-caption",
    ] {
        let name = if id == "table-caption" {
            "table-show-caption"
        } else {
            id
        };
        if let Some(widget) = find_widget(&context, name) {
            let prev = widget.prev_sibling();
            context.remove(&widget);
            let slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
            slot.set_widget_name(&format!("format-row-elements-{id}"));
            slot.append(&widget);
            context.insert_child_after(&slot, prev.as_ref());
        }
    }
    content.remove(&context);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    body.append(&context);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(440);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&body));
    content.append(&scroll);
    content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    content.append(&insert);
    pop.set_child(Some(&content));
    menu.set_popover(Some(&pop));
    bar.append(&menu);
    menu
}

pub(super) fn refresh_table_controls(ui: &Rc<Ui>, pop: &gtk::Popover) {
    let root = pop
        .child()
        .or_else(|| ui.elements.borrow().as_ref().and_then(|e| e.body_for(pop)));
    let Some(root) = root else {
        return;
    };
    let value = ui.current.borrow();
    let doc = value.as_ref();
    if let Some(w) = find_widget(&root, "table-context") {
        let targeted = doc.is_some_and(|d| {
            wordcraft_gnome::tables::in_table(&d.session)
                || wordcraft_gnome::captions::table_target(&d.session).is_some()
        });
        w.set_sensitive(targeted);
        root.set_tooltip_text(
            (!targeted).then_some(wordcraft_gnome::t!("Place the caret in a table.")),
        );
    }
    if let Some(w) = find_widget(&root, "table-merge") {
        w.set_sensitive(
            doc.is_some_and(|d| wordcraft_gnome::tables::selected_cells(&d.session) > 1),
        );
    }
    if let Some(button) =
        find_widget(&root, "table-show-caption").and_then(|w| w.downcast::<gtk::CheckButton>().ok())
    {
        let shown = doc
            .and_then(|d| {
                wordcraft_gnome::captions::table_target(&d.session)
                    .and_then(|t| wordcraft_gnome::captions::for_target(&d.session.doc, &t))
            })
            .is_some_and(|c| c.visible);
        let body =
            doc.is_some_and(|d| wordcraft_gnome::captions::table_target(&d.session).is_some());
        button.set_sensitive(
            body && doc.is_some_and(|d| !wordcraft_engine::cmd::protect::enforced(&d.session.doc)),
        );
        button.set_tooltip_text((!body).then_some(wordcraft_gnome::t!(
            "Captions are available in the document body."
        )));
        button.set_active(shown);
    }
}
