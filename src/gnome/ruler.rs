use super::*;
use wordcraft_gnome::ruler::{self, Lease, Marker, State};
#[derive(Clone)]
struct Drag {
    state: State,
    lease: Lease,
    marker: Marker,
    start_x: f64,
    start_y: f64,
    distance: f32,
    drop: bool,
}
struct View {
    state: State,
    lease: Lease,
    page_x: f64,
    page: usize,
    editable: bool,
    zoom: f64,
}
impl View {
    fn x(&self, d: f32) -> f64 {
        self.page_x + self.state.x(d) as f64 * self.zoom
    }
    fn distance(&self, x: f64) -> f32 {
        self.state
            .distance_at(((x - self.page_x) / self.zoom) as f32)
    }
}
const BAND: f32 = 21.0;

#[derive(Clone, Copy)]
enum Edge {
    Top,
    Left,
    Bottom,
    Right,
}

impl Edge {
    fn band(self, sheet: &gtk::graphene::Rect, outside: bool) -> gtk::graphene::Rect {
        let (x, y, w, h) = (sheet.x(), sheet.y(), sheet.width(), sheet.height());
        match (self, outside) {
            (Self::Top, false) => gtk::graphene::Rect::new(x, y, w, BAND),
            (Self::Top, true) => gtk::graphene::Rect::new(x, y - BAND, w, BAND),
            (Self::Bottom, false) => gtk::graphene::Rect::new(x, y + h - BAND, w, BAND),
            (Self::Bottom, true) => gtk::graphene::Rect::new(x, y + h, w, BAND),
            (Self::Left, false) => gtk::graphene::Rect::new(x, y, BAND, h),
            (Self::Left, true) => gtk::graphene::Rect::new(x - BAND, y, BAND, h),
            (Self::Right, false) => gtk::graphene::Rect::new(x + w - BAND, y, BAND, h),
            (Self::Right, true) => gtk::graphene::Rect::new(x + w, y, BAND, h),
        }
    }
}

fn visible_sheet(sheet: &gtk::graphene::Rect, viewport: &gtk::Widget) -> gtk::graphene::Rect {
    let width = viewport.width().max(0) as f32;
    let height = viewport.height().max(0) as f32;
    let x = sheet.x().clamp(0., width);
    let y = sheet.y().clamp(0., height);
    let right = (sheet.x() + sheet.width()).clamp(x, width);
    let bottom = (sheet.y() + sheet.height()).clamp(y, height);
    gtk::graphene::Rect::new(x, y, right - x, bottom - y)
}

fn clip_sheet(cr: &gtk::cairo::Context, sheet: &gtk::graphene::Rect) {
    let (x, y, w, h) = (
        sheet.x() as f64,
        sheet.y() as f64,
        sheet.width() as f64,
        sheet.height() as f64,
    );
    let r = 6.0_f64.min(w / 2.).min(h / 2.);
    let quarter = std::f64::consts::FRAC_PI_2;
    cr.new_path();
    for (i, (cx, cy)) in [
        (x + w - r, y + r),
        (x + w - r, y + h - r),
        (x + r, y + h - r),
        (x + r, y + r),
    ]
    .into_iter()
    .enumerate()
    {
        cr.arc(cx, cy, r, (i as f64 - 1.) * quarter, i as f64 * quarter);
    }
    cr.close_path();
    cr.clip();
}

struct MarginDrag {
    document: (u64, u64),
    section: usize,
    horizontal: bool,
    leading: bool,
    zoom: f64,
    setup: PageSetup,
    delta: f32,
    gutter: f32,
}

impl MarginDrag {
    fn value(&mut self) -> &mut f32 {
        match (self.horizontal, self.leading) {
            (true, true) => &mut self.setup.left,
            (true, false) => &mut self.setup.right,
            (false, true) => &mut self.setup.top,
            (false, false) => &mut self.setup.bottom,
        }
    }

    fn update(&mut self, dx: f64, dy: f64) {
        let (offset, available) = if self.horizontal {
            (
                dx,
                self.setup.width - self.setup.left - self.setup.right - self.gutter,
            )
        } else {
            (dy, self.setup.height - self.setup.top - self.setup.bottom)
        };
        let sign = if self.leading { 1. } else { -1. };
        let minimum = -*self.value();
        let maximum = (available - wordcraft_engine::cmd::page::MIN_TEXT_ROOM).max(0.);
        self.delta = ((offset / self.zoom * sign) as f32).clamp(minimum, maximum);
    }
}

