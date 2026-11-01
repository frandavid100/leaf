use super::*;

// Shared chrome for the four editing popovers. No document state is stored here.
pub(super) fn format_chevron() -> gtk::Widget {
    let a = gtk::DrawingArea::new();
    a.set_content_width(16);
    a.set_content_height(16);
    a.set_halign(gtk::Align::Center);
    a.set_valign(gtk::Align::Center);
    a.set_draw_func(|w, c, _, _| {
        let f = w.color();
        c.set_source_rgba(
            f.red() as f64,
            f.green() as f64,
            f.blue() as f64,
            f.alpha() as f64,
        );
        c.set_line_width(1.6);
        c.set_line_cap(gtk::cairo::LineCap::Round);
        c.set_line_join(gtk::cairo::LineJoin::Round);
        c.move_to(4.5, 6.5);
        c.line_to(8.0, 10.0);
        c.line_to(11.5, 6.5);
        let _ = c.stroke();
    });
    a.upcast()
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
}
impl FormatHeader {
    pub(super) fn home(&self) {
        self.label.set_text(&self.title);
        let parent = self.parent_back.borrow().clone();
        self.arrow.set_visible(parent.is_some());
        self.button.set_can_target(parent.is_some());
        self.button.set_focusable(parent.is_some());
        self.back.replace(parent);
    }
    pub(super) fn back(&self, label: &str, callback: impl Fn() + 'static) {
        self.label.set_text(label);
        self.arrow.set_visible(true);
        self.button.set_can_target(true);
        self.button.set_focusable(true);
        self.back.replace(Some(Rc::new(callback)));
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
                if !p.is_visible() || surface == p.surface() {
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
thread_local! {pub(super) static EMBEDDED_FORMAT:RefCell<Vec<(glib::WeakRef<gtk::Popover>,glib::WeakRef<gtk::Popover>,glib::WeakRef<gtk::ToggleButton>)>>=const{RefCell::new(Vec::new())};}
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
    icon.set_content_width(18);
    icon.set_content_height(18);
    icon.set_halign(gtk::Align::Center);
    icon.set_valign(gtk::Align::Center);
    icon.set_draw_func(|w, c, _, _| {
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
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
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

pub(super) fn popover_pin(pop: &gtk::Popover) -> gtk::ToggleButton {
    if let Some((_, pin)) = embedded_format(pop) {
        return pin;
    }
    find_widget(&pop.child().unwrap(), "format-pin")
        .unwrap()
        .downcast()
        .unwrap()
}
