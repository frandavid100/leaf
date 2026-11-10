//! Image metadata and a non-destructive crop draft; only commit runs an engine command.
use super::*;
use serde_json::json;
use wordcraft_doc::para::InlineObject;
fn object(ui: &Rc<Ui>) -> Option<(Pos, InlineObject)> {
    let current = ui.current.borrow();
    let s = &current.as_ref()?.session;
    let (pos, _, _) = picture_properties_target(s)?;
    Some((
        pos.clone(),
        s.doc.para_at(&pos)?.object_at(pos.off)?.clone(),
    ))
}
fn valid(ui: &Rc<Ui>, identity: Option<(u64, u64)>, pos: &Pos) -> bool {
    !ui.closed.get()
        && document_identity(ui) == identity
        && object(ui).is_some_and(|(p, _)| &p == pos)
        && ui
            .host
            .upgrade()
            .is_some_and(|h| selected_ui(&h).is_some_and(|u| Rc::ptr_eq(ui, &u)))
}
fn dialog(ui: &Rc<Ui>, title: &str, action: &str) -> Option<(adw::Dialog, gtk::Box, gtk::Button)> {
    let host = ui.host.upgrade()?;
    let window = adw::Dialog::new();
    window.set_title(title);
    window.set_content_width(560);
    window.set_content_height(490);
    let toolbar = ToolbarView::new();
    let header = HeaderBar::new();
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    header.pack_start(&cancel);
    let save = gtk::Button::with_label(action);
    save.add_css_class("suggested-action");
    header.pack_end(&save);
    toolbar.add_top_bar(&header);
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.set_margin_top(12);
    root.set_margin_bottom(12);
    root.set_margin_start(16);
    root.set_margin_end(16);
    toolbar.set_content(Some(&root));
    window.set_child(Some(&toolbar));
    let weak = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = weak.upgrade() {
            w.close();
        }
    });
    window.present(Some(&host.window));
    Some((window, root, save))
}
pub(super) fn alt_text(ui: &Rc<Ui>) {
    let Some((pos, image)) = object(ui) else {
        return;
    };
    let identity = document_identity(ui);
    let Some((window, root, save)) = dialog(
        ui,
        wordcraft_gnome::t!("Alternative Text"),
        wordcraft_gnome::t!("Save"),
    ) else {
        return;
    };
    window.set_widget_name("image-alt-dialog");
    save.set_widget_name("image-alt-save");
    let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
        "Describe the image's meaning. A visible caption is separate from alternative text."
    )));
    hint.set_wrap(true);
    hint.set_xalign(0.);
    root.append(&hint);
    let text = gtk::TextView::new();
    text.set_widget_name("image-alt-text");
    text.set_wrap_mode(gtk::WrapMode::WordChar);
    text.buffer().set_text(image.alt_text());
    accessible_name(&text, wordcraft_gnome::t!("Alternative Text"));
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_vexpand(true);
    scroll.set_child(Some(&text));
    root.append(&scroll);
    let decorative = gtk::CheckButton::with_label(wordcraft_gnome::t!("Mark as Decorative"));
    decorative.set_widget_name("image-decorative");
    decorative.set_active(image.is_decorative());
    root.append(&decorative);
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    error.add_css_class("error");
    root.append(&error);
    let weak = Rc::downgrade(ui);
    let w = window.downgrade();
    save.connect_clicked(move |_| {
        let (Some(ui), Some(w)) = (weak.upgrade(), w.upgrade()) else {
            return;
        };
        if !valid(&ui, identity, &pos) {
            error.set_text(wordcraft_gnome::t!(
                "The document has changed; open the image tools again."
            ));
            return;
        }
        let b = text.buffer();
        let value = b.text(&b.start_iter(), &b.end_iter(), true).to_string();
        let decorate = decorative.is_active();
        let pos = pos.clone();
        if tools_change(&ui, move |s| {
            pictures::select(s, &pos)?;
            s.run(
                "object.altText",
                &json!({"text":value,"decorative":decorate}),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
        }) {
            w.close();
        }
    });
}
pub(super) fn rotate(ui: &Rc<Ui>, direction: &str) {
    if let Some(p) = ui.picture.borrow().as_ref() {
        let direction = direction.to_owned();
        p.execute(move |s| {
            s.run("arrange.rotate", &json!({"direction":direction}))
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
    }
}
fn image_rect(width: f64, height: f64, ratio: f64) -> (f64, f64, f64, f64) {
    let w = width.min(height * ratio);
    let h = w / ratio;
    ((width - w) / 2., (height - h) / 2., w, h)
}
pub(super) fn crop(ui: &Rc<Ui>) {
    let Some((
        pos,
        InlineObject::Image {
            media, w, h, crop, ..
        },
    )) = object(ui)
    else {
        return;
    };
    let identity = document_identity(ui);
    let data = ui
        .current
        .borrow()
        .as_ref()
        .and_then(|d| d.session.doc.media.get(&media).map(|v| v.to_vec()));
    let Some(data) = data else { return };
    let Ok(texture) = gdk::Texture::from_bytes(&glib::Bytes::from_owned(data)) else {
        ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
            "This image cannot be previewed in the crop editor."
        )));
        return;
    };
    let Some((window, root, save)) = dialog(
        ui,
        wordcraft_gnome::t!("Crop Image"),
        wordcraft_gnome::t!("Crop"),
    ) else {
        return;
    };
    window.set_widget_name("image-crop-dialog");
    save.set_widget_name("image-crop-apply");
    let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
        "Drag a rectangle to keep that part of the original image. Cancel leaves the image unchanged."
    )));
    hint.set_wrap(true);
    hint.set_xalign(0.);
    root.append(&hint);
    let preview = gtk::Overlay::new();
    preview.set_vexpand(true);
    let picture = gtk::Picture::for_paintable(&texture);
    picture.set_content_fit(gtk::ContentFit::Contain);
    picture.set_can_shrink(true);
    preview.set_child(Some(&picture));
    let area = gtk::DrawingArea::new();
    area.set_widget_name("image-crop-preview");
    area.set_content_width(250);
    area.set_content_height(220);
    area.set_cursor_from_name(Some("crosshair"));
    preview.add_overlay(&area);
    root.append(&preview);
    let draft = Rc::new(RefCell::new(crop.map(|v| v as f64)));
    let updating = Rc::new(Cell::new(false));
    let mut fields = Vec::new();
    let grid = gtk::Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(4);
    root.append(&grid);
    for (i, (label, id)) in [
        (wordcraft_gnome::t!("Left (%)"), "left"),
        (wordcraft_gnome::t!("Top (%)"), "top"),
        (wordcraft_gnome::t!("Right (%)"), "right"),
        (wordcraft_gnome::t!("Bottom (%)"), "bottom"),
    ]
    .into_iter()
    .enumerate()
    {
        let l = gtk::Label::new(Some(label));
        l.set_xalign(0.);
        let n = gtk::SpinButton::with_range(0., 95., 0.5);
        n.set_digits(2);
        n.set_width_chars(5);
        n.set_value(crop[i] as f64 * 100.);
        n.set_widget_name(&format!("image-crop-{id}"));
        accessible_name(&n, label);
        validate_numeric_input(&n);
        grid.attach(&l, (i % 2 * 2) as i32, (i / 2) as i32, 1, 1);
        grid.attach(&n, (i % 2 * 2 + 1) as i32, (i / 2) as i32, 1, 1);
        fields.push(n);
    }
    let error = gtk::Label::new(None);
    error.set_wrap(true);
    error.add_css_class("error");
    root.append(&error);
    for (i, n) in fields.iter().enumerate() {
        let draft = draft.clone();
        let weak = area.downgrade();
        let updating = updating.clone();
        n.connect_value_changed(move |n| {
            if updating.get() {
                return;
            }
            draft.borrow_mut()[i] = n.value() / 100.;
            if let Some(a) = weak.upgrade() {
                a.queue_draw();
            }
        });
    }
    let ratio = texture.width() as f64 / texture.height() as f64;
    let drawn = draft.clone();
    area.set_draw_func(move |_, c, width, height| {
        let (x, y, w, h) = image_rect(width as f64, height as f64, ratio);
        let d = *drawn.borrow();
        let x0 = x + w * d[0];
        let y0 = y + h * d[1];
        let cw = w * (1. - d[0] - d[2]);
        let ch = h * (1. - d[1] - d[3]);
        c.set_source_rgba(0., 0., 0., 0.5);
        c.rectangle(x, y, w, (y0 - y).max(0.));
        c.rectangle(x, y0, (x0 - x).max(0.), ch.max(0.));
        c.rectangle(x0 + cw, y0, (x + w - x0 - cw).max(0.), ch.max(0.));
        c.rectangle(x, y0 + ch, w, (y + h - y0 - ch).max(0.));
        let _ = c.fill();
        c.set_source_rgb(1., 1., 1.);
        c.set_line_width(2.);
        c.rectangle(x0, y0, cw, ch);
        let _ = c.stroke();
    });
    let drag = gtk::GestureDrag::new();
    drag.set_button(gdk::BUTTON_PRIMARY);
    let start = Rc::new(Cell::new((0., 0.)));
    let s = start.clone();
    drag.connect_drag_begin(move |_, x, y| s.set((x, y)));
    let d = draft.clone();
    let controls = fields.clone();
    let a = area.downgrade();
    let update = updating.clone();
    drag.connect_drag_update(move |_, dx, dy| {
        let Some(area) = a.upgrade() else { return };
        let (x, y, w, h) = image_rect(area.width() as f64, area.height() as f64, ratio);
        let (sx, sy) = start.get();
        let ax = ((sx - x) / w).clamp(0., 1.);
        let ay = ((sy - y) / h).clamp(0., 1.);
        let bx = ((sx + dx - x) / w).clamp(0., 1.);
        let by = ((sy + dy - y) / h).clamp(0., 1.);
        let left = ax.min(bx).min(0.95);
        let top = ay.min(by).min(0.95);
        let right = (1. - ax.max(bx)).min(0.95 - left);
        let bottom = (1. - ay.max(by)).min(0.95 - top);
        let value = [left, top, right, bottom];
        d.replace(value);
        update.set(true);
        for (i, n) in controls.iter().enumerate() {
            n.set_value(value[i] * 100.);
        }
        update.set(false);
        area.queue_draw();
    });
    area.add_controller(drag);
    let weak = Rc::downgrade(ui);
    let win = window.downgrade();
    save.connect_clicked(move |_|{let(Some(ui),Some(window))=(weak.upgrade(),win.upgrade())else{return};if !valid(&ui,identity,&pos){error.set_text(wordcraft_gnome::t!("The document has changed; open the image tools again."));return;}
        let mut value=[0.;4];for(i,n)in fields.iter().enumerate(){let text=n.text().replace(',',".");let Ok(v)=text.parse::<f64>()else{n.add_css_class("error");return};if !v.is_finite()||!(0. ..=95.).contains(&v){n.add_css_class("error");return;}value[i]=v/100.;}
        if value[0]+value[2]>0.95||value[1]+value[3]>0.95{error.set_text(wordcraft_gnome::t!("Keep at least 5% of the image in each direction."));return;}
        let width=w as f64*(1.-value[0]-value[2])/(1.-crop[0] as f64-crop[2] as f64);let height=h as f64*(1.-value[1]-value[3])/(1.-crop[1] as f64-crop[3] as f64);let pos=pos.clone();if tools_change(&ui,move|s|{pictures::select(s,&pos)?;s.run("picture.crop",&json!({"left":value[0],"top":value[1],"right":value[2],"bottom":value[3],"width":width,"height":height})).map(|_|()).map_err(|e|e.to_string())}){window.close();}
    });
}
