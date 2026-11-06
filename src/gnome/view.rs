use super::*;

pub(super) fn texture(img: wordcraft_gnome::PageImage) -> gdk::MemoryTexture {
    let bytes = glib::Bytes::from_owned(img.rgba);
    gdk::MemoryTexture::new(
        img.width as i32,
        img.height as i32,
        gdk::MemoryFormat::R8g8b8a8Premultiplied,
        &bytes,
        img.width as usize * 4,
    )
}

pub(super) fn document_page_picture() -> gtk::Picture {
    let image = gtk::Picture::new();
    image.set_can_shrink(false);
    image.set_content_fit(gtk::ContentFit::Fill);
    image
}

pub(super) fn update_status(ui: &Rc<Ui>) {
    if let Some(ruler) = ui.ruler.borrow().as_ref() {
        ruler.refresh();
    }
    // Un cambio de tamaño puede emitir una señal de scroll de forma reentrante.
    let Ok(mut current) = ui.current.try_borrow_mut() else {
        return;
    };
    let pages_tip = current
        .as_ref()
        .filter(|d| !d.sizes.is_empty())
        .map(|d| {
            wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Pages: {current} of {total}"),
                &[
                    ("current", (d.active.min(d.sizes.len() - 1) + 1).to_string()),
                    ("total", d.sizes.len().to_string()),
                ],
            )
        })
        .unwrap_or_else(|| wordcraft_gnome::t!("Pages").into());
    if let Some(doc) = current.as_mut() {
        if doc.fit_to_width {
            doc.zoom = doc.page_zoom(doc.active);
        }
        ui.zoom_label
            .set_text(&format!("{:.0} %", doc.page_zoom(doc.active) * 100.0));
        ui.zoom_out
            .set_sensitive(doc.zoom > viewer_layout::MIN_ZOOM);
        ui.zoom_in.set_sensitive(doc.zoom < viewer_layout::MAX_ZOOM);
        if doc.status_active != Some(doc.active) {
            if let Some(previous) = doc.status_active {
                doc.thumb_buttons[previous].remove_css_class("selected-thumb");
            }
            doc.thumb_buttons[doc.active].add_css_class("selected-thumb");
            doc.status_active = Some(doc.active);
            ui.thumbnail_reveal_pending.set(true);
            let weak = Rc::downgrade(ui);
            glib::idle_add_local_once(move || {
                if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
                    reveal_active_thumbnail(&ui);
                }
            });
        }
    } else {
        ui.zoom_label.set_text("100 %");
    }
    // Updating tooltip text can synchronously query the canvas tooltip.
    drop(current);
    if ui.pages_button.tooltip_text().as_deref() != Some(pages_tip.as_str()) {
        ui.pages_button.set_tooltip_text(Some(&pages_tip));
        ui.pages_button
            .update_property(&[gtk::accessible::Property::Description(&pages_tip)]);
    }
}

pub(super) fn refresh_document_appearance(ui: &Rc<Ui>) {
    if adw::StyleManager::default().is_dark() {
        ui.sidebar.add_css_class("leaf-dark-document");
    } else {
        ui.sidebar.remove_css_class("leaf-dark-document");
    }
    if let Some(d) = ui.current.borrow_mut().as_mut() {
        d.painted_pages.clear();
        for p in &d.pages {
            p.set_paintable(None::<&gdk::Paintable>);
        }
        d.painted_thumbs.clear();
        for p in &d.thumbs {
            p.set_paintable(None::<&gdk::Paintable>);
        }
    }
    refresh_pages(ui);
    refresh_thumbnails(ui);
    refresh_selection(ui);
}

/// Viewport geometry is separate from the canonical editing/thumbnail target.
pub(super) fn viewport_page(ui: &Ui, doc: &Current) -> usize {
    doc.arrangement.page_at(
        ui.scroll.hadjustment().value() + ui.scroll.hadjustment().page_size() * 0.3,
        ui.scroll.vadjustment().value() + ui.scroll.vadjustment().page_size() * 0.3,
    )
}

