//! Transient, theme-aware decoration of an explicitly associated object/caption.
use super::*;
use wordcraft_gnome::captions::{self, Cue};

fn cached_caption_cues(ui: &Rc<Ui>, d: &mut Current) -> Vec<Cue> {
    let key = (d.session.document_id(), d.session.rev());
    if let Some((id, revision, cues)) = ui.caption_cues.borrow().as_ref() {
        if (*id, *revision) == key {
            return cues.clone();
        }
    }
    let cues = captions::cues(&mut d.session);
    ui.caption_cues.replace(Some((key.0, key.1, cues.clone())));
    cues
}
fn active_caption_id(ui: &Rc<Ui>, d: &Current, cues: &[Cue]) -> Option<String> {
    ui.caption_hover
        .borrow()
        .as_ref()
        .and_then(|id| cues.iter().find(|c| &c.caption.id == id))
        .or_else(|| {
            cues.iter().find(|c| {
                c.caption.pos.story == d.session.sel.focus.story
                    && c.caption.pos.path == d.session.sel.focus.path
            })
        })
        .or_else(|| {
            let (p, _, _) = pictures::selected(&d.session)?;
            cues.iter()
                .find(|c| c.caption.owner == captions::Target::Image(p.clone()))
        })
        .map(|c| c.caption.id.clone())
}
pub(super) fn caption_cue_pages(ui: &Rc<Ui>, d: &mut Current) -> Vec<usize> {
    let cues = cached_caption_cues(ui, d);
    let id = active_caption_id(ui, d, &cues);
    cues.iter()
        .filter(|c| Some(&c.caption.id) == id.as_ref())
        .flat_map(|c| c.rects().map(|r| r.page))
        .collect()
}
pub(super) fn update_caption_hover(ui: &Rc<Ui>, point: Option<(f64, f64)>) {
    let id = point.and_then(|(x, y)| {
        let (page, px, py) = table_pointer::page_point(ui, x, y, None)?;
        let mut current = ui.current.try_borrow_mut().ok()?;
        let d = current.as_mut()?;
        let cues = cached_caption_cues(ui, d);
        if let Some(hit) =
            d.session
                .layout()
                .object_at_in(page, px, py, 0., wordcraft_doc::StoryRef::Body)
        {
            if let Some(c) = cues
                .iter()
                .find(|c| c.caption.owner == captions::Target::Image(hit.pos()))
            {
                return Some(c.caption.id.clone());
            }
        }
        cues.iter()
            .flat_map(|c| {
                c.rects()
                    .filter(move |r| r.contains(page, px, py))
                    .map(move |r| (c, r.w * r.h))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(c, _)| c.caption.id.clone())
    });
    if *ui.caption_hover.borrow() != id {
        ui.caption_hover.replace(id);
        if let Some(d) = ui.current.borrow().as_ref() {
            for area in &d.selection_areas {
                area.queue_draw();
            }
        }
    }
}
pub(super) fn paint_caption_cue(
    ui: &Rc<Ui>,
    widget: &gtk::DrawingArea,
    cr: &gtk::cairo::Context,
    page: usize,
    z: f64,
) {
    let Ok(mut current) = ui.current.try_borrow_mut() else {
        return;
    };
    let Some(d) = current.as_mut() else { return };
    let cues = cached_caption_cues(ui, d);
    let id = active_caption_id(ui, d, &cues);
    let Some(cue) = cues.iter().find(|c| Some(&c.caption.id) == id.as_ref()) else {
        return;
    };
    let color = widget.color();
    let _ = cr.save();
    cr.set_source_rgba(
        color.red() as f64,
        color.green() as f64,
        color.blue() as f64,
        0.055,
    );
    for r in cue.rects().filter(|r| r.page == page) {
        cr.rectangle(
            r.x as f64 * z - 2.,
            r.y as f64 * z - 2.,
            r.w as f64 * z + 4.,
            r.h as f64 * z + 4.,
        );
    }
    let _ = cr.fill();
    let _ = cr.restore();
}
