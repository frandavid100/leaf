use super::*;

// Native document tools; engine adapters own all mutations.
pub(super) const PT_PER_MM: f64 = 72.0 / 25.4;
pub(super) fn mm_input(value: f64, min: f64, max: f64) -> gtk::SpinButton {
    let n = gtk::SpinButton::with_range(min, max, 1.0);
    n.set_digits(2);
    n.set_value(value);
    n.set_width_chars(7);
    n.set_valign(gtk::Align::Center);
    n
}
pub(super) fn document_identity(ui: &Ui) -> Option<(u64, u64)> {
    ui.current
        .borrow()
        .as_ref()
        .map(|d| (d.session.document_id(), d.session.rev()))
}
pub(super) fn tools_change(
    ui: &Rc<Ui>,
    f: impl FnOnce(&mut Session) -> Result<(), String>,
) -> bool {
    let result = {
        let mut state = ui.current.borrow_mut();
        let Some(d) = state.as_mut() else {
            return false;
        };
        let rev = d.session.rev();
        f(&mut d.session).map(|_| d.session.rev() != rev)
    };
    match result {
        Ok(changed) => {
            if changed {
                after_content_change(ui);
            } else {
                refresh_selection(ui);
            }
            true
        }
        Err(e) => {
            ui.toasts.add_toast(Toast::new(&e));
            false
        }
    }
}
