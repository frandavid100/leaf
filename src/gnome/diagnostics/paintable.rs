use super::*;

pub(super) fn hidpi_ui_test(ui:&Rc<Ui>,app:&Application){
    opened(ui,sample_session(),"Prueba HiDPI");
    ui.sidebar.set_show_sidebar(true);
    ui.current.borrow_mut().as_mut().unwrap().fit_to_width=false;
    zoom_to(ui,1.0);
    let u=ui.clone();let a=app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(350),move||{
        refresh_pages(&u);refresh_thumbnails(&u);hidpi_assert(&u,1.0);
        let identity=document_identity(&u);
        u.current.borrow_mut().as_mut().unwrap().raster_scale=0;
        refresh_pages(&u);refresh_thumbnails(&u);
        assert_eq!(document_identity(&u),identity);
        zoom_to(&u,2.0);
        glib::timeout_add_local_once(std::time::Duration::from_millis(350),move||{
            refresh_pages(&u);refresh_thumbnails(&u);hidpi_assert(&u,2.0);
            assert_eq!(document_identity(&u),identity);
            eprintln!("HIDPI_UI_OK scale={}",u.document_box.scale_factor());a.quit();
        });
    });
}