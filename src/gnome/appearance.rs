use super::*;

thread_local! {static CSS_INSTALLED:Cell<bool>=const{Cell::new(false)};}
pub(super) fn apply_theme(mode: u32) {
    adw::StyleManager::default().set_color_scheme(match mode {
        1 => adw::ColorScheme::ForceLight,
        2 => adw::ColorScheme::ForceDark,
        _ => adw::ColorScheme::Default,
    });
}
pub(super) fn theme_selector() -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.set_widget_name("theme-selector");
    row.set_halign(gtk::Align::Center);
    row.set_margin_top(6);
    row.set_margin_bottom(6);
    let mut buttons: Vec<gtk::ToggleButton> = Vec::new();
    for (mode, label) in [
        wordcraft_gnome::t!("Follow System"),
        wordcraft_gnome::t!("Light"),
        wordcraft_gnome::t!("Dark"),
    ]
    .into_iter()
    .enumerate()
    {
        let button = gtk::ToggleButton::new();
        button.set_widget_name(&format!("theme-{mode}"));
        button.add_css_class("circular");
        button.add_css_class("flat");
        accessible_name(&button, label);
        button.set_tooltip_text(Some(label));
        if let Some(first) = buttons.first() {
            button.set_group(Some(first));
        }
        let circle = gtk::DrawingArea::new();
        circle.set_content_width(32);
        circle.set_content_height(32);
        let weak = button.downgrade();
        circle.set_draw_func(move |_, cr, _, _| {
            cr.arc(16.0, 16.0, 11.0, 0.0, std::f64::consts::TAU);
            cr.set_source_rgb(
                if mode == 2 { 0.2 } else { 0.95 },
                if mode == 2 { 0.2 } else { 0.95 },
                if mode == 2 { 0.2 } else { 0.95 },
            );
            let _ = cr.fill_preserve();
            cr.set_source_rgb(0.5, 0.5, 0.5);
            cr.set_line_width(1.0);
            let _ = cr.stroke();
            if mode == 0 {
                cr.arc(
                    16.0,
                    16.0,
                    10.0,
                    -std::f64::consts::FRAC_PI_2,
                    std::f64::consts::FRAC_PI_2,
                );
                cr.close_path();
                cr.set_source_rgb(0.2, 0.2, 0.2);
                let _ = cr.fill();
            }
            if weak.upgrade().is_some_and(|b| b.is_active()) {
                cr.set_source_rgb(0.2, 0.5, 0.9);
                cr.arc(25.0, 25.0, 7.0, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
                cr.set_source_rgb(1.0, 1.0, 1.0);
                cr.set_line_width(2.0);
                cr.move_to(21.0, 25.0);
                cr.line_to(24.0, 28.0);
                cr.line_to(29.0, 22.0);
                let _ = cr.stroke();
            }
        });
        button.set_child(Some(&circle));
        button.set_active(wordcraft_gnome::preferences::theme() == mode as u32);
        let weak = circle.downgrade();
        button.connect_toggled(move |b| {
            if let Some(c) = weak.upgrade() {
                c.queue_draw();
            }
            if b.is_active() && wordcraft_gnome::preferences::theme() != mode as u32 {
                if wordcraft_gnome::preferences::set_theme(mode as u32).is_ok() {
                    apply_theme(mode as u32);
                }
            }
        });
        row.append(&button);
        buttons.push(button);
    }
    row.connect_map(move |row| {
        if let Some(button) = find_widget(
            row,
            &format!("theme-{}", wordcraft_gnome::preferences::theme()),
        )
        .and_then(|w| w.downcast::<gtk::ToggleButton>().ok())
        {
            button.set_active(true);
        }
    });
    row
}
pub(super) fn install_css() {
    if CSS_INSTALLED.with(|s| s.replace(true)) {
        return;
    }
    let provider = gtk::CssProvider::new();
    // Adwaita suministra color, espaciados y estados; CSS mínimo para papel.
    provider.load_from_data("
      .document-scroll { background: @window_bg_color; }
      .document-sheet { background: white; border-radius: 6px; box-shadow: 0 1px 4px alpha(black, 0.15); }
      .leaf-dark-document .document-sheet, .leaf-dark-document .thumbnail-sheet { background: #333333; }
      .thumbnail-sheet { background: white; border-radius: 6px; border: 1px solid alpha(black, 0.12); background-clip: content-box; }
      .thumbnail-button { padding: 12px; min-width: 0; min-height: 0; }
      .thumbnail-button.selected-thumb { background: alpha(@window_fg_color, 0.09); }
      .sidebar-selector button { border-radius: 10px; }
      .comment-thread.selected { outline: 2px solid @accent_color; outline-offset: -2px; }
      button.mixed { background: alpha(@accent_bg_color, 0.18); }
      .sidebar-selector button:checked { background: alpha(@window_fg_color, 0.12); }
      button.outline-expander { min-width: 18px; min-height: 24px; padding: 0; }
      .search-options > button { background: transparent; box-shadow: none; }
      button.search-replace { min-width: 24px; min-height: 24px; padding: 0; }
      .style-popover button, .format-popover button { font-weight: normal; }
      .style-row.current-style { background: alpha(@window_fg_color, 0.14); border-radius: 6px; }
      .format-popover button:checked:not(.split-action) { background: alpha(@window_fg_color, 0.14); }
      .page-indicator, .zoom-value { font-feature-settings: 'tnum'; }
    ");
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
