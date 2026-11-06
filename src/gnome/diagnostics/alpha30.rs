use super::*;
use serde_json::json;

fn button(root: &impl IsA<gtk::Widget>, name: &str) -> gtk::Button {
    find_widget(root, name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .downcast()
        .unwrap()
}
pub(super) fn alpha30_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    window.set_default_size(1200, 950);
    window.set_title(Some("Leaf Alpha30 Regression"));
    let file = std::env::temp_dir().join(format!("leaf30-{}.png", std::process::id()));
    std::fs::write(
        &file,
        wordcraft_engine::cmd::insert::base64_decode(include_str!(
            "../../../tests/fixtures/picture.b64"
        ))
        .unwrap(),
    )
    .unwrap();
    let text = (0..140)
        .map(|i| format!("Paragraph {i} with enough text to produce a long document.\n"))
        .collect::<String>();
    let mut s = Session::new(wordcraft_doc::Document::from_text(&text));
    selection::set_caret(&mut s, &Pos::body(125, 0), false).unwrap();
    pictures::insert(&mut s, &file).unwrap();
    pictures::resize(&mut s, 150., 90., false, true).unwrap();
    opened(ui, s, "Viewport regression");
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        caret_ensure_visible(&u);
        refresh_selection(&u);
        let active = u.current.borrow().as_ref().unwrap().active;
        assert!(active > 0);
        u.scroll.vadjustment().set_value(0.);
        refresh_pages(&u);
        update_status(&u);
        assert_eq!(
            u.current.borrow().as_ref().unwrap().active,
            active,
            "scroll must not change active thumbnail"
        );
        caret_ensure_visible(&u);
        assert!(
            u.scroll.vadjustment().value() > 100.,
            "object selection reveals the image instead of a stale text caret"
        );
        let old = u.scroll.vadjustment().value();
        tools_change(&u, |s| pictures::resize(s, 190., 110., false, true));
        let (u, w, a) = (u.clone(), w.clone(), a.clone());
        glib::timeout_add_local_once(std::time::Duration::from_millis(450), move || {
            assert!(
                (u.scroll.vadjustment().value() - old).abs() < 80.,
                "image edit must preserve its viewport"
            );
            let mut s = Session::new(wordcraft_doc::Document::from_text(
                "First heading\nA paragraph\nSecond heading\nMore text",
            ));
            for n in [0, 2] {
                selection::set_caret(&mut s, &Pos::body(n, 0), false).unwrap();
                s.run("para.style", &json!({"style":"Heading1"})).unwrap();
            }
            selection::set_caret(&mut s, &Pos::body(0, 0), false).unwrap();
            wordcraft_gnome::references::insert_toc(
                &mut s,
                &json!({"title":"Contents","levels":3,"hyperlinks":true}),
            )
            .unwrap();
            opened(&u, s, "TOC interaction");
            let (u, w, a) = (u.clone(), w.clone(), a.clone());
            glib::timeout_add_local_once(std::time::Duration::from_millis(450), move || {
                alpha30_controls(&u, &w, &a)
            });
        });
    });
}
fn alpha30_controls(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let depth = ui.current.borrow().as_ref().unwrap().session.undo_depth();
    let tools = ui.text_tools.borrow().as_ref().unwrap().clone();
    let text = format_button(ui, "text-menu").unwrap();
    text.popup();
    tools.home();
    let pop = text.popover().unwrap();
    assert!(find_widget(&pop, "text-font-property").unwrap().is_mapped());
    assert!(!ui.font_input.is_mapped());
    button(&pop, "text-font-property").emit_clicked();
    assert!(gtk::prelude::WidgetExt::is_visible(&ui.font_input));
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.undo_depth(),
        depth
    );
    tools.home();
    tools.page("options");
    assert!(
        tools
            .palette
            .borrow()
            .as_ref()
            .unwrap()
            .superscript
            .parent()
            .as_ref()
            == Some(tools.detail.upcast_ref())
    );
    tools.home();
    immediate_format(ui, "format.font", json!({"name":"serif"})).unwrap();
    format_execute(ui, "leaf.proofText", json!({"choice":"es-ES"})).unwrap();
    {
        let mut c = ui.current.borrow_mut();
        selection::set_caret(&mut c.as_mut().unwrap().session, &Pos::body(3, 0), false).unwrap();
    }
    refresh_selection(ui);
    button(&pop, "text-font-apply").emit_clicked();
    assert_eq!(selection_character(ui)["font"], "serif");
    text.popup();
    tools.home();
    button(&pop, "text-language-apply").emit_clicked();
    assert_eq!(
        wordcraft_gnome::proofing_defaults::choice(&ui.current.borrow().as_ref().unwrap().session),
        "es-ES"
    );
    text.popdown();
    let para = ui.paragraph.borrow().as_ref().unwrap().clone();
    para.execute("para.shading", json!({"color":"DDEBF7"}));
    para.execute(
        "para.borders",
        json!({"kind":"outside","color":"2266AA","width":1.5}),
    );
    let props = selection_paragraph(ui);
    {
        let mut c = ui.current.borrow_mut();
        let end = c
            .as_ref()
            .unwrap()
            .session
            .doc
            .end_of(wordcraft_doc::StoryRef::Body);
        selection::set_caret(&mut c.as_mut().unwrap().session, &end, false).unwrap();
    }
    refresh_selection(ui);
    para.home();
    let undo = ui.current.borrow().as_ref().unwrap().session.undo_depth();
    button(&para.main, "paragraph-borders").emit_clicked();
    assert_eq!(selection_paragraph(ui)["borders"], props["borders"]);
    assert_eq!(selection_paragraph(ui)["shading"], props["shading"]);
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.undo_depth(),
        undo + 1
    );
    apply_user_edit(ui, Edit::Undo);
    let elements = ui.elements.borrow().as_ref().unwrap().clone();
    elements.show("image");
    assert!(
        !find_widget(&elements.pop, "picture-context")
            .unwrap()
            .is_sensitive()
    );
    assert!(
        find_widget(&elements.pop, "picture-context")
            .unwrap()
            .is_visible()
    );
    assert!(
        find_widget(&elements.pop, "picture-insert")
            .unwrap()
            .is_sensitive()
    );
    elements.pop.popdown();
    alpha30_all_repeats(ui);
    let outline = ui.outline.borrow().as_ref().unwrap().clone();
    outline.show();
    let filter = find_widget(&ui.navigation_pages, "navigation-filter")
        .unwrap()
        .downcast::<gtk::DropDown>()
        .unwrap();
    let document = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    filter.set_selected(3);
    assert!(outline.status.is_visible());
    filter.set_selected(5);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, document);
    filter.set_selected(0);
    // Return to the TOC and emit coordinates for a real ordinary pointer click.
    let entry = {
        let c = ui.current.borrow();
        let s = &c.as_ref().unwrap().session;
        s.doc
            .para_paths(wordcraft_doc::StoryRef::Body)
            .into_iter()
            .find(|p| {
                s.doc
                    .para(wordcraft_doc::StoryRef::Body, p)
                    .unwrap()
                    .props
                    .style
                    .as_deref()
                    == Some("TOC1")
            })
            .unwrap()
    };
    let pos = Pos {
        story: wordcraft_doc::StoryRef::Body,
        path: entry,
        off: 1,
    };
    let expected = {
        let c = ui.current.borrow();
        let s = &c.as_ref().unwrap().session;
        let url = wordcraft_gnome::references::toc_destination(s, &pos).unwrap();
        s.doc
            .bookmarks()
            .into_iter()
            .find(|(n, _)| Some(n.as_str()) == url.strip_prefix('#'))
            .unwrap()
            .1
    };
    {
        let mut c = ui.current.borrow_mut();
        selection::set_caret(&mut c.as_mut().unwrap().session, &pos, false).unwrap();
    }
    refresh_selection(ui);
    caret_ensure_visible(ui);
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    glib::timeout_add_local_once(std::time::Duration::from_millis(450), move || {
        let (x, y) = {
            let mut c = u.current.borrow_mut();
            let d = c.as_mut().unwrap();
            let c = d
                .session
                .layout()
                .caret_on(&pos, d.session.page_hint)
                .unwrap();
            let r = d.page_frames[c.page].compute_bounds(&w).unwrap();
            (
                r.x() as f64 + c.x as f64 * d.page_zoom(c.page) as f64 + 3.,
                r.y() as f64 + (c.top + c.height / 2.) as f64 * d.page_zoom(c.page) as f64,
            )
        };
        eprintln!(
            "ALPHA30_TOC_LINK {} {}",
            (x * w.scale_factor() as f64) as i32,
            (y * w.scale_factor() as f64) as i32
        );
        let tries = Cell::new(0);
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            tries.set(tries.get() + 1);
            assert!(
                tries.get() < 250,
                "ordinary TOC pointer navigation timed out"
            );
            let target = u
                .current
                .borrow()
                .as_ref()
                .unwrap()
                .session
                .sel
                .focus
                .clone();
            if target.path != expected.path {
                return glib::ControlFlow::Continue;
            }
            assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, document);
            eprintln!(
                "ALPHA30_UI_OK caret-page=true image-viewport=true font-language-repeat=true paragraph-pair=true disabled-elements=true type-filter=true toc-click=true"
            );
            a.quit();
            glib::ControlFlow::Break
        });
    });
}

