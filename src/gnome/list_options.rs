//! One list gallery at a time; remembered designs contain no list-instance IDs.
use super::*;
use serde_json::{Value, json};
pub(super) fn build(s: &Rc<ParagraphPopover>, ui: &Rc<Ui>, page: &str, state: &Value) -> bool {
    if matches!(page, "list-level" | "list-start") {
        let level = page == "list-level";
        let start = ui
            .current
            .borrow()
            .as_ref()
            .and_then(|d| {
                let num = state["numbering"]["num"].as_u64()? as u32;
                let level = state["numbering"]["level"].as_u64()? as u8;
                d.session
                    .doc
                    .numbering
                    .nums
                    .iter()
                    .find(|n| n.id == num)
                    .and_then(|n| {
                        n.start_overrides
                            .iter()
                            .find(|(l, _)| *l == level)
                            .map(|(_, v)| *v)
                    })
                    .or_else(|| d.session.doc.numbering.level(num, level).map(|l| l.start))
            })
            .unwrap_or(1);
        let spin = s.number(
            if level {
                wordcraft_gnome::t!("Level (1–9)")
            } else {
                wordcraft_gnome::t!("Start at")
            },
            if level {
                state["numbering"]["level"].as_f64().unwrap_or(0.) + 1.
            } else {
                start as f64
            },
            if level { 1. } else { 0. },
            if level { 9. } else { 32767. },
        );
        spin.set_digits(0);
        spin.set_widget_name(if level {
            "paragraph-list-level"
        } else {
            "paragraph-list-start"
        });
        if state["_mixed"]
            .as_array()
            .is_some_and(|m| m.iter().any(|v| v == "numbering"))
        {
            spin.set_text("");
        }
        let weak = Rc::downgrade(ui);
        bind_numeric(ui, &spin, move |v| {
            let ui = weak.upgrade().ok_or("Document closed")?;
            if v.fract() != 0. {
                return Err(wordcraft_gnome::t!("Enter a whole number.").into());
            }
            if level {
                immediate_format(
                    &ui,
                    "para.listLevel",
                    json!({"level":(v as u32).saturating_sub(1)}),
                )
            } else {
                tools_change(&ui, |s| wordcraft_gnome::list_design::start_at(s, v as u32));
                Ok(())
            }
        });
        return true;
    }
    if !matches!(page, "list" | "bullets" | "numbering" | "multilevel") {
        return false;
    }
    let category = if page == "numbering" {
        "numbering"
    } else if page == "multilevel" {
        "multilevel"
    } else {
        "bullets"
    };
    let tabs = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    tabs.add_css_class("linked");
    s.detail.append(&tabs);
    for (label, target) in [
        (wordcraft_gnome::t!("Bullets"), "bullets"),
        (wordcraft_gnome::t!("Numbering"), "numbering"),
        (wordcraft_gnome::t!("Multilevel"), "multilevel"),
    ] {
        let b = gtk::Button::with_label(label);
        b.set_widget_name(&format!("paragraph-more-{target}"));
        b.set_hexpand(true);
        if target == category {
            b.add_css_class("suggested-action");
        }
        tabs.append(&b);
        let weak = Rc::downgrade(s);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.page(target);
            }
        });
    }
    let choices: Vec<(&str, &str, &str)> = match category {
        "numbering" => vec![
            ("1. 2. 3.", "numbered", "1.  Abc\n2.  Abc\n3.  Abc"),
            ("1) 2) 3)", "numberedParen", "1)  Abc\n2)  Abc\n3)  Abc"),
            ("A. B. C.", "upperLetter", "A.  Abc\nB.  Abc\nC.  Abc"),
            ("a) b) c)", "lowerLetter", "a)  Abc\nb)  Abc\nc)  Abc"),
            ("i. ii. iii.", "lowerRoman", "i.  Abc\nii.  Abc\niii.  Abc"),
        ],
        "multilevel" => vec![
            (
                "1 / 1.1 / 1.1.1",
                "legal",
                "1.  Abc\n  1.1.  Abc\n    1.1.1.  Abc",
            ),
            (
                wordcraft_gnome::t!("Outline"),
                "outline",
                "I.  Abc\n  A.  Abc\n    1.  Abc",
            ),
        ],
        _ => vec![
            (wordcraft_gnome::t!("• Disc"), "•", "•  Abc\n•  Abc\n•  Abc"),
            (
                wordcraft_gnome::t!("○ Circle"),
                "○",
                "○  Abc\n○  Abc\n○  Abc",
            ),
            (
                wordcraft_gnome::t!("▪ Square"),
                "▪",
                "▪  Abc\n▪  Abc\n▪  Abc",
            ),
            (wordcraft_gnome::t!("– Dash"), "–", "–  Abc\n–  Abc\n–  Abc"),
        ],
    };
    let mixed = state["_mixed"]
        .as_array()
        .is_some_and(|v| v.iter().any(|v| v == "numbering"));
    let gallery = gtk::FlowBox::new();
    gallery.set_selection_mode(gtk::SelectionMode::None);
    gallery.set_min_children_per_line(2);
    gallery.set_max_children_per_line(2);
    gallery.set_homogeneous(true);
    gallery.set_row_spacing(4);
    gallery.set_column_spacing(4);
    s.detail.append(&gallery);
    for (label, kind, sample) in choices {
        let (on, enabled) = {
            let c = ui.current.borrow();
            let session = &c.as_ref().unwrap().session;
            (
                wordcraft_gnome::list_design::matches(session, kind),
                wordcraft_gnome::list_design::can_apply(session, kind),
            )
        };
        let b = gtk::Button::new();
        b.set_widget_name(&format!("paragraph-list-{kind}"));
        b.set_tooltip_text(Some(if enabled {label}else{wordcraft_gnome::t!("WordCraft cannot yet change this numbering design without restarting it. Customize the current list definition instead.")}));
        b.set_sensitive(enabled);
        accessible_name(&b, label);
        let col = gtk::Box::new(gtk::Orientation::Vertical, 3);
        let l = gtk::Label::new(Some(sample));
        l.set_xalign(0.);
        col.append(&l);
        let check = gtk::Label::new(Some(if on { "✓" } else { "" }));
        check.set_halign(gtk::Align::End);
        col.append(&check);
        b.set_child(Some(&col));
        if on {
            b.add_css_class("current-style");
        }
        b.update_state(&[gtk::accessible::State::Pressed(if mixed {
            gtk::AccessibleTristate::Mixed
        } else if on {
            gtk::AccessibleTristate::True
        } else {
            gtk::AccessibleTristate::False
        })]);
        gallery.insert(&b, -1);
        let weak = Rc::downgrade(s);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.execute(
                    if category == "bullets" {
                        "para.bullets"
                    } else if category == "multilevel" {
                        "para.multilevel"
                    } else {
                        "para.numbering"
                    },
                    json!({"kind":kind}),
                );
            }
        });
    }
    let status = gtk::Label::new(None);
    status.set_widget_name("paragraph-list-state");
    status.set_xalign(0.);
    status.add_css_class("heading");
    s.detail.append(&status);
    s.detail.append(&s.action(
        wordcraft_gnome::t!("No List"),
        "paragraph-list-none",
        "para.bullets",
        json!({"off":true}),
    ));
    let custom = style_menu_button(wordcraft_gnome::t!("Customize List…"));
    custom.set_widget_name("paragraph-list-custom");
    s.detail.append(&custom);
    let weak = Rc::downgrade(ui);
    custom.connect_clicked(move |_| {
        if let Some(ui) = weak.upgrade() {
            customize(&ui);
        }
    });
    let valid_list =
        state["numbering"].is_object() && state["numbering"]["num"].as_u64().is_some_and(|n| n > 0);
    let level = s.number(
        wordcraft_gnome::t!("Level (1–9)"),
        state["numbering"]["level"].as_f64().unwrap_or(0.) + 1.,
        1.,
        9.,
    );
    level.set_digits(0);
    level.set_sensitive(valid_list);
    level.set_widget_name("paragraph-list-level");
    let weak = Rc::downgrade(ui);
    bind_numeric(ui, &level, move |v| {
        let ui = weak.upgrade().ok_or("Document closed")?;
        if v.fract() != 0. {
            return Err(wordcraft_gnome::t!("Enter a whole list level from 1 to 9.").into());
        }
        immediate_format(
            &ui,
            "para.listLevel",
            json!({"level":(v as u32).saturating_sub(1)}),
        )
    });
    let continuation = style_menu_button(wordcraft_gnome::t!("Continue Numbering"));
    continuation.set_widget_name("paragraph-list-continue");
    continuation.set_sensitive(
        ui.current.borrow().as_ref().is_some_and(|d| {
            wordcraft_gnome::list_design::previous_compatible(&d.session).is_some()
        }),
    );
    s.detail.append(&continuation);
    let weak = Rc::downgrade(ui);
    continuation.connect_clicked(move |_| {
        if let Some(ui) = weak.upgrade() {
            tools_change(&ui, wordcraft_gnome::list_design::continue_previous);
        }
    });
    let restart = s.action(
        wordcraft_gnome::t!("Restart Numbering at 1"),
        "paragraph-list-restart",
        "para.restartNumbering",
        json!({}),
    );
    restart.set_sensitive(valid_list);
    s.detail.append(&restart);
    let start = s.number(wordcraft_gnome::t!("Start at"), 1., 0., 32767.);
    start.set_digits(0);
    start.set_sensitive(valid_list);
    start.set_widget_name("paragraph-list-start");
    let apply = style_menu_button(wordcraft_gnome::t!("Set Starting Number"));
    apply.set_sensitive(valid_list);
    apply.set_widget_name("paragraph-list-set-start");
    s.detail.append(&apply);
    let weak = Rc::downgrade(ui);
    apply.connect_clicked(move |_| {
        let Ok(value) = start.text().trim().parse::<u32>() else {
            return;
        };
        if let Some(ui) = weak.upgrade() {
            tools_change(&ui, |s| wordcraft_gnome::list_design::start_at(s, value));
        }
    });
    sync(s, state);
    true
}
pub(super) fn customize(ui: &Rc<Ui>) {
    let Some(host) = ui.host.upgrade() else {
        return;
    };
    let identity = document_identity(ui);
    let existing = {
        let c = ui.current.borrow();
        let s = &c.as_ref().unwrap().session;
        let p = s.doc.para_at(&s.sel.focus);
        p.and_then(|p| s.doc.styles.resolve_para(&p.props).numbering)
            .filter(|n| n.num != 0)
            .map(|n| n.num)
    };
    let w = preferences_dialog(wordcraft_gnome::t!("Customize List"));
    w.set_search_enabled(false);
    w.set_widget_name("list-custom-dialog");
    let p = panel_page(&w, wordcraft_gnome::t!("List"));
    let g = panel_group(&p, wordcraft_gnome::t!("List Definition"));
    g.set_description(Some(wordcraft_gnome::t!(
        "Edits affect every list sharing this definition. Heading links are preserved."
    )));
    let level = panel_number(&g, wordcraft_gnome::t!("Level (1–9)"), 1., 1., 9.);
    level.set_digits(0);
    level.set_widget_name("list-custom-level");
    let formats = [
        "decimal",
        "upperLetter",
        "lowerLetter",
        "upperRoman",
        "lowerRoman",
        "bullet",
        "none",
    ];
    let mode = adw::ComboRow::new();
    mode.set_title(wordcraft_gnome::t!("Number Format"));
    mode.set_widget_name("list-custom-format");
    mode.set_model(Some(&gtk::StringList::new(&[
        "1, 2, 3",
        "A, B, C",
        "a, b, c",
        "I, II, III",
        "i, ii, iii",
        "•",
        wordcraft_gnome::t!("None"),
    ])));
    g.add(&mode);
    let text = panel_entry(&g, wordcraft_gnome::t!("Marker Template"), "%1.");
    text.set_widget_name("list-custom-template");
    text.set_tooltip_text(Some(wordcraft_gnome::t!(
        "Use %1 through %9 for level numbers, or a bullet character."
    )));
    let base = ui
        .current
        .borrow()
        .as_ref()
        .map(|d| {
            (0..9)
                .map(|i| {
                    existing
                        .and_then(|id| d.session.doc.numbering.level(id, i).cloned())
                        .unwrap_or_else(|| {
                            wordcraft_doc::numbering::levels_for(
                                wordcraft_doc::numbering::ListKind::Numbered,
                            )[i as usize]
                                .clone()
                        })
                })
                .collect::<Vec<_>>()
        })
        .unwrap();
    let drafts = Rc::new(RefCell::new(vec![json!({}); 9]));
    let active = Rc::new(Cell::new(0usize));
    let syncing = Rc::new(Cell::new(true));
    mode.set_selected(
        formats
            .iter()
            .position(|v| *v == base[0].format.ooxml())
            .map_or(gtk::INVALID_LIST_POSITION, |i| i as u32),
    );
    text.set_text(&base[0].text);
    syncing.set(false);
    {
        let drafts = drafts.clone();
        let active = active.clone();
        let syncing = syncing.clone();
        mode.connect_selected_notify(move |m| {
            if !syncing.get() {
                if let Some(format) = formats.get(m.selected() as usize) {
                    drafts.borrow_mut()[active.get()]["format"] = json!(format);
                }
            }
        });
    }
    {
        let drafts = drafts.clone();
        let active = active.clone();
        let syncing = syncing.clone();
        text.connect_changed(move |e| {
            if !syncing.get() {
                drafts.borrow_mut()[active.get()]["text"] = json!(e.text().as_str());
            }
        });
    }
    {
        let drafts = drafts.clone();
        let active = active.clone();
        let syncing = syncing.clone();
        let mode = mode.clone();
        let text = text.clone();
        level.connect_value_changed(move |n| {
            let index = (n.value_as_int() - 1).clamp(0, 8) as usize;
            active.set(index);
            syncing.set(true);
            let draft = &drafts.borrow()[index];
            let format = draft["format"]
                .as_str()
                .unwrap_or(base[index].format.ooxml());
            mode.set_selected(
                formats
                    .iter()
                    .position(|f| *f == format)
                    .map_or(gtk::INVALID_LIST_POSITION, |i| i as u32),
            );
            text.set_text(draft["text"].as_str().unwrap_or(&base[index].text));
            syncing.set(false);
        });
    }
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
    apply.set_widget_name("list-custom-apply");
    g.add(&apply);
    let guard = panel_target(ui, &w);
    let weak = Rc::downgrade(ui);
    let dialog = w.downgrade();
    apply.connect_clicked(move |_| {
        let (Some(ui), Some(w)) = (weak.upgrade(), dialog.upgrade()) else {
            return;
        };
        if !guard() || document_identity(&ui) != identity {
            return;
        }
        let levels = drafts.borrow().clone();
        if levels
            .iter()
            .all(|v| v.as_object().is_some_and(|o| o.is_empty()))
        {
            w.close();
            return;
        }
        let mut args = json!({"levels":levels,"apply":existing.is_none()});
        if let Some(id) = existing {
            args["num"] = json!(id);
        }
        if tools_change(&ui, move |s| {
            s.run("list.define", &args)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }) {
            if let Some(p) = ui.paragraph.borrow().as_ref() {
                p.groups.borrow_mut().remove("list");
            }
            w.close();
        }
    });
    w.present(Some(&host.window));
}

