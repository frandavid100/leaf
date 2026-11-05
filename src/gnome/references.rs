use super::*;
use serde_json::json;
use wordcraft_gnome::references;

fn reference_action(ui: &Rc<Ui>, action: impl FnOnce(&mut Session) -> Result<(), String>) {
    if tools_change(ui, action) {
        ui.document_box.grab_focus();
        caret_ensure_visible(ui);
    }
}

pub(super) fn install_reference_actions(host: &Rc<Workspace>) {
    window_action(host, "insert-equation", |h| {
        if let Some(ui) = selected_ui(h) {
            show_equation(&ui, &h.window);
        }
    });

    window_action(host, "insert-link", |h| {
        if let Some(ui) = selected_ui(h) {
            show_link(&ui);
        }
    });
    window_action(host, "follow-note", |h| {
        if let Some(ui) = selected_ui(h) {
            reference_action(&ui, |s| {
                if matches!(s.sel.focus.story, wordcraft_doc::StoryRef::Part(_)) {
                    references::return_to_reference(s)
                } else {
                    references::open_footnote(s, false)
                }
            });
        }
    });

    for (name, action) in [
        (
            "insert-footnote",
            references::insert_footnote as fn(&mut Session) -> Result<(), String>,
        ),
        ("edit-footnote", |s: &mut Session| {
            references::open_footnote(s, false)
        }),
        ("next-footnote", |s: &mut Session| {
            references::open_footnote(s, true)
        }),
        ("footnote-reference", references::return_to_reference),
        ("update-references", references::update),
    ] {
        window_action(host, name, move |host| {
            if let Some(ui) = selected_ui(host) {
                reference_action(&ui, action);
            }
        });
    }
    window_action(host, "insert-toc", |h| {
        if let Some(ui) = selected_ui(h) {
            show_insert_contents_dialog(&ui, &h.window);
        }
    });
    window_action(host, "insert-cross-reference", |h| {
        if let Some(ui) = selected_ui(h) {
            show_cross_reference_dialog(&ui, &h.window);
        }
    });
    window_action(host, "insert-caption", |h| {
        if let Some(ui) = selected_ui(h) {
            reference_action(&ui, references::insert_caption);
        }
    });
    window_action(host, "footnote-options", |h| {
        if let Some(ui) = selected_ui(h) {
            show_footnote_options(&ui, &h.window);
        }
    });
    for (name, instr) in [
        ("field-page", "PAGE"),
        ("field-pages", "NUMPAGES"),
        ("field-author", "AUTHOR"),
        ("field-title", "TITLE"),
        ("field-filename", "FILENAME"),
        ("field-date", r#"DATE \@ "yyyy-MM-dd""#),
        ("field-time", r#"TIME \@ "HH:mm""#),
    ] {
        window_action(host, name, move |h| {
            if let Some(ui) = selected_ui(h) {
                reference_action(&ui, |s| references::insert_field(s, instr));
            }
        });
    }
}

pub(super) fn reference_window(
    parent: &ApplicationWindow,
    title: &str,
    inserting: bool,
) -> (adw::Window, gtk::Box, gtk::Button) {
    let window = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(440)
        .resizable(false)
        .build();
    let view = ToolbarView::new();
    let header = HeaderBar::new();
    header.set_show_end_title_buttons(false);
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    let apply = gtk::Button::with_label(if inserting {
        wordcraft_gnome::t!("Insert")
    } else {
        wordcraft_gnome::t!("Apply")
    });
    apply.add_css_class("suggested-action");
    apply.set_widget_name("reference-apply");
    header.pack_start(&cancel);
    header.pack_end(&apply);
    view.add_top_bar(&header);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(18);
    content.set_margin_bottom(18);
    content.set_margin_start(18);
    content.set_margin_end(18);
    view.set_content(Some(&content));
    window.set_content(Some(&view));
    let weak = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    (window, content, apply)
}

pub(super) fn reference_row(root: &gtk::Box, label: &str, control: &impl IsA<gtk::Widget>) {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let text = gtk::Label::new(Some(label));
    text.set_xalign(0.0);
    row.append(&text);
    row.append(control);
    root.append(&row);
}

pub(super) fn reference_apply(
    ui: &Rc<Ui>,
    window: &adw::Window,
    root: &gtk::Box,
    apply: &gtk::Button,
    action: impl Fn(&mut Session) -> Result<(), String> + 'static,
) {
    let identity = document_identity(ui);
    let selection = ui.current.borrow().as_ref().map(|d| d.session.sel.clone());
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    error.add_css_class("error");
    root.append(&error);
    let weak_ui = Rc::downgrade(ui);
    let weak_window = window.downgrade();
    apply.connect_clicked(move |_| {
        let (Some(ui), Some(window)) = (weak_ui.upgrade(), weak_window.upgrade()) else {
            return;
        };
        if ui.closed.get()
            || document_identity(&ui) != identity
            || ui.current.borrow().as_ref().map(|d| d.session.sel.clone()) != selection
        {
            error.set_text(wordcraft_gnome::t!(
                "The document has changed; open this dialog again."
            ));
            return;
        }
        let result = {
            let mut current = ui.current.borrow_mut();
            let Some(doc) = current.as_mut() else { return };
            action(&mut doc.session)
        };
        match result {
            Ok(()) => {
                window.close();
                after_content_change(&ui);
                ui.document_box.grab_focus();
                caret_ensure_visible(&ui);
            }
            Err(e) => error.set_text(&e),
        }
    });
    window.present();
}

pub(super) fn show_contents_dialog(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let existing = {
        let c = ui.current.borrow();
        let d = c.as_ref().unwrap();
        if wordcraft_engine::cmd::references::caret_in_toc(&d.session) {
            references::toc_field(&d.session)
        } else {
            None
        }
    };
    contents_window(ui, parent, existing);
}

pub(super) fn show_insert_contents_dialog(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    contents_window(ui, parent, None);
}
fn contents_window(ui: &Rc<Ui>, parent: &ApplicationWindow, existing: Option<(Pos, String)>) {
    let (w, root, apply) = reference_window(
        parent,
        wordcraft_gnome::t!("Table of Contents"),
        existing.is_none(),
    );
    let action = contents_controls(ui, &root, existing);
    reference_apply(ui, &w, &root, &apply, move |s| action(s));
}
pub(super) type ContentsAction = Box<dyn Fn(&mut Session) -> Result<(), String>>;
pub(super) fn contents_controls(
    ui: &Rc<Ui>,
    root: &gtk::Box,
    existing: Option<(Pos, String)>,
) -> ContentsAction {
    let title = gtk::Entry::new();
    title.set_text(wordcraft_gnome::t!("Contents"));
    let levels = gtk::SpinButton::with_range(1., 9., 1.);
    levels.set_value(3.);
    let pages = gtk::CheckButton::with_label(wordcraft_gnome::t!("Show Page Numbers"));
    pages.set_active(true);
    let hyperlinks = gtk::CheckButton::with_label(wordcraft_gnome::t!("Clickable Entries"));
    hyperlinks.set_active(true);
    let right = gtk::CheckButton::with_label(wordcraft_gnome::t!("Right-align Page Numbers"));
    right.set_active(true);
    let mut initial = (3, true, true, true);
    if let Some((at, code)) = &existing {
        let f = wordcraft_engine::cmd::references::FieldCode::parse(code);
        let max = f
            .arg('o')
            .and_then(wordcraft_engine::cmd::references::level_range)
            .map(|(_, b)| b)
            .unwrap_or(3);
        levels.set_value(max as f64);
        pages.set_active(!f.has('n'));
        hyperlinks.set_active(f.has('h'));
        right.set_active(!f.has('p'));
        let c = ui.current.borrow();
        let p = c.as_ref().unwrap().session.doc.para_at(at).unwrap();
        title.set_sensitive(p.props.style.as_deref() == Some("TOCHeading"));
        title.set_text(&p.text[..at.off].replace(wordcraft_doc::para::OBJ, ""));
        initial = (
            max as i32,
            pages.is_active(),
            hyperlinks.is_active(),
            right.is_active(),
        );
    }
    reference_row(&root, wordcraft_gnome::t!("Title"), &title);
    reference_row(&root, wordcraft_gnome::t!("Heading Levels"), &levels);
    root.append(&pages);
    root.append(&hyperlinks);
    root.append(&right);
    Box::new(move |s| {
        if let Some((_, code)) = &existing {
            let mut f = wordcraft_engine::cmd::references::FieldCode::parse(code);
            let mut set = |key: char, value: Option<Option<String>>| {
                if let Some(value) = value {
                    f.switches.retain(|(k, _)| *k != key);
                    if let Some(value) = value {
                        f.switches
                            .push((key, if value.is_empty() { None } else { Some(value) }));
                    }
                }
            };
            if levels.value_as_int() != initial.0 {
                set('o', Some(Some(format!("1-{}", levels.value_as_int()))));
            }
            if pages.is_active() != initial.1 {
                set(
                    'n',
                    Some(if pages.is_active() {
                        None
                    } else {
                        Some(format!("1-{}", levels.value_as_int()))
                    }),
                );
            }
            if hyperlinks.is_active() != initial.2 {
                set('h', Some(hyperlinks.is_active().then(String::new)));
            }
            if right.is_active() != initial.3 {
                set(
                    'p',
                    Some(if right.is_active() {
                        None
                    } else {
                        Some(" ".into())
                    }),
                );
            }
            let quote = |s: &str| format!("\"{}\"", s.replace('\\', "\\\\").replace('\"', "\\\""));
            let mut code = f.name;
            for arg in f.args {
                code.push(' ');
                code.push_str(&quote(&arg));
            }
            for (key, arg) in f.switches {
                code.push_str(&format!(" \\{key}"));
                if let Some(arg) = arg {
                    code.push(' ');
                    code.push_str(&quote(&arg));
                }
            }
            references::edit_toc(
                s,
                &code,
                title.is_sensitive().then(|| title.text()).as_deref(),
            )
        } else {
            references::insert_toc(
                s,
                &json!({"title":title.text().as_str(),"levels":levels.value_as_int(),"pageNumbers":pages.is_active(),"hyperlinks":hyperlinks.is_active(),"rightAlign":right.is_active()}),
            )
        }
    })
}

fn show_cross_reference_dialog(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let (w, root, apply) = reference_window(parent, wordcraft_gnome::t!("Cross-reference"), true);
    let kinds = ["heading", "figure", "table", "bookmark"];
    let kind = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("Heading"),
        wordcraft_gnome::t!("Figure"),
        wordcraft_gnome::t!("Table"),
        wordcraft_gnome::t!("Bookmark"),
    ]);
    let target = gtk::DropDown::from_strings(&[]);
    let show = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("Text"),
        wordcraft_gnome::t!("Page Number"),
    ]);
    reference_row(&root, wordcraft_gnome::t!("Reference Type"), &kind);
    reference_row(&root, wordcraft_gnome::t!("Target"), &target);
    reference_row(&root, wordcraft_gnome::t!("Show"), &show);
    let weak = Rc::downgrade(ui);
    let t = target.clone();
    let refresh = move |kind: &gtk::DropDown| {
        let Some(ui) = weak.upgrade() else { return };
        let current = ui.current.borrow();
        let Some(doc) = current.as_ref() else { return };
        let targets = wordcraft_engine::cmd::citations::ref_targets(
            &doc.session,
            kinds[kind.selected() as usize],
        );
        let model = gtk::StringList::new(&[]);
        for (text, _) in targets {
            model.append(&text);
        }
        t.set_model(Some(&model));
        t.set_selected(0);
    };
    refresh(&kind);
    kind.connect_selected_notify(refresh);
    reference_apply(ui, &w, &root, &apply, move |s| {
        let text = target
            .selected_item()
            .and_then(|x| x.downcast::<gtk::StringObject>().ok())
            .ok_or(wordcraft_gnome::t!(
                "No references of this type are available."
            ))?
            .string();
        s.run(
            "insert.crossReference",
            &json!({"to":kinds[kind.selected() as usize],"target":text.as_str(),
            "show":if show.selected()==0 {"text"} else {"page"}}),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    });
}

