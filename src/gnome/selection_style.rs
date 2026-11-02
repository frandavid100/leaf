use super::*;

pub(super) fn selection_colors(
    widget: &impl IsA<gtk::Widget>,
    focused: bool,
) -> (gdk::RGBA, gdk::RGBA) {
    let style = widget.as_ref().style_context();
    let color = |names: &[&str]| names.iter().find_map(|name| style.lookup_color(name));
    let bg = if focused {
        color(&["theme_selected_bg_color", "accent_bg_color"])
    } else {
        color(&[
            "theme_unfocused_selected_bg_color",
            "theme_selected_bg_color",
            "accent_bg_color",
        ])
    }
    .unwrap_or_else(|| widget.as_ref().color());
    let fg = if focused {
        color(&["theme_selected_fg_color", "accent_fg_color"])
    } else {
        color(&[
            "theme_unfocused_selected_fg_color",
            "theme_selected_fg_color",
            "accent_fg_color",
        ])
    }
    .or_else(|| color(&["window_bg_color", "theme_bg_color"]))
    .unwrap_or_else(|| widget.as_ref().color());
    (bg, fg)
}
pub(super) fn paint_selection(
    ui: &Rc<Ui>,
    widget: &gtk::DrawingArea,
    cr: &gtk::cairo::Context,
    page: usize,
    z: f64,
    rects: &[selection::Rect],
) {
    if !rects.iter().any(|r| r.page == page) {
        return;
    }
    let focused =
        ui.document_box.has_focus() && ui.host.upgrade().is_some_and(|h| h.window.is_active());
    let (bg, fg) = selection_colors(widget, focused);
    let _ = cr.save();
    for r in rects.iter().filter(|r| r.page == page) {
        cr.rectangle(r.x * z, r.y * z, r.width * z, r.height * z);
    }
    cr.clip();
    cr.set_source_rgba(
        bg.red() as f64,
        bg.green() as f64,
        bg.blue() as f64,
        bg.alpha() as f64,
    );
    let _ = cr.paint();
    let raster = {
        let Ok(mut value) = ui.current.try_borrow_mut() else {
            let _ = cr.restore();
            return;
        };
        let Some(d) = value.as_mut() else {
            let _ = cr.restore();
            return;
        };
        let density = d.raster_scale.max(1);
        let key = format!(
            "{}:{}:{page}:{}:{density}:{fg:?}",
            d.session.document_id(),
            d.session.rev(),
            z.to_bits()
        );
        let cached = ui
            .selection_rasters
            .borrow()
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone());
        if let Some(surface) = cached {
            Some((surface, density))
        } else {
            let ink = wordcraft_doc::props::Rgb(
                (fg.red() * 255.0).round() as u8,
                (fg.green() * 255.0).round() as u8,
                (fg.blue() * 255.0).round() as u8,
            );
            wordcraft_gnome::selection_ink::render(
                &mut d.session,
                page,
                (z * density as f64) as f32,
                ink,
            )
            .and_then(|mut image| {
                for pixel in image.rgba.chunks_exact_mut(4) {
                    if cfg!(target_endian = "little") {
                        pixel.swap(0, 2);
                    } else {
                        pixel.rotate_right(1);
                    }
                }
                let surface = gtk::cairo::ImageSurface::create_for_data(
                    image.rgba,
                    gtk::cairo::Format::ARgb32,
                    image.width as i32,
                    image.height as i32,
                    image.width as i32 * 4,
                )
                .ok()?;
                let mut cache = ui.selection_rasters.borrow_mut();
                if cache.len() >= 2 {
                    cache.remove(0);
                }
                cache.push((key, surface.clone()));
                Some((surface, density))
            })
        }
    };
    if let Some((surface, density)) = raster {
        cr.scale(1.0 / density as f64, 1.0 / density as f64);
        if cr.set_source_surface(&surface, 0.0, 0.0).is_ok() {
            let _ = cr.paint_with_alpha(fg.alpha() as f64);
        }
    }
    let _ = cr.restore();
}
