//! One native palette/editor for every RGB property. Semantic choices belong to
//! each property; viewing a color never records a recent choice or edits data.
use super::*;

const COLORS: &[&str] = &[
    "FFFFFF", "F6F5F4", "DEDDDA", "C0BFBC", "9A9996", "77767B", "3D3846", "000000", "99C1F1",
    "62A0EA", "3584E4", "1C71D8", "1A5FB4", "0000FF", "00FFFF", "B4F9F8", "8FF0A4", "57E389",
    "33D17A", "2EC27E", "26A269", "00FF00", "008000", "C1F1CC", "F9F06B", "F8E45C", "F6D32D",
    "F5C211", "E5A50A", "FFFF00", "FFA500", "F8E9B8", "FFBE6F", "FFA348", "FF7800", "E66100",
    "C64600", "F66151", "FF0000", "C01C28", "DC8ADD", "C061CB", "9141AC", "813D9C", "613583",
    "FF00FF", "A51D2D", "63452C",
];
thread_local! {
    static RECENT: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static PICKERS: RefCell<Vec<std::rc::Weak<InlineColor>>> = const { RefCell::new(Vec::new()) };
}
fn normalized(value: &str) -> Option<String> {
    gdk::RGBA::parse(value)
        .ok()
        .map(|c| rgba_hex(&c))
        .or_else(|| {
            (value.len() == 6 && value.bytes().all(|c| c.is_ascii_hexdigit()))
                .then(|| value.to_uppercase())
        })
}
fn record(value: &str) {
    RECENT.with(|colors| {
        let mut colors = colors.borrow_mut();
        colors.retain(|c| c != value);
        colors.insert(0, value.to_owned());
        colors.truncate(8);
    });
    let pickers = PICKERS.with(|items| {
        let mut items = items.borrow_mut();
        items.retain(|w| w.strong_count() > 0);
        items
            .iter()
            .filter_map(std::rc::Weak::upgrade)
            .collect::<Vec<_>>()
    });
    for picker in pickers {
        picker.refresh_recents();
    }
}
#[allow(deprecated)]
pub(super) struct InlineColor {
    pub(super) widget: gtk::Box,
    pub(super) editor: gtk::ColorChooserWidget,
    stack: gtk::Stack,
    recent: gtk::FlowBox,
    recent_title: gtk::Label,
    current: RefCell<Option<String>>,
    mixed: Cell<bool>,
    cells: RefCell<Vec<(String, gtk::Button)>>,
    custom: gtk::Button,
    syncing: Cell<bool>,
    apply: Rc<dyn Fn(&str) -> bool>,
}
#[allow(deprecated)]
impl InlineColor {
    pub(super) fn sync(&self, color: Option<&str>) {
        self.sync_state(color, false);
    }
    pub(super) fn sync_state(&self, color: Option<&str>, mixed: bool) {
        let color = color.and_then(normalized);
        self.current.replace(color.clone());
        self.mixed.set(mixed);
        self.syncing.set(true);
        if let Some(rgba) = color
            .as_deref()
            .and_then(|s| gdk::RGBA::parse(&format!("#{s}")).ok())
        {
            self.editor.set_rgba(&rgba);
        }
        self.syncing.set(false);
        self.mark();
    }
    fn mark(&self) {
        let color = self.current.borrow();
        for (value, button) in self.cells.borrow().iter() {
            let selected = !self.mixed.get() && color.as_ref() == Some(value);
            button.update_state(&[gtk::accessible::State::Pressed(if selected {
                gtk::AccessibleTristate::True
            } else {
                gtk::AccessibleTristate::False
            })]);
            if selected {
                button.add_css_class("current-style");
            } else {
                button.remove_css_class("current-style");
            }
            button.queue_draw();
        }
        let custom = color.as_ref().filter(|c| !COLORS.contains(&c.as_str()));
        self.custom.set_label(&if self.mixed.get() {
            format!(
                "{} — {}",
                wordcraft_gnome::t!("Custom Color…"),
                wordcraft_gnome::t!("Mixed")
            )
        } else if let Some(color) = custom {
            format!("{} — #{color}", wordcraft_gnome::t!("Custom Color…"))
        } else {
            wordcraft_gnome::t!("Custom Color…").into()
        });
    }
    fn choose(&self, color: &str) {
        if self.syncing.get() {
            return;
        }
        let Some(value) = normalized(color) else {
            return;
        };
        if (self.apply)(&value) {
            self.current.replace(Some(value.clone()));
            self.mixed.set(false);
            record(&value);
            self.mark();
        }
    }
    fn cell(self: &Rc<Self>, value: &str) -> gtk::Button {
        let button = gtk::Button::new();
        button.add_css_class("flat");
        button.set_widget_name(&format!("color-swatch-{}", value.to_lowercase()));
        let label = wordcraft_gnome::i18n::interpolate(
            wordcraft_gnome::t!("Color #{color}"),
            &[("color", value.to_owned())],
        );
        accessible_name(&button, &label);
        button.set_tooltip_text(Some(&label));
        let swatch = gtk::DrawingArea::new();
        swatch.set_content_width(24);
        swatch.set_content_height(24);
        let rgb = gdk::RGBA::parse(&format!("#{value}")).unwrap();
        let (weak, key) = (Rc::downgrade(self), value.to_owned());
        swatch.set_draw_func(move |w, c, width, height| {
            c.set_source_rgb(rgb.red() as f64, rgb.green() as f64, rgb.blue() as f64);
            c.rectangle(1.5, 1.5, width as f64 - 3., height as f64 - 3.);
            let _ = c.fill_preserve();
            let fg = w.color();
            c.set_source_rgba(fg.red() as f64, fg.green() as f64, fg.blue() as f64, 0.45);
            c.set_line_width(1.);
            let _ = c.stroke();
            if weak.upgrade().is_some_and(|s| {
                !s.mixed.get() && s.current.borrow().as_deref() == Some(key.as_str())
            }) {
                let dark = rgb.red() * 0.2126 + rgb.green() * 0.7152 + rgb.blue() * 0.0722 > 0.5;
                let v = if dark { 0. } else { 1. };
                c.set_source_rgb(v, v, v);
                c.set_line_width(2.);
                c.move_to(6., height as f64 / 2.);
                c.line_to(width as f64 / 2. - 1., height as f64 - 7.);
                c.line_to(width as f64 - 5., 6.);
                let _ = c.stroke();
            }
        });
        button.set_child(Some(&swatch));
        let (weak, key) = (Rc::downgrade(self), value.to_owned());
        button.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.choose(&key);
            }
        });
        self.cells
            .borrow_mut()
            .push((value.to_owned(), button.clone()));
        button
    }
    fn refresh_recents(self: &Rc<Self>) {
        let colors = RECENT.with(|c| c.borrow().clone());
        self.recent_title.set_visible(!colors.is_empty());
        self.recent.set_visible(!colors.is_empty());
        // Preserve standard swatches and their focus; the small recent row is
        // refreshed only after a real choice, never on selection synchronization.
        self.cells.borrow_mut().retain(|(_, b)| {
            b.ancestor(gtk::FlowBox::static_type()).as_ref() != Some(self.recent.upcast_ref())
        });
        while let Some(child) = self.recent.first_child() {
            self.recent.remove(&child);
        }
        for color in colors {
            self.recent.insert(&self.cell(&color), -1);
        }
        self.mark();
    }
}
fn flow() -> gtk::FlowBox {
    let flow = gtk::FlowBox::new();
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_min_children_per_line(4);
    flow.set_max_children_per_line(8);
    flow.set_homogeneous(true);
    flow.set_row_spacing(3);
    flow.set_column_spacing(3);
    flow
}
#[allow(deprecated)]
pub(super) fn embedded_color_chooser(
    name: &str,
    initial: Option<&str>,
    apply: impl Fn(&str) -> bool + 'static,
) -> Rc<InlineColor> {
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 6);
    widget.set_widget_name(name);
    let stack = gtk::Stack::new();
    stack.set_hhomogeneous(false);
    stack.set_vhomogeneous(false);
    let main = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let palette = flow();
    main.append(&palette);
    let recent_title = gtk::Label::new(Some(wordcraft_gnome::t!("Recent Colors")));
    recent_title.set_xalign(0.);
    recent_title.add_css_class("dim-label");
    main.append(&recent_title);
    let recent = flow();
    recent.set_widget_name("color-recents");
    main.append(&recent);
    let custom = style_menu_button(wordcraft_gnome::t!("Custom Color…"));
    custom.set_widget_name("color-custom");
    main.append(&custom);
    stack.add_named(&main, Some("palette"));
    let editor = gtk::ColorChooserWidget::new();
    editor.set_widget_name("color-custom-editor");
    editor.set_use_alpha(false);
    stack.add_named(&editor, Some("custom"));
    stack.set_visible_child_name("palette");
    widget.append(&stack);
    let picker = Rc::new(InlineColor {
        widget,
        editor,
        stack,
        recent,
        recent_title,
        custom,
        current: RefCell::new(None),
        mixed: Cell::new(false),
        cells: RefCell::new(Vec::new()),
        syncing: Cell::new(false),
        apply: Rc::new(apply),
    });
    for color in COLORS {
        palette.insert(&picker.cell(color), -1);
    }
    let weak = Rc::downgrade(&picker);
    picker.custom.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            s.syncing.set(true);
            s.editor.set_show_editor(true);
            s.syncing.set(false);
            s.stack.set_visible_child_name("custom");
        }
    });
    let armed = Rc::new(Cell::new(false));
    let click = gtk::GestureClick::new();
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    let a = armed.clone();
    click.connect_pressed(move |_, _, _, _| a.set(true));
    picker.editor.add_controller(click);
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let a = armed.clone();
    keys.connect_key_pressed(move |_, _, _, _| {
        a.set(true);
        glib::Propagation::Proceed
    });
    picker.editor.add_controller(keys);
    let (weak, a) = (Rc::downgrade(&picker), armed.clone());
    picker.editor.connect_rgba_notify(move |editor| {
        if a.get() && editor.is_mapped() {
            if let Some(s) = weak.upgrade().filter(|s| !s.syncing.get()) {
                s.choose(&rgba_hex(&editor.rgba()));
            }
        }
    });
    let weak = Rc::downgrade(&picker);
    picker.editor.connect_show_editor_notify(move |editor| {
        if !editor.shows_editor() {
            if let Some(s) = weak.upgrade() {
                s.stack.set_visible_child_name("palette");
            }
        }
    });
    picker.sync(initial);
    picker.refresh_recents();
    PICKERS.with(|p| p.borrow_mut().push(Rc::downgrade(&picker)));
    picker
}