pub(super) fn sync(s: &ParagraphPopover, state: &Value) {
    let Some(ui) = s.ui.upgrade() else { return };
    let current = ui.current.borrow();
    let Some(d) = current.as_ref() else { return };
    let mixed = state["_mixed"]
        .as_array()
        .is_some_and(|v| v.iter().any(|k| k == "numbering"));
    let valid_list = state["numbering"]["num"].as_u64().is_some_and(|n| n > 0);
    if let Some(b) = find_widget(&s.detail, "paragraph-list-none").and_downcast::<gtk::Button>() {
        b.update_state(&[gtk::accessible::State::Pressed(if mixed {
            gtk::AccessibleTristate::Mixed
        } else if !valid_list {
            gtk::AccessibleTristate::True
        } else {
            gtk::AccessibleTristate::False
        })]);
        if !valid_list && !mixed {
            b.add_css_class("current-style");
        } else {
            b.remove_css_class("current-style");
        }
    }
    for name in [
        "paragraph-list-level",
        "paragraph-list-start",
        "paragraph-list-set-start",
        "paragraph-list-restart",
    ] {
        if let Some(w) = find_widget(&s.detail, name) {
            w.set_sensitive(valid_list && !mixed);
        }
    }
    if let Some(w) = find_widget(&s.detail, "paragraph-list-continue") {
        w.set_sensitive(
            wordcraft_gnome::list_design::previous_compatible(&d.session).is_some() && !mixed,
        );
    }
    if let Some(label) = find_widget(&s.detail, "paragraph-list-state").and_downcast::<gtk::Label>()
    {
        let builtin = [
            "•",
            "○",
            "▪",
            "–",
            "numbered",
            "numberedParen",
            "upperLetter",
            "lowerLetter",
            "lowerRoman",
            "legal",
            "outline",
        ]
        .iter()
        .any(|kind| wordcraft_gnome::list_design::matches(&d.session, kind));
        label.set_text(if mixed {
            wordcraft_gnome::t!("Mixed")
        } else if valid_list && !builtin {
            wordcraft_gnome::t!("Custom")
        } else {
            ""
        });
    }
    for kind in [
        "•",
        "○",
        "▪",
        "–",
        "numbered",
        "numberedParen",
        "upperLetter",
        "lowerLetter",
        "lowerRoman",
        "legal",
        "outline",
    ] {
        if let Some(b) =
            find_widget(&s.detail, &format!("paragraph-list-{kind}")).and_downcast::<gtk::Button>()
        {
            let on = wordcraft_gnome::list_design::matches(&d.session, kind);
            b.set_sensitive(wordcraft_gnome::list_design::can_apply(&d.session, kind));
            if on {
                b.add_css_class("current-style");
            } else {
                b.remove_css_class("current-style");
            }
            b.update_state(&[gtk::accessible::State::Pressed(if mixed {
                gtk::AccessibleTristate::Mixed
            } else if on {
                gtk::AccessibleTristate::True
            } else {
                gtk::AccessibleTristate::False
            })]);
            if let Some(mark) = b
                .child()
                .and_then(|w| w.last_child())
                .and_downcast::<gtk::Label>()
            {
                mark.set_text(if on { "✓" } else { "" });
            }
        }
    }
}