fn alpha30_all_repeats(ui: &Rc<Ui>) {
    let restore = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "First paragraph\nSecond paragraph\nThird paragraph",
        )),
        "Property repeats",
    );
    let p = ui.paragraph.borrow().as_ref().unwrap().clone();
    let target = |row| {
        let mut c = ui.current.borrow_mut();
        selection::set_caret(&mut c.as_mut().unwrap().session, &Pos::body(row, 0), false).unwrap();
        drop(c);
        refresh_selection(ui);
    };
    p.execute("para.align", json!({"value":"center"}));
    target(1);
    p.home();
    button(&p.main, "paragraph-alignment-apply").emit_clicked();
    assert_eq!(selection_paragraph(ui)["align"], "center");
    p.execute("para.multilevel", json!({"kind":"outline"}));
    p.execute("para.listLevel", json!({"level":1}));
    target(2);
    p.home();
    let before = ui.current.borrow().as_ref().unwrap().session.undo_depth();
    button(&p.main, "paragraph-list-apply").emit_clicked();
    assert_eq!(
        selection_paragraph(ui)["numbering"]["level"],
        1,
        "repeat retains list family and level"
    );
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.undo_depth(),
        before + 1
    );
    p.execute(
        "para.indents",
        json!({"left":24.,"right":12.,"firstLine":6.}),
    );
    target(0);
    p.home();
    button(&p.main, "paragraph-more-indents-apply").emit_clicked();
    let state = selection_paragraph(ui);
    assert_eq!(state["indentLeft"], 24.);
    assert_eq!(state["indentRight"], 12.);
    assert_eq!(state["indentFirst"], 6.);
    p.execute("para.lineSpacing", json!({"value":1.5}));
    target(1);
    p.home();
    button(&p.main, "paragraph-spacing").emit_clicked();
    assert_eq!(selection_paragraph(ui)["lineSpacing"]["value"], 1.5);
    let editor = columns_editor(ui, || {});
    editor.controls.count.set_value(2.);
    editor.apply.emit_clicked();
    assert_eq!(
        document_properties::ColumnSetup::current(&ui.current.borrow().as_ref().unwrap().session)
            .count,
        2
    );
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "Another document\nOrdinary text",
        )),
        "Column repeat",
    );
    p.home();
    button(&p.main, "paragraph-columns-apply").emit_clicked();
    assert_eq!(
        document_properties::ColumnSetup::current(&ui.current.borrow().as_ref().unwrap().session)
            .count,
        2
    );
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.undo_depth(),
        1
    );
    opened(ui, Session::new(restore), "TOC interaction");
}