fn show_footnote_options(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let (w, root, apply) = reference_window(parent, wordcraft_gnome::t!("Footnote Options"), false);
    let format = gtk::DropDown::from_strings(&["1, 2, 3", "i, ii, iii", "I, II, III", "a, b, c"]);
    let restart = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("Continuous"),
        wordcraft_gnome::t!("Each Section"),
        wordcraft_gnome::t!("Each Page"),
    ]);
    let start = gtk::SpinButton::with_range(1., 32767., 1.);
    let mut formats = vec![
        "decimal".to_string(),
        "lowerRoman".to_string(),
        "upperRoman".to_string(),
        "lowerLetter".to_string(),
    ];
    if let Some(d) = ui.current.borrow().as_ref() {
        let section = wordcraft_engine::cmd::page::sect(&d.session);
        let options = d.session.doc.note_options(&section, false);
        let current = options.num_fmt.ooxml().to_string();
        let index = if let Some(i) = formats.iter().position(|f| *f == current) {
            i
        } else {
            formats.push(current.clone());
            let model =
                gtk::StringList::new(&["1, 2, 3", "i, ii, iii", "I, II, III", "a, b, c", &current]);
            format.set_model(Some(&model));
            formats.len() - 1
        };
        format.set_selected(index as u32);
        restart.set_selected(match options.num_restart {
            wordcraft_doc::section::NoteRestart::EachSect => 1,
            wordcraft_doc::section::NoteRestart::EachPage => 2,
            _ => 0,
        });
        start.set_value(options.num_start as f64);
    }
    let scope = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("This Section"),
        wordcraft_gnome::t!("Whole Document"),
    ]);
    reference_row(&root, wordcraft_gnome::t!("Apply To"), &scope);
    reference_row(&root, wordcraft_gnome::t!("Number Format"), &format);
    reference_row(&root, wordcraft_gnome::t!("Restart Numbering"), &restart);
    reference_row(&root, wordcraft_gnome::t!("Start At"), &start);
    reference_apply(ui, &w, &root, &apply, move |s| {
        s.run("references.noteOptions", &json!({"kind":"footnote","scope":if scope.selected()==0 {"section"}else{"document"},
            "numFmt":formats[format.selected() as usize],
            "numRestart":(["continuous","eachSect","eachPage"][restart.selected() as usize]),"numStart":start.value() as u32}))
            .map_err(|e|e.to_string())?;
        Ok(())
    });
}

