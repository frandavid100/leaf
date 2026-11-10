//! Paragraph toolbar glyphs, independent of the settings form.
use super::*;

pub(super) fn paragraph_icon(name: &str) -> gtk::Widget {
    let area = gtk::DrawingArea::new();
    area.set_widget_name(&format!("paragraph-glyph-{name}"));
    area.set_content_width(18);
    area.set_content_height(18);
    area.set_halign(gtk::Align::Center);
    area.set_valign(gtk::Align::Center);
    let kind = name.to_string();
    area.set_draw_func(move |w, cr, _, _| {
        let c = w.color();
        cr.set_source_rgba(
            c.red() as f64,
            c.green() as f64,
            c.blue() as f64,
            c.alpha() as f64,
        );
        cr.set_line_width(1.4);
        match kind.as_str() {
            "format-justify-left-symbolic"
            | "format-justify-center-symbolic"
            | "format-justify-right-symbolic"
            | "format-justify-fill-symbolic"
            | "format-justify-distribute-symbolic" => {
                for (i, width) in [14.0, 10.0, 14.0, 8.0].into_iter().enumerate() {
                    let width = if kind.contains("distribute") || (kind.contains("fill") && i < 3) {
                        14.0
                    } else {
                        width
                    };
                    let x = if kind.contains("center") {
                        (18.0 - width) / 2.0
                    } else if kind.contains("right") {
                        16.0 - width
                    } else {
                        2.0
                    };
                    cr.move_to(x, 3.0 + i as f64 * 4.0);
                    cr.line_to(x + width, 3.0 + i as f64 * 4.0);
                }
                let _ = cr.stroke();
            }
            "view-list-bullet-symbolic" | "view-list-ordered-symbolic" => {
                for i in 0..3 {
                    let y = 4.0 + i as f64 * 5.0;
                    cr.move_to(7.0, y);
                    cr.line_to(17.0, y);
                    let _ = cr.stroke();
                    if kind.contains("bullet") {
                        cr.arc(2.0, y, 1.2, 0.0, std::f64::consts::TAU);
                        let _ = cr.fill();
                    } else {
                        cr.set_font_size(6.5);
                        cr.move_to(0.0, y + 2.0);
                        let _ = cr.show_text(&(i + 1).to_string());
                    }
                }
            }
            "list-remove" => {
                for y in [4., 9., 14.] {
                    cr.move_to(7., y);
                    cr.line_to(17., y);
                }
                cr.move_to(0.5, 6.5);
                cr.line_to(4.5, 10.5);
                cr.move_to(4.5, 6.5);
                cr.line_to(0.5, 10.5);
                let _ = cr.stroke();
            }
            "format-indent-less-symbolic" | "format-indent-more-symbolic" => {
                for y in [2.0, 16.0] {
                    cr.move_to(1.0, y);
                    cr.line_to(17.0, y);
                }
                for y in [6.0, 10.0] {
                    cr.move_to(8.0, y);
                    cr.line_to(17.0, y);
                }
                if kind.contains("less") {
                    cr.move_to(5.0, 5.0);
                    cr.line_to(1.0, 9.0);
                    cr.line_to(5.0, 13.0);
                } else {
                    cr.move_to(1.0, 5.0);
                    cr.line_to(5.0, 9.0);
                    cr.line_to(1.0, 13.0);
                }
                let _ = cr.stroke();
            }
            "replace" => {
                cr.move_to(1.0, 5.0);
                cr.line_to(16.0, 5.0);
                cr.line_to(12.0, 1.0);
                cr.move_to(16.0, 5.0);
                cr.line_to(12.0, 9.0);
                cr.move_to(17.0, 13.0);
                cr.line_to(2.0, 13.0);
                cr.line_to(6.0, 9.0);
                cr.move_to(2.0, 13.0);
                cr.line_to(6.0, 17.0);
                let _ = cr.stroke();
            }
            "spacing" => {
                for y in [4.0, 9.0, 14.0] {
                    cr.move_to(8.0, y);
                    cr.line_to(17.0, y);
                }
                cr.move_to(3.0, 2.0);
                cr.line_to(3.0, 16.0);
                cr.move_to(0.5, 5.0);
                cr.line_to(3.0, 2.0);
                cr.line_to(5.5, 5.0);
                cr.move_to(0.5, 13.0);
                cr.line_to(3.0, 16.0);
                cr.line_to(5.5, 13.0);
                let _ = cr.stroke();
            }
            "shading" => {
                cr.rectangle(2.0, 3.0, 14.0, 12.0);
                let _ = cr.stroke();
                cr.rectangle(4.0, 8.0, 10.0, 5.0);
                let _ = cr.fill();
            }
            _ => {
                cr.set_dash(&[2.0, 2.0], 0.0);
                cr.rectangle(2.0, 2.0, 14.0, 14.0);
                let _ = cr.stroke();
                cr.set_dash(&[], 0.0);
                cr.move_to(2.0, 16.0);
                cr.line_to(16.0, 16.0);
                let _ = cr.stroke();
            }
        }
    });
    area.upcast()
}

pub(super) fn list_choice_content(label: &str, kind: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.append(&paragraph_icon(kind));
    let text = gtk::Label::new(Some(label));
    text.set_xalign(0.);
    text.set_hexpand(true);
    row.append(&text);
    row
}
