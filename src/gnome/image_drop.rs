//! Passive drop geometry is separate from the typing selection and async import.
use super::*;
#[derive(Clone)]
pub(super) struct Preview {
    pub(super) identity: Option<(u64, u64)>,
    pub(super) pos: Pos,
    pub(super) rect: selection::Rect,
    point: (f64, f64),
}
fn file_path(value: &glib::Value) -> Option<PathBuf> {
    let files = value.get::<gdk::FileList>().ok()?.files();
    if files.len() != 1 {
        return None;
    }
    let path = files[0].path()?;
    let extension = path.extension()?.to_str()?;
    [
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "wmf", "emf",
    ]
    .iter()
    .any(|v| v.eq_ignore_ascii_case(extension))
    .then_some(path)
}
fn supported(value: Option<glib::Value>) -> bool {
    value.is_some_and(|v| file_path(&v).is_some() || v.get::<gdk::Texture>().is_ok())
}
pub(super) fn clear_preview(ui: &Rc<Ui>) {
    let was_visible = ui.image_drop.borrow_mut().take().is_some();
    if was_visible {
        refresh_overlay_layers(ui);
    }
    if ui.drag_anchor.borrow().is_none() && ui.picture_drag.borrow().is_none() {
        stop_document_drag(ui);
    }
}
pub(super) fn cancel(ui: &Rc<Ui>) {
    clear_preview(ui);
    if let Some(task) = ui.image_read.borrow_mut().take() {
        task.abort();
    }
}
pub(super) fn motion(ui: &Rc<Ui>, x: f64, y: f64) -> bool {
    let inside = ui.current.borrow().as_ref().is_some_and(|d| {
        d.page_frames.iter().any(|f| {
            f.compute_bounds(&ui.document_box).is_some_and(|b| {
                x >= b.x() as f64
                    && y >= b.y() as f64
                    && x <= (b.x() + b.width()) as f64
                    && y <= (b.y() + b.height()) as f64
            })
        })
    });
    let target = if inside {
        inspect_canvas_point(ui, x, y)
    } else {
        None
    };
    let preview = target.and_then(|(page, pos)| {
        let mut current = ui.current.borrow_mut();
        let d = current.as_mut()?;
        d.session.doc.para_at(&pos)?;
        let caret = d.session.layout().caret_on(&pos, page)?;
        Some(Preview {
            identity: Some((d.session.document_id(), d.session.rev())),
            pos,
            point: (x, y),
            rect: selection::Rect {
                page: caret.page,
                x: caret.x as f64,
                y: caret.top as f64,
                width: 2.,
                height: caret.height as f64,
            },
        })
    });
    let valid = preview.is_some();
    ui.image_drop.replace(preview);
    refresh_overlay_layers(ui);
    if valid {
        if let Some(b) = ui.document_box.compute_bounds(&ui.scroll) {
            ui.drag_point
                .set(Some((x + b.x() as f64, y + b.y() as f64)));
            schedule_drag_scroll(ui);
        }
    }
    valid
}
pub(super) fn paint(ui: &Ui, cr: &gtk::cairo::Context, page: usize, zoom: f64) {
    let preview = ui.image_drop.borrow();
    let Some(p) = preview
        .as_ref()
        .filter(|p| p.rect.page == page && document_identity(ui) == p.identity)
    else {
        return;
    };
    let (bg, _) = selection_colors(&ui.document_box, true);
    cr.set_source_rgba(bg.red() as f64, bg.green() as f64, bg.blue() as f64, 1.);
    cr.set_line_width(2.);
    let (x, y, h) = (p.rect.x * zoom, p.rect.y * zoom, p.rect.height * zoom);
    cr.move_to(x, y);
    cr.line_to(x, y + h);
    cr.move_to(x - 4., y);
    cr.line_to(x + 4., y);
    cr.move_to(x - 4., y + h);
    cr.line_to(x + 4., y + h);
    let _ = cr.stroke();
}
fn read_image(path: &Path) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 32 * 1024 * 1024 {
        return Err(wordcraft_gnome::t!("The image is too large.").into());
    }
    if let Some((w, h)) = wordcraft_render::image_size(&bytes) {
        if w > 16384 || h > 16384 || u64::from(w) * u64::from(h) > 64_000_000 {
            return Err(wordcraft_gnome::t!("The image is too large.").into());
        }
    } else if wordcraft_render::metafile_size(&bytes).is_none() {
        return Err(wordcraft_gnome::t!("Could Not Read Image").into());
    }
    Ok(bytes)
}
pub(super) fn install(ui: &Rc<Ui>) {
    let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    target.set_types(&[gdk::FileList::static_type(), gdk::Texture::static_type()]);
    target.set_preload(true);
    let point = Rc::new(Cell::new(None));
    for entering in [true, false] {
        let weak = Rc::downgrade(ui);
        let point = point.clone();
        let callback = move |target: &gtk::DropTarget, x, y| {
            point.set(Some((x, y)));
            let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                return gdk::DragAction::empty();
            };
            // Preloading is asynchronous. Keep negotiation alive until value-notify
            // can validate the payload; a drop still requires a visible valid caret.
            if target.value().is_none() {
                clear_preview(&ui);
                return gdk::DragAction::COPY;
            }
            if supported(target.value()) && motion(&ui, x, y) {
                gdk::DragAction::COPY
            } else {
                clear_preview(&ui);
                gdk::DragAction::empty()
            }
        };
        if entering {
            target.connect_enter(callback);
        } else {
            target.connect_motion(callback);
        }
    }
    let weak = Rc::downgrade(ui);
    let point_ref = point.clone();
    target.connect_value_notify(move |target| {
        if let (Some(ui), Some((x, y))) = (weak.upgrade(), point_ref.get()) {
            if supported(target.value()) {
                motion(&ui, x, y);
            } else {
                clear_preview(&ui);
            }
        }
    });
    let weak = Rc::downgrade(ui);
    target.connect_leave(move |_| {
        point.set(None);
        if let Some(ui) = weak.upgrade() {
            clear_preview(&ui);
        }
    });
    let weak = Rc::downgrade(ui);
    target.connect_drop(move |_, value, x, y| {
        let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
            return false;
        };
        let Some(preview) = ui.image_drop.borrow().clone() else {
            return false;
        };
        if preview.identity != document_identity(&ui)
            || (preview.point.0 - x).abs() > 3.
            || (preview.point.1 - y).abs() > 3.
            || !supported(Some(value.clone()))
        {
            eprintln!(
                "DROP36 rejected point={:?} identity={:?} actual={:?}",
                preview.point,
                preview.identity,
                document_identity(&ui)
            );
            clear_preview(&ui);
            return false;
        }
        clear_preview(&ui);
        if let Some(old) = ui.image_read.borrow_mut().take() {
            old.abort();
        }
        let selection = ui.current.borrow().as_ref().map(|d| d.session.sel.clone());
        let path = file_path(value);
        let texture = value.get::<gdk::Texture>().ok();
        let weak = Rc::downgrade(&ui);
        let task = glib::spawn_future_local(async move {
            let bytes = if let Some(path) = path {
                gio::spawn_blocking(move || read_image(&path))
                    .await
                    .unwrap_or_else(|_| Err(wordcraft_gnome::t!("Could Not Read Image").into()))
            } else if let Some(texture) = texture {
                if texture.width() > 16384
                    || texture.height() > 16384
                    || i64::from(texture.width()) * i64::from(texture.height()) > 64_000_000
                {
                    Err(wordcraft_gnome::t!("The image is too large.").into())
                } else {
                    Ok(texture.save_to_png_bytes().as_ref().to_vec())
                }
            } else {
                return;
            };
            let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                return;
            };
            ui.image_read.borrow_mut().take();
            if document_identity(&ui) != preview.identity
                || ui.current.borrow().as_ref().map(|d| d.session.sel.clone()) != selection
            {
                return;
            }
            match bytes {
                Ok(bytes) => {
                    tools_change(&ui, |s| {
                        wordcraft_gnome::operations::atomic(
                            s,
                            wordcraft_gnome::t!("Insert Image"),
                            |s| {
                                selection::set_caret(s, &preview.pos, false)?;
                                s.page_hint = preview.rect.page;
                                wordcraft_gnome::pictures::insert_data(s, &bytes)
                            },
                        )
                    });
                    ui.document_box.grab_focus();
                    caret_ensure_visible(&ui);
                }
                Err(e) => ui.toasts.add_toast(Toast::new(&e)),
            }
        });
        ui.image_read.replace(Some(task));
        true
    });
    ui.document_box.add_controller(target);
}