pub(super) fn schedule_derived_refresh(ui: &Rc<Ui>) {
    if let Some(id) = ui.derived_source.borrow_mut().take() {
        id.remove();
    }
    let identity = document_identity(ui);
    let weak = Rc::downgrade(ui);
    let source = glib::timeout_add_local_once(std::time::Duration::from_millis(450), move || {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        ui.derived_source.borrow_mut().take();
        let modal = ui.host.upgrade().is_some_and(|h| {
            gtk::Window::list_toplevels()
                .iter()
                .filter_map(|w| w.downcast_ref::<gtk::Window>())
                .any(|w| {
                    w.is_visible()
                        && w.is_modal()
                        && w.transient_for().as_ref() == Some(h.window.upcast_ref())
                })
        });
        if modal || ui.picture_drag.borrow().is_some() || table_pointer_active(&ui) {
            schedule_derived_refresh(&ui);
            return;
        }
        if document_identity(&ui) != identity {
            return;
        }
        let scroll = (
            ui.scroll.hadjustment().value(),
            ui.scroll.vadjustment().value(),
        );
        let result = {
            let mut c = ui.current.borrow_mut();
            c.as_mut()
                .map(|d| references::refresh_derived(&mut d.session))
        };
        match result {
            Some(Ok(true)) => {
                refresh_after_content_change(&ui);
                let weak = Rc::downgrade(&ui);
                glib::idle_add_local_once(move || {
                    if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
                        ui.scroll.hadjustment().set_value(scroll.0);
                        ui.scroll.vadjustment().set_value(scroll.1);
                    }
                });
            }
            Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
            _ => {}
        }
    });
    ui.derived_source.replace(Some(source));
}

