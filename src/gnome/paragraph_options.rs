//! Focused paragraph editors. Read-only construction; public Session commands own edits.
use super::*;
use serde_json::{Value, json};

fn heading(s: &ParagraphPopover, text: &str) {
    let l = gtk::Label::new(Some(text));
    l.set_xalign(0.);
    l.add_css_class("heading");
    s.detail.append(&l);
}
fn note(s: &ParagraphPopover, text: &str) {
    let l = gtk::Label::new(Some(text));
    l.set_wrap(true);
    l.set_max_width_chars(34);
    l.set_xalign(0.);
    l.add_css_class("dim-label");
    s.detail.append(&l);
}
fn selected(button: &gtk::Button, value: Option<bool>) {
    button.update_state(&[gtk::accessible::State::Pressed(match value {
        Some(true) => gtk::AccessibleTristate::True,
        Some(false) => gtk::AccessibleTristate::False,
        None => gtk::AccessibleTristate::Mixed,
    })]);
    if value == Some(true) {
        button.add_css_class("current-style");
    }
}
fn preview(s: &ParagraphPopover, state: &Value, indent: bool) {
    let d = gtk::DrawingArea::new();
    d.set_content_width(235);
    d.set_content_height(95);
    let state = state.clone();
    d.set_draw_func(move |w, c, width, _| {
        let fg = w.color();
        c.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 0.65);
        c.set_line_width(1.5);
        let scale = (width as f64 - 24.) / 450.;
        let left = state["indentLeft"].as_f64().unwrap_or(0.) * scale;
        let right = state["indentRight"].as_f64().unwrap_or(0.) * scale;
        let first = state["indentFirst"].as_f64().unwrap_or(0.) * scale;
        let line = if indent {
            11.
        } else {
            match state["lineSpacing"]["rule"].as_str() {
                Some("multiple") => 11. * state["lineSpacing"]["value"].as_f64().unwrap_or(1.),
                Some("exactly" | "atLeast") => {
                    state["lineSpacing"]["value"].as_f64().unwrap_or(14.) * 0.6
                }
                _ => 11.,
            }
            .clamp(5., 20.)
        };
        let gap = if indent {
            7.
        } else {
            state["spaceAfter"]
                .as_f64()
                .unwrap_or(0.)
                .max(state["spaceBefore"].as_f64().unwrap_or(0.))
                * 0.5
                + 5.
        };
        let mut y = 10.;
        for _ in 0..2 {
            for i in 0..3 {
                let x = 12.
                    + if indent {
                        left + if i == 0 { first } else { 0. }
                    } else {
                        0.
                    };
                let end = width as f64
                    - 12.
                    - if indent { right } else { 0. }
                    - if i == 2 { 25. } else { 0. };
                c.move_to(x.clamp(3., width as f64 - 20.), y);
                c.line_to(end.max(x + 5.), y);
                y += line;
            }
            y += gap;
        }
        let _ = c.stroke();
    });
    s.detail.append(&d);
}
fn number(
    s: &Rc<ParagraphPopover>,
    ui: &Rc<Ui>,
    label: &str,
    name: &str,
    value: Option<f64>,
    min: f64,
    max: f64,
    command: &'static str,
    key: &'static str,
    factor: f64,
) -> gtk::SpinButton {
    let n = s.number(
        label,
        value.unwrap_or(0.) / factor,
        min / factor,
        max / factor,
    );
    n.set_digits(3);
    n.set_width_chars(6);
    n.set_widget_name(name);
    if value.is_none() {
        n.set_text("");
        n.set_tooltip_text(Some(wordcraft_gnome::t!("Mixed")));
    }
    let weak = Rc::downgrade(ui);
    bind_numeric(ui, &n, move |v| {
        let ui = weak.upgrade().ok_or("Document closed")?;
        immediate_format(&ui, command, json!({key:v*factor}))
    });
    n
}
fn page_link(s: &Rc<ParagraphPopover>, title: &str, page: &'static str) {
    let b = style_menu_button(title);
    b.set_widget_name(&format!("format-row-paragraph-{page}"));
    s.detail.append(&b);
    let weak = Rc::downgrade(s);
    b.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            s.page(page);
        }
    });
}

