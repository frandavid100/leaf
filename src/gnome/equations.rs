use super::*;
use wordcraft_gnome::equations;
pub(super) fn show_equation(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let existing = ui
        .current
        .borrow()
        .as_ref()
        .and_then(|d| equations::target(&d.session));
    let (window, root, apply) = reference_window(parent, wordcraft_gnome::t!("Equation"));
    let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
        "UnicodeMath: (a+b)/(c+d), x^2, x_1, √(x). Use ordinary Unicode symbols; this is not a LaTeX editor."
    )));
    hint.set_wrap(true);
    hint.set_max_width_chars(46);
    hint.set_xalign(0.);
    root.append(&hint);
    let input = gtk::Entry::new();
    input.set_widget_name("equation-input");
    input.set_hexpand(true);
    accessible_name(&input, wordcraft_gnome::t!("Mathematical Expression"));
    input.set_text(
        existing
            .as_ref()
            .map(|(_, text, _, _)| text.as_str())
            .unwrap_or("x^2"),
    );
    reference_row(
        &root,
        wordcraft_gnome::t!("Mathematical Expression"),
        &input,
    );
    let templates = gtk::FlowBox::new();
    templates.set_selection_mode(gtk::SelectionMode::None);
    templates.set_max_children_per_line(3);
    for (name, text) in [
        (wordcraft_gnome::t!("Fraction"), "(a+b)/(c+d)"),
        (wordcraft_gnome::t!("Power"), "x^2"),
        (wordcraft_gnome::t!("Subscript"), "x_1"),
        (wordcraft_gnome::t!("Square Root"), "√(x)"),
        (wordcraft_gnome::t!("Greek Symbols"), "α+β=γ"),
        (
            wordcraft_gnome::t!("Quadratic Formula"),
            "x=(-b±√(b^2-4ac))/(2a)",
        ),
    ] {
        let b = gtk::Button::with_label(name);
        let weak = input.downgrade();
        b.connect_clicked(move |_| {
            if let Some(input) = weak.upgrade() {
                input.set_text(text);
                input.grab_focus();
            }
        });
        templates.insert(&b, -1);
    }
    root.append(&templates);
    let display = gtk::CheckButton::with_label(wordcraft_gnome::t!("Display on Its Own Line"));
    display.set_active(existing.as_ref().is_some_and(|(_, _, mode, _)| *mode));
    root.append(&display);
    let supported = existing
        .as_ref()
        .is_none_or(|(_, _, _, editable)| *editable);
    if !supported {
        input.set_sensitive(false);
        templates.set_sensitive(false);
        display.set_sensitive(false);
        hint.set_text(wordcraft_gnome::t!("This imported equation cannot be edited faithfully with the basic editor. Its original structure is preserved."));
    }
    apply.set_sensitive(supported);
    reference_apply(ui, &window, &root, &apply, move |s| {
        equations::apply(s, &input.text(), display.is_active())
    });
}
