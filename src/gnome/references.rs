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
            show_contents_dialog(&ui, &h.window);
        }
    });
    window_action(host, "insert-cross-reference", |h| {
        if let Some(ui) = selected_ui(h) {
            show_cross_reference_dialog(&ui, &h.window);
        }
    });
    window_action(host, "insert-caption", |h| {
        if let Some(ui) = selected_ui(h) {
            show_caption_dialog(&ui, &h.window);
        }
    });
    window_action(host, "footnote-options", |h| {
        if let Some(ui) = selected_ui(h) {
            show_footnote_options(&ui, &h.window);
        }
    });
}

fn reference_window(
    parent: &ApplicationWindow,
    title: &str,
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
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
    apply.add_css_class("suggested-action");
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

fn reference_row(root: &gtk::Box, label: &str, control: &impl IsA<gtk::Widget>) {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let text = gtk::Label::new(Some(label));
    text.set_xalign(0.0);
    row.append(&text);
    row.append(control);
    root.append(&row);
}

fn reference_apply(
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

fn show_contents_dialog(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let (w, root, apply) = reference_window(parent, wordcraft_gnome::t!("Table of Contents"));
    let title = gtk::Entry::new();
    title.set_text(wordcraft_gnome::t!("Contents"));
    let levels = gtk::SpinButton::with_range(1., 9., 1.);
    levels.set_value(3.);
    let pages = gtk::CheckButton::with_label(wordcraft_gnome::t!("Show Page Numbers"));
    pages.set_active(true);
    reference_row(&root, wordcraft_gnome::t!("Title"), &title);
    reference_row(&root, wordcraft_gnome::t!("Heading Levels"), &levels);
    root.append(&pages);
    reference_apply(ui, &w, &root, &apply, move |s| {
        s.run(
            "references.toc",
            &json!({"title":title.text().as_str(),"levels":levels.value() as u8,
            "pageNumbers":pages.is_active(),"hyperlinks":true}),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    });
}

fn show_cross_reference_dialog(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let (w, root, apply) = reference_window(parent, wordcraft_gnome::t!("Cross-reference"));
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

fn show_caption_dialog(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let (w, root, apply) = reference_window(parent, wordcraft_gnome::t!("Caption"));
    let kind =
        gtk::DropDown::from_strings(&[wordcraft_gnome::t!("Figure"), wordcraft_gnome::t!("Table")]);
    let text = gtk::Entry::new();
    reference_row(&root, wordcraft_gnome::t!("Label"), &kind);
    reference_row(&root, wordcraft_gnome::t!("Text"), &text);
    reference_apply(ui, &w, &root, &apply, move |s| {
        s.run(
            "references.caption",
            &json!({"label":if kind.selected()==0 {"Figure"} else {"Table"},
            "text":text.text().as_str(),"position":"below"}),
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    });
}

fn show_footnote_options(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let (w, root, apply) = reference_window(parent, wordcraft_gnome::t!("Footnote Options"));
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