pub(super) fn build(s: &Rc<ParagraphPopover>, ui: &Rc<Ui>, page: &str, state: &Value) -> bool {
    match page {
        "alignment" => {
            if state["align"].is_null() {
                note(s, wordcraft_gnome::t!("Mixed"));
            }
            for (title, value, icon, key) in [
                (
                    wordcraft_gnome::t!("Align Left"),
                    "left",
                    "format-justify-left-symbolic",
                    "Ctrl+L",
                ),
                (
                    wordcraft_gnome::t!("Center"),
                    "center",
                    "format-justify-center-symbolic",
                    "Ctrl+E",
                ),
                (
                    wordcraft_gnome::t!("Align Right"),
                    "right",
                    "format-justify-right-symbolic",
                    "Ctrl+R",
                ),
                (
                    wordcraft_gnome::t!("Justify"),
                    "justify",
                    "format-justify-fill-symbolic",
                    "Ctrl+J",
                ),
                (
                    wordcraft_gnome::t!("Distributed Alignment"),
                    "distribute",
                    "format-justify-distribute-symbolic",
                    "Ctrl+Shift+J",
                ),
            ] {
                let b = s.action(
                    title,
                    &format!("paragraph-align-{value}"),
                    "para.align",
                    json!({"value":value}),
                );
                let row = list_choice_content(title, icon);
                let shortcut = gtk::Label::new(Some(key));
                shortcut.add_css_class("dim-label");
                row.append(&shortcut);
                let check = gtk::Label::new(Some(if state["align"] == value { "✓" } else { "" }));
                row.append(&check);
                b.set_child(Some(&row));
                selected(&b, state["align"].as_str().map(|a| a == value));
                if value == "justify" || value == "distribute" {
                    b.set_tooltip_text(Some(if value == "justify" {
                        wordcraft_gnome::t!(
                            "Justify lines; keep the final line at its natural width."
                        )
                    } else {
                        wordcraft_gnome::t!(
                            "Distribute text across the full line, including the final line."
                        )
                    }));
                }
                s.detail.append(&b);
            }
        }
        "direction" => {
            for (label, rtl, command, id) in [
                (
                    wordcraft_gnome::t!("Left to Right"),
                    false,
                    "para.ltr",
                    "paragraph-ltr",
                ),
                (
                    wordcraft_gnome::t!("Right to Left"),
                    true,
                    "para.rtl",
                    "paragraph-rtl",
                ),
            ] {
                let b = s.action(label, id, command, json!({"value":true}));
                selected(&b, state["bidi"].as_bool().map(|v| v == rtl));
                s.detail.append(&b);
            }
        }
        "indents" => {
            for (title, name, command, key) in [
                (
                    wordcraft_gnome::t!("Increase Indent"),
                    "paragraph-indent",
                    "para.indent",
                    "Ctrl+M",
                ),
                (
                    wordcraft_gnome::t!("Decrease Indent"),
                    "paragraph-outdent",
                    "para.outdent",
                    "Ctrl+Shift+M",
                ),
            ] {
                let b = s.action(title, name, command, json!({}));
                b.set_tooltip_text(Some(key));
                s.detail.append(&b);
            }
            // The current ruler is centimetres, without a separate unit preference.
            let cm = 72. / 2.54;
            number(
                s,
                ui,
                wordcraft_gnome::t!("Left (cm)"),
                "paragraph-indentLeft",
                state["indentLeft"].as_f64(),
                -1584.,
                1584.,
                "para.indents",
                "left",
                cm,
            );
            number(
                s,
                ui,
                wordcraft_gnome::t!("Right (cm)"),
                "paragraph-indentRight",
                state["indentRight"].as_f64(),
                -1584.,
                1584.,
                "para.indents",
                "right",
                cm,
            );
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let label = gtk::Label::new(Some(wordcraft_gnome::t!("Special")));
            label.set_hexpand(true);
            label.set_xalign(0.);
            row.append(&label);
            let mode = gtk::DropDown::from_strings(&[
                wordcraft_gnome::t!("None"),
                wordcraft_gnome::t!("First Line"),
                wordcraft_gnome::t!("Hanging"),
            ]);
            mode.set_widget_name("paragraph-special-indent");
            let first = state["indentFirst"].as_f64();
            mode.set_selected(first.map_or(gtk::INVALID_LIST_POSITION, |v| {
                if v == 0. {
                    0
                } else if v > 0. {
                    1
                } else {
                    2
                }
            }));
            row.append(&mode);
            s.detail.append(&row);
            let distance = s.number(
                wordcraft_gnome::t!("Distance (cm)"),
                first.unwrap_or(0.).abs() / cm,
                0.,
                1584. / cm,
            );
            distance.set_digits(3);
            distance.set_widget_name("paragraph-indentFirst");
            distance.set_sensitive(mode.selected() != 0);
            if first.is_none() {
                distance.set_text("");
            }
            let weak = Rc::downgrade(ui);
            let m = mode.clone();
            bind_numeric(ui, &distance, move |v| {
                let ui = weak.upgrade().ok_or("Document closed")?;
                let sign = if m.selected() == 2 { -1. } else { 1. };
                immediate_format(
                    &ui,
                    "para.indents",
                    json!({"firstLine":if m.selected()==0 {0.}else{v*cm*sign}}),
                )
            });
            let weak = Rc::downgrade(ui);
            let n = distance.clone();
            mode.connect_selected_notify(move |m| {
                n.set_sensitive(m.selected() != 0);
                if let Some(ui) = weak.upgrade() {
                    if m.selected() == 0 {
                        let _ = format_execute(&ui, "para.indents", json!({"firstLine":0.}));
                    } else if let Ok(v) = n.text().replace(',', ".").parse::<f64>() {
                        let _ = format_execute(
                            &ui,
                            "para.indents",
                            json!({"firstLine":v*cm*if m.selected()==2 {-1.}else{1.}}),
                        );
                    }
                }
            });
            preview(s, state, true);
            s.detail.append(&s.action(
                wordcraft_gnome::t!("Remove Indents"),
                "paragraph-remove-indents",
                "para.indents",
                json!({"left":0.,"right":0.,"firstLine":0.}),
            ));
        }
        "spacing" => {
            heading(s, wordcraft_gnome::t!("Line Spacing"));
            for (value, label) in [
                (1., wordcraft_gnome::t!("Single")),
                (1.15, wordcraft_gnome::t!("1.15 Lines")),
                (1.5, wordcraft_gnome::t!("1.5 Lines")),
                (2., wordcraft_gnome::t!("Double")),
            ] {
                let b = s.action(
                    label,
                    &format!("paragraph-line-{value}"),
                    "para.lineSpacing",
                    json!({"value":value}),
                );
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let glyph = gtk::DrawingArea::new();
                glyph.set_content_width(24);
                glyph.set_content_height(30);
                glyph.set_draw_func(move |w, c, _, _| {
                    let fg = w.color();
                    c.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 1.);
                    c.set_line_width(1.2);
                    for i in 0..3 {
                        let y = 5. + i as f64 * 5. * value;
                        c.move_to(1., y);
                        c.line_to(23., y);
                    }
                    let _ = c.stroke();
                });
                row.append(&glyph);
                let l = gtk::Label::new(Some(label));
                l.set_hexpand(true);
                l.set_xalign(0.);
                row.append(&l);
                let on = state["lineSpacing"]["rule"] == "multiple"
                    && state["lineSpacing"]["value"]
                        .as_f64()
                        .is_some_and(|v| (v - value).abs() < 0.001);
                row.append(&gtk::Label::new(Some(if on { "✓" } else { "" })));
                b.set_child(Some(&row));
                selected(&b, (!state["lineSpacing"].is_null()).then_some(on));
                s.detail.append(&b);
            }
            heading(s, wordcraft_gnome::t!("Custom"));
            let mode = gtk::DropDown::from_strings(&[
                wordcraft_gnome::t!("Multiple"),
                wordcraft_gnome::t!("At Least (pt)"),
                wordcraft_gnome::t!("Exactly (pt)"),
            ]);
            mode.set_widget_name("paragraph-spacing-mode");
            mode.set_selected(match state["lineSpacing"]["rule"].as_str() {
                Some("multiple") => 0,
                Some("atLeast") => 1,
                Some("exactly") => 2,
                _ => gtk::INVALID_LIST_POSITION,
            });
            s.detail.append(&mode);
            let n = s.number(
                wordcraft_gnome::t!("Value"),
                state["lineSpacing"]["value"].as_f64().unwrap_or(1.),
                0.1,
                500.,
            );
            n.set_digits(2);
            n.set_widget_name("paragraph-exact-line-spacing");
            if state["lineSpacing"].is_null() {
                n.set_text("");
            }
            let weak = Rc::downgrade(ui);
            let m = mode.clone();
            bind_numeric(ui, &n, move |v| {
                let ui = weak.upgrade().ok_or("Document closed")?;
                let key = match m.selected() {
                    0 => "value",
                    1 => "atLeast",
                    2 => "exactly",
                    _ => return Ok(()),
                };
                immediate_format(&ui, "para.lineSpacing", json!({key:v}))
            });
            // Choosing a mode enables the corresponding unit; confirming its value applies.
            let n2 = n.clone();
            mode.connect_selected_notify(move |m| {
                n2.set_tooltip_text(Some(if m.selected() == 0 {
                    wordcraft_gnome::t!("Line height multiplier")
                } else {
                    wordcraft_gnome::t!("Distance in points")
                }));
            });
            heading(s, wordcraft_gnome::t!("Paragraph Spacing"));
            number(
                s,
                ui,
                wordcraft_gnome::t!("Before paragraph (pt)"),
                "paragraph-spaceBefore",
                state["spaceBefore"].as_f64(),
                0.,
                1584.,
                "para.spacing",
                "before",
                1.,
            );
            number(
                s,
                ui,
                wordcraft_gnome::t!("After paragraph (pt)"),
                "paragraph-spaceAfter",
                state["spaceAfter"].as_f64(),
                0.,
                1584.,
                "para.spacing",
                "after",
                1.,
            );
            preview(s, state, false);
            let help = gtk::Expander::new(Some(wordcraft_gnome::t!("About paragraph spacing")));
            let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
                "Between paragraphs, the larger adjoining Before or After value is used. Styles can suppress this spacing."
            )));
            hint.set_widget_name("paragraph-spacing-hint");
            hint.set_wrap(true);
            hint.set_max_width_chars(32);
            help.set_child(Some(&hint));
            s.detail.append(&help);
        }
        "sort" => {
            note(
                s,
                wordcraft_gnome::t!(
                    "Sort the selected paragraphs. Select at least two paragraphs."
                ),
            );
            let valid = ui.current.borrow().as_ref().is_some_and(|d| {
                wordcraft_gnome::text_properties::paragraph_positions(&d.session).len() > 1
            });
            for (title, id, descending) in [
                (
                    wordcraft_gnome::t!("Ascending (A–Z)"),
                    "paragraph-sort-asc",
                    false,
                ),
                (
                    wordcraft_gnome::t!("Descending (Z–A)"),
                    "paragraph-sort-desc",
                    true,
                ),
            ] {
                let b = s.action(title, id, "para.sort", json!({"descending":descending}));
                b.set_sensitive(valid);
                s.detail.append(&b);
            }
        }
        "tabs" => tabs(s, ui, state),
        "borders" => borders(s, ui, state),
        "border-color" => border_color(s, ui, state),
        "options" => {
            page_link(s, wordcraft_gnome::t!("Text Direction"), "direction");
            let marks = ui
                .current
                .borrow()
                .as_ref()
                .is_some_and(|d| d.session.view.marks);
            s.toggle(
                wordcraft_gnome::t!("Show Formatting Marks (¶)"),
                marks,
                "view.marks",
            );
            heading(s, wordcraft_gnome::t!("Pagination"));
            for (label, key, cmd) in [
                (
                    wordcraft_gnome::t!("Keep with Next"),
                    "keepNext",
                    "para.keepNext",
                ),
                (
                    wordcraft_gnome::t!("Keep Lines Together"),
                    "keepLines",
                    "para.keepLines",
                ),
                (
                    wordcraft_gnome::t!("Page Break Before"),
                    "pageBreakBefore",
                    "para.pageBreakBefore",
                ),
                (
                    wordcraft_gnome::t!("Widow and Orphan Control"),
                    "widowControl",
                    "para.widowControl",
                ),
            ] {
                s.toggle(label, state[key].as_bool().unwrap_or(false), cmd);
            }
            page_link(s, wordcraft_gnome::t!("Sort"), "sort");
            page_link(s, wordcraft_gnome::t!("Tab Stops"), "tabs");
            s.detail
                .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            let b = s.action(
                wordcraft_gnome::t!("Reset Paragraph Formatting"),
                "paragraph-reset",
                "para.reset",
                json!({}),
            );
            let outer = gtk::Box::new(gtk::Orientation::Vertical, 0);
            outer.set_widget_name("format-row-paragraph-reset");
            outer.append(&b);
            s.detail.append(&outer);
        }
        _ => return false,
    }
    true
}
fn tabs(s: &Rc<ParagraphPopover>, _ui: &Rc<Ui>, state: &Value) {
    note(
        s,
        wordcraft_gnome::t!(
            "Positions are measured from the left text indent, as on the ruler. Style tab stops are inherited; removing one adds an explicit override."
        ),
    );
    let cm = 72. / 2.54;
    let current = state["tabs"].as_array().cloned().unwrap_or_default();
    if state["tabs"].is_null() {
        note(s, wordcraft_gnome::t!("Mixed"));
    }
    let list = gtk::ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::Single);
    list.set_widget_name("paragraph-tab-list");
    s.detail.append(&list);
    for stop in &current {
        let l = gtk::Label::new(Some(&format!(
            "{:.3} cm · {} · {}",
            stop["pos"].as_f64().unwrap_or(0.) / cm,
            stop["align"].as_str().unwrap_or("left"),
            stop["leader"].as_str().unwrap_or("none")
        )));
        l.set_margin_top(5);
        l.set_margin_bottom(5);
        list.append(&l);
    }
    let pos = s.number(
        wordcraft_gnome::t!("Position (cm)"),
        1.27,
        -1584. / cm,
        1584. / cm,
    );
    pos.set_digits(3);
    pos.set_widget_name("paragraph-tab-position");
    let align = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("Align Left"),
        wordcraft_gnome::t!("Center"),
        wordcraft_gnome::t!("Align Right"),
        wordcraft_gnome::t!("Decimal"),
        wordcraft_gnome::t!("Bar"),
    ]);
    s.detail.append(&align);
    let leader = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("None"),
        wordcraft_gnome::t!("Dots"),
        wordcraft_gnome::t!("Dashes"),
        wordcraft_gnome::t!("Underline"),
    ]);
    s.detail.append(&leader);
    let selected_pos = Rc::new(Cell::new(None::<f64>));
    let old = selected_pos.clone();
    let p = pos.clone();
    let a = align.clone();
    let l = leader.clone();
    let stops = current.clone();
    list.connect_row_selected(move |_, row| {
        let Some(v) = row.and_then(|r| stops.get(r.index() as usize)) else {
            return;
        };
        let vpos = v["pos"].as_f64().unwrap_or(0.);
        old.set(Some(vpos));
        p.set_value(vpos / cm);
        a.set_selected(
            ["left", "center", "right", "decimal", "bar"]
                .iter()
                .position(|k| v["align"] == *k)
                .unwrap_or(0) as u32,
        );
        l.set_selected(
            ["none", "dot", "hyphen", "underscore"]
                .iter()
                .position(|k| v["leader"] == *k)
                .unwrap_or(0) as u32,
        );
    });
    let b = style_menu_button(wordcraft_gnome::t!("Add or Update Tab Stop"));
    b.set_widget_name("paragraph-tabs-set");
    s.detail.append(&b);
    let weak = Rc::downgrade(s);
    let old = selected_pos.clone();
    b.connect_clicked(move |_|{let Ok(v)=pos.text().replace(',',".").parse::<f64>() else{return};if !v.is_finite()||v.abs()>1584./cm{return}if let Some(s)=weak.upgrade(){let clear=old.get().filter(|o|(o-v*cm).abs()>=0.5).map(|v|vec![v]).unwrap_or_default();s.execute("para.tabs",json!({"clear":clear,"set":[{"pos":v*cm,"align":(["left","center","right","decimal","bar"][align.selected() as usize]),"leader":(["none","dot","hyphen","underscore"][leader.selected() as usize])}]}));}});
    let remove = style_menu_button(wordcraft_gnome::t!("Remove Selected Tab Stop"));
    remove.set_widget_name("paragraph-tabs-remove");
    s.detail.append(&remove);
    let weak = Rc::downgrade(s);
    remove.connect_clicked(move |_| {
        if let (Some(s), Some(pos)) = (weak.upgrade(), selected_pos.get()) {
            s.execute("para.tabs", json!({"clear":[pos]}));
        }
    });
    s.detail.append(&s.action(
        wordcraft_gnome::t!("Clear Tab Stops"),
        "paragraph-tabs-clear",
        "para.tabs",
        json!({"clearAll":true}),
    ));
}