pub(super) fn refresh_pages(ui: &Rc<Ui>) {
    let _perf = perf_scope("leaf.refresh_pages");
    sync_raster_scale(ui);
    let adj = ui.scroll.vadjustment();
    let top = adj.value();
    let bottom = top + adj.page_size().max(600.0);
    let mut errors = Vec::new();
    if let Ok(mut borrowed) = ui.current.try_borrow_mut() {
        if let Some(doc) = borrowed.as_mut() {
            // Raster visibility never changes the editing target or active thumbnail.
            let visible = doc.arrangement.visible(top, bottom);
            let keep: HashSet<usize> = visible.iter().copied().collect();
            let old: Vec<usize> = doc.painted_pages.difference(&keep).copied().collect();
            for index in old {
                doc.pages[index].set_paintable(None::<&gdk::Paintable>);
                doc.painted_pages.remove(&index);
            }
            for index in visible {
                if !doc.painted_pages.contains(&index) {
                    let zoom = doc.page_zoom(index);
                    match {
                        let _p = perf_scope("boundary.page_raster");
                        render_page(
                            &mut doc.session,
                            index,
                            zoom * doc.raster_scale as f32,
                            adw::StyleManager::default().is_dark(),
                        )
                    } {
                        Ok(img) => {
                            doc.pages[index].set_paintable(Some(&page_paintable::new(
                                img,
                                (doc.sizes[index].0 * zoom).ceil() as i32,
                                (doc.sizes[index].1 * zoom).ceil() as i32,
                            )));
                            doc.painted_pages.insert(index);
                        }
                        Err(error) => errors.push(error),
                    }
                }
            }
        }
    }
    for e in errors {
        ui.toasts.add_toast(Toast::new(&e));
    }
    update_status(ui);
}

pub(super) fn refresh_thumbnails(ui: &Rc<Ui>) {
    let _perf = perf_scope("leaf.refresh_thumbnails");
    sync_raster_scale(ui);
    if !ui.sidebar.shows_sidebar()
        || ui.sidebar_pages.visible_child_name().as_deref() != Some("navigation")
        || ui.navigation_pages.visible_child_name().as_deref() != Some("thumbnails")
    {
        return;
    }
    let adj = ui.thumbnails_scroll.vadjustment();
    let top = adj.value();
    let bottom = top + adj.page_size().max(600.0);
    if let Ok(mut borrowed) = ui.current.try_borrow_mut() {
        if let Some(doc) = borrowed.as_mut() {
            if doc.sizes.is_empty() {
                return;
            }
            let allocation = |i: usize| {
                perf_count("thumbnail_bounds", 1);
                doc.thumb_buttons[i]
                    .compute_bounds(&ui.thumbnails_box)
                    .map(|r| (r.y() as f64, (r.y() + r.height()) as f64))
                    .filter(|(start, end)| end > start && *end > 0.0)
            };
            // Newly appended GTK children can all report the same origin before
            // allocation. Treat that as an unallocated list, not every page visible.
            let allocated = match (allocation(0), allocation(doc.sizes.len() - 1)) {
                (Some(first), Some(last)) => doc.sizes.len() == 1 || last.0 >= first.1,
                _ => false,
            };
            let visible = viewer_layout::ordered_visible_range(
                doc.sizes.len(),
                top - 250.0,
                bottom + 250.0,
                |i| {
                    if allocated {
                        allocation(i).unwrap_or(doc.thumbnail_fallback[i])
                    } else {
                        doc.thumbnail_fallback[i]
                    }
                },
            );
            let old: Vec<usize> = doc
                .painted_thumbs
                .iter()
                .copied()
                .filter(|i| !visible.contains(i))
                .collect();
            for i in old {
                doc.painted_thumbs.remove(&i);
                doc.thumbs[i].set_paintable(None::<&gdk::Paintable>);
            }
            for i in visible {
                if !doc.painted_thumbs.contains(&i) {
                    let scale = viewer_layout::thumbnail_scale(doc.sizes[i].0);
                    if let Ok(image) = {
                        let _p = perf_scope("boundary.thumbnail_raster");
                        render_page(
                            &mut doc.session,
                            i,
                            scale * doc.raster_scale as f32,
                            adw::StyleManager::default().is_dark(),
                        )
                    } {
                        doc.thumbs[i].set_paintable(Some(&page_paintable::new(
                            image,
                            (doc.sizes[i].0 * scale).ceil() as i32,
                            (doc.sizes[i].1 * scale).ceil() as i32,
                        )));
                        doc.painted_thumbs.insert(i);
                    }
                }
            }
        }
    }
}

