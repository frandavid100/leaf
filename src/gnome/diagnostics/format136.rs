use super::*;
use serde_json::json;

fn editable_child(root: &gtk::Widget) -> Option<gtk::Widget> {
    if let Some(entry) = root.downcast_ref::<gtk::Entry>() {
        let value = entry.text();
        if value.trim_start_matches('#').len() == 6 {
            return Some(root.clone());
        }
    }
    let mut child = root.first_child();
    while let Some(w) = child {
        child = w.next_sibling();
        if let Some(found) = editable_child(&w) {
            return Some(found);
        }
    }
    None
}
fn scroll_to_control(widget: &gtk::Widget) {
    let mut parent = widget.parent();
    while let Some(w) = parent {
        parent = w.parent();
        if let Some(scroll) = w.downcast_ref::<gtk::ScrolledWindow>() {
            if let Some(r) = widget.compute_bounds(scroll) {
                let a = scroll.vadjustment();
                let delta = if r.y() < 0. {
                    r.y()
                } else {
                    (r.y() + r.height() - scroll.height() as f32).max(0.)
                };
                if delta.abs() > 0.5 {
                    a.set_value(
                        (a.value() + delta as f64)
                            .clamp(a.lower(), (a.upper() - a.page_size()).max(a.lower())),
                    );
                }
            }
        }
    }
}
fn action_widget(root: &gtk::Widget, name: &str) -> Option<gtk::Widget> {
    if root
        .clone()
        .dynamic_cast::<gtk::Actionable>()
        .ok()
        .is_some_and(|w| w.action_name().as_deref() == Some(name))
    {
        return Some(root.clone());
    }
    let mut child = root.first_child();
    while let Some(w) = child {
        child = w.next_sibling();
        if let Some(found) = action_widget(&w, name) {
            return Some(found);
        }
    }
    None
}
#[allow(deprecated)]
pub(super) fn format136_ui_test(
    ui: &Rc<Ui>,
    window: &ApplicationWindow,
    app: &Application,
    text_menu: &gtk::MenuButton,
    paragraph_menu: &gtk::MenuButton,
    style_menu: &gtk::MenuButton,
    style: &Rc<StylePopover>,
) {
    let compact = std::env::var("GDK_SCALE").as_deref() == Ok("2");
    window.set_default_size(
        if compact { 480 } else { 1250 },
        if compact { 540 } else { 950 },
    );
    if compact {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
        gtk::Settings::default()
            .unwrap()
            .set_property("gtk-font-name", "Cantarell 16");
    }
    let mut session = Session::new(wordcraft_doc::Document::from_text(
        "Format sample.\nSecond paragraph.",
    ));
    let end = session.doc.end_of(wordcraft_doc::StoryRef::Body);
    selection::set_caret(&mut session, &end, false).unwrap();
    session
        .run("insert.table", &json!({"rows":2,"cols":2}))
        .unwrap();
    selection::select_range(&mut session, &Pos::body(0, 0), &Pos::body(0, 14)).unwrap();
    opened(ui, session, "Native formatting");
    if compact {
        // Match the normal editing path and the existing narrow-input test:
        // a modal overlay sidebar must be dismissed before editor menus open.
        ui.sidebar.set_show_sidebar(false);
        ui.sidebar_button.set_active(false);
    }
    let cases = vec![
        (0, "text-underline", wordcraft_gnome::t!("Underline")),
        (0, "text-strike", wordcraft_gnome::t!("Strikethrough")),
        (0, "text-font-property", wordcraft_gnome::t!("Font")),
        (0, "text-language", wordcraft_gnome::t!("Language")),
        (0, "text-ink", wordcraft_gnome::t!("Text Color")),
        (0, "text-highlight", wordcraft_gnome::t!("Highlight")),
        (1, "paragraph-alignment", wordcraft_gnome::t!("Alignment")),
        (1, "paragraph-list", wordcraft_gnome::t!("List")),
        (1, "paragraph-more-indents", wordcraft_gnome::t!("Indents")),
        (
            1,
            "paragraph-more-spacing",
            wordcraft_gnome::t!("Line Spacing"),
        ),
        (1, "paragraph-more-borders", wordcraft_gnome::t!("Borders")),
        (1, "paragraph-more-shading", wordcraft_gnome::t!("Shading")),
        (1, "paragraph-columns", wordcraft_gnome::t!("Columns")),
        (2, "style-new", wordcraft_gnome::t!("New Style")),
        (3, "table", wordcraft_gnome::t!("Table")),
        (3, "image", wordcraft_gnome::t!("Image")),
        (3, "toc", wordcraft_gnome::t!("Table of Contents")),
        // Revisit direct children after visiting unrelated nested pages.
        (0, "text-font-property", wordcraft_gnome::t!("Font")),
        (0, "text-underline", wordcraft_gnome::t!("Underline")),
    ];
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    let (text, para_menu, styles_menu, styles) = (
        text_menu.clone(),
        paragraph_menu.clone(),
        style_menu.clone(),
        style.clone(),
    );
    let tools = u.text_tools.borrow().as_ref().unwrap().clone();
    let para = u.paragraph.borrow().as_ref().unwrap().clone();
    let elements = u.elements.borrow().as_ref().unwrap().clone();
    let index = Cell::new(0usize);
    let phase = Cell::new(0u8);
    let ticket = Cell::new(0u32);
    let ticks = Cell::new(0u32);
    let original: RefCell<Option<wordcraft_doc::Document>> = RefCell::new(None);
    let depth = Cell::new(0usize);
    let page_wait = Cell::new(0u8);
    let selected = RefCell::new(None);
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        ticks.set(ticks.get() + 1);
        assert!(
            ticks.get() < 1400,
            "format136 stalled case={} phase={}",
            index.get(),
            phase.get()
        );
        w.set_title(Some("Leaf Format136 Regression"));
        if index.get() >= cases.len() {
            eprintln!(
                "FORMAT136_UI_OK families=4 native_colors=true direct_fonts=true pointer_back=true"
            );
            a.quit();
            return glib::ControlFlow::Break;
        }
        let (family, row, title) = cases[index.get()];
        let (pop, header) = match family {
            0 => (text.popover().unwrap(), tools.header.clone()),
            1 => (para_menu.popover().unwrap(), para.header.clone()),
            2 => (styles_menu.popover().unwrap(), styles.header.clone()),
            _ => (elements.pop.clone(), elements.header.clone()),
        };
        let ack = PathBuf::from(std::env::var_os("LEAF29_ACK_DIR").unwrap())
            .join(format!("format136-{}", ticket.get()));
        match phase.get() {
            0 => {
                if family == 3 {
                    let mut current = u.current.borrow_mut();
                    let s = &mut current.as_mut().unwrap().session;
                    let path = s
                        .doc
                        .para_paths(wordcraft_doc::StoryRef::Body)
                        .into_iter()
                        .find(|p| p.cell().is_some())
                        .unwrap();
                    selection::set_caret(
                        s,
                        &Pos {
                            story: wordcraft_doc::StoryRef::Body,
                            path,
                            off: 0,
                        },
                        false,
                    )
                    .unwrap();
                    drop(current);
                    refresh_selection(&u);
                }
                original.replace(Some(
                    u.current.borrow().as_ref().unwrap().session.doc.clone(),
                ));
                depth.set(u.current.borrow().as_ref().unwrap().session.undo_depth());
                selected.replace(Some(
                    u.current.borrow().as_ref().unwrap().session.sel.clone(),
                ));
                page_wait.set(0);
                header.pin.set_active(index.get() % 2 == 1);
                match family {
                    0 => {
                        tools.home();
                        text.popup();
                    }
                    1 => {
                        para.home();
                        para_menu.popup();
                    }
                    2 => {
                        styles.main();
                        styles_menu.popup();
                    }
                    _ => {
                        elements.home();
                        elements.menu.popup();
                    }
                }
                phase.set(1);
            }
            1 if pop.is_mapped() => {
                let target = if family == 3 {
                    let target = action_widget(pop.upcast_ref(), &format!("win.elements-{row}"));
                    if let Some(target) = target.filter(|t| t.is_sensitive()) {
                        target
                    } else {
                        // Existing no-target actions remain disabled; inspect
                        // their shared contained body without inventing a target.
                        elements.show(row);
                        phase.set(3);
                        return glib::ControlFlow::Continue;
                    }
                } else {
                    find_widget(&pop, row).unwrap()
                };
                scroll_to_control(&target);
                if [
                    "text-font-property",
                    "text-language",
                    "text-ink",
                    "text-highlight",
                    "paragraph-alignment",
                    "paragraph-more-spacing",
                    "paragraph-more-borders",
                    "paragraph-more-shading",
                ]
                .contains(&row)
                {
                    let split = target
                        .parent()
                        .unwrap()
                        .downcast::<adw::SplitButton>()
                        .unwrap();
                    // libadwaita owns the content-derived image/text classes.
                    // Both toolbar history and property rows use this native
                    // component's flat presentation, with its own separator.
                    assert!(split.has_css_class("flat") && !split.has_css_class("raised"));
                    let repeat = target
                        .next_sibling()
                        .unwrap()
                        .next_sibling()
                        .unwrap()
                        .downcast::<gtk::MenuButton>()
                        .unwrap();
                    assert!(!target.downcast_ref::<gtk::Button>().unwrap().has_frame());
                    assert!(!repeat.has_frame());
                    eprintln!(
                        "FORMAT136_SPLIT row={row} classes={:?} repeat_active={}",
                        split.css_classes(),
                        repeat.is_active()
                    );
                }
                if text_pointer_target(
                    &format!("FORMAT136_READY {} click", ticket.get()),
                    &pop,
                    &target,
                ) {
                    phase.set(2);
                }
            }
            2 if ack.exists() => {
                ticket.set(ticket.get() + 1);
                phase.set(3);
            }
            3 => {
                assert_eq!(
                    u.current.borrow().as_ref().unwrap().session.sel,
                    *selected.borrow().as_ref().unwrap(),
                    "opening {row} preserves the actual editing selection"
                );
                if header.label.text().as_str() != title {
                    page_wait.set(page_wait.get() + 1);
                    assert!(
                        page_wait.get() < 20,
                        "opening {row}: expected header {title}, actual={:?}, focus={:?}",
                        header.label.text(),
                        gtk::prelude::GtkWindowExt::focus(&w)
                            .map(|f| (f.type_().name(), f.widget_name()))
                    );
                    return glib::ControlFlow::Continue;
                }
                assert_eq!(header.label.text(), title);
                eprintln!("FORMAT136_PAGE family={family} row={row} title={title}");
                assert_eq!(
                    u.current.borrow().as_ref().unwrap().session.doc,
                    *original.borrow().as_ref().unwrap(),
                    "opening a property never edits content"
                );
                if row == "text-font-property" {
                    assert!(find_widget(&pop, "text-font-list").unwrap().is_mapped());
                    assert!(find_widget(&pop, "text-font-choices").is_none());
                    assert!(u.font_input.is_mapped());
                }
                if row == "text-language" {
                    assert!(find_widget(&pop, "language-search").unwrap().is_mapped());
                }
                if matches!(
                    row,
                    "text-ink" | "text-highlight" | "paragraph-more-shading"
                ) {
                    let picker = find_widget(
                        &pop,
                        match row {
                            "text-ink" => "text-ink-chooser",
                            "text-highlight" => "text-highlight-chooser",
                            _ => "paragraph-shading-chooser",
                        },
                    )
                    .unwrap()
                    .downcast::<gtk::ColorChooserWidget>()
                    .unwrap();
                    assert!(picker.is_mapped());
                    picker.set_show_editor(true);
                    phase.set(4);
                } else if row == "text-language" {
                    // The new page is mapped, but its long list has not yet
                    // received an allocation. Scroll only after that layout.
                    phase.set(6);
                } else {
                    phase.set(7);
                }
            }
            4 => {
                let picker = find_widget(
                    &pop,
                    match row {
                        "text-ink" => "text-ink-chooser",
                        "text-highlight" => "text-highlight-chooser",
                        _ => "paragraph-shading-chooser",
                    },
                )
                .unwrap();
                if let Some(entry) = editable_child(&picker) {
                    scroll_to_control(&entry);
                    if text_pointer_target(
                        &format!("FORMAT136_READY {} hex", ticket.get()),
                        &pop,
                        &entry,
                    ) {
                        phase.set(5);
                    }
                }
            }
            5 if ack.exists() => {
                ticket.set(ticket.get() + 1);
                let state = selection_character(&u);
                if row == "text-ink" {
                    assert_eq!(state["color"], json!({"Rgb":[35,69,103]}));
                } else if row == "paragraph-more-shading" {
                    assert_eq!(selection_paragraph(&u)["shading"], json!([35, 69, 103]));
                } else {
                    assert!(wordcraft_gnome::highlight::matches(
                        &u.current.borrow().as_ref().unwrap().session,
                        "234567"
                    ));
                }
                assert_eq!(
                    u.current.borrow().as_ref().unwrap().session.undo_depth(),
                    depth.get() + 1,
                    "one custom colour edit is one Undo step"
                );
                apply_user_edit(&u, Edit::Undo);
                assert_eq!(
                    u.current.borrow().as_ref().unwrap().session.doc,
                    *original.borrow().as_ref().unwrap()
                );
                phase.set(7);
            }
            6 => {
                let footer = find_widget(&pop, "install-dictionaries").unwrap();
                let scroll = footer
                    .ancestor(gtk::ScrolledWindow::static_type())
                    .unwrap()
                    .downcast::<gtk::ScrolledWindow>()
                    .unwrap();
                let ad = scroll.vadjustment();
                assert!(footer.is_mapped() && ad.page_size() > 0.);
                ad.set_value((ad.upper() - ad.page_size()).max(0.));
                eprintln!(
                    "FORMAT136_LANGUAGE_SCROLL upper={} page={} value={}",
                    ad.upper(),
                    ad.page_size(),
                    ad.value()
                );
                phase.set(7);
            }
            7 => {
                if row == "text-language" {
                    let footer = find_widget(&pop, "install-dictionaries").unwrap();
                    let scroll = footer
                        .ancestor(gtk::ScrolledWindow::static_type())
                        .unwrap()
                        .downcast::<gtk::ScrolledWindow>()
                        .unwrap();
                    let bounds = footer.compute_bounds(&scroll).unwrap();
                    assert!(
                        bounds.y() >= -1.
                            && bounds.y() + bounds.height() <= scroll.height() as f32 + 1.,
                        "final language action is fully reachable: {bounds:?}"
                    );
                }
                if text_pointer_target(
                    &format!("FORMAT136_READY {} click", ticket.get()),
                    &pop,
                    &header.button,
                ) {
                    phase.set(8);
                }
            }
            8 if ack.exists() => {
                ticket.set(ticket.get() + 1);
                let expected = match family {
                    0 => wordcraft_gnome::t!("Text"),
                    1 => wordcraft_gnome::t!("Paragraph"),
                    2 => wordcraft_gnome::t!("Style"),
                    _ => wordcraft_gnome::t!("Elements"),
                };
                assert_eq!(
                    header.label.text(),
                    expected,
                    "Back returns to the actual family home"
                );
                assert_eq!(
                    u.current.borrow().as_ref().unwrap().session.doc,
                    *original.borrow().as_ref().unwrap()
                );
                pop.popdown();
                index.set(index.get() + 1);
                phase.set(0);
            }
            _ => {}
        }
        glib::ControlFlow::Continue
    });
}
