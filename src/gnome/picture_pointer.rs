use super::*;
use serde_json::json;

type PictureRect = (f32, f32, f32, f32);
pub(super) struct PictureDrag {
    document: (u64, u64),
    pos: Pos,
    page: usize,
    target_page: usize,
    edges: (i8, i8),
    inline: bool,
    rect: PictureRect,
    preview: PictureRect,
    start: (f64, f64),
    grab: (f32, f32),
    zoom: f32,
    moved: bool,
    valid: bool,
    destination: Option<Pos>,
    image: Option<gtk::cairo::ImageSurface>,
}

fn edges_at(rect: PictureRect, px: f32, py: f32, reach: f32) -> Option<(i8, i8)> {
    let (x, y, w, h) = rect;
    if px < x - reach || px > x + w + reach || py < y - reach || py > y + h + reach {
        return None;
    }
    let ex = if (px - x).abs() <= reach {
        -1
    } else if (px - x - w).abs() <= reach {
        1
    } else {
        0
    };
    let ey = if (py - y).abs() <= reach {
        -1
    } else if (py - y - h).abs() <= reach {
        1
    } else {
        0
    };
    Some((ex, ey))
}

pub(super) fn picture_drag_begin(ui: &Rc<Ui>, x: f64, y: f64) -> bool {
    let mut current = ui.current.borrow_mut();
    let Some(doc) = current.as_mut() else {
        return false;
    };
    let Some((pos, _, _)) = pictures::selected(&doc.session) else {
        return false;
    };
    let Some(hit) = doc.session.layout().object(&pos, doc.session.page_hint) else {
        return false;
    };
    let Some(frame) = doc.page_frames[hit.page].compute_bounds(&ui.document_box) else {
        return false;
    };
    let z = doc.page_zoom(hit.page);
    let px = (x - frame.x() as f64) as f32 / z;
    let py = (y - frame.y() as f64) as f32 / z;
    let rect = (hit.rect.x, hit.rect.y, hit.rect.w, hit.rect.h);
    let Some(edges) = edges_at(rect, px, py, 7. / z) else {
        return false;
    };
    let inline =
        pictures::layout(&doc.session).is_some_and(|f| f.wrap == wordcraft_doc::para::Wrap::Inline);
    // Crop the already rendered document representation once, retaining image crop/effects.
    let image = render_page(
        &mut doc.session,
        hit.page,
        1.0,
        adw::StyleManager::default().is_dark(),
    )
    .ok()
    .and_then(|r| {
        let left = hit.rect.x.floor().max(0.) as usize;
        let top = hit.rect.y.floor().max(0.) as usize;
        let w = (hit.rect.w.ceil() as usize).min((r.width as usize).saturating_sub(left));
        let h = (hit.rect.h.ceil() as usize).min((r.height as usize).saturating_sub(top));
        if w == 0 || h == 0 || w.saturating_mul(h) > 8_000_000 {
            return None;
        }
        let mut bytes = Vec::with_capacity(w * h * 4);
        for row in top..top + h {
            let start = (row * r.width as usize + left) * 4;
            bytes.extend_from_slice(&r.rgba[start..start + w * 4]);
        }
        for p in bytes.chunks_exact_mut(4) {
            if cfg!(target_endian = "little") {
                p.swap(0, 2)
            } else {
                p.rotate_right(1)
            }
        }
        gtk::cairo::ImageSurface::create_for_data(
            bytes,
            gtk::cairo::Format::ARgb32,
            w as i32,
            h as i32,
            w as i32 * 4,
        )
        .ok()
    });
    ui.document_box
        .set_cursor_from_name(Some(if edges == (0, 0) {
            "grabbing"
        } else {
            resize_cursor(edges)
        }));
    ui.picture_drag.replace(Some(PictureDrag {
        document: (doc.session.document_id(), doc.session.rev()),
        pos,
        page: hit.page,
        target_page: hit.page,
        edges,
        inline,
        rect,
        preview: rect,
        start: (x, y),
        grab: ((px - rect.0) / rect.2, (py - rect.1) / rect.3),
        zoom: z,
        moved: false,
        valid: true,
        destination: None,
        image,
    }));
    true
}