pub(super) fn jump_to(ui: &Rc<Ui>, index: usize) {
    {
        let mut current = ui.current.borrow_mut();
        let Some(doc) = current.as_mut() else { return };
        let layout = doc.session.layout();
        let Some(page) = layout.pages.get(index) else {
            return;
        };
        if let Some(pos) = layout.hit(
            index,
            page.body.x,
            page.body.y,
            wordcraft_doc::StoryRef::Body,
        ) {
            let _ = selection::set_caret(&mut doc.session, &pos, false);
        }
        doc.active = index;
        doc.session.page_hint = index;
    }
    refresh_selection(ui);
    if ui.sidebar.is_collapsed() {
        ui.sidebar.set_show_sidebar(false);
    }
    scroll_to_page(ui, index);
}

// A viewport refit preserves an open sidebar. Only explicit page navigation
// dismisses the collapsed overlay; background layout must not close review tools.
pub(super) fn scroll_to_page(ui: &Rc<Ui>, index: usize) {
    let target = {
        let borrowed = ui.current.borrow();
        borrowed
            .as_ref()
            .and_then(|doc| doc.arrangement.pages.get(index).map(|p| p.y))
    };
    if let Some(top) = target {
        let adj = ui.scroll.vadjustment();
        let max = (adj.upper() - adj.page_size()).max(0.0);
        adj.set_value((top - 10.0).max(0.0).min(max));
        refresh_pages(ui);
    }
}

pub(super) fn zoom_to(ui: &Rc<Ui>, zoom: f32) {
    if let Some(doc) = ui.current.borrow_mut().as_mut() {
        doc.fit_to_width = false;
    }
    apply_zoom(ui, zoom);
}

fn apply_zoom(ui: &Rc<Ui>, zoom: f32) {
    let zoom = viewer_layout::zoom_clamped(zoom);
    let current_page;
    {
        let mut borrowed = ui.current.borrow_mut();
        let Some(doc) = borrowed.as_mut() else { return };
        current_page = doc.active;
        doc.zoom = zoom;
    }
    ui.selection_zoom.set(zoom);
    layout_pages(ui);
    refresh_selection(ui);
    update_status(ui);
    let weak = Rc::downgrade(ui);
    // Wait for allocation without keeping a closed document alive.
    glib::idle_add_local_once(move || {
        if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
            scroll_to_page(&ui, current_page);
        }
    });
}

pub(super) fn zoom_step(ui: &Rc<Ui>, positive: bool) {
    let value = ui.current.borrow().as_ref().map(|d| d.zoom);
    let Some(zoom) = value else { return };
    let next = if positive {
        STEPS
            .iter()
            .copied()
            .find(|&v| v > zoom + 0.005)
            .unwrap_or(2.0)
    } else {
        STEPS
            .iter()
            .copied()
            .rev()
            .find(|&v| v < zoom - 0.005)
            .unwrap_or(0.5)
    };
    if let Some(doc) = ui.current.borrow_mut().as_mut() {
        doc.fit_to_width = false;
    }
    zoom_to(ui, next);
}

pub(super) fn refit_if_needed(ui: &Rc<Ui>) {
    let fit = ui
        .current
        .try_borrow()
        .ok()
        .and_then(|s| s.as_ref().map(|d| d.fit_to_width))
        .unwrap_or(false);
    if fit {
        fit_width(ui);
    }
}

pub(super) fn viewport_width(ui: &Ui) -> f64 {
    let split = ui.sidebar.width() as f64;
    let side = if ui.sidebar.shows_sidebar() && !ui.sidebar.is_collapsed() {
        ui.sidebar
            .sidebar()
            .map(|w| w.width() as f64)
            .unwrap_or(0.0)
    } else {
        0.0
    };
    (ui.scroll.width() as f64)
        .min((split - side).max(0.0))
        .max(250.0)
}

