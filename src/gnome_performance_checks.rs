// GUI regressions for the host-only invalidation and visibility policies.
fn host_cache_ui_test(ui:&Rc<Ui>,app:&Application){
    ui.document_box.settings().set_property("gtk-enable-animations",false);
    let text=(0..400).map(|i|format!("Row {i}: {} useful document content café.",if i%25==0{"needle"}else{"ordinary"})).collect::<Vec<_>>().join("\n");
    let opening=perf_phase("open_unallocated",||opened(ui,Session::new(wordcraft_doc::Document::from_text(&text)),"Host cache regression"));
    let count=ui.current.borrow().as_ref().unwrap().sizes.len();
    assert!(opening["inclusive_spans"]["boundary.thumbnail_raster"]["calls"].as_u64().unwrap_or(0)<count as u64,"opening must not rasterize every unallocated thumbnail");
    let u=ui.clone();let a=app.clone();glib::timeout_add_local_once(std::time::Duration::from_millis(500),move||{
        let ui=&u;let pages=ui.current.borrow().as_ref().unwrap().sizes.len();assert!(pages>5);
        let original=ui.current.borrow().as_ref().unwrap().session.doc.clone();
        let result=perf_phase("cursor_regression",||{for _ in 0..40{selected_command(ui,"right",false);}});
        assert!(result["counts"]["overlay_queue"].as_u64().unwrap()<120,"only old/new caret layers need repainting");
        selected_command(ui,"select-all",false);
        let marked=ui.current.borrow().as_ref().unwrap().overlay_pages.len();assert!(marked>5);
        let clear=perf_phase("clear_selection",||selected_command(ui,"docEnd",false));
        assert!(clear["counts"]["overlay_queue"].as_u64().unwrap()>=marked as u64,"erase every formerly selected page");
        assert_eq!(ui.current.borrow().as_ref().unwrap().overlay_pages.len(),1);
        let search=ui.search.borrow().as_ref().unwrap().clone();search.show(false);search.query.set_text("needle");search.refresh();assert_eq!(search.hits.borrow().len(),16);
        let search_pages=ui.current.borrow().as_ref().unwrap().overlay_pages.len();assert!(search_pages>2);
        let clear=perf_phase("close_search",||search.toggle.set_active(false));
        assert!(clear["counts"]["overlay_queue"].as_u64().unwrap()>=search_pages as u64);
        assert_eq!(ui.current.borrow().as_ref().unwrap().overlay_pages.len(),1);
        let hidden=perf_phase("hidden_search",||{search.refresh();search.schedule();});
        assert!(hidden["inclusive_spans"].get("boundary.search_query").is_none());
        selected_command(ui,"docStart",false);apply_user_edit(ui,Edit::Insert("needle ".into()));
        search.show(false);search.refresh();assert_eq!(search.hits.borrow().len(),17,"reopening search must refresh stale hidden results");
        apply_user_edit(ui,Edit::Undo);search.refresh();assert_eq!(search.hits.borrow().len(),16);search.toggle.set_active(false);
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,original);
        for fraction in [0.0,0.5,1.0]{
            let adj=ui.thumbnails_scroll.vadjustment();adj.set_value((adj.upper()-adj.page_size()).max(0.0)*fraction);refresh_thumbnails(ui);
            let d=ui.current.borrow();let d=d.as_ref().unwrap();let top=adj.value();let bottom=top+adj.page_size().max(600.0);
            let expected:HashSet<_>=d.thumb_buttons.iter().enumerate().filter_map(|(i,b)|{let r=b.compute_bounds(&ui.thumbnails_box).unwrap();let start=r.y()as f64;let end=(r.y()+r.height())as f64;(start<=bottom+250.0&&end>=top-250.0).then_some(i)}).collect();
            assert_eq!(d.painted_thumbs,expected,"indexed lookup must match all actual GTK allocations");
        }
        eprintln!("LEAF_HOST_CACHE_UI_OK pages={pages}");a.quit();
    });
}