pub(super) struct Ruler {
    ui: std::rc::Weak<Ui>,
    pub(super) area: gtk::DrawingArea,
    pub(super) reveal: gtk::Revealer,
    overlay: glib::WeakRef<gtk::Overlay>,
    strip: gtk::Box,
    pub(super) vertical: gtk::DrawingArea,
    pub(super) vertical_reveal: gtk::Revealer,
    vertical_strip: gtk::Box,
    pub(super) bottom: gtk::Revealer,
    pub(super) right: gtk::Revealer,
    hovered_page: Cell<Option<usize>>,
    hovered: Cell<bool>,
    focused: Cell<bool>,
    drag: RefCell<Option<Drag>>,
    margin_drag: RefCell<Option<MarginDrag>>,
    selected: Cell<usize>,
    hide: RefCell<Option<glib::SourceId>>,
    announced: RefCell<String>,
}
impl Ruler {
    fn margin_at(&self, x: f64, y: f64) -> Option<MarginDrag> {
        let ui = self.ui.upgrade()?;
        let overlay = self.overlay.upgrade()?;
        let mut current = ui.current.try_borrow_mut().ok()?;
        let doc = current.as_mut()?;
        let index = self.hovered_page.get()?;
        let sheet = doc.page_frames.get(index)?.compute_bounds(&overlay)?;
        let visible = visible_sheet(&sheet, overlay.upcast_ref());
        let (section, body) = doc
            .session
            .layout()
            .pages
            .get(index)
            .map(|p| (p.section, p.body))?;
        let zoom = f64::from(doc.zoom);
        let point = gtk::graphene::Point::new(x as f32, y as f32);
        for (edge, revealer) in [
            (Edge::Top, &self.reveal),
            (Edge::Bottom, &self.bottom),
            (Edge::Left, &self.vertical_reveal),
            (Edge::Right, &self.right),
        ] {
            if !revealer.reveals_child() || !edge.band(&visible, false).contains_point(&point) {
                continue;
            }
            let horizontal = matches!(edge, Edge::Top | Edge::Bottom);
            // Leave the indent and tab marker lanes on the top ruler available.
            let local_y = y - f64::from(visible.y());
            if matches!(edge, Edge::Top) && (local_y < 7.5 || local_y >= 15.0) {
                continue;
            }
            let (position, origin, start, end) = if horizontal {
                (x, sheet.x(), body.x, body.x + body.w)
            } else {
                (y, sheet.y(), body.y, body.y + body.h)
            };
            let leading = (position - f64::from(origin) - f64::from(start) * zoom).abs();
            let trailing = (position - f64::from(origin) - f64::from(end) * zoom).abs();
            if leading.min(trailing) > 7.0 {
                continue;
            }
            let props = doc.session.doc.sections().get(section)?.1;
            return Some(MarginDrag {
                document: (doc.session.document_id(), doc.session.rev()),
                section,
                horizontal,
                leading: leading <= trailing,
                zoom,
                setup: PageSetup {
                    width: props.page_w,
                    height: props.page_h,
                    left: props.margin_left,
                    right: props.margin_right,
                    top: props.margin_top,
                    bottom: props.margin_bottom,
                },
                delta: 0.,
                gutter: props.gutter,
            });
        }
        None
    }

    fn view(&self) -> Option<View> {
        let ui = self.ui.upgrade().filter(|u| !u.closed.get())?;
        let mut value = ui.current.try_borrow_mut().ok()?;
        let d = value.as_mut()?;
        let state = ruler::state(&mut d.session)?;
        let lease = Lease::capture(&d.session);
        let page = self.hovered_page.get().unwrap_or(d.active);
        let rect = d.page_frames.get(page)?.compute_bounds(&self.area)?;
        let editable = state.page == page;
        Some(View {
            page,
            editable,
            state,
            lease,
            page_x: rect.x() as f64,
            zoom: d.zoom as f64,
        })
    }
    pub(super) fn margin_preview_delta(&self) -> Option<f32> {
        self.margin_drag.borrow().as_ref().map(|drag| drag.delta)
    }