pub(super) fn layout_pages(ui: &Rc<Ui>) {
    let width = viewport_width(ui);
    let Ok(mut value) = ui.current.try_borrow_mut() else {
        return;
    };
    let Some(doc) = value.as_mut() else {
        return;
    };
    let scales = if doc.fit_to_width {
        viewer_layout::fitted_scales(&doc.sizes, &doc.sections, width)
    } else {
        vec![doc.zoom; doc.sizes.len()]
    };
    if scales != doc.page_scales {
        for (i, picture) in doc.pages.iter().enumerate() {
            picture.set_paintable(None::<&gdk::Paintable>);
            let w = (doc.sizes[i].0 * scales[i]).ceil() as i32;
            let h = (doc.sizes[i].1 * scales[i]).ceil() as i32;
            doc.page_frames[i].set_size_request(w, h);
            picture.set_size_request(w, h);
        }
        doc.painted_pages.clear();
        doc.page_scales = scales;
    }
    if doc.fit_to_width {
        doc.zoom = doc.page_zoom(doc.active);
    }
    let displayed: Vec<_> = doc
        .sizes
        .iter()
        .zip(&doc.page_scales)
        .map(|(&(w, h), z)| (w * z, h * z))
        .collect();
    let next = viewer_layout::arrange_sections(&displayed, &doc.sections, 1.0, width);
    for (frame, p) in doc.page_frames.iter().zip(&next.pages) {
        ui.document_box.move_(frame, p.x, p.y);
    }
    ui.document_box
        .set_size_request(next.width.ceil() as i32, next.height.ceil() as i32);
    doc.arrangement = next;
}

pub(super) fn fit_width(ui: &Rc<Ui>) {
    let width = viewport_width(ui);
    let zoom = {
        let mut value = ui.current.borrow_mut();
        let Some(doc) = value.as_mut() else { return };
        doc.fit_to_width = true;
        viewer_layout::fitted_scales(&doc.sizes, &doc.sections, width)
            .get(doc.active)
            .copied()
            .unwrap_or(1.0)
    };
    apply_zoom(ui, zoom);
}

pub(super) fn schedule_layout(ui: &Rc<Ui>) {
    if ui.layout_pending.replace(true) {
        return;
    }
    let weak = Rc::downgrade(ui);
    glib::timeout_add_local_once(std::time::Duration::from_millis(80), move || {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        ui.layout_pending.set(false);
        let fit = ui.current.borrow().as_ref().is_some_and(|d| d.fit_to_width);
        if fit {
            fit_width(&ui);
        } else {
            layout_pages(&ui);
            refresh_pages(&ui);
        }
    });
}

pub(super) fn clear_canvas(canvas: &impl IsA<gtk::Fixed>) {
    let canvas: &gtk::Fixed = canvas.as_ref();
    // Tool popovers belong to the tab, not to a page. Preserve their parent
    // and native surface when rebuilding pages or replacing the document.
    let mut child = canvas.first_child();
    while let Some(w) = child {
        child = w.next_sibling();
        if !w.is::<gtk::Popover>() {
            canvas.remove(&w);
        }
    }
}

