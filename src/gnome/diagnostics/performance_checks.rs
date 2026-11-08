use super::*;

// GUI regressions for the host-only invalidation and visibility policies.
pub(super) fn host_cache_ui_test(ui: &Rc<Ui>, app: &Application) {
    ui.document_box
        .settings()
        .set_property("gtk-enable-animations", false);
    let text = (0..400)
        .map(|i| {
            format!(
                "Row {i}: {} useful document content café.",
                if i % 25 == 0 { "needle" } else { "ordinary" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let opening = perf_phase("open_unallocated", || {
        opened(
            ui,
            Session::new(wordcraft_doc::Document::from_text(&text)),
            "Host cache regression",
        )
    });
    let count = ui.current.borrow().as_ref().unwrap().sizes.len();
    assert!(
        opening["inclusive_spans"]["boundary.thumbnail_raster"]["calls"]
            .as_u64()
            .unwrap_or(0)
            < count as u64,
        "opening must not rasterize every unallocated thumbnail"
    );
    let u = ui.clone();
    let a = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
        let ui = &u;
        let pages = ui.current.borrow().as_ref().unwrap().sizes.len();
        assert!(pages > 5);
        let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
        let styles = ui.style.borrow().as_ref().unwrap().clone();
        styles.main();
        assert!(!styles.list.is_mapped());
        let result = perf_phase("cursor_regression", || {
            for _ in 0..40 {
                selected_command(ui, "right", false);
            }
        });
        assert!(
            result["counts"]["overlay_queue"].as_u64().unwrap() < 120,
            "only old/new caret layers need repainting"
        );
        assert_eq!(
            result["counts"]["toc_lookup_scans"].as_u64().unwrap_or(0),
            0,
            "caret moves must reuse the current revision's TOC ranges"
        );
        assert!(
            result["counts"]["ruler_refresh"].as_u64().unwrap_or(0) <= 80,
            "one selection refresh must not repeat the same ruler work"
        );
        assert_eq!(
            result["counts"]["style_rows_visited"].as_u64().unwrap_or(0),
            0,
            "a populated hidden style list must not be traversed on caret moves"
        );
        let style_menu = format_button(ui, "style-menu").unwrap();
        style_menu.popup();
        perf_pump();
        assert!(styles.list.is_mapped());
        let shown = perf_phase("visible_styles", || styles.sync_selection());
        assert!(shown["counts"]["style_rows_visited"].as_u64().unwrap() > 0);
        let apply = find_widget(&styles.list, "style-apply-Normal").unwrap();
        assert!(apply.parent().unwrap().has_css_class("current-style"));
        style_menu.popover().unwrap().popdown();
        perf_pump();
        let window = ui.host.upgrade().unwrap().window.clone();
        let context = show_document_context(ui, &window, 60, 80, DocumentContext::Text);
        context.set_visible_submenu(Some(wordcraft_gnome::t!("Style")));
        perf_pump();
        assert!(styles.list.is_mapped());
        assert!(!style_menu.popover().unwrap().is_mapped());
        let embedded = perf_phase("contextual_styles", || styles.sync_selection());
        assert!(embedded["counts"]["style_rows_visited"].as_u64().unwrap() > 0);
        context.popdown();
        perf_pump();
        selected_command(ui, "select-all", false);
        let marked = ui.current.borrow().as_ref().unwrap().overlay_pages.len();
        assert!(marked > 5);
        let clear = perf_phase("clear_selection", || selected_command(ui, "docEnd", false));
        assert!(
            clear["counts"]["overlay_queue"].as_u64().unwrap() >= marked as u64,
            "erase every formerly selected page"
        );
        assert_eq!(ui.current.borrow().as_ref().unwrap().overlay_pages.len(), 1);
        let search = ui.search.borrow().as_ref().unwrap().clone();
        search.show(false);
        search.query.set_text("needle");
        search.refresh();
        assert_eq!(search.hits.borrow().len(), 16);
        let search_pages = ui.current.borrow().as_ref().unwrap().overlay_pages.len();
        assert!(search_pages > 2);
        let clear = perf_phase("close_search", || search.toggle.set_active(false));
        assert!(clear["counts"]["overlay_queue"].as_u64().unwrap() >= search_pages as u64);
        assert_eq!(ui.current.borrow().as_ref().unwrap().overlay_pages.len(), 1);
        let hidden = perf_phase("hidden_search", || {
            search.refresh();
            search.schedule();
        });
        assert!(
            hidden["inclusive_spans"]
                .get("boundary.search_query")
                .is_none()
        );
        selected_command(ui, "docStart", false);
        apply_user_edit(ui, Edit::Insert("needle ".into()));
        search.show(false);
        search.refresh();
        assert_eq!(
            search.hits.borrow().len(),
            17,
            "reopening search must refresh stale hidden results"
        );
        apply_user_edit(ui, Edit::Undo);
        search.refresh();
        assert_eq!(search.hits.borrow().len(), 16);
        // Simulate the coalesced edit invalidation while thumbnails are hidden.
        {
            let mut d = ui.current.borrow_mut();
            let d = d.as_mut().unwrap();
            for i in d.painted_thumbs.drain() {
                d.thumbs[i].set_paintable(None::<&gdk::Paintable>);
            }
        }
        search.toggle.set_active(false);
        assert!(
            !ui.current
                .borrow()
                .as_ref()
                .unwrap()
                .painted_thumbs
                .is_empty(),
            "closing search restores invalidated thumbnail rasters immediately"
        );
        assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, original);
        for fraction in [0.0, 0.5, 1.0] {
            let adj = ui.thumbnails_scroll.vadjustment();
            adj.set_value((adj.upper() - adj.page_size()).max(0.0) * fraction);
            refresh_thumbnails(ui);
            let d = ui.current.borrow();
            let d = d.as_ref().unwrap();
            let top = adj.value();
            let bottom = top + adj.page_size().max(600.0);
            let content = ui.thumbnails_box.parent().unwrap();
            let expected: HashSet<_> = d
                .thumb_buttons
                .iter()
                .enumerate()
                .filter_map(|(i, b)| {
                    let r = b.compute_bounds(&content).unwrap();
                    let start = r.y() as f64;
                    let end = (r.y() + r.height()) as f64;
                    (start <= bottom + 250.0 && end >= top - 250.0).then_some(i)
                })
                .collect();
            assert_eq!(
                d.painted_thumbs, expected,
                "indexed lookup must match all actual GTK allocations"
            );
        }
        verify_toc_cache_revision_and_identity();
        eprintln!("LEAF_HOST_CACHE_UI_OK pages={pages}");
        a.quit();
    });
}

/// Range caches are keyed by both document identity and content revision.
fn verify_toc_cache_revision_and_identity() {
    let mut source = Session::new(wordcraft_doc::Document::from_text("Heading\nBody"));
    source.run("para.heading1", &serde_json::json!({})).unwrap();
    source
        .run("references.toc", &serde_json::json!({"levels":2}))
        .unwrap();
    let mut session = Session::new(source.doc.clone());
    let position = wordcraft_gnome::references::toc_field(&session).unwrap().0;
    let mut cache = TocCache::default();
    let expected = wordcraft_gnome::references::toc_span_at(&session, &position);
    assert!(expected.is_some());
    let first = perf_phase("toc_initial", || {
        assert_eq!(cache.at(&session, &position), expected)
    });
    assert_eq!(first["counts"]["toc_lookup_scans"], 1);
    let warm = perf_phase("toc_warm", || {
        assert_eq!(cache.at(&session, &position), expected)
    });
    assert_eq!(warm["counts"]["toc_lookup_scans"].as_u64().unwrap_or(0), 0);
    let end = session.doc.end_of(wordcraft_doc::StoryRef::Body);
    selection::set_caret(&mut session, &end, false).unwrap();
    session
        .run("text.insert", &serde_json::json!({"text":" changed"}))
        .unwrap();
    for command in [None, Some("edit.undo"), Some("edit.redo")] {
        if let Some(command) = command {
            session.run(command, &serde_json::json!({})).unwrap();
        }
        let expected = wordcraft_gnome::references::toc_span_at(&session, &position);
        let changed = perf_phase("toc_new_revision", || {
            assert_eq!(cache.at(&session, &position), expected)
        });
        assert_eq!(changed["counts"]["toc_lookup_scans"], 1);
    }
    let fresh = Session::new(source.doc);
    assert!(cache.at(&fresh, &position).is_some());
    let other = Session::new(wordcraft_doc::Document::from_text("No table of contents"));
    assert_eq!(fresh.rev(), other.rev());
    assert!(cache.at(&other, &Pos::body(0, 0)).is_none());
}
