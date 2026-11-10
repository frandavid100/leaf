use super::*;

// Shared chrome for the four editing popovers. No document state is stored here.
pub(super) fn format_chevron() -> gtk::Widget {
    let icon = gtk::Image::from_icon_name("go-down-symbolic");
    icon.set_pixel_size(16);
    icon.upcast()
}
pub(super) fn format_menu_label(menu: &gtk::MenuButton, name: &str) {
    menu.add_css_class("flat");
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    row.append(&gtk::Label::new(Some(name)));
    row.append(&format_chevron());
    menu.set_child(Some(&row));
}
pub(super) type FormatBack = Rc<RefCell<Option<Rc<dyn Fn()>>>>;
pub(super) struct FormatHeader {
    pub(super) button: gtk::Button,
    pub(super) label: gtk::Label,
    pub(super) arrow: gtk::Image,
    pub(super) pin: gtk::ToggleButton,
    pub(super) back: FormatBack,
    pub(super) parent_back: FormatBack,
    pub(super) title: String,
    pub(super) entry_page: RefCell<Option<String>>,
}
impl FormatHeader {
    pub(super) fn home(&self) {
        self.label.set_text(&self.title);
        self.label.set_tooltip_text(Some(&self.title));
        let parent = self.parent_back.borrow().clone();
        self.arrow.set_visible(parent.is_some());
        self.button.set_can_target(parent.is_some());
        self.button.set_focusable(parent.is_some());
        self.back.replace(parent);
    }
    pub(super) fn back(&self, label: &str, callback: impl Fn() + 'static) {
        self.label.set_text(label);
        self.label.set_tooltip_text(Some(label));
        self.arrow.set_visible(true);
        self.button.set_can_target(true);
        self.button.set_focusable(true);
        self.back.replace(Some(Rc::new(callback)));
    }
    pub(super) fn page(&self, page: &str, label: &str, callback: impl Fn() + 'static) {
        self.back(label, callback);
        if self.entry_page.borrow().as_deref() == Some(page) {
            self.arrow.set_visible(false);
            self.button.set_can_target(false);
            self.button.set_focusable(false);
            self.back.borrow_mut().take();
        }
    }
}
#[derive(Default)]
pub(super) struct FormatPopoverGroup {
    pub(super) pinned: Cell<bool>,
    pub(super) entries: RefCell<Vec<(glib::WeakRef<gtk::Popover>, Rc<FormatHeader>)>>,
    pub(super) hosts: RefCell<Vec<glib::WeakRef<gtk::Popover>>>,
}
impl FormatPopoverGroup {
    pub(super) fn pin(&self, on: bool) {
        if self.pinned.replace(on) == on {
            return;
        }
        for (_, h) in self.entries.borrow().iter() {
            h.pin.set_active(on);
            h.pin.set_tooltip_text(Some(if on {
                wordcraft_gnome::t!("Unpin")
            } else {
                wordcraft_gnome::t!("Pin")
            }));
        }
    }
    pub(super) fn show(&self, pop: &gtk::Popover) {
        for weak in self.hosts.borrow().iter() {
            if let Some(p) = weak.upgrade() {
                if p != *pop {
                    p.popdown();
                }
            }
        }
        for (weak, h) in self.entries.borrow().iter() {
            if let Some(p) = weak.upgrade() {
                if p != *pop {
                    p.popdown();
                }
                h.pin.set_active(self.pinned.get());
            }
        }
    }
    pub(super) fn outside(
        &self,
        gesture: &gtk::GestureClick,
        window: &ApplicationWindow,
        x: f64,
        y: f64,
    ) {
        if self.pinned.get() {
            return;
        }
        let surface = gesture.current_event().and_then(|e| e.surface());
        let hosts = self
            .hosts
            .borrow()
            .iter()
            .cloned()
            .chain(self.entries.borrow().iter().map(|(p, _)| p.clone()))
            .collect::<Vec<_>>();
        for weak in &hosts {
            if let Some(p) = weak.upgrade() {
                if std::env::args()
                    .any(|a| a == "--split-states-demo" || a == "--text18-pointer-test")
                    && p.is_visible()
                {
                    eprintln!(
                        "SPLIT_OUTSIDE event={surface:?} popup={:?} same={} xy={x},{y}",
                        p.surface(),
                        surface == p.surface()
                    );
                }
                let mut ancestor = surface.clone();
                let mut in_popup = false;
                while let Some(s) = ancestor {
                    if Some(&s) == p.surface().as_ref() {
                        in_popup = true;
                        break;
                    }
                    ancestor = s
                        .downcast::<gdk::Popup>()
                        .ok()
                        .and_then(|s| gdk::prelude::PopupExt::parent(&s));
                }
                if !p.is_visible() || in_popup {
                    continue;
                }
                let anchor = p
                    .parent()
                    .filter(|w| w.is::<gtk::MenuButton>())
                    .and_then(|w| w.compute_bounds(window));
                if anchor.is_some_and(|r| {
                    x >= r.x() as f64
                        && y >= r.y() as f64
                        && x <= (r.x() + r.width()) as f64
                        && y <= (r.y() + r.height()) as f64
                }) {
                    continue;
                }
                p.popdown();
            }
        }
    }
}
type EmbeddedFormat = (
    glib::WeakRef<gtk::Popover>,
    glib::WeakRef<gtk::Popover>,
    glib::WeakRef<gtk::ToggleButton>,
);
thread_local! {
    pub(super) static EMBEDDED_FORMAT: RefCell<Vec<EmbeddedFormat>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn embedded_format(pop: &gtk::Popover) -> Option<(gtk::Popover, gtk::ToggleButton)> {
    EMBEDDED_FORMAT.with(|items| {
        items.borrow().iter().find_map(|(origin, host, pin)| {
            if origin.upgrade().as_ref() == Some(pop) {
                Some((host.upgrade()?, pin.upgrade()?))
            } else {
                None
            }
        })
    })
}
pub(super) fn close_format_popover(pop: &gtk::Popover) {
    if let Some((host, pin)) = embedded_format(pop) {
        if !pin.is_active() {
            host.popdown();
        }
    } else if !popover_pin(pop).is_active() {
        pop.popdown();
    }
}
pub(super) fn format_popover_header(
    pop: &gtk::Popover,
    root: &gtk::Box,
    title: &str,
    group: &Rc<FormatPopoverGroup>,
) -> Rc<FormatHeader> {
    root.set_margin_top(0);
    root.set_spacing(6);
    // Modality never changes on a mapped popup: pinning cannot blink/remap it.
    pop.set_autohide(false);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let button = gtk::Button::new();
    button.add_css_class("flat");
    button.set_hexpand(true);
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let arrow = gtk::Image::from_icon_name("go-previous-symbolic");
    content.append(&arrow);
    let label = gtk::Label::new(Some(title));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label.add_css_class("heading");
    content.append(&label);
    button.set_child(Some(&content));
    row.append(&button);
    let pin = gtk::ToggleButton::new();
    pin.add_css_class("flat");
    pin.set_widget_name("format-pin");
    pin.set_tooltip_text(Some(wordcraft_gnome::t!("Pin")));
    accessible_name(&pin, wordcraft_gnome::t!("Pin"));
    let icon = gtk::DrawingArea::new();
    icon.set_content_width(16);
    icon.set_content_height(16);
    icon.set_halign(gtk::Align::Center);
    icon.set_valign(gtk::Align::Center);
    icon.set_draw_func(|w, c, _, _| {
        c.scale(16. / 18., 16. / 18.);
        let f = w.color();
        c.set_source_rgba(
            f.red() as f64,
            f.green() as f64,
            f.blue() as f64,
            f.alpha() as f64,
        );
        c.set_line_width(1.4);
        c.set_line_join(gtk::cairo::LineJoin::Round);
        c.move_to(5.0, 2.5);
        c.line_to(13.0, 2.5);
        c.move_to(6.0, 2.5);
        c.line_to(6.0, 7.5);
        c.line_to(3.5, 11.0);
        c.line_to(14.5, 11.0);
        c.line_to(12.0, 7.5);
        c.line_to(12.0, 2.5);
        c.move_to(9.0, 11.0);
        c.line_to(9.0, 16.0);
        let _ = c.stroke();
    });
    pin.set_child(Some(&icon));
    row.append(&pin);
    root.append(&row);
    root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let back: FormatBack = Rc::new(RefCell::new(None));
    let action = back.clone();
    button.connect_clicked(move |_| {
        let f = action.borrow().clone();
        if let Some(f) = f {
            f();
        }
    });
    let h = Rc::new(FormatHeader {
        button,
        label,
        arrow,
        pin: pin.clone(),
        back,
        parent_back: Rc::new(RefCell::new(None)),
        title: title.into(),
        entry_page: RefCell::new(None),
    });
    let weak_header = Rc::downgrade(&h);
    pop.connect_closed(move |p| {
        p.set_pointing_to(None::<&gdk::Rectangle>);
        if let Some(h) = weak_header.upgrade() {
            h.entry_page.borrow_mut().take();
        }
    });
    h.home();
    group
        .entries
        .borrow_mut()
        .push((pop.downgrade(), h.clone()));
    let weak = Rc::downgrade(group);
    pin.connect_toggled(move |b| {
        if let Some(g) = weak.upgrade() {
            g.pin(b.is_active());
        }
    });
    let weak = Rc::downgrade(group);
    pop.connect_show(move |p| {
        if let Some(g) = weak.upgrade() {
            g.show(p);
        }
    });
    let keys = gtk::EventControllerKey::new();
    let weak = pop.downgrade();
    let back = h.back.clone();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            let parent = back.borrow().clone();
            if let Some(parent) = parent {
                parent();
                return glib::Propagation::Stop;
            }
            if let Some(p) = weak.upgrade() {
                p.popdown();
                if let Some(anchor) = p.parent() {
                    anchor.grab_focus();
                }
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    pop.add_controller(keys);
    h
}

pub(super) fn rgba_hex(c: &gdk::RGBA) -> String {
    format!(
        "{:02X}{:02X}{:02X}",
        (c.red() * 255.).round() as u8,
        (c.green() * 255.).round() as u8,
        (c.blue() * 255.).round() as u8
    )
}
pub(super) fn rgb_value_hex(v: &serde_json::Value) -> Option<String> {
    let c = v.as_array().filter(|a| a.len() == 3)?;
    Some(format!(
        "#{:02X}{:02X}{:02X}",
        c[0].as_u64()?,
        c[1].as_u64()?,
        c[2].as_u64()?
    ))
}

pub(super) fn popover_pin(pop: &gtk::Popover) -> gtk::ToggleButton {
    if let Some((_, pin)) = embedded_format(pop) {
        return pin;
    }
    find_widget(&pop.child().unwrap(), "format-pin")
        .unwrap()
        .downcast()
        .unwrap()
}

// Same AdwSplitButton and native child presentation as history_button. A
// create-popup callback is the public MenuButton activation hook: this secondary
// action reapplies a property and deliberately creates no menu.
pub(super) fn property_split(open: &gtk::Button, repeat: &gtk::Button) -> adw::SplitButton {
    let split = adw::SplitButton::new();
    split.add_css_class("flat");
    split.add_css_class("image-button");
    split.add_css_class("leaf-format-split");
    split.set_hexpand(true);
    let native = split
        .first_child()
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
    let arrow = native
        .next_sibling()
        .unwrap()
        .next_sibling()
        .unwrap()
        .downcast::<gtk::MenuButton>()
        .unwrap();
    if let Some(child) = open.child() {
        open.set_child(None::<&gtk::Widget>);
        split.set_child(Some(&child));
    }
    // The secondary uses a custom icon rather than a popup arrow. Keep both
    // native children explicitly flat outside the HeaderBar's toolbar context.
    native.set_has_frame(false);
    arrow.set_has_frame(false);
    native.set_widget_name(&open.widget_name());
    native.set_tooltip_text(open.tooltip_text().as_deref());
    native.set_sensitive(open.is_sensitive());
    let source = open.clone();
    native.connect_clicked(move |_| source.emit_clicked());
    let target = native.downgrade();
    open.connect_notify_local(Some("tooltip-text"), move |source, _| {
        if let Some(target) = target.upgrade() {
            target.set_tooltip_text(source.tooltip_text().as_deref());
            if let Some(label) = source.tooltip_text() {
                accessible_name(&target, &label);
            }
        }
    });
    let target = native.downgrade();
    open.connect_sensitive_notify(move |source| {
        if let Some(target) = target.upgrade() {
            target.set_sensitive(source.is_sensitive());
        }
    });

    // Forward activation through a public callback, retaining the original
    // secondary action object so each caller owns its weak target callback.
    arrow.set_icon_name("view-refresh-symbolic");
    arrow.set_widget_name(&repeat.widget_name());
    arrow.set_tooltip_text(repeat.tooltip_text().as_deref());
    arrow.set_sensitive(repeat.is_sensitive());
    let action = repeat.clone();
    arrow.set_create_popup_func(move |button| {
        action.emit_clicked();
        let weak = button.downgrade();
        glib::idle_add_local_once(move || {
            if let Some(button) = weak.upgrade() {
                button.set_active(false);
            }
        });
    });
    let target = arrow.downgrade();
    repeat.connect_sensitive_notify(move |source| {
        if let Some(target) = target.upgrade() {
            target.set_sensitive(source.is_sensitive());
        }
    });
    split
}

/// Property body reapplies its remembered value; the native secondary navigates
/// within the current panel and deliberately does not create another popup.
pub(super) fn reapply_split(open: &gtk::Button, apply: &gtk::Button) -> adw::SplitButton {
    if let Some(child) = open.child() {
        open.set_child(None::<&gtk::Widget>);
        apply.set_child(Some(&child));
    }
    let split = property_split(apply, open);
    let arrow = split
        .last_child()
        .unwrap()
        .downcast::<gtk::MenuButton>()
        .unwrap();
    arrow.set_icon_name("go-next-symbolic");
    split
}