pub(super) fn opened(ui: &Rc<Ui>, mut session: Session, label: &str) {
    let _perf = perf_scope("leaf.opened");
    *ui.table_pointer.borrow_mut() = TablePointer::default();
    ui.picture_drag.borrow_mut().take();
    ui.reference_hover.borrow_mut().take();
    ui.selection_rasters.borrow_mut().clear();
    wordcraft_gnome::proofing::configure(&mut session);
    if ui.closed.get() {
        return;
    }
    if let Some(s) = ui.spelling.borrow().as_ref() {
        s.invalidate();
    }
    if !session.recovery_ready() {
        if let Err(e) = session.enable_recovery(
            wordcraft_gnome::preferences::recovery_root(),
            std::sync::Arc::new(|| {}),
        ) {
            ui.toasts
                .add_toast(Toast::new(&wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Recovery: {e}"),
                    &[("e", format!("{}", e))],
                )));
        }
    }
    let layout = {
        let _p = perf_scope("boundary.layout_opened");
        session.layout()
    };
    let sizes: Vec<(f32, f32)> = layout.pages.iter().map(|p| (p.w, p.h)).collect();
    if sizes.is_empty() {
        ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
            "The document contains no pages"
        )));
        return;
    }
    let sections: Vec<_> = layout.pages.iter().map(|page| page.section).collect();
    let zoom = 1.0;
    let arrangement = viewer_layout::arrange_sections(&sizes, &sections, zoom, viewport_width(ui));
    ui.caret_motion.borrow_mut().retarget(None, 0, false);
    ui.search_highlights.borrow_mut().clear();
    if let Some(s) = ui.search.borrow().as_ref() {
        s.schedule();
    }
    if let Some(s) = ui.outline.borrow().as_ref() {
        s.schedule();
    }
    if let Some(s) = ui.annotations.borrow().as_ref() {
        s.schedule();
    }
    clear_canvas(&ui.document_box);
    clear_box(&ui.thumbnails_box);

    let mut pages = Vec::with_capacity(sizes.len());
    let mut page_frames = Vec::with_capacity(sizes.len());
    let mut selection_areas = Vec::with_capacity(sizes.len());
    let mut thumbs = Vec::with_capacity(sizes.len());
    let mut thumb_buttons = Vec::with_capacity(sizes.len());

    for (i, &(width, height)) in sizes.iter().enumerate() {
        perf_count("page_widgets", 1);
        let picture = document_page_picture();
        picture.set_size_request(width.ceil() as i32, height.ceil() as i32);
        picture.set_halign(gtk::Align::Center);
        picture.set_margin_top(11);
        picture.set_margin_bottom(11);
        let frame = gtk::Overlay::new();
        frame.add_css_class("document-sheet");
        frame.set_overflow(gtk::Overflow::Hidden);
        frame.set_size_request(width.ceil() as i32, height.ceil() as i32);
        frame.set_halign(gtk::Align::Center);
        frame.set_valign(gtk::Align::Start);
        picture.set_margin_top(0);
        picture.set_margin_bottom(0);
        frame.set_child(Some(&picture));
        let area = gtk::DrawingArea::new();
        area.set_can_target(false);
        area.set_halign(gtk::Align::Fill);
        area.set_valign(gtk::Align::Fill);
        {
            let data = ui.selection_data.clone();
            let motion = ui.caret_motion.clone();
            let search = ui.search_highlights.clone();
            let weak = Rc::downgrade(ui);
            area.set_draw_func(move |widget, cr, _, _| {
                let selection = data.borrow();
                let Some(ui_for_scale) = weak.upgrade() else {
                    return;
                };
                let z = ui_for_scale
                    .current
                    .try_borrow()
                    .ok()
                    .and_then(|c| c.as_ref().map(|d| f64::from(d.page_zoom(i))))
                    .unwrap_or(1.0);
                if let Some(ui) = weak.upgrade() {
                    if let Ok(mut value) = ui.current.try_borrow_mut() {
                        if let Some(d) = value.as_mut() {
                            if let Some(region) = header_footer::active(&d.session) {
                                let layout = d.session.layout();
                                if let Some(page) = layout.pages.get(i) {
                                    let y = if region == Region::Header {
                                        page.body.y
                                    } else {
                                        page.body.bottom()
                                    };
                                    cr.set_source_rgba(0.4, 0.5, 0.7, 0.7);
                                    cr.set_line_width(1.0);
                                    cr.set_dash(&[4.0, 4.0], 0.0);
                                    cr.move_to(0.0, y as f64 * z);
                                    cr.line_to(page.w as f64 * z, y as f64 * z);
                                    let _ = cr.stroke();
                                    cr.set_dash(&[], 0.0);
                                }
                            }
                        }
                    }
                }
                for r in search.borrow().iter().filter(|r| r.page == i) {
                    cr.set_source_rgba(0.94, 0.57, 0.12, 0.35);
                    cr.rectangle(r.x * z, r.y * z, r.width * z, r.height * z);
                    let _ = cr.fill();
                }
                if let Some(ui) = weak.upgrade() {
                    paint_reference_cue(&ui, widget, cr, i, z);
                    paint_selection(&ui, widget, cr, i, z, &selection.highlights);
                    paint_table_pointer(&ui, widget, cr, i, z);
                    paint_picture_frame(&ui, cr, i, z);
                }
                if let Some(ui) = weak.upgrade().filter(caret_is_shown) {
                    let now = widget.frame_clock().map(|c| c.frame_time()).unwrap_or(0);
                    let alpha = caret_opacity(&ui, now);
                    if let Some(r) = motion.borrow().position().filter(|r| r.page == i) {
                        let color = if adw::StyleManager::default().is_dark() {
                            1.0
                        } else {
                            0.0
                        };
                        cr.set_source_rgba(color, color, color, alpha);
                        let density = f64::from(widget.scale_factor().max(1));
                        cr.rectangle(
                            (r.x * z * density).round() / density,
                            (r.y * z * density).round() / density,
                            (r.width * z * density).round().max(1.0) / density,
                            (r.height * z * density).round().max(2.0) / density,
                        );
                        let _ = cr.fill();
                    }
                }
            });
        }
        frame.add_overlay(&area);
        let position = &arrangement.pages[i];
        ui.document_box.put(&frame, position.x, position.y);
        pages.push(picture);
        page_frames.push(frame);
        selection_areas.push(area);

        let thumb_scale = viewer_layout::thumbnail_scale(width);
        let thumb = gtk::Picture::new();
        thumb.set_can_shrink(false);
        thumb.set_content_fit(gtk::ContentFit::Fill);
        thumb.set_halign(gtk::Align::Center);
        thumb.set_hexpand(false);
        thumb.set_vexpand(false);
        thumb.set_size_request(
            (width * thumb_scale).ceil() as i32,
            (height * thumb_scale).ceil() as i32,
        );
        thumb.add_css_class("thumbnail-sheet");
        thumb.set_overflow(gtk::Overflow::Hidden);
        let number = gtk::Label::new(Some(&wordcraft_gnome::i18n::interpolate(
            wordcraft_gnome::t!("{current} of {total}"),
            &[
                ("current", (i + 1).to_string()),
                ("total", sizes.len().to_string()),
            ],
        )));
        number.add_css_class("dim-label");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 5);
        content.set_halign(gtk::Align::Center);
        content.set_vexpand(false);
        let button = gtk::Button::new();
        button.add_css_class("flat");
        button.add_css_class("thumbnail-button");
        button.set_halign(gtk::Align::Center);
        button.set_valign(gtk::Align::Start);
        button.set_vexpand(false);
        thumb.set_valign(gtk::Align::Center);
        content.append(&thumb);
        content.append(&number);
        button.set_child(Some(&content));
        button.set_margin_top(6);
        button.set_margin_bottom(10);
        {
            let ui = ui.clone();
            button.connect_clicked(glib::clone!(
                #[weak]
                ui,
                move |_| jump_to(&ui, i)
            ));
        }
        ui.thumbnails_box.append(&button);
        thumbs.push(thumb);
        thumb_buttons.push(button);
    }

    let mut offset = 12.0;
    let thumbnail_fallback = sizes
        .iter()
        .map(|&(w, h)| {
            let height =
                (f64::from(h) * f64::from(viewer_layout::thumbnail_scale(w))).ceil() + 46.0;
            let bounds = (offset, offset + height);
            offset += height;
            bounds
        })
        .collect();
    ui.current.replace(Some(Current {
        page_scales: vec![zoom; sizes.len()],
        session,
        sizes,
        sections,
        arrangement,
        zoom,
        fit_to_width: true,
        pages,
        page_frames,
        selection_areas,
        thumbs,
        thumb_buttons,
        raster_scale: 0,
        painted_pages: HashSet::new(),
        painted_thumbs: HashSet::new(),
        overlay_pages: HashSet::new(),
        overlay_header: None,
        thumbnail_fallback,
        active: 0,
        status_active: None,
    }));
    *ui.drag_anchor.borrow_mut() = None;
    ui.selection_zoom.set(zoom);
    ui.document_name.replace(label.to_owned());
    ui.dirty.set(false);
    sync_tab(ui);
    ui.save_path.replace(None);
    ui.source_path.replace(
        ui.current
            .borrow()
            .as_ref()
            .and_then(|d| d.session.path.clone()),
    );

    if let Some(a) = ui.adaptive.borrow().as_ref() {
        a.sync_width(ui.viewport_width.get());
    }
    ui.stack.set_visible_child_name("pages");
    layout_pages(ui);
    ui.scroll.vadjustment().set_value(0.0);
    ui.thumbnails_scroll.vadjustment().set_value(0.0);
    if let Some(outline) = ui.outline.borrow().as_ref() {
        outline.document_opened();
    }
    update_status(ui);
    refresh_pages(ui);
    refresh_thumbnails(ui);
    refresh_selection(ui);
    let weak = Rc::downgrade(ui);
    glib::idle_add_local_once(move || {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        refresh_pages(&ui);
        refresh_thumbnails(&ui);
    });
}

