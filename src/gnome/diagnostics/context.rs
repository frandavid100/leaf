use super::*;

fn has_submenu(menu: &gio::MenuModel, label: &str) -> bool {
    (0..menu.n_items()).any(|i| {
        (menu
            .item_attribute_value(i, "label", None)
            .and_then(|v| v.get::<String>())
            .as_deref()
            == Some(label)
            && menu.item_link(i, "submenu").is_some())
            || menu
                .item_link(i, "section")
                .is_some_and(|section| has_submenu(&section, label))
    })
}

pub(super) fn context_ui_test(ui: &Rc<Ui>, parent: &ApplicationWindow, app: &Application) {
    use gdk::prelude::PopupExt;
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "Hello world, texto de prueba\nSegundo párrafo",
        )),
        wordcraft_gnome::t!("Contexts"),
    );
    {
        let mut value = ui.current.borrow_mut();
        let d = value.as_mut().unwrap();
        d.session
            .run(
                "select.range",
                &serde_json::json!({"anchor":Pos::body(0,0),"focus":Pos::body(0,11)}),
            )
            .unwrap();
    }
    refresh_selection(ui);
    let ui = ui.clone();
    let parent = parent.clone();
    let app = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
        let (x, y, sel, rev) = {
            let mut value = ui.current.borrow_mut();
            let d = value.as_mut().unwrap();
            let c = d.session.layout().caret_on(&Pos::body(0, 4), 0).unwrap();
            let r = d.page_frames[c.page].compute_bounds(&parent).unwrap();
            (
                r.x() as f64 + c.x as f64 * d.zoom as f64,
                r.y() as f64 + (c.top + c.height / 2.0) as f64 * d.zoom as f64,
                d.session.sel.clone(),
                d.session.rev(),
            )
        };
        eprintln!("CONTEXT_POINTER_READY {} {}", x as i32, y as i32);
        glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
            {
                let value = ui.current.borrow();
                let d = value.as_ref().unwrap();
                assert_eq!(
                    d.session.sel, sel,
                    "right click must preserve selected text"
                );
                assert_eq!(d.session.rev(), rev);
            }
            let pop = find_widget(&ui.document_box, "document-context-menu")
                .unwrap()
                .downcast::<gtk::PopoverMenu>()
                .unwrap();
            assert!(pop.is_mapped());
            assert!(
                !ui.document_box.has_focus(),
                "context menu must receive keyboard focus"
            );
            assert_eq!(pop.menu_model().unwrap().n_items(), 3);
            let row = context_model_row(&pop, wordcraft_gnome::t!("Copy")).unwrap();
            let r = row.compute_bounds(&pop).unwrap();
            let surface = pop.surface().unwrap().downcast::<gdk::Popup>().unwrap();
            let (tx, ty) = pop.surface_transform();
            eprintln!(
                "CONTEXT_ACTION_READY {} {}",
                (surface.position_x() as f64 - tx + r.x() as f64 + r.width() as f64 / 2.0) as i32,
                (surface.position_y() as f64 - ty + r.y() as f64 + r.height() as f64 / 2.0) as i32
            );
            glib::timeout_add_local_once(std::time::Duration::from_millis(700), move || {
                assert!(
                    RICH_CLIPBOARD.with(|c| c.borrow().is_some()),
                    "real native row click must copy"
                );
                assert!(
                    ui.document_box.has_focus(),
                    "closing context must return keyboard focus to editor"
                );

                {
                    let mut value = ui.current.borrow_mut();
                    selection::set_caret(
                        &mut value.as_mut().unwrap().session,
                        &Pos::body(1, 0),
                        false,
                    )
                    .unwrap();
                }
                let edit = show_document_context(&ui, &parent, 100, 150, DocumentContext::Text);
                context_activate_row(&edit, wordcraft_gnome::t!("Paste"));
                assert!(ui.current.borrow().as_ref().unwrap().session.can_undo());
                apply_user_edit(&ui, Edit::Undo);
                {
                    let mut value = ui.current.borrow_mut();
                    value
                        .as_mut()
                        .unwrap()
                        .session
                        .run(
                            "select.range",
                            &serde_json::json!({"anchor":Pos::body(0,0),"focus":Pos::body(0,11)}),
                        )
                        .unwrap();
                }
                let edit = show_document_context(&ui, &parent, 100, 150, DocumentContext::Text);
                context_activate_row(&edit, wordcraft_gnome::t!("Cut"));
                assert_eq!(
                    ui.current
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        .session
                        .doc
                        .para_at(&Pos::body(0, 0))
                        .unwrap()
                        .text,
                    ", texto de prueba"
                );
                apply_user_edit(&ui, Edit::Undo);
                let edit = show_document_context(&ui, &parent, 100, 150, DocumentContext::Text);
                context_activate_row(&edit, wordcraft_gnome::t!("Select All"));
                assert!(
                    !ui.current
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .session
                        .sel
                        .is_collapsed()
                );
                for (label, name) in [
                    (wordcraft_gnome::t!("Style"), "style-menu"),
                    (wordcraft_gnome::t!("Paragraph"), "paragraph-menu"),
                    (wordcraft_gnome::t!("Text"), "text-menu"),
                ] {
                    let edit = show_document_context(&ui, &parent, 100, 150, DocumentContext::Text);
                    context_activate_row(&edit, label);
                    let m = format_button(&ui, name).unwrap();
                    assert!(
                        !m.popover().unwrap().is_visible(),
                        "context must not open toolbar popup"
                    );
                    assert!(
                        m.popover().unwrap().child().is_none(),
                        "controls must be embedded in context"
                    );
                    assert_eq!(edit.visible_submenu().as_deref(), Some(label));
                    edit.popdown();
                    assert!(
                        m.popover().unwrap().child().is_some(),
                        "closing context must return controls"
                    );
                }

                {
                    let mut value = ui.current.borrow_mut();
                    value
                        .as_mut()
                        .unwrap()
                        .session
                        .run(
                            "select.range",
                            &serde_json::json!({"anchor":Pos::body(0,0),"focus":Pos::body(0,11)}),
                        )
                        .unwrap();
                }
                // Clicking outside the selection moves the caret, without a mutation.
                let (cx, cy) = {
                    let mut value = ui.current.borrow_mut();
                    let d = value.as_mut().unwrap();
                    let c = d.session.layout().caret_on(&Pos::body(1, 3), 0).unwrap();
                    let r = d.page_frames[c.page]
                        .compute_bounds(&ui.document_box)
                        .unwrap();
                    (
                        r.x() as f64 + c.x as f64 * d.zoom as f64,
                        r.y() as f64 + (c.top + c.height / 2.0) as f64 * d.zoom as f64,
                    )
                };
                assert_eq!(context_prepare(&ui, cx, cy), Some(DocumentContext::Text));
                assert!(
                    ui.current
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .session
                        .sel
                        .is_collapsed()
                );
                table_command(&ui, "insert.table", serde_json::json!({"rows":2,"cols":2}));
                let table = show_document_context(&ui, &parent, 100, 150, DocumentContext::Table);
                assert!(has_submenu(
                    &table.menu_model().unwrap(),
                    wordcraft_gnome::t!("Add")
                ));
                context_activate_row(&table, wordcraft_gnome::t!("Select"));
                context_activate_row(&table, wordcraft_gnome::t!("Column"));
                assert!(
                    wordcraft_gnome::tables::selected_cells(
                        &ui.current.borrow().as_ref().unwrap().session
                    ) > 1
                );
                let table = show_document_context(&ui, &parent, 100, 150, DocumentContext::Table);
                context_activate_row(&table, wordcraft_gnome::t!("More Table Options…"));
                assert!(ui.table_popover.borrow().as_ref().unwrap().is_visible());
                ui.table_popover.borrow().as_ref().unwrap().popdown();
                document_tools_fixture(&ui);
                let u = ui.clone();
                let p = parent.clone();
                let a = app.clone();
                glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
                    let (x, y, rev) = {
                        let mut value = u.current.borrow_mut();
                        let d = value.as_mut().unwrap();
                        let hit = d.session.layout().find_object(0, |_| true).unwrap();
                        let r = d.page_frames[hit.page]
                            .compute_bounds(&u.document_box)
                            .unwrap();
                        let rect = hit.bounds();
                        let point = (
                            r.x() as f64 + (rect.x + rect.w / 2.0) as f64 * d.zoom as f64,
                            r.y() as f64 + (rect.y + rect.h / 2.0) as f64 * d.zoom as f64,
                            d.session.rev(),
                        );
                        d.session
                            .run("caret.docStart", &serde_json::json!({}))
                            .unwrap();
                        point
                    };
                    assert_eq!(context_prepare(&u, x, y), Some(DocumentContext::Picture));
                    assert_eq!(u.current.borrow().as_ref().unwrap().session.rev(), rev);
                    let pop =
                        show_document_context(&u, &p, x as i32, y as i32, DocumentContext::Picture);
                    context_activate_row(&pop, wordcraft_gnome::t!("Image Size…"));
                    assert!(u.picture.borrow().as_ref().unwrap().pop.is_visible());
                    u.picture.borrow().as_ref().unwrap().pop.popdown();
                    let pop =
                        show_document_context(&u, &p, x as i32, y as i32, DocumentContext::Picture);
                    context_activate_row(&pop, wordcraft_gnome::t!("Copy"));
                    // A separate document must receive the image fragment, not U+FFFC.
                    opened(
                        &u,
                        Session::new(wordcraft_doc::Document::from_text("")),
                        wordcraft_gnome::t!("Destination"),
                    );
                    context_paste(&u);
                    assert!(
                        u.current
                            .borrow_mut()
                            .as_mut()
                            .unwrap()
                            .session
                            .layout()
                            .find_object(0, |_| true)
                            .is_some()
                    );
                    apply_user_edit(&u, Edit::Undo);
                    assert!(
                        u.current
                            .borrow_mut()
                            .as_mut()
                            .unwrap()
                            .session
                            .layout()
                            .find_object(0, |_| true)
                            .is_none()
                    );
                    // An external clipboard owner invalidates our rich fragment.
                    u.document_box.clipboard().set_text("External");
                    context_paste(&u);
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(400),
                        move || {
                            let mut state = u.current.borrow_mut();
                            let d = state.as_mut().unwrap();
                            assert!(d.session.layout().find_object(0, |_| true).is_none());
                            let v = d
                                .session
                                .run("select.all", &serde_json::json!({"whole":true}))
                                .unwrap();
                            drop(v);
                            assert_eq!(selection::copy_text(&mut d.session).unwrap(), "External");
                            eprintln!("CONTEXT_UI_OK");
                            a.quit();
                        },
                    );
                });
            });
        });
    });
}