pub(super) fn picture_drag_at(ui: &Rc<Ui>, x: f64, y: f64) -> bool {
    let mut drag = ui.picture_drag.borrow_mut();
    let Some(drag) = drag.as_mut() else {
        return false;
    };
    let (dx, dy) = (
        (x - drag.start.0) as f32 / drag.zoom,
        (y - drag.start.1) as f32 / drag.zoom,
    );
    drag.moved = (x - drag.start.0).abs().max((y - drag.start.1).abs()) > 4.;
    let (rx, ry, w, h) = drag.rect;
    let mut current = ui.current.borrow_mut();
    let Some(doc) = current.as_mut() else {
        return true;
    };
    if drag.edges != (0, 0) {
        let (ex, ey) = drag.edges;
        let mut nw = if ex == 0 {
            w
        } else {
            (w + dx * ex as f32).clamp(1., 4000.)
        };
        let mut nh = if ey == 0 {
            h
        } else {
            (h + dy * ey as f32).clamp(1., 4000.)
        };
        if ex != 0 && ey != 0 {
            let scale = if (nw / w - 1.).abs() >= (nh / h - 1.).abs() {
                nw / w
            } else {
                nh / h
            };
            let scale = scale.clamp((1. / w).max(1. / h), (4000. / w).min(4000. / h));
            nw = w * scale;
            nh = h * scale;
        }
        drag.preview = (
            if ex < 0 && !drag.inline {
                rx + w - nw
            } else {
                rx
            },
            if ey < 0 && !drag.inline {
                ry + h - nh
            } else {
                ry
            },
            nw,
            nh,
        );
        drag.valid = true;
    } else {
        let target = doc.page_frames.iter().enumerate().find_map(|(page, f)| {
            let r = f.compute_bounds(&ui.document_box)?;
            (x >= r.x() as f64
                && x <= (r.x() + r.width()) as f64
                && y >= r.y() as f64
                && y <= (r.y() + r.height()) as f64)
                .then_some((page, r))
        });
        drag.valid = false;
        if let Some((page, r)) = target {
            let z = doc.page_zoom(page);
            let px = (x - r.x() as f64) as f32 / z;
            let py = (y - r.y() as f64) as f32 / z;
            let (pw, ph) = doc.sizes[page];
            let nx = (px - drag.grab.0 * w).clamp(0., (pw - w).max(0.));
            let ny = (py - drag.grab.1 * h).clamp(0., (ph - h).max(0.));
            let to = doc.session.layout().hit(page, px, py, drag.pos.story);
            if let Some(to) = to {
                drag.target_page = page;
                drag.preview = (nx, ny, w, h);
                drag.destination = Some(to);
                drag.valid = true;
            }
        }
    }
    ui.pointer_moved.set(drag.moved);
    for area in &doc.selection_areas {
        area.queue_draw()
    }
    true
}

pub(super) fn picture_drag_update(ui: &Rc<Ui>, dx: f64, dy: f64) -> bool {
    let start = ui.picture_drag.borrow().as_ref().map(|d| d.start);
    let Some(start) = start else { return false };
    picture_drag_at(ui, start.0 + dx, start.1 + dy);
    if let Some(r) = ui.document_box.compute_bounds(&ui.scroll) {
        ui.drag_point.set(Some((
            r.x() as f64 + start.0 + dx,
            r.y() as f64 + start.1 + dy,
        )));
        schedule_drag_scroll(ui);
    }
    true
}

pub(super) fn picture_drag_finish(ui: &Rc<Ui>) -> bool {
    let Some(drag) = ui.picture_drag.borrow_mut().take() else {
        return false;
    };
    stop_document_drag(ui);
    ui.document_box.set_cursor_from_name(None);
    if drag.moved && drag.valid && document_identity(ui) == Some(drag.document) {
        let (x, y, w, h) = drag.preview;
        tools_change(ui, |s| {
            pictures::select(s, &drag.pos)?;
            if drag.edges == (0, 0) {
                pictures::drop_at(s, drag.target_page, drag.destination.as_ref(), x, y)?;
            } else {
                let args = if drag.inline {
                    json!({"width":w,"height":h})
                } else {
                    json!({"x":x,"y":y,"width":w,"height":h,"page":drag.page})
                };
                s.run("arrange.bounds", &args).map_err(|e| e.to_string())?;
            }
            Ok(())
        });
    }
    refresh_selection(ui);
    true
}

pub(super) fn picture_drag_cancel(ui: &Rc<Ui>) -> bool {
    let active = ui.picture_drag.borrow_mut().take().is_some();
    if active {
        ui.document_box.set_cursor_from_name(None);
        stop_document_drag(ui);
        refresh_selection(ui)
    }
    active
}