pub(super) fn schedule_thumbnail_refresh(ui: &Rc<Ui>) {
    if ui.thumbnail_refresh_pending.replace(true) {
        return;
    }
    let weak = Rc::downgrade(ui);
    glib::timeout_add_local_once(std::time::Duration::from_millis(120), move || {
        let Some(ui) = weak.upgrade() else {
            return;
        };
        ui.thumbnail_refresh_pending.set(false);
        if ui.closed.get() {
            return;
        }
        if let Some(doc) = ui.current.borrow_mut().as_mut() {
            for index in doc.painted_thumbs.drain() {
                doc.thumbs[index].set_paintable(None::<&gdk::Paintable>);
            }
        }
        refresh_thumbnails(&ui);
    });
}

/// Only called for an active-page change or when the Pages panel is revealed.
/// Ordinary sidebar browsing and redraws never snap back to this entry.
pub(super) fn reveal_active_thumbnail(ui: &Rc<Ui>) {
    if !ui.sidebar.shows_sidebar()
        || ui.sidebar_pages.visible_child_name().as_deref() != Some("navigation")
        || ui.navigation_pages.visible_child_name().as_deref() != Some("thumbnails")
    {
        return;
    }
    ui.thumbnail_reveal_pending.set(false);
    reveal_active_thumbnail_now(ui);
    let target = ui
        .current
        .borrow()
        .as_ref()
        .map(|d| (d.session.document_id(), d.active));
    let weak = Rc::downgrade(ui);
    let frames = Cell::new(0);
    // The selected CSS border and lazy paintable can change allocation after
    // the idle callback. Follow only this active-page event through layout;
    // ordinary redraws/manual browsing never schedule this correction.
    ui.thumbnails_box.add_tick_callback(move |_, _| {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return glib::ControlFlow::Break;
        };
        let current = ui
            .current
            .borrow()
            .as_ref()
            .map(|d| (d.session.document_id(), d.active));
        if current != target || !ui.sidebar.shows_sidebar() {
            return glib::ControlFlow::Break;
        }
        reveal_active_thumbnail_now(&ui);
        frames.set(frames.get() + 1);
        if frames.get() >= 2 {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn reveal_active_thumbnail_now(ui: &Rc<Ui>) {
    if !ui.sidebar.shows_sidebar()
        || ui.sidebar_pages.visible_child_name().as_deref() != Some("navigation")
        || ui.navigation_pages.visible_child_name().as_deref() != Some("thumbnails")
    {
        return;
    }
    let target = ui.current.try_borrow().ok().and_then(|c| {
        let d = c.as_ref()?;
        d.thumb_buttons
            .get(d.active)?
            .compute_bounds(&ui.thumbnails_box)
    });
    if let Some(r) = target {
        let a = ui.thumbnails_scroll.vadjustment();
        let top = r.y() as f64;
        let bottom = (r.y() + r.height()) as f64;
        let value = if top < a.value() {
            top
        } else if bottom > a.value() + a.page_size() {
            bottom - a.page_size()
        } else {
            return;
        };
        a.set_value(value.clamp(a.lower(), (a.upper() - a.page_size()).max(a.lower())));
    }
}
