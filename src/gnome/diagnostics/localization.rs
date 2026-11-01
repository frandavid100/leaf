use super::*;

// Focused end-to-end checks run in separate processes for each locale.
pub(super) fn localization_ui_test(ui: &Rc<Ui>, app: &Application) {
    let language = std::env::var("LANGUAGE").unwrap_or_default();
    let spanish = language.starts_with("es");
    assert_eq!(
        wordcraft_gnome::t!("New Document"),
        if spanish {
            "Documento nuevo"
        } else {
            "New Document"
        }
    );
    assert_eq!(
        ui.document_name.borrow().as_str(),
        wordcraft_gnome::t!("New Document")
    );
    assert_eq!(
        wordcraft_gnome::t!("Save"),
        if spanish { "Guardar" } else { "Save" }
    );
    let menu = gtk::MenuButton::new();
    install_main_menu(&menu);
    let section = menu.menu_model().unwrap().item_link(1, "section").unwrap();
    let label = section
        .item_attribute_value(0, "label", None)
        .unwrap()
        .get::<String>()
        .unwrap();
    assert_eq!(
        label,
        if spanish {
            "Ventana nueva"
        } else {
            "New Window"
        }
    );
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let rev = ui.current.borrow().as_ref().unwrap().session.rev();
    let selection = ui.current.borrow().as_ref().unwrap().session.sel.clone();
    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    let weak = Rc::downgrade(ui);
    let app = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(350), move || {
        let ui = weak.upgrade().unwrap();
        assert!(ui.sidebar.has_css_class("leaf-dark-document"));
        {
            let mut current = ui.current.borrow_mut();
            let doc = current.as_mut().unwrap();
            assert_eq!(
                page_paintable::paper_pixel(&doc.pages[0].paintable().unwrap()),
                [51, 51, 51, 255]
            );
            assert_eq!(
                page_paintable::paper_pixel(&doc.thumbs[0].paintable().unwrap()),
                [51, 51, 51, 255]
            );
            assert_eq!(doc.session.doc, before);
            assert_eq!(doc.session.rev(), rev);
            assert_eq!(doc.session.sel, selection);
            let dark =
                wordcraft_gnome::render_page_screen(&mut doc.session, 0, 0.25, true).unwrap();
            assert_eq!(&dark.rgba[..4], &[51, 51, 51, 255]);
            let light =
                wordcraft_gnome::render_page_premultiplied(&mut doc.session, 0, 0.25).unwrap();
            assert_eq!(&light.rgba[..4], &[255, 255, 255, 255]);
            assert_eq!(doc.session.doc, before);
        }
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceLight);
        let weak = Rc::downgrade(&ui);
        glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            let ui = weak.upgrade().unwrap();
            assert!(!ui.sidebar.has_css_class("leaf-dark-document"));
            let current = ui.current.borrow();
            let doc = current.as_ref().unwrap();
            assert_eq!(
                page_paintable::paper_pixel(&doc.pages[0].paintable().unwrap()),
                [255, 255, 255, 255]
            );
            assert_eq!(
                page_paintable::paper_pixel(&doc.thumbs[0].paintable().unwrap()),
                [255, 255, 255, 255]
            );
            assert_eq!(doc.session.doc, before);
            eprintln!("LOCALIZATION_UI_OK language={language}");
            app.quit();
        });
    });
}