    pub(super) fn refresh(&self) {
        let (Some(ui), Some(overlay)) = (self.ui.upgrade(), self.overlay.upgrade()) else {
            return;
        };
        let Ok(c) = ui.current.try_borrow() else {
            return;
        };
        let Some(d) = c.as_ref() else {
            return;
        };
        let page = self.hovered_page.get().unwrap_or(d.active);
        let Some(sheet) = d
            .page_frames
            .get(page)
            .and_then(|p| p.compute_bounds(&overlay))
        else {
            return;
        };
        let sheet = visible_sheet(&sheet, overlay.upcast_ref());
        let (width, height) = (overlay.width() as f32, overlay.height() as f32);
        for (widget, edge, outside) in [
            (self.reveal.upcast_ref::<gtk::Widget>(), Edge::Top, false),
            (self.strip.upcast_ref(), Edge::Top, true),
            (self.vertical_reveal.upcast_ref(), Edge::Left, false),
            (self.vertical_strip.upcast_ref(), Edge::Left, true),
            (self.bottom.upcast_ref(), Edge::Bottom, false),
            (self.right.upcast_ref(), Edge::Right, false),
        ] {
            let band = edge.band(&sheet, outside);
            let x = band.x().clamp(0., width);
            let y = band.y().clamp(0., height);
            let right = (band.x() + band.width()).clamp(0., width);
            let bottom = (band.y() + band.height()).clamp(0., height);
            widget.set_margin_start(x.round() as i32);
            widget.set_margin_top(y.round() as i32);
            widget.set_size_request((right - x).round() as i32, (bottom - y).round() as i32);
            widget.set_visible(right > x && bottom > y);
        }
        for revealer in [&self.bottom, &self.right] {
            if let Some(child) = revealer.child() {
                child.queue_draw();
            }
        }
        self.area.queue_draw();
        self.vertical.queue_draw();
    }
    pub(super) fn stop(&self) {
        if let Some(id) = self.hide.borrow_mut().take() {
            id.remove();
        }
        self.drag.borrow_mut().take();
        self.margin_drag.borrow_mut().take();
        self.reveal.set_reveal_child(false);
        self.vertical_reveal.set_reveal_child(false);
        self.bottom.set_reveal_child(false);
        self.right.set_reveal_child(false);
        self.hovered_page.set(None);
    }
    fn show(&self) {
        if let Some(id) = self.hide.borrow_mut().take() {
            id.remove();
        }
        self.refresh();
        self.reveal
            .set_transition_duration(if self.area.settings().is_gtk_enable_animations() {
                175
            } else {
                0
            });
        self.reveal.set_reveal_child(true);
        self.area.queue_draw();
    }
    pub(super) fn show_and_focus(&self) {
        self.hovered_page.set(None);
        if let Some(ui) = self.ui.upgrade() {
            let page = ui.current.borrow().as_ref().map(|doc| doc.active);
            if let Some(page) = page {
                super::view::scroll_to_page(&ui, page);
            }
        }
        self.show();
        self.area.grab_focus();
    }
    fn hide_later(self: &Rc<Self>) {
        if self.hovered.get() || self.focused.get() || self.drag.borrow().is_some() {
            return;
        }
        if self.hide.borrow().is_some() {
            return;
        }
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(std::time::Duration::from_millis(180), move || {
            if let Some(r) = weak.upgrade() {
                r.hide.borrow_mut().take();
                if !r.hovered.get() && !r.focused.get() && r.drag.borrow().is_none() {
                    r.reveal.set_reveal_child(false);
                }
            }
        });
        self.hide.replace(Some(id));
    }
    fn markers(s: &State) -> Vec<Marker> {
        let mut out = vec![Marker::Left, Marker::First, Marker::Right];
        out.extend((0..s.tabs.len()).map(Marker::Tab));
        out
    }
    fn hit(v: &View, x: f64, y: f64) -> Marker {
        let preferred = if y < 9.0 { Marker::First } else { Marker::Left };
        let mut candidates = Self::markers(&v.state);
        candidates.sort_by_key(|m| if *m == preferred { 0 } else { 1 });
        candidates
            .into_iter()
            .find(|m| {
                (v.x(v.state.distance(*m)) - x).abs() < 9.0
                    && match m {
                        Marker::First => y < 9.0,
                        Marker::Left | Marker::Right => y >= 9.0,
                        _ => true,
                    }
            })
            .unwrap_or(Marker::NewTab)
    }
    fn commit(&self, state: &State, lease: &Lease, marker: Marker, distance: f32, drop: bool) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if !ui
            .host
            .upgrade()
            .and_then(|h| selected_ui(&h))
            .is_some_and(|u| Rc::ptr_eq(&u, &ui))
        {
            return;
        }
        let result = ui
            .current
            .borrow_mut()
            .as_mut()
            .map(|d| ruler::edit(&mut d.session, lease, state, marker, distance, drop));
        match result {
            Some(Ok(true)) => after_content_change(&ui),
            Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
            _ => {}
        }
        self.area.queue_draw();
    }
    fn draw(&self, widget: &gtk::DrawingArea, cr: &gtk::cairo::Context, width: i32, _height: i32) {
        let Some(v) = self.view() else {
            return;
        };
        let state = &v.state;
        let markers = Self::markers(state);
        let marker = markers[self.selected.get().min(markers.len() - 1)];
        let name = match marker {
            Marker::Left => wordcraft_gnome::t!("Leading indent"),
            Marker::First => wordcraft_gnome::t!("First line indent"),
            Marker::Right => wordcraft_gnome::t!("Trailing indent"),
            _ => wordcraft_gnome::t!("Tab stop"),
        };
        let announcement = wordcraft_gnome::i18n::interpolate(
            wordcraft_gnome::t!("Ruler: {marker}, {value} pt, page {page}"),
            &[
                ("marker", name.into()),
                ("value", format!("{:.1}", state.distance(marker))),
                ("page", (v.page + 1).to_string()),
            ],
        );
        if *self.announced.borrow() != announcement {
            self.announced.replace(announcement.clone());
            accessible_name(widget, &announcement);
        }
        self.draw_scale(widget, cr, Edge::Top);
        cr.set_source_rgba(0.16, 0.16, 0.16, 0.85);
        cr.set_line_width(1.);
        if !v.editable {
            return;
        }
        let drag = self.drag.borrow();
        let markers = Self::markers(state);
        for (index, m) in markers.iter().enumerate() {
            let distance = drag
                .as_ref()
                .filter(|d| d.marker == *m)
                .map_or(state.distance(*m), |d| d.distance);
            let x = v.x(distance);
            if x < -10.0 || x > width as f64 + 10.0 {
                continue;
            }
            if self.area.has_focus() && self.selected.get() == index {
                cr.rectangle(
                    x - 7.0,
                    if *m == Marker::First { 0.0 } else { 11.0 },
                    14.0,
                    12.0,
                );
                let _ = cr.stroke();
            }
            match m {
                Marker::First => {
                    cr.move_to(x - 5.0, 3.0);
                    cr.line_to(x + 5.0, 3.0);
                    cr.line_to(x, 7.0);
                    cr.close_path();
                    let _ = cr.fill();
                }
                Marker::Left | Marker::Right => {
                    cr.move_to(x - 5.0, 20.0);
                    cr.line_to(x + 5.0, 20.0);
                    cr.line_to(x, 16.0);
                    cr.close_path();
                    let _ = cr.fill();
                }
                Marker::Tab(n) => {
                    use wordcraft_doc::props::TabAlign;
                    cr.move_to(x, 15.0);
                    cr.line_to(x, 19.0);
                    match state.tabs[*n].align {
                        TabAlign::Left => cr.line_to(x + 5.0, 21.0),
                        TabAlign::Right => cr.line_to(x - 5.0, 21.0),
                        TabAlign::Bar | TabAlign::Clear => {}
                        TabAlign::Center | TabAlign::Decimal => {
                            cr.move_to(x - 5.0, 19.0);
                            cr.line_to(x + 5.0, 19.0);
                        }
                    }
                    let _ = cr.stroke();
                    if state.tabs[*n].align == TabAlign::Decimal {
                        cr.arc(x + 3.0, 20.0, 1.0, 0.0, std::f64::consts::TAU);
                        let _ = cr.fill();
                    }
                }
                _ => {}
            }
        }
        if let Some(d) = drag
            .as_ref()
            .filter(|d| d.marker == Marker::NewTab && !d.drop)
        {
            let x = v.x(d.distance);
            cr.move_to(x, 15.0);
            cr.line_to(x, 20.0);
            cr.line_to(x + 5.0, 20.0);
            let _ = cr.stroke();
        }
    }
    fn draw_scale(&self, widget: &gtk::DrawingArea, cr: &gtk::cairo::Context, edge: Edge) {
        let Some(ui) = self.ui.upgrade() else { return };
        let Ok(mut current) = ui.current.try_borrow_mut() else {
            return;
        };
        let Some(doc) = current.as_mut() else { return };
        let page = self.hovered_page.get().unwrap_or(doc.active);
        let Some(sheet) = doc
            .page_frames
            .get(page)
            .and_then(|frame| frame.compute_bounds(widget))
        else {
            return;
        };
        let Some(&(width, height)) = doc.sizes.get(page) else {
            return;
        };
        let Some(body) = doc.session.layout().pages.get(page).map(|page| page.body) else {
            return;
        };
        let horizontal = matches!(edge, Edge::Top | Edge::Bottom);
        let zoom = f64::from(doc.zoom);
        let origin = f64::from(if horizontal { sheet.x() } else { sheet.y() });
        let length = f64::from(if horizontal {
            widget.width()
        } else {
            widget.height()
        });
        let depth = f64::from(if horizontal {
            widget.height()
        } else {
            widget.width()
        });
        let (margin, content) = if horizontal {
            (body.x, body.w)
        } else {
            (body.y, body.h)
        };
        clip_sheet(cr, &sheet);
        cr.set_source_rgba(1., 1., 1., 0.72);
        let _ = cr.paint();
        let (mut start, mut span) = (origin + f64::from(margin) * zoom, f64::from(content) * zoom);
        if let Some(drag) = self
            .margin_drag
            .borrow()
            .as_ref()
            .filter(|drag| drag.horizontal == horizontal)
        {
            let change = f64::from(drag.delta) * zoom;
            if drag.leading {
                start += change;
            }
            span -= change;
        }
        cr.set_source_rgba(0.30, 0.30, 0.30, 0.10);
        if horizontal {
            cr.rectangle(start, 0., span, depth);
        } else {
            cr.rectangle(0., start, depth, span);
        }
        let _ = cr.fill();
        cr.set_source_rgba(0.22, 0.22, 0.22, 0.19);
        cr.set_line_width(1.);
        let boundary = if matches!(edge, Edge::Top | Edge::Left) {
            depth - 0.5
        } else {
            0.5
        };
        if horizontal {
            cr.move_to(0., boundary);
            cr.line_to(length, boundary);
        } else {
            cr.move_to(boundary, 0.);
            cr.line_to(boundary, length);
        }
        let _ = cr.stroke();
        // Paint marks with the same composited grey as the inward separator.
        cr.set_source_rgba(0.22, 0.22, 0.22, 0.19);
        cr.set_font_size(8.);
        let unit = 72. / 2.54 / 2.;
        let extent = if horizontal { width } else { height };
        for n in 1..=(extent / unit).floor() as i32 {
            let offset = origin + f64::from(n as f32 * unit) * zoom;
            if !(3. ..length - 3.).contains(&offset) {
                continue;
            }
            let (x, y) = if horizontal {
                (offset, depth / 2.)
            } else {
                (depth / 2., offset)
            };
            if n % 2 != 0 {
                if horizontal {
                    cr.move_to(x, y - 1.5);
                    cr.line_to(x, y + 1.5);
                } else {
                    cr.move_to(x - 1.5, y);
                    cr.line_to(x + 1.5, y);
                }
                let _ = cr.stroke();
                continue;
            }
            let label = (n / 2).to_string();
            let Ok(metrics) = cr.text_extents(&label) else {
                continue;
            };
            if offset - metrics.width() / 2. <= 3. || offset + metrics.width() / 2. >= length - 3. {
                continue;
            }
            let _ = cr.save();
            cr.translate(x, y);
            if !horizontal {
                cr.rotate(std::f64::consts::FRAC_PI_2);
            }
            cr.move_to(
                -metrics.x_bearing() - metrics.width() / 2.,
                -metrics.y_bearing() - metrics.height() / 2.,
            );
            let _ = cr.show_text(&label);
            let _ = cr.restore();
        }
    }
}
pub(super) fn install_ruler(ui: &Rc<Ui>, overlay: &gtk::Overlay) -> Rc<Ruler> {
    let reveal = gtk::Revealer::new();
    reveal.set_can_target(false);
    reveal.set_transition_type(gtk::RevealerTransitionType::Crossfade);
    reveal.set_transition_duration(175);
    reveal.set_halign(gtk::Align::Start);
    reveal.set_valign(gtk::Align::Start);

    let area = gtk::DrawingArea::new();
    area.set_content_height(BAND as i32);
    area.set_focusable(true);
    area.set_widget_name("document-ruler");
    accessible_name(&area, wordcraft_gnome::t!("Ruler"));
    area.set_tooltip_text(Some(wordcraft_gnome::t!("Ruler")));
    area.update_property(&[gtk::accessible::Property::Description(wordcraft_gnome::t!("Ruler (cm). Drag indents or tab stops; click to add a left tab. Arrow keys adjust the selected marker, Up/Down choose a marker, Insert adds a tab, and Delete removes it."))]);
    reveal.set_child(Some(&area));
    let strip = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    strip.set_widget_name("ruler-activation-strip");
    let vertical = gtk::DrawingArea::new();
    vertical.set_content_width(BAND as i32);
    vertical.set_widget_name("vertical-document-ruler");
    accessible_name(&vertical, wordcraft_gnome::t!("Vertical ruler (cm)"));
    let vertical_reveal = gtk::Revealer::new();

    vertical_reveal.set_can_target(false);
    vertical_reveal.set_transition_type(gtk::RevealerTransitionType::Crossfade);
    vertical_reveal.set_transition_duration(175);
    vertical_reveal.set_child(Some(&vertical));
    let passive = || {
        let revealer = gtk::Revealer::new();
        revealer.set_can_target(false);
        revealer.set_transition_type(gtk::RevealerTransitionType::Crossfade);
        revealer.set_child(Some(&gtk::DrawingArea::new()));
        revealer
    };
    let bottom = passive();
    let right = passive();
    let vertical_strip = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vertical_strip.set_widget_name("vertical-ruler-activation-strip");
    for w in [
        reveal.upcast_ref::<gtk::Widget>(),
        strip.upcast_ref(),
        vertical_reveal.upcast_ref(),
        vertical_strip.upcast_ref(),
        bottom.upcast_ref(),
        right.upcast_ref(),
    ] {
        w.set_halign(gtk::Align::Start);
        w.set_valign(gtk::Align::Start);
        overlay.add_overlay(w);
        overlay.set_measure_overlay(w, false);
        overlay.set_clip_overlay(w, true);
    }
    let r = Rc::new(Ruler {
        ui: Rc::downgrade(ui),
        area,
        overlay: overlay.downgrade(),
        strip: strip.clone(),
        vertical,
        vertical_reveal,
        vertical_strip,
        reveal,
        bottom,
        right,
        hovered_page: Cell::new(None),
        hovered: Cell::new(false),
        focused: Cell::new(false),
        drag: RefCell::new(None),
        margin_drag: RefCell::new(None),
        selected: Cell::new(0),
        hide: RefCell::new(None),
        announced: RefCell::new(String::new()),
    });
    strip.set_can_target(false);
    r.vertical_strip.set_can_target(false);
    r.reveal.connect_reveal_child_notify(move |reveal| {
        reveal.set_can_target(reveal.reveals_child());
    });
    let weak = Rc::downgrade(&r);
    r.area.set_draw_func(move |w, c, x, y| {
        if let Some(r) = weak.upgrade() {
            r.draw(w, c, x, y);
        }
    });
    for revealer in [&r.reveal, &r.vertical_reveal, &r.bottom, &r.right] {
        let weak = Rc::downgrade(&r);
        revealer.connect_child_revealed_notify(move |_| {
            let Some(r) = weak.upgrade() else { return };
            if [&r.reveal, &r.vertical_reveal, &r.bottom, &r.right]
                .iter()
                .all(|edge| !edge.reveals_child() && !edge.is_child_revealed())
            {
                r.hovered_page.set(None);
            }
        });
    }
    let focus = gtk::EventControllerFocus::new();
    let weak = Rc::downgrade(&r);
    focus.connect_enter(move |_| {
        if let Some(r) = weak.upgrade() {
            r.focused.set(true);
            r.show();
        }
    });
    let weak = Rc::downgrade(&r);
    focus.connect_leave(move |_| {
        if let Some(r) = weak.upgrade() {
            r.focused.set(false);
            r.hide_later();
        }
    });
    r.area.add_controller(focus);
    for (area, edge) in [
        (r.vertical.clone(), Edge::Left),
        (r.bottom.child().unwrap().downcast().unwrap(), Edge::Bottom),
        (r.right.child().unwrap().downcast().unwrap(), Edge::Right),
    ] {
        let weak = Rc::downgrade(&r);
        area.set_draw_func(move |area, cr, _, _| {
            if let Some(r) = weak.upgrade() {
                r.draw_scale(area, cr, edge);
            }
        });
    }
    // Refresh nonmeasuring overlay geometry on viewport allocation/scroll, and
    // on pointer entry after an initially empty tab. No permanent frame callback.
    for adjustment in [ui.scroll.hadjustment(), ui.scroll.vadjustment()] {
        let weak = Rc::downgrade(&r);
        adjustment.connect_changed(move |_| {
            let weak = weak.clone();
            glib::idle_add_local_once(move || {
                if let Some(r) = weak.upgrade() {
                    r.refresh();
                }
            });
        });
    }
    let motion = gtk::EventControllerMotion::new();
    motion.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = Rc::downgrade(&r);
    let canvas = overlay.downgrade();
    motion.connect_motion(move |_, x, y| {
        let (Some(r), Some(canvas)) = (weak.upgrade(), canvas.upgrade()) else {
            return;
        };
        let Some(ui) = r.ui.upgrade() else { return };
        if r.drag.borrow().is_some()
            || r.margin_drag.borrow().is_some()
            || ui.drag_anchor.borrow().is_some()
        {
            return;
        }
        let edges = [Edge::Top, Edge::Left, Edge::Bottom, Edge::Right];
        let point = gtk::graphene::Point::new(x as f32, y as f32);
        let target = ui.current.try_borrow().ok().and_then(|current| {
            let doc = current.as_ref()?;
            let adjustment = ui.scroll.vadjustment();
            let top = adjustment.value();
            doc.arrangement
                .visible(
                    top - f64::from(BAND),
                    top + adjustment.page_size() + f64::from(BAND),
                )
                .into_iter()
                .filter_map(|page| {
                    let frame = doc.page_frames.get(page)?;
                    if r.focused.get() && page != doc.active {
                        return None;
                    }
                    let sheet = frame.compute_bounds(&canvas)?;
                    let visible = visible_sheet(&sheet, canvas.upcast_ref());
                    if visible.width() <= 0. || visible.height() <= 0. {
                        return None;
                    }
                    let hits = edges.map(|edge| {
                        edge.band(&visible, false).contains_point(&point)
                            || edge.band(&visible, true).contains_point(&point)
                    });
                    let dx = point.x() - point.x().clamp(sheet.x(), sheet.x() + sheet.width());
                    let dy = point.y() - point.y().clamp(sheet.y(), sheet.y() + sheet.height());
                    hits.iter()
                        .any(|hit| *hit)
                        .then_some((page, hits, dx * dx + dy * dy))
                })
                .min_by(|(_, _, first), (_, _, second)| first.total_cmp(second))
        });
        let hits = if let Some((page, hits, _)) = target {
            if r.hovered_page.get() != Some(page) {
                for revealer in [&r.reveal, &r.vertical_reveal, &r.bottom, &r.right] {
                    revealer.set_transition_duration(0);
                    revealer.set_reveal_child(false);
                }
                r.hovered_page.set(Some(page));
            }
            r.refresh();
            hits
        } else {
            [false; 4]
        };
        r.hovered.set(hits[0]);
        if hits[0] {
            r.show();
        } else if hits[1..].iter().any(|hit| *hit) && !r.focused.get() {
            r.reveal.set_reveal_child(false);
        } else {
            r.hide_later();
        }
        let duration = if r.area.settings().is_gtk_enable_animations() {
            175
        } else {
            0
        };
        for (revealer, hit) in [&r.vertical_reveal, &r.bottom, &r.right]
            .into_iter()
            .zip(hits[1..].iter().copied())
        {
            revealer.set_transition_duration(duration);
            revealer.set_reveal_child(hit);
        }
        let cursor = r.margin_at(x, y).map(|margin| {
            if margin.horizontal {
                "ew-resize"
            } else {
                "ns-resize"
            }
        });
        canvas.set_cursor_from_name(cursor);
        r.area.set_cursor_from_name(cursor);
    });
    let weak = Rc::downgrade(&r);
    motion.connect_leave(move |_| {
        if let Some(r) = weak.upgrade() {
            if r.margin_drag.borrow().is_some() {
                return;
            }
            r.hovered.set(false);
            r.hide_later();
            r.vertical_reveal.set_reveal_child(false);
            r.bottom.set_reveal_child(false);
            r.right.set_reveal_child(false);
        }
    });
    overlay.add_controller(motion);
    let margins = gtk::GestureDrag::new();
    margins.set_button(gdk::BUTTON_PRIMARY);
    margins.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = Rc::downgrade(&r);
    margins.connect_drag_begin(move |gesture, x, y| {
        let Some(r) = weak.upgrade() else { return };
        let Some(margin) = r.margin_at(x, y) else {
            gesture.set_state(gtk::EventSequenceState::Denied);
            return;
        };
        r.margin_drag.replace(Some(margin));
        gesture.set_state(gtk::EventSequenceState::Claimed);
    });
    let weak = Rc::downgrade(&r);
    margins.connect_drag_update(move |_, dx, dy| {
        if let Some(r) = weak.upgrade() {
            if let Some(margin) = r.margin_drag.borrow_mut().as_mut() {
                margin.update(dx, dy);
            }
            r.refresh();
        }
    });
    let weak = Rc::downgrade(&r);
    margins.connect_drag_end(move |_, _, _| {
        let Some(r) = weak.upgrade() else { return };
        let Some(mut margin) = r.margin_drag.borrow_mut().take() else {
            return;
        };
        r.refresh();
        let Some(ui) =
            r.ui.upgrade()
                .filter(|ui| document_identity(ui) == Some(margin.document))
        else {
            return;
        };
        if margin.delta.abs() < 0.5 {
            return;
        }
        let delta = margin.delta;
        *margin.value() += delta;
        let result = {
            let mut current = ui.current.borrow_mut();
            let Some(doc) = current.as_mut() else { return };
            document_properties::apply_at(&mut doc.session, margin.setup, false, margin.section)
        };
        match result {
            Ok(true) => after_content_change(&ui),
            Ok(false) => {}
            Err(error) => ui.toasts.add_toast(Toast::new(&error)),
        }
    });
    let weak = Rc::downgrade(&r);
    margins.connect_cancel(move |_, _| {
        if let Some(r) = weak.upgrade() {
            r.margin_drag.borrow_mut().take();
            r.refresh();
        }
    });
    overlay.add_controller(margins);

    // The top ruler is interactive and sits beside the scroller in the
    // overlay. The three passive rulers let the scroller receive input directly.
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
    scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak = Rc::downgrade(ui);
    scroll.connect_scroll(move |controller, dx, dy| {
        let Some(ui) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        for (adjustment, delta) in [(ui.scroll.hadjustment(), dx), (ui.scroll.vadjustment(), dy)] {
            let scale = if controller.unit() == gdk::ScrollUnit::Surface {
                1.0
            } else {
                adjustment.step_increment().max(24.0)
            };
            adjustment.set_value(adjustment.value() + delta * scale);
        }
        glib::Propagation::Stop
    });
    r.reveal.add_controller(scroll);

    let drag = gtk::GestureDrag::new();
    drag.set_button(gdk::BUTTON_PRIMARY);
    let weak = Rc::downgrade(&r);
    drag.connect_drag_begin(move |g, x, y| {
        let Some(r) = weak.upgrade() else {
            return;
        };
        let Some(v) = r.view() else {
            return;
        };
        if !v.editable {
            g.set_state(gtk::EventSequenceState::Denied);
            return;
        }
        let marker = Ruler::hit(&v, x, y);
        if marker == Marker::NewTab && !(0.0..=v.state.width).contains(&v.distance(x)) {
            g.set_state(gtk::EventSequenceState::Denied);
            return;
        }
        let index = Ruler::markers(&v.state)
            .iter()
            .position(|m| *m == marker)
            .unwrap_or(0);
        r.selected.set(index);
        let distance = if marker == Marker::NewTab {
            v.distance(x)
        } else {
            v.state.distance(marker)
        };
        r.drag.replace(Some(Drag {
            state: v.state,
            lease: v.lease,
            marker,
            start_x: x,
            start_y: y,
            distance,
            drop: false,
        }));
        r.area.grab_focus();
        r.focused.set(false);
        g.set_state(gtk::EventSequenceState::Claimed);
        r.show();
    });
    let weak = Rc::downgrade(&r);
    drag.connect_drag_update(move |_, dx, dy| {
        if let Some(r) = weak.upgrade() {
            if let Some(v) = r.view() {
                if let Some(d) = r.drag.borrow_mut().as_mut() {
                    let x = d.start_x + dx;
                    let y = d.start_y + dy;
                    d.distance = (v.distance(x) * 2.0).round() / 2.0;
                    d.drop = !(-18.0..=42.0).contains(&y);
                    r.hovered.set(
                        (-f64::from(BAND)..f64::from(BAND)).contains(&y)
                            && (0.0..f64::from(r.area.width())).contains(&x),
                    );
                }
            }
            r.area.queue_draw();
        }
    });
    let weak = Rc::downgrade(&r);
    drag.connect_drag_end(move |_, _, _| {
        if let Some(r) = weak.upgrade() {
            let d = r.drag.borrow_mut().take();
            if let Some(d) = d {
                r.commit(&d.state, &d.lease, d.marker, d.distance, d.drop);
            }
            r.hide_later();
        }
    });
    let weak = Rc::downgrade(&r);
    drag.connect_cancel(move |_, _| {
        if let Some(r) = weak.upgrade() {
            r.drag.borrow_mut().take();
            r.area.queue_draw();
            r.hide_later();
        }
    });
    r.area.add_controller(drag);
    let keys = gtk::EventControllerKey::new();
    let weak = Rc::downgrade(&r);
    keys.connect_key_pressed(move |_, key, _, mods| {
        let Some(r) = weak.upgrade() else {
            return glib::Propagation::Proceed;
        };
        let Some(v) = r.view() else {
            return glib::Propagation::Proceed;
        };
        if mods.contains(gdk::ModifierType::CONTROL_MASK)
            && matches!(key, gdk::Key::z | gdk::Key::Z)
        {
            if let Some(ui) = r.ui.upgrade() {
                apply_user_edit(
                    &ui,
                    if mods.contains(gdk::ModifierType::SHIFT_MASK) {
                        Edit::Redo
                    } else {
                        Edit::Undo
                    },
                );
                r.area.grab_focus();
            }
            return glib::Propagation::Stop;
        }
        let markers = Ruler::markers(&v.state);
        let selected = r.selected.get().min(markers.len() - 1);
        let marker = markers[selected];
        if key == gdk::Key::Escape {
            r.drag.borrow_mut().take();
            if let Some(u) = r.ui.upgrade() {
                u.document_box.grab_focus();
            }
            r.hide_later();
            return glib::Propagation::Stop;
        }
        if !v.editable {
            return glib::Propagation::Proceed;
        }
        if matches!(key, gdk::Key::Up | gdk::Key::Down) {
            r.selected.set(if key == gdk::Key::Down {
                (selected + 1) % markers.len()
            } else {
                (selected + markers.len() - 1) % markers.len()
            });
            r.area.queue_draw();
            return glib::Propagation::Stop;
        }
        if matches!(key, gdk::Key::Left | gdk::Key::Right) {
            let step = if mods.contains(gdk::ModifierType::SHIFT_MASK) {
                10.0
            } else {
                1.0
            };
            let sign = if (key == gdk::Key::Right) != v.state.rtl {
                1.0
            } else {
                -1.0
            };
            r.commit(
                &v.state,
                &v.lease,
                marker,
                v.state.distance(marker) + sign * step,
                false,
            );
            return glib::Propagation::Stop;
        }
        if key == gdk::Key::Insert {
            r.commit(
                &v.state,
                &v.lease,
                Marker::NewTab,
                72.0_f32.min(v.state.width),
                false,
            );
            return glib::Propagation::Stop;
        }
        if key == gdk::Key::Delete && matches!(marker, Marker::Tab(_)) {
            r.commit(&v.state, &v.lease, marker, 0.0, true);
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    r.area.add_controller(keys);
    r
}
