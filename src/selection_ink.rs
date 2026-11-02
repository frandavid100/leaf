//! A disposable, screen-only recoloring of the public display list for native selection.
use wordcraft_doc::props::Rgb;
use wordcraft_engine::{
    Session,
    layout::display::{Draw, page_display},
};
fn text_only(items: &mut Vec<Draw>, ink: Rgb) {
    items.retain_mut(|item| match item {
        Draw::Glyphs {
            color,
            alpha,
            text_bg,
            ..
        }
        | Draw::Line {
            color,
            alpha,
            text_bg,
            ..
        } => {
            *color = ink;
            *alpha = 1.0;
            *text_bg = None;
            true
        }
        Draw::Turned { items, .. } | Draw::Rotated { items, .. } | Draw::Clip { items, .. } => {
            text_only(items, ink);
            !items.is_empty()
        }
        Draw::Figure { draws, .. } => {
            text_only(draws, ink);
            !draws.is_empty()
        }
        _ => false,
    });
}
/// Keep the disposable overlay bounded, preserving its logical page coordinates.
pub fn raster_scale(w: f32, h: f32, requested: f32) -> f32 {
    let requested = if requested.is_finite() && requested > 0.0 {
        requested
    } else {
        1.0
    };
    requested
        .min((11_000_000.0 / (w * h).max(1.0)).sqrt())
        .min(8191.0 / w.max(h).max(1.0))
}
pub fn render(s: &mut Session, page: usize, scale: f32, ink: Rgb) -> Option<crate::PageImage> {
    let layout = s.layout();
    let p = layout.pages.get(page)?;
    let scale = raster_scale(p.w, p.h, scale);
    let opts = wordcraft_render::RenderOptions::default();
    let mut items = page_display(&s.doc, p, &opts.display);
    text_only(&mut items, ink);
    let mut bounds = p.body;
    bounds.x = 0.0;
    bounds.y = 0.0;
    bounds.w = p.w;
    bounds.h = p.h;
    let rendered = wordcraft_render::render_items(&s.doc, &items, bounds, scale, &opts);
    Some(crate::PageImage {
        width: rendered.width,
        height: rendered.height,
        rgba: rendered.pixels,
        total_pages: layout.pages.len(),
    })
}
