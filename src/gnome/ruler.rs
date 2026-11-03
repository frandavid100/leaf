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
const BAND: f32 = 24.0;

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

pub(super) struct Ruler {
    ui: std::rc::Weak<Ui>,
    pub(super) area: gtk::DrawingArea,
    pub(super) reveal: gtk::Revealer,
    overlay: glib::WeakRef<gtk::Overlay>,
    strip: gtk::Box,
    pub(super) vertical: gtk::DrawingArea,
    pub(super) vertical_reveal: gtk::Revealer,
    vertical_strip: gtk::Box,
    bottom: gtk::Revealer,
    right: gtk::Revealer,
    hovered_page: Cell<Option<usize>>,
    hovered: Cell<bool>,
    focused: Cell<bool>,
    drag: RefCell<Option<Drag>>,
    selected: Cell<usize>,
    hide: RefCell<Option<glib::SourceId>>,
    announced: RefCell<String>,
}
impl Ruler {
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
        if let Some(id) = self.hide.borrow_mut().take() {
            id.remove();
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
                    cr.move_to(x - 5.0, 22.0);
                    cr.line_to(x + 5.0, 22.0);
                    cr.line_to(x, 16.0);
                    cr.close_path();
                    let _ = cr.fill();
                }
                Marker::Tab(n) => {
                    use wordcraft_doc::props::TabAlign;
                    cr.move_to(x, 15.0);
                    cr.line_to(x, 21.0);
                    match state.tabs[*n].align {
                        TabAlign::Left => cr.line_to(x + 5.0, 21.0),
                        TabAlign::Right => cr.line_to(x - 5.0, 21.0),
                        TabAlign::Bar | TabAlign::Clear => {}
                        TabAlign::Center | TabAlign::Decimal => {
                            cr.move_to(x - 5.0, 21.0);
                            cr.line_to(x + 5.0, 21.0);
                        }
                    }
                    let _ = cr.stroke();
                    if state.tabs[*n].align == TabAlign::Decimal {
                        cr.arc(x + 3.0, 22.0, 1.0, 0.0, std::f64::consts::TAU);
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
            cr.line_to(x, 22.0);
            cr.line_to(x + 5.0, 22.0);
            let _ = cr.stroke();
        }
    }
    fn draw_scale(&self, widget: &gtk::DrawingArea, cr: &gtk::cairo::Context, edge: Edge) {
        let Some(ui) = self.ui.upgrade() else { return };
        let Ok(current) = ui.current.try_borrow() else {
            return;
        };
        let Some(doc) = current.as_ref() else { return };
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
        clip_sheet(cr, &sheet);
        cr.set_source_rgba(0.16, 0.16, 0.16, 0.58);
        cr.set_line_width(1.);
        cr.set_font_size(8.);
        let horizontal = matches!(edge, Edge::Top | Edge::Bottom);
        let unit = 72. / 2.54 / 2.;
        let extent = if horizontal { width } else { height };
        let band = edge.band(&sheet, false);
        for n in 0..=(extent / unit).floor() as i32 {
            let offset = (n as f32 * unit * doc.zoom) as f64;
            let major = n % 2 == 0;
            let length = if major { 6. } else { 3. };
            if horizontal {
                let x = sheet.x() as f64 + offset;
                let y = band.y() as f64 + if matches!(edge, Edge::Top) { 15. } else { 5. };
                let direction = if matches!(edge, Edge::Top) { -1. } else { 1. };
                cr.move_to(x.round() + 0.5, y);
                cr.line_to(x.round() + 0.5, y + length * direction);
                let _ = cr.stroke();
                if major {
                    cr.move_to(x + 3., y + if direction < 0. { -5. } else { 15. });
                    let _ = cr.show_text(&(n / 2).to_string());
                }
            } else {
                let y = sheet.y() as f64 + offset;
                let x = band.x() as f64 + if matches!(edge, Edge::Left) { 20. } else { 4. };
                let direction = if matches!(edge, Edge::Left) { -1. } else { 1. };
                cr.move_to(x, y.round() + 0.5);
                cr.line_to(x + length * direction, y.round() + 0.5);
                let _ = cr.stroke();
                if major {
                    cr.move_to(
                        band.x() as f64 + if direction < 0. { 3. } else { 11. },
                        y + 8.,
                    );
                    let _ = cr.show_text(&(n / 2).to_string());
                }
            }
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
    area.set_content_height(24);
    area.set_focusable(true);
    area.set_widget_name("document-ruler");
    accessible_name(&area, wordcraft_gnome::t!("Ruler"));
    area.set_tooltip_text(Some(wordcraft_gnome::t!("Ruler")));
    area.update_property(&[gtk::accessible::Property::Description(wordcraft_gnome::t!("Ruler (cm). Drag indents or tab stops; click to add a left tab. Arrow keys adjust the selected marker, Up/Down choose a marker, Insert adds a tab, and Delete removes it."))]);
    reveal.set_child(Some(&area));
    let strip = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    strip.set_widget_name("ruler-activation-strip");
    let vertical = gtk::DrawingArea::new();
    vertical.set_content_width(24);
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
        if r.drag.borrow().is_some() || ui.drag_anchor.borrow().is_some() {
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
                    let hits = edges.map(|edge| {
                        edge.band(&sheet, false).contains_point(&point)
                            || edge.band(&sheet, true).contains_point(&point)
                    });
                    hits.iter().any(|hit| *hit).then_some((
                        page,
                        hits,
                        sheet.contains_point(&point),
                    ))
                })
                .max_by_key(|(_, _, inside)| *inside)
        });
        let hits = if let Some((page, hits, _)) = target {
            r.hovered_page.set(Some(page));
            r.refresh();
            hits
        } else {
            r.hovered_page.set(None);
            r.refresh();
            [false; 4]
        };
        r.hovered.set(hits[0]);
        if hits[0] {
            r.show();
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
    });
    let weak = Rc::downgrade(&r);
    motion.connect_leave(move |_| {
        if let Some(r) = weak.upgrade() {
            r.hovered.set(false);
            r.hovered_page.set(None);
            r.hide_later();
            r.vertical_reveal.set_reveal_child(false);
            r.bottom.set_reveal_child(false);
            r.right.set_reveal_child(false);
        }
    });
    overlay.add_controller(motion);
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
                    d.distance = (v.distance(d.start_x + dx) * 2.0).round() / 2.0;
                    d.drop = d.start_y + dy < -18.0 || d.start_y + dy > 42.0;
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
