use super::*;

pub(super) fn document_tools_fixture(ui: &Rc<Ui>) {
    let file = std::env::temp_dir().join(format!("wordcraft-picture-{}.png", std::process::id()));
    std::fs::write(
        &file,
        wordcraft_engine::cmd::insert::base64_decode(include_str!(
            "../../../tests/fixtures/picture.b64"
        ))
        .unwrap(),
    )
    .unwrap();
    let mut session = Session::new(wordcraft_doc::Document::from_text(
        "Documento con imagen\nUna imagen en línea conserva su lugar entre los párrafos.\n\nEl documento puede guardarse como DOCX u ODT y exportarse a PDF.",
    ));
    session
        .run("caret.set", &serde_json::json!({"pos":Pos::body(2,0)}))
        .unwrap();
    pictures::insert(&mut session, &file).unwrap();
    pictures::resize(&mut session, 240.0, 120.0, false, true).unwrap();
    opened(ui, session, "Documento con imagen.docx");
}

pub(super) fn document_tools_ui_test(
    ui: &Rc<Ui>,
    panel: &Rc<PicturePopover>,
    menu: &gtk::MenuButton,
    parent: &ApplicationWindow,
    app: &Application,
) {
    let ui = ui.clone();
    let panel = panel.clone();
    let menu = menu.clone();
    let parent = parent.clone();
    let app = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
        let (px, py, rev) = {
            let mut state = ui.current.borrow_mut();
            let d = state.as_mut().unwrap();
            let hit = d.session.layout().find_object(0, |_| true).unwrap();
            let frame = d.page_frames[hit.page].compute_bounds(&parent).unwrap();
            let point = (
                frame.x() as f64 + (hit.rect.x + hit.rect.w / 2.0) as f64 * d.zoom as f64,
                frame.y() as f64 + (hit.rect.y + hit.rect.h / 2.0) as f64 * d.zoom as f64,
                d.session.rev(),
            );
            d.session
                .run("caret.docStart", &serde_json::json!({}))
                .unwrap();
            point
        };
        refresh_selection(&ui);
        eprintln!("IMAGE_POINTER_READY {} {}", px as i32, py as i32);
        glib::timeout_add_local_once(std::time::Duration::from_millis(1500), move || {
            {
                let state = ui.current.borrow();
                let d = state.as_ref().unwrap();
                assert!(
                    wordcraft_engine::cmd::objects::object_selection(&d.session).is_some(),
                    "real pointer must select the picture"
                );
                assert_eq!(d.session.rev(), rev);
            }
            menu.popup();
            popover_pin(&panel.pop).set_active(true);
            panel.width.set_value(50.0);
            find_widget(&panel.pop.child().unwrap(), "picture-apply")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            {
                let state = ui.current.borrow();
                let (_, w, h) = pictures::selected(&state.as_ref().unwrap().session).unwrap();
                assert!((w / h - 2.0).abs() < 0.01);
                assert!((w as f64 / PT_PER_MM - 50.0).abs() < 0.1);
            }
            find_widget(&panel.pop.child().unwrap(), "picture-original")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            assert_eq!(
                pictures::selected(&ui.current.borrow().as_ref().unwrap().session)
                    .unwrap()
                    .1,
                120.0
            );
            find_widget(&panel.pop.child().unwrap(), "picture-delete")
                .unwrap()
                .downcast::<gtk::Button>()
                .unwrap()
                .emit_clicked();
            assert!(!panel.context.is_visible());
            apply_user_edit(&ui, Edit::Undo);
            assert!(panel.context.is_visible());
            panel.pop.popdown();
            let props = show_document_properties(&ui, &parent);
            props.width.set_value(297.0);
            props.height.set_value(210.0);
            props.top.set_value(20.0);
            props.bottom.set_value(20.0);
            props.left.set_value(25.0);
            props.right.set_value(25.0);
            props.apply.emit_clicked();
            {
                let mut state = ui.current.borrow_mut();
                let d = state.as_mut().unwrap();
                let p = PageSetup::current(&d.session);
                assert!(p.width > p.height);
                let (id, dirty, depth, rev) = (
                    d.session.document_id(),
                    d.session.dirty,
                    d.session.undo_depth(),
                    d.session.rev(),
                );
                pdf_export::export(&mut d.session, Path::new("document-tools-test.pdf")).unwrap();
                assert_eq!(
                    (
                        d.session.document_id(),
                        d.session.dirty,
                        d.session.undo_depth(),
                        d.session.rev()
                    ),
                    (id, dirty, depth, rev)
                );
            }
            assert!(!props.window.is_visible());
            assert_eq!(app.windows().len(), 1);
            eprintln!("DOCUMENT_TOOLS_OK");
            app.quit();
        });
    });
}
