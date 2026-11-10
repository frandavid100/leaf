//! GTK's standard embedded color chooser, shared by every RGB property.
//! Automatic/None remain property actions; synchronization never applies a color.
use super::*;

// GTK 4 retains this native palette and custom editor. Keep the same chooser
// Leaf used before alpha.43, without an application-defined palette or recents.
#[allow(deprecated)]
pub(super) struct InlineColor {
    pub(super) widget: gtk::Box,
    pub(super) editor: gtk::ColorChooserWidget,
    syncing: Rc<Cell<bool>>,
}
#[allow(deprecated)]
impl InlineColor {
    pub(super) fn sync(&self, color: Option<&str>) {
        self.sync_state(color, false);
    }
    pub(super) fn sync_state(&self, color: Option<&str>, mixed: bool) {
        self.syncing.set(true);
        if let Some(rgba) = color.and_then(|value| {
            gdk::RGBA::parse(value)
                .or_else(|_| gdk::RGBA::parse(format!("#{value}")))
                .ok()
        }) {
            self.editor.set_rgba(&rgba);
        }
        self.editor
            .set_tooltip_text(mixed.then(|| wordcraft_gnome::t!("Mixed")));
        self.syncing.set(false);
    }
}
#[allow(deprecated)]
pub(super) fn embedded_color_chooser(
    name: &str,
    initial: Option<&str>,
    apply: impl Fn(&str) -> bool + 'static,
) -> Rc<InlineColor> {
    let widget = gtk::Box::new(gtk::Orientation::Vertical, 6);
    widget.set_widget_name(name);
    let editor = gtk::ColorChooserWidget::new();
    editor.set_widget_name("color-custom-editor");
    editor.set_use_alpha(false);
    widget.append(&editor);
    let syncing = Rc::new(Cell::new(false));
    let armed = Rc::new(Cell::new(false));
    let click = gtk::GestureClick::new();
    click.set_propagation_phase(gtk::PropagationPhase::Capture);
    let input = armed.clone();
    click.connect_pressed(move |_, _, _, _| input.set(true));
    editor.add_controller(click);
    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let input = armed.clone();
    keys.connect_key_pressed(move |_, _, _, _| {
        input.set(true);
        glib::Propagation::Proceed
    });
    editor.add_controller(keys);
    // Native widget callbacks own their small guards/action, not the controller
    // or parent widget. Builders may drop InlineColor without losing its action.
    let apply: Rc<dyn Fn(&str) -> bool> = Rc::new(apply);
    let (guard, action) = (syncing.clone(), apply.clone());
    editor.connect_rgba_notify(move |editor| {
        if armed.get() && editor.is_mapped() && !guard.get() {
            action(&rgba_hex(&editor.rgba()));
        }
    });
    let guard = syncing.clone();
    editor.connect_color_activated(move |editor, rgba| {
        if editor.is_mapped() && !guard.get() {
            apply(&rgba_hex(rgba));
        }
    });
    let picker = Rc::new(InlineColor {
        widget,
        editor,
        syncing,
    });
    picker.sync(initial);
    picker
}
