use super::*;
use serde_json::json;

pub(super) struct PictureDrag {
    document: (u64, u64),
    pos: Pos,
    page: usize,
    resize: bool,
    rect: (f32, f32, f32, f32),
    preview: (f32, f32, f32, f32),
    zoom: f32,
    moved: bool,
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
    let reach = 7. / z;
    let resize = (px - hit.rect.right()).abs() <= reach && (py - hit.rect.bottom()).abs() <= reach;
    if !resize
        && (px < hit.rect.x || py < hit.rect.y || px > hit.rect.right() || py > hit.rect.bottom())
    {
        return false;
    }
    let rect = (hit.rect.x, hit.rect.y, hit.rect.w, hit.rect.h);
    ui.picture_drag.replace(Some(PictureDrag {
        document: (doc.session.document_id(), doc.session.rev()),
        pos,
        page: hit.page,
        resize,
        rect,
        preview: rect,
        zoom: z,
        moved: false,
    }));
    true
}

pub(super) fn picture_drag_update(ui: &Rc<Ui>, dx: f64, dy: f64) -> bool {
    let mut drag = ui.picture_drag.borrow_mut();
    let Some(drag) = drag.as_mut() else {
        return false;
    };
    drag.moved = dx.abs().max(dy.abs()) > 4.;
    let (x, y, w, h) = drag.rect;
    let (dx, dy) = (dx as f32 / drag.zoom, dy as f32 / drag.zoom);
    drag.preview = if drag.resize {
        let change = if dx.abs() / w >= dy.abs() / h {
            dx / w
        } else {
            dy / h
        };
        let scale = (1. + change).clamp((1. / w).max(1. / h), (4000. / w).min(4000. / h));
        (x, y, w * scale, h * scale)
    } else {
        (x + dx, y + dy, w, h)
    };
    ui.pointer_moved.set(drag.moved);
    if let Some(doc) = ui.current.borrow().as_ref() {
        doc.selection_areas[drag.page].queue_draw();
    }
    true
}

pub(super) fn picture_drag_finish(ui: &Rc<Ui>) -> bool {
    let Some(drag) = ui.picture_drag.borrow_mut().take() else {
        return false;
    };
    if drag.moved && document_identity(ui) == Some(drag.document) {
        let (x, y, w, h) = drag.preview;
        tools_change(ui, |s| {
            pictures::select(s, &drag.pos)?;
            let args = if drag.resize {
                json!({"width":w,"height":h})
            } else {
                json!({"x":x,"y":y,"page":drag.page})
            };
            s.run("arrange.bounds", &args).map_err(|e| e.to_string())?;
            Ok(())
        });
    }
    refresh_selection(ui);
    true
}

pub(super) fn picture_drag_cancel(ui: &Rc<Ui>) -> bool {
    let active = ui.picture_drag.borrow_mut().take().is_some();
    if active {
        refresh_selection(ui);
    }
    active
}

pub(super) fn paint_picture_frame(ui: &Rc<Ui>, cr: &gtk::cairo::Context, page: usize, zoom: f64) {
    let preview = ui
        .picture_drag
        .borrow()
        .as_ref()
        .filter(|d| d.page == page)
        .map(|d| d.preview);
    let rect = preview.or_else(|| {
        let mut c = ui.current.try_borrow_mut().ok()?;
        let d = c.as_mut()?;
        let (pos, _, _) = pictures::selected(&d.session)?;
        let hit = d.session.layout().object(&pos, d.session.page_hint)?;
        (hit.page == page).then_some((hit.rect.x, hit.rect.y, hit.rect.w, hit.rect.h))
    });
    if let Some((x, y, w, h)) = rect {
        let (x, y, w, h) = (
            x as f64 * zoom,
            y as f64 * zoom,
            w as f64 * zoom,
            h as f64 * zoom,
        );
        let _ = cr.save();
        cr.set_source_rgba(0.20, 0.45, 0.85, 0.9);
        cr.set_line_width(1.5);
        cr.rectangle(x, y, w, h);
        let _ = cr.stroke();
        cr.rectangle(x + w - 4., y + h - 4., 8., 8.);
        let _ = cr.fill();
        let _ = cr.restore();
    }
}
