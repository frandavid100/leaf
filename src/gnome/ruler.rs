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
    page_width: f32,
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
pub(super) struct Ruler {
    ui: std::rc::Weak<Ui>,
    pub(super) area: gtk::DrawingArea,
    pub(super) reveal: gtk::Revealer,
    overlay: glib::WeakRef<gtk::Overlay>,
    strip: gtk::Box,
    pub(super) vertical: gtk::DrawingArea,
    pub(super) vertical_reveal: gtk::Revealer,
    vertical_strip: gtk::Box,
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
        let page = d.active;
        let rect = d.page_frames.get(page)?.compute_bounds(&self.area)?;
        let page_width = d.sizes.get(page)?.0;
        let editable = state.page == page;
        Some(View {
            page,
            page_width,
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
        let Some(rect) = d
            .page_frames
            .get(d.active)
            .and_then(|p| p.compute_bounds(&overlay))
        else {
            return;
        };
        let width = overlay.width() as f32;
        let height = overlay.height() as f32;
        if width <= 0. || height <= 0. {
            return;
        }
        // The horizontal ruler stays below the titlebar, with a small gap.
        // Both axes use the real sheet bounds and are clipped by the viewport.
        let x = rect.x().max(0.).min(width);
        let right = (rect.x() + rect.width()).max(x).min(width);
        let y = rect.y().max(8.).min(height);
        let bottom = (rect.y() + rect.height()).max(y).min(height);
        for w in [
            self.reveal.upcast_ref::<gtk::Widget>(),
            self.strip.upcast_ref(),
        ] {
            w.set_margin_start(x.round() as i32);
            w.set_margin_top(8);
            w.set_size_request(
                (right - x).round().max(1.) as i32,
                if w == self.strip.upcast_ref::<gtk::Widget>() {
                    5
                } else {
                    24
                },
            );
            w.set_visible(right > x);
        }
        for w in [
            self.vertical_reveal.upcast_ref::<gtk::Widget>(),
            self.vertical_strip.upcast_ref(),
        ] {
            w.set_margin_start((rect.x() - 28.).max(0.).min((width - 24.).max(0.)).round() as i32);
            w.set_margin_top(y.round() as i32);
            w.set_size_request(
                if w == self.vertical_strip.upcast_ref::<gtk::Widget>() {
                    5
                } else {
                    24
                },
                (bottom - y).round().max(1.) as i32,
            );
            w.set_visible(bottom > y);
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
    fn draw(&self, widget: &gtk::DrawingArea, cr: &gtk::cairo::Context, width: i32, height: i32) {
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
        let fg = gdk::RGBA::new(0.16, 0.16, 0.16, 1.0);
        cr.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 0.0);
        cr.rectangle(0.0, 0.0, width as f64, height as f64);
        let _ = cr.fill();
        let (a, b) = (v.x(0.0), v.x(state.width));
        cr.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 0.07);
        if v.editable {
            cr.rectangle(a.min(b), 2.0, (b - a).abs(), height as f64 - 4.0);
        }
        let _ = cr.fill();
        cr.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 1.0);
        cr.set_line_width(1.0);
        cr.set_font_size(8.0);
        // Centimetres are converted from document points, never device pixels.
        let unit = 72.0 / 2.54_f32;
        let max = (v.page_width / unit * 10.0).ceil() as i32;
        for n in 0..=max {
            let x = v.page_x + (n as f32 * unit / 10.0) as f64 * v.zoom;
            if x < 0.0 || x > width as f64 {
                continue;
            }
            let major = n % 10 == 0;
            let length = if major {
                4.0
            } else if n % 5 == 0 {
                4.0
            } else {
                3.0
            };
            cr.move_to(x.round() + 0.5, 15.0);
            cr.line_to(x.round() + 0.5, 15.0 - length);
            let _ = cr.stroke();
            if major {
                cr.move_to(x + 3.0, 9.0);
                let _ = cr.show_text(&(n / 10).to_string());
            }
        }
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
}
pub(super) fn install_ruler(ui: &Rc<Ui>, overlay: &gtk::Overlay) -> Rc<Ruler> {
    let reveal = gtk::Revealer::new();
    reveal.set_can_target(false);
    reveal.set_transition_type(gtk::RevealerTransitionType::Crossfade);
    reveal.set_transition_duration(175);
    reveal.set_halign(gtk::Align::Start);
    reveal.set_valign(gtk::Align::Start);

    let area = gtk::DrawingArea::new();
    area.add_css_class("leaf-ruler");
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
    vertical.add_css_class("leaf-ruler");
    vertical.set_content_width(24);
    vertical.set_widget_name("vertical-document-ruler");
    accessible_name(&vertical, wordcraft_gnome::t!("Vertical ruler (cm)"));
    let vertical_reveal = gtk::Revealer::new();

    vertical_reveal.set_can_target(false);
    vertical_reveal.set_transition_type(gtk::RevealerTransitionType::Crossfade);
    vertical_reveal.set_transition_duration(175);
    vertical_reveal.set_child(Some(&vertical));
    let vertical_strip = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vertical_strip.set_widget_name("vertical-ruler-activation-strip");
    for w in [
        reveal.upcast_ref::<gtk::Widget>(),
        strip.upcast_ref(),
        vertical_reveal.upcast_ref(),
        vertical_strip.upcast_ref(),
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
        hovered: Cell::new(false),
        focused: Cell::new(false),
        drag: RefCell::new(None),
        selected: Cell::new(0),
        hide: RefCell::new(None),
        announced: RefCell::new(String::new()),
    });
    let weak_strip = strip.downgrade();
    r.reveal.connect_reveal_child_notify(move |reveal| {
        reveal.set_can_target(reveal.reveals_child());
        if let Some(strip) = weak_strip.upgrade() {
            strip.set_can_target(!reveal.reveals_child());
        }
    });
    let weak = Rc::downgrade(&r);
    r.area.set_draw_func(move |w, c, x, y| {
        if let Some(r) = weak.upgrade() {
            r.draw(w, c, x, y);
        }
    });
    let motion = gtk::EventControllerMotion::new();
    let weak = Rc::downgrade(&r);
    motion.connect_enter(move |_, _, _| {
        if let Some(r) = weak.upgrade() {
            if r.ui
                .upgrade()
                .is_some_and(|u| u.current.borrow().is_some() && u.drag_anchor.borrow().is_none())
            {
                r.show();
            }
        }
    });
    strip.add_controller(motion);
    let hover = gtk::EventControllerMotion::new();
    let weak = Rc::downgrade(&r);
    hover.connect_enter(move |_, _, _| {
        if let Some(r) = weak.upgrade() {
            r.hovered.set(true);
            r.show();
        }
    });
    let weak = Rc::downgrade(&r);
    hover.connect_leave(move |_| {
        if let Some(r) = weak.upgrade() {
            r.hovered.set(false);
            r.hide_later();
        }
    });
    r.area.add_controller(hover);
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
    let weak = Rc::downgrade(&r);
    r.vertical.set_draw_func(move |w, cr, _, height| {
        let Some(r) = weak.upgrade() else {
            return;
        };
        let Some(ui) = r.ui.upgrade() else {
            return;
        };
        let Ok(c) = ui.current.try_borrow() else {
            return;
        };
        let Some(d) = c.as_ref() else {
            return;
        };
        let Some((_, page_height)) = d.sizes.get(d.active).copied() else {
            return;
        };
        let Some(rect) = d
            .page_frames
            .get(d.active)
            .and_then(|p| p.compute_bounds(w))
        else {
            return;
        };
        cr.set_source_rgb(0.16, 0.16, 0.16);
        cr.set_line_width(1.);
        cr.set_font_size(8.);
        let unit = 72. / 2.54_f32;
        for n in 0..=(page_height / unit * 10.).ceil() as i32 {
            let y = rect.y() as f64 + (n as f32 * unit / 10. * d.zoom) as f64;
            if y < 0. || y > height as f64 {
                continue;
            }
            let length = if n % 10 == 0 {
                7.
            } else if n % 5 == 0 {
                5.
            } else {
                3.
            };
            cr.move_to(23., y.round() + 0.5);
            cr.line_to(23. - length, y.round() + 0.5);
            let _ = cr.stroke();
            if n % 10 == 0 {
                cr.move_to(2., (y + 3.).clamp(8., (height as f64 - 2.).max(8.)));
                let _ = cr.show_text(&(n / 10).to_string());
            }
        }
    });
    let weak_strip = r.vertical_strip.downgrade();
    r.vertical_reveal
        .connect_reveal_child_notify(move |reveal| {
            reveal.set_can_target(reveal.reveals_child());
            if let Some(strip) = weak_strip.upgrade() {
                strip.set_can_target(!reveal.reveals_child());
            }
        });
    let hover = gtk::EventControllerMotion::new();
    let weak = Rc::downgrade(&r);
    hover.connect_enter(move |_, _, _| {
        if let Some(r) = weak.upgrade() {
            r.refresh();
            r.vertical_reveal.set_transition_duration(
                if r.area.settings().is_gtk_enable_animations() {
                    175
                } else {
                    0
                },
            );
            r.vertical_reveal.set_reveal_child(true);
        }
    });
    r.vertical_strip.add_controller(hover);
    let hover = gtk::EventControllerMotion::new();
    let weak = Rc::downgrade(&r);
    hover.connect_leave(move |_| {
        if let Some(r) = weak.upgrade() {
            r.vertical_reveal.set_reveal_child(false);
        }
    });
    r.vertical.add_controller(hover);
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
        r.refresh();
        let within = |w: &gtk::Widget| {
            w.compute_bounds(&canvas).is_some_and(|b| {
                x >= b.x() as f64
                    && x < (b.x() + b.width()) as f64
                    && y >= b.y() as f64
                    && y < (b.y() + b.height()) as f64
            })
        };
        let horizontal = within(r.strip.upcast_ref())
            || (r.reveal.reveals_child() && within(r.reveal.upcast_ref()));
        r.hovered.set(horizontal);
        if !horizontal {
            r.hide_later();
        }
        let vertical = within(r.vertical_strip.upcast_ref())
            || (r.vertical_reveal.reveals_child() && within(r.vertical_reveal.upcast_ref()));
        if !vertical {
            r.vertical_reveal.set_reveal_child(false);
        }
    });
    let weak = Rc::downgrade(&r);
    motion.connect_leave(move |_| {
        if let Some(r) = weak.upgrade() {
            r.hovered.set(false);
            r.hide_later();
            r.vertical_reveal.set_reveal_child(false);
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