pub(super) fn follow_note_number_at(ui: &Rc<Ui>, x: f64, y: f64) -> bool {
    let target = {
        let mut current = ui.current.borrow_mut();
        let Some(d) = current.as_mut() else {
            return false;
        };
        let Some((page, r)) = d.page_frames.iter().enumerate().find_map(|(i, f)| {
            let r = f.compute_bounds(&ui.document_box)?;
            (x >= r.x() as f64
                && x <= (r.x() + r.width()) as f64
                && y >= r.y() as f64
                && y <= (r.y() + r.height()) as f64)
                .then_some((i, r))
        }) else {
            return false;
        };
        let z = d.page_zoom(page);
        let px = (x - r.x() as f64) as f32 / z;
        let py = (y - r.y() as f64) as f32 / z;
        let layout = d.session.layout();
        let story = layout
            .story_at(page, px, py)
            .unwrap_or(wordcraft_doc::StoryRef::Body);
        let Some(hit) = layout.hit(page, px, py, story) else {
            return false;
        };
        let Some(p) = d.session.doc.para_at(&hit) else {
            return false;
        };
        let mut found = None;
        for off in p.object_offsets() {
            if let Some(wordcraft_doc::para::InlineObject::NoteRef {
                kind: wordcraft_doc::para::NoteKind::Footnote,
                id,
                ..
            }) = p.object_at(off)
            {
                let a = Pos { off, ..hit.clone() };
                let b = Pos {
                    off: off + wordcraft_doc::para::OBJ.len_utf8(),
                    ..a.clone()
                };
                if layout
                    .selection_rects(&d.session.doc, &a, &b, page)
                    .iter()
                    .any(|(i, r)| {
                        *i == page
                            && px >= r.x - 2. / z
                            && px <= r.right() + 2. / z
                            && py >= r.y
                            && py <= r.bottom()
                    })
                {
                    found = Some(if story == wordcraft_doc::StoryRef::Body {
                        d.session.doc.end_of(wordcraft_doc::StoryRef::Part(*id))
                    } else {
                        let Some((mut reference, _)) = references::footnote_refs(&d.session)
                            .into_iter()
                            .find(|(_, n)| n == id)
                        else {
                            return false;
                        };
                        reference.off += wordcraft_doc::para::OBJ.len_utf8();
                        reference
                    });
                    break;
                }
            }
        }
        found
    };
    let Some(target) = target else { return false };
    {
        let mut c = ui.current.borrow_mut();
        let _ = selection::set_caret(&mut c.as_mut().unwrap().session, &target, false);
    }
    refresh_selection(ui);
    caret_ensure_visible(ui);
    ui.document_box.grab_focus();
    true
}