fn border_color(s: &Rc<ParagraphPopover>, _ui: &Rc<Ui>, state: &Value) {
    let current = state["borders"]
        .as_object()
        .and_then(|b| b.values().find_map(|v| rgb_value_hex(&v["color"])));
    let weak = Rc::downgrade(s);
    let picker =
        embedded_color_chooser("paragraph-border-chooser", current.as_deref(), move |hex| {
            let Some(s) = weak.upgrade() else {
                return false;
            };
            s.border.borrow_mut()["color"] = json!(hex);
            change_edges(&s, None, Some(("color", json!(hex))))
        });
    s.detail.append(&picker.widget);
}
fn change_edges(
    s: &Rc<ParagraphPopover>,
    placement: Option<&str>,
    property: Option<(&str, Value)>,
) -> bool {
    let Some(ui) = s.ui.upgrade() else {
        return false;
    };
    let recipe = s.border.borrow().clone();
    let ok = tools_change(&ui, |session| {
        let styles = session.doc.styles.clone();
        wordcraft_engine::cmd::para::fmt(session, &|p| {
            let mut borders = styles.resolve_para(p).borders.unwrap_or_default();
            if let Some((key, value)) = &property {
                for edge in [
                    &mut borders.top,
                    &mut borders.bottom,
                    &mut borders.left,
                    &mut borders.right,
                    &mut borders.between,
                ]
                .into_iter()
                .flatten()
                {
                    match *key {
                        "width" => edge.width = value.as_f64().unwrap_or(0.5) as f32,
                        "style" => {
                            edge.style = wordcraft_doc::props::BorderStyle::from_ooxml(
                                value.as_str().unwrap_or("single"),
                            )
                        }
                        "color" => {
                            edge.color = value.as_str().and_then(wordcraft_doc::props::Rgb::parse)
                        }
                        _ => {}
                    }
                }
            }
            if let Some(kind) = placement {
                let line = wordcraft_doc::props::Border {
                    style: wordcraft_doc::props::BorderStyle::from_ooxml(
                        recipe["style"].as_str().unwrap_or("single"),
                    ),
                    width: recipe["width"].as_f64().unwrap_or(0.5) as f32,
                    color: recipe["color"]
                        .as_str()
                        .and_then(wordcraft_doc::props::Rgb::parse),
                    space: 1.,
                };
                if kind == "none" {
                    borders = Default::default();
                } else {
                    for edge in ["top", "bottom", "left", "right"] {
                        if edge == kind || matches!(kind, "all" | "outside") {
                            match edge {
                                "top" => borders.top = Some(line),
                                "bottom" => borders.bottom = Some(line),
                                "left" => borders.left = Some(line),
                                _ => borders.right = Some(line),
                            }
                        }
                    }
                    if kind == "all" || kind == "inside" {
                        borders.between = Some(line);
                    }
                }
            }
            p.borders = Some(borders);
        })
        .map_err(|e| e.to_string())?;
        Ok(())
    });
    if ok {
        s.remember_property("para.borders", &recipe, &ui);
        if !ui.format_group.pinned.get() {
            if let Some(pop) = s.popover.upgrade() {
                close_format_popover(&pop);
            }
        }
    }
    ok
}
fn borders(s: &Rc<ParagraphPopover>, ui: &Rc<Ui>, state: &Value) {
    heading(s, wordcraft_gnome::t!("Paragraph borders"));
    if state["borders"].is_null()
        && state["_mixed"]
            .as_array()
            .is_some_and(|m| m.iter().any(|v| v == "borders"))
    {
        note(s, wordcraft_gnome::t!("Mixed"));
    }
    for (title, kind) in [
        (wordcraft_gnome::t!("Bottom"), "bottom"),
        (wordcraft_gnome::t!("Top"), "top"),
        (wordcraft_gnome::t!("Left"), "left"),
        (wordcraft_gnome::t!("Right"), "right"),
        (wordcraft_gnome::t!("Outside"), "outside"),
        (wordcraft_gnome::t!("All"), "all"),
        (wordcraft_gnome::t!("No Borders"), "none"),
    ] {
        let b = style_menu_button(title);
        b.set_widget_name(&format!("paragraph-border-{kind}"));
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let sample = gtk::DrawingArea::new();
        sample.set_content_width(24);
        sample.set_content_height(24);
        sample.set_draw_func(move |w, c, _, _| {
            let fg = w.color();
            c.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 0.25);
            c.rectangle(3., 3., 18., 18.);
            let _ = c.stroke();
            c.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 1.);
            c.set_line_width(1.8);
            for (side, x1, y1, x2, y2) in [
                ("top", 3., 3., 21., 3.),
                ("bottom", 3., 21., 21., 21.),
                ("left", 3., 3., 3., 21.),
                ("right", 21., 3., 21., 21.),
            ] {
                if side == kind || matches!(kind, "outside" | "all") {
                    c.move_to(x1, y1);
                    c.line_to(x2, y2);
                }
            }
            if kind == "all" {
                c.move_to(3., 12.);
                c.line_to(21., 12.);
            }
            if kind == "none" {
                c.move_to(2., 22.);
                c.line_to(22., 2.);
            }
            let _ = c.stroke();
        });
        row.append(&sample);
        let label = gtk::Label::new(Some(title));
        label.set_hexpand(true);
        label.set_xalign(0.);
        row.append(&label);
        if state["borders"][kind].is_object() {
            row.append(&gtk::Label::new(Some("✓")));
        }
        b.set_child(Some(&row));
        s.detail.append(&b);
        let weak = Rc::downgrade(s);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.border.borrow_mut()["kind"] = json!(kind);
                change_edges(&s, Some(kind), None);
            }
        });
    }
    let widths = state["borders"]
        .as_object()
        .map(|o| {
            o.values()
                .filter_map(|v| v["width"].as_f64())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let width = widths
        .first()
        .copied()
        .filter(|v| widths.iter().all(|w| (v - w).abs() < 0.001));
    let n = s.number(
        wordcraft_gnome::t!("Thickness (pt)"),
        width.unwrap_or(0.5),
        0.25,
        6.,
    );
    n.set_widget_name("paragraph-border-thickness");
    if width.is_none() && !widths.is_empty() {
        n.set_text("");
    }
    let weak = Rc::downgrade(s);
    bind_numeric(ui, &n, move |v| {
        if let Some(s) = weak.upgrade() {
            s.border.borrow_mut()["width"] = json!(v);
            if change_edges(&s, None, Some(("width", json!(v)))) {
                Ok(())
            } else {
                Err("Cannot change borders".into())
            }
        } else {
            Err("Document closed".into())
        }
    });
    let styles = [
        "single", "double", "dotted", "dashed", "thick", "triple", "dotDash", "wave",
    ];
    let mode = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("Single"),
        wordcraft_gnome::t!("Double"),
        wordcraft_gnome::t!("Dotted"),
        wordcraft_gnome::t!("Dashed"),
        wordcraft_gnome::t!("Thick"),
        wordcraft_gnome::t!("Triple"),
        wordcraft_gnome::t!("Dot Dash"),
        wordcraft_gnome::t!("Wavy"),
    ]);
    let current: Vec<_> = state["borders"]
        .as_object()
        .map(|o| o.values().filter_map(|v| v["style"].as_str()).collect())
        .unwrap_or_default();
    let style = current
        .first()
        .copied()
        .filter(|v| current.iter().all(|w| v == w));
    mode.set_selected(
        style
            .and_then(|v| styles.iter().position(|x| *x == v))
            .map_or(gtk::INVALID_LIST_POSITION, |i| i as u32),
    );
    accessible_name(&mode, wordcraft_gnome::t!("Line Style"));
    s.detail.append(&mode);
    let weak = Rc::downgrade(s);
    mode.connect_selected_notify(move |m| {
        if let Some(s) = weak.upgrade() {
            if let Some(v) = styles.get(m.selected() as usize) {
                s.border.borrow_mut()["style"] = json!(v);
                change_edges(&s, None, Some(("style", json!(v))));
            }
        }
    });
    page_link(s, wordcraft_gnome::t!("Color"), "border-color");
}