pub(super) fn paint_picture_frame(ui: &Rc<Ui>, cr: &gtk::cairo::Context, page: usize, zoom: f64) {
    let drag = ui.picture_drag.borrow();
    let preview = drag.as_ref().filter(|d| d.target_page == page && d.moved);
    let rect = preview.map(|d| d.preview).or_else(|| {
        let mut c = ui.current.try_borrow_mut().ok()?;
        let d = c.as_mut()?;
        let (pos, _, _) = pictures::selected(&d.session)?;
        let hit = d.session.layout().object(&pos, d.session.page_hint)?;
        (hit.page == page).then_some((hit.rect.x, hit.rect.y, hit.rect.w, hit.rect.h))
    });
    let Some((x, y, w, h)) = rect else { return };
    let (x, y, w, h) = (
        x as f64 * zoom,
        y as f64 * zoom,
        w as f64 * zoom,
        h as f64 * zoom,
    );
    let _ = cr.save();
    if let Some(image) = preview.and_then(|d| d.image.as_ref()) {
        let _ = cr.save();
        cr.rectangle(x, y, w, h);
        cr.clip();
        cr.translate(x, y);
        cr.scale(w / image.width() as f64, h / image.height() as f64);
        if cr.set_source_surface(image, 0., 0.).is_ok() {
            let _ = cr.paint_with_alpha(0.88);
        }
        let _ = cr.restore();
    }
    let (bg, _) = selection_colors(&ui.document_box, true);
    cr.set_source_rgba(bg.red() as f64, bg.green() as f64, bg.blue() as f64, 0.95);
    cr.set_line_width(1.5);
    cr.rectangle(x, y, w, h);
    let _ = cr.stroke();
    for (hx, hy) in [
        (x, y),
        (x + w / 2., y),
        (x + w, y),
        (x, y + h / 2.),
        (x + w, y + h / 2.),
        (x, y + h),
        (x + w / 2., y + h),
        (x + w, y + h),
    ] {
        cr.rectangle(hx - 3., hy - 3., 6., 6.);
        let _ = cr.fill();
    }
    let _ = cr.restore();
}

pub(super) fn picture_hover(ui: &Rc<Ui>, x: f64, y: f64) -> Option<&'static str> {
    let mut c = ui.current.try_borrow_mut().ok()?;
    let d = c.as_mut()?;
    let layout = d.session.layout();
    if let Some((pos, _, _)) = pictures::selected(&d.session) {
        if let Some(hit) = layout.object(&pos, d.session.page_hint) {
            let r = d
                .page_frames
                .get(hit.page)?
                .compute_bounds(&ui.document_box)?;
            let z = d.page_zoom(hit.page);
            if let Some(edges) = edges_at(
                (hit.rect.x, hit.rect.y, hit.rect.w, hit.rect.h),
                (x - r.x() as f64) as f32 / z,
                (y - r.y() as f64) as f32 / z,
                7. / z,
            ) {
                return Some(if edges == (0, 0) {
                    "grab"
                } else {
                    resize_cursor(edges)
                });
            }
        }
    }
    // Unselected images advertise selection, without nonexistent resize handles.
    for (page, frame) in d.page_frames.iter().enumerate() {
        let Some(r) = frame.compute_bounds(&ui.document_box) else {
            continue;
        };
        let z = d.page_zoom(page) as f64;
        let (px, py) = (
            ((x - r.x() as f64) / z) as f32,
            ((y - r.y() as f64) / z) as f32,
        );
        if let Some(hit) = layout.object_at_in(page, px, py, 0., wordcraft_doc::StoryRef::Body) {
            if matches!(
                d.session
                    .doc
                    .para_at(&hit.pos())
                    .and_then(|p| p.object_at(hit.off)),
                Some(wordcraft_doc::para::InlineObject::Image { .. })
            ) {
                return Some("pointer");
            }
        }
    }
    None
}
fn resize_cursor(edges: (i8, i8)) -> &'static str {
    match edges {
        (-1, -1) | (1, 1) => "nwse-resize",
        (-1, 1) | (1, -1) => "nesw-resize",
        (_, 0) if edges.0 != 0 => "ew-resize",
        (0, _) if edges.1 != 0 => "ns-resize",
        _ => "grab",
    }
}
