use super::*;
use serde_json::json;
use wordcraft_engine::cmd::{table_mouse as mouse, table_resize as resize};

#[derive(Clone)]
enum Hit {
    Resize(resize::ResizeTarget),
    Mouse(mouse::Target),
}
#[derive(Clone)]
pub(super) struct TableDrag {
    id: u64,
    rev: u64,
    page: usize,
    start: (f32, f32),
    end: (f32, f32),
    hit: Hit,
    selection: wordcraft_engine::Selection,
    reach: f32,
    anchor_row: Option<usize>,
    shift: bool,
    ctrl: bool,
    moved: bool,
    inserted: bool,
}
#[derive(Default)]
pub(super) struct TablePointer {
    drag: Option<TableDrag>,
    hover: Option<(usize, resize::ResizeTarget, Option<mouse::Target>)>,
    cancelled: bool,
}
fn page_point(ui: &Ui, x: f64, y: f64, page: Option<usize>) -> Option<(usize, f32, f32)> {
    let current = ui.current.borrow();
    let d = current.as_ref()?;
    d.page_frames.iter().enumerate().find_map(|(i, f)| {
        if page.is_some_and(|p| p != i) {
            return None;
        }
        let r = f.compute_bounds(&ui.document_box)?;
        if page.is_none()
            && (x < r.x() as f64
                || y < r.y() as f64
                || x > (r.x() + r.width()) as f64
                || y > (r.y() + r.height()) as f64)
        {
            return None;
        }
        Some((
            i,
            ((x - r.x() as f64) / d.zoom as f64) as f32,
            ((y - r.y() as f64) / d.zoom as f64) as f32,
        ))
    })
}
fn hit(ui: &Ui, p: usize, x: f32, y: f32) -> Option<Hit> {
    let mut c = ui.current.borrow_mut();
    let d = c.as_mut()?;
    let l = d.session.layout();
    let story = d.session.sel.focus.story;
    if let Some(mut t) = resize::resize_target(&l, &d.session.doc, p, x, y, 3.0 / d.zoom) {
        t.limit_to_selection(&d.session, y);
        return (t.story == story).then_some(Hit::Resize(t));
    }
    mouse::target_at(&l, &d.session.doc, story, p, x, y, 3.0 / d.zoom).map(Hit::Mouse)
}
fn redraw(ui: &Ui) {
    if let Some(d) = ui.current.borrow().as_ref() {
        for a in &d.selection_areas {
            a.queue_draw();
        }
    }
}
fn command(ui: &Rc<Ui>, cmd: &str, args: serde_json::Value, edit: bool) -> bool {
    let result = ui
        .current
        .borrow_mut()
        .as_mut()
        .map(|d| d.session.run(cmd, &args));
    match result {
        Some(Ok(_)) => {
            if edit {
                after_content_change(ui);
            } else {
                refresh_selection(ui);
            }
            true
        }
        Some(Err(e)) => {
            ui.toasts.add_toast(Toast::new(&e.to_string()));
            false
        }
        None => false,
    }
}
pub(super) fn table_pointer_press(
    ui: &Rc<Ui>,
    count: i32,
    x: f64,
    y: f64,
    mods: gdk::ModifierType,
) -> bool {
    ui.table_pointer.borrow_mut().cancelled = false;
    let Some((page, px, py)) = page_point(ui, x, y, None) else {
        return false;
    };
    let Some(target) = hit(ui, page, px, py) else {
        return false;
    };
    stop_document_drag(ui);
    ui.document_box.grab_focus();
    if count == 2
        && matches!(&target,Hit::Resize(t) if matches!(t.edge,resize::Edge::Column(n) if n>0))
    {
        ui.table_pointer.borrow_mut().cancelled = true;
        command(
            ui,
            "table.fitColumn",
            json!({"page":page,"x":px,"y":py}),
            true,
        );
        return true;
    }
    if count != 1 {
        return true;
    }
    let (id, rev, selection) = {
        let c = ui.current.borrow();
        let s = &c.as_ref().unwrap().session;
        (s.document_id(), s.rev(), s.sel.clone())
    };
    let reach = ui
        .current
        .borrow()
        .as_ref()
        .map(|d| 3.0 / d.zoom)
        .unwrap_or(3.0);
    let anchor_row = match &target {
        Hit::Mouse(t) if mods.contains(gdk::ModifierType::SHIFT_MASK) => match t.spot {
            mouse::Spot::Row(r) => ui
                .current
                .borrow()
                .as_ref()
                .map(|d| mouse::anchor_row(&d.session, &t.geom.table, r)),
            _ => None,
        },
        _ => None,
    };
    let mut drag = TableDrag {
        id,
        rev,
        page,
        start: (px, py),
        end: (px, py),
        hit: target.clone(),
        selection,
        reach,
        anchor_row,
        shift: mods.contains(gdk::ModifierType::SHIFT_MASK),
        ctrl: mods.contains(gdk::ModifierType::CONTROL_MASK),
        moved: false,
        inserted: false,
    };
    if let Hit::Mouse(t) = &target {
        let insert = matches!(
            t.spot,
            mouse::Spot::InsertRow(_) | mouse::Spot::InsertColumn(_)
        );
        let cmd = if insert {
            "table.insertAt"
        } else {
            "table.selectAt"
        };
        drag.inserted = insert;
        command(
            ui,
            cmd,
            json!({"page":page,"x":px,"y":py,"reach":drag.reach,"anchorRow":drag.anchor_row,"extend":drag.shift}),
            insert,
        );
        if insert {
            if let Some(d) = ui.current.borrow().as_ref() {
                drag.rev = d.session.rev();
            }
        }
    }
    let mut state = ui.table_pointer.borrow_mut();
    state.cancelled = false;
    state.drag = Some(drag);
    drop(state);
    redraw(ui);
    true
}
pub(super) fn table_pointer_active(ui: &Ui) -> bool {
    let p = ui.table_pointer.borrow();
    p.drag.is_some() || p.cancelled
}
pub(super) fn table_pointer_update(ui: &Rc<Ui>, x: f64, y: f64, mods: gdk::ModifierType) -> bool {
    let state = ui.table_pointer.borrow();
    let Some(mut drag) = state.drag.clone() else {
        return state.cancelled;
    };
    drop(state);
    if !ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| d.session.document_id() == drag.id && d.session.rev() == drag.rev)
    {
        table_pointer_cancel(ui);
        return true;
    }
    let Some((_, px, py)) = page_point(ui, x, y, Some(drag.page)) else {
        return true;
    };
    drag.end = (px, py);
    drag.moved |= (px - drag.start.0).abs() > 3.0 || (py - drag.start.1).abs() > 3.0;
    drag.shift = mods.contains(gdk::ModifierType::SHIFT_MASK);
    drag.ctrl = mods.contains(gdk::ModifierType::CONTROL_MASK);
    if drag.moved
        && !drag.inserted
        && matches!(&drag.hit,Hit::Mouse(t) if !matches!(t.spot,mouse::Spot::Handle))
    {
        command(
            ui,
            "table.selectAt",
            json!({"page":drag.page,"x":drag.start.0,"y":drag.start.1,"toX":px,"toY":py,"reach":drag.reach,"anchorRow":drag.anchor_row,"extend":drag.shift}),
            false,
        );
    }
    ui.table_pointer.borrow_mut().drag = Some(drag);
    redraw(ui);
    true
}
pub(super) fn table_pointer_finish(ui: &Rc<Ui>) -> bool {
    let mut state = ui.table_pointer.borrow_mut();
    let cancelled = std::mem::take(&mut state.cancelled);
    let drag = state.drag.take();
    drop(state);
    let Some(d) = drag else {
        return cancelled;
    };
    if d.moved
        && !d.inserted
        && ui
            .current
            .borrow()
            .as_ref()
            .is_some_and(|c| c.session.document_id() == d.id && c.session.rev() == d.rev)
    {
        let cmd = match d.hit {
            Hit::Resize(_) => Some("table.resizeDrag"),
            Hit::Mouse(t) if t.spot == mouse::Spot::Handle && mouse::movable(&t.geom) => {
                Some("table.moveDrag")
            }
            _ => None,
        };
        if let Some(cmd) = cmd {
            command(
                ui,
                cmd,
                json!({"page":d.page,"x":d.start.0,"y":d.start.1,"toX":d.end.0,"toY":d.end.1,"shift":d.shift,"ctrl":d.ctrl,"reach":d.reach}),
                true,
            );
        }
    }
    redraw(ui);
    true
}
pub(super) fn table_pointer_cancel(ui: &Rc<Ui>) -> bool {
    let drag = ui.table_pointer.borrow_mut().drag.take();
    let Some(d) = drag else {
        return false;
    };
    if !d.inserted {
        if let Some(c) = ui.current.borrow_mut().as_mut() {
            if c.session.document_id() == d.id && c.session.rev() == d.rev {
                c.session.sel = d.selection;
            }
        }
    }
    ui.table_pointer.borrow_mut().cancelled = true;
    refresh_selection(ui);
    redraw(ui);
    true
}
pub(super) fn install_table_hover(ui: &Rc<Ui>) {
    let motion = gtk::EventControllerMotion::new();
    let weak = Rc::downgrade(ui);
    motion.connect_motion(move |_, x, y| {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if table_pointer_active(&ui) {
            return;
        }
        let hover = page_point(&ui, x, y, None).and_then(|(p, x, y)| {
            let mut c = ui.current.borrow_mut();
            let d = c.as_mut()?;
            let l = d.session.layout();
            let story = d.session.sel.focus.story;
            let t = mouse::target_at(&l, &d.session.doc, story, p, x, y, 3.0 / d.zoom);
            let g = mouse::handle_table(&l, &d.session.doc, story, p, x, y, 3.0 / d.zoom)
                .or_else(|| resize::table_under(&l, &d.session.doc, p, x, y, 3.0 / d.zoom))?;
            Some((p, g, t))
        });
        let cursor = page_point(&ui, x, y, None)
            .and_then(|(p, x, y)| hit(&ui, p, x, y))
            .map(|t| match t {
                Hit::Resize(t) => match t.edge {
                    resize::Edge::Column(_) => "col-resize",
                    resize::Edge::Row(_) => "row-resize",
                    resize::Edge::Corner => "nwse-resize",
                },
                Hit::Mouse(t) => match t.spot {
                    mouse::Spot::Handle => "move",
                    mouse::Spot::InsertRow(_) | mouse::Spot::InsertColumn(_) => "copy",
                    mouse::Spot::Column(_) => "s-resize",
                    _ => "e-resize",
                },
            })
            .unwrap_or("text");
        ui.document_box.set_cursor_from_name(Some(cursor));
        ui.table_pointer.borrow_mut().hover = hover;
        redraw(&ui);
    });
    let weak = Rc::downgrade(ui);
    motion.connect_leave(move |_| {
        if let Some(ui) = weak.upgrade() {
            ui.table_pointer.borrow_mut().hover = None;
            redraw(&ui);
        }
    });
    ui.document_box.add_controller(motion);
}
pub(super) fn paint_table_pointer(
    ui: &Rc<Ui>,
    widget: &gtk::DrawingArea,
    cr: &gtk::cairo::Context,
    page: usize,
    z: f64,
) {
    let state = ui.table_pointer.borrow();
    let mut geom = None;
    let mut target = None;
    if let Some((p, g, t)) = &state.hover {
        if *p == page {
            geom = Some(g.clone());
            target = t.clone();
        }
    }
    if let Some(d) = &state.drag {
        if d.page == page {
            geom = Some(match &d.hit {
                Hit::Resize(g) => g.clone(),
                Hit::Mouse(t) => t.geom.clone(),
            });
        }
    }
    let Some(g) = geom else {
        return;
    };
    let (bg, _) = selection_colors(widget, true);
    let _ = cr.save();
    cr.scale(z, z);
    cr.set_line_width(1.0 / z);
    cr.set_source_rgba(bg.red() as f64, bg.green() as f64, bg.blue() as f64, 1.0);

    // The engine decides the actual first/last page, including repeated header rows.
    if let Some(c) = ui.current.borrow_mut().as_mut() {
        let l = c.session.layout();
        let (a, b) = mouse::caret_handles(&l, &c.session.doc, g.story, &g.table, page);
        for r in [a, b].into_iter().flatten() {
            cr.rectangle(r.x as f64, r.y as f64, r.w as f64, r.h as f64);
            let _ = cr.stroke();
        }
    }
    if let Some(t) = target {
        if let Some(m) = mouse::insert_mark(&t) {
            cr.move_to(m.from.0 as f64, m.from.1 as f64);
            cr.line_to(m.to.0 as f64, m.to.1 as f64);
            let _ = cr.stroke();
            cr.arc(
                m.center.0 as f64,
                m.center.1 as f64,
                4.0,
                0.0,
                std::f64::consts::TAU,
            );
            let _ = cr.stroke();
            cr.move_to(m.center.0 as f64 - 2.0, m.center.1 as f64);
            cr.line_to(m.center.0 as f64 + 2.0, m.center.1 as f64);
            cr.move_to(m.center.0 as f64, m.center.1 as f64 - 2.0);
            cr.line_to(m.center.0 as f64, m.center.1 as f64 + 2.0);
            let _ = cr.stroke();
        }
    }
    if let Some(d) = &state.drag {
        if d.page == page && d.moved && !d.inserted {
            let dx = d.end.0 - d.start.0;
            let dy = d.end.1 - d.start.1;
            cr.set_dash(&[4.0 / z, 3.0 / z], 0.0);
            let (mut x, mut y, mut w, mut h) = (
                g.left(),
                g.top(),
                g.right() - g.left(),
                g.bottom() - g.top(),
            );
            match &d.hit {
                Hit::Resize(t) => {
                    let share = if d.shift {
                        resize::Share::LeftOnly
                    } else if d.ctrl {
                        resize::Share::Proportional
                    } else {
                        resize::Share::Neighbour
                    };
                    let plan = t.plan(dx, dy, share);
                    x += plan.indent_by.unwrap_or(0.0);
                    if let Some(widths) = &plan.widths {
                        w = widths.iter().sum();
                        let mut edge = x;
                        for width in widths {
                            edge += width;
                            cr.move_to(edge as f64, y as f64);
                            cr.line_to(edge as f64, (y + h) as f64);
                        }
                    }
                    for (row, _) in &plan.cells {
                        if let Some(r) = t.rows.iter().find(|r| r.row == *row) {
                            for line in 0..t.cols.len() {
                                let at = t.planned_x(&plan, Some(*row), line);
                                cr.move_to(at as f64, r.top as f64);
                                cr.line_to(at as f64, r.bottom as f64);
                            }
                        }
                    }
                    for (row, height) in plan.heights {
                        if let Some(r) = t.rows.iter().find(|r| r.row == row) {
                            let end = r.top + height;
                            cr.move_to(x as f64, end as f64);
                            cr.line_to((x + w) as f64, end as f64);
                            h = h.max(end - y);
                        }
                    }
                }
                Hit::Mouse(t) if t.spot == mouse::Spot::Handle => {
                    x += dx;
                    y += dy;
                }
                _ => {}
            }
            cr.rectangle(x as f64, y as f64, w as f64, h as f64);
            let _ = cr.stroke();
        }
    }
    let _ = cr.restore();
}
