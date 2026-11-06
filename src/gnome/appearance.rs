use super::*;

thread_local! {static CSS_INSTALLED:Cell<bool>=const{Cell::new(false)};}
pub(super) fn apply_theme(mode: u32) {
    adw::StyleManager::default().set_color_scheme(match mode {
        1 => adw::ColorScheme::ForceLight,
        2 => adw::ColorScheme::ForceDark,
        _ => adw::ColorScheme::Default,
    });
}
// GNOME Text Editor's grouped-checkbutton theme selector; see data/THEME_SELECTOR_SOURCES.md.
pub(super) fn theme_selector() -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    row.set_widget_name("theme-selector");
    row.add_css_class("leaf-theme-selector");
    row.set_hexpand(true);
    let mut buttons: Vec<gtk::CheckButton> = Vec::new();
    for (mode, label) in [
        wordcraft_gnome::t!("Follow System"),
        wordcraft_gnome::t!("Light"),
        wordcraft_gnome::t!("Dark"),
    ]
    .into_iter()
    .enumerate()
    {
        let button = gtk::CheckButton::new();
        button.set_widget_name(&format!("theme-{mode}"));
        button.add_css_class("theme-selector");
        button.add_css_class(["follow", "light", "dark"][mode]);
        button.set_hexpand(true);
        button.set_halign(gtk::Align::Center);
        button.set_focus_on_click(false);
        accessible_name(&button, label);
        button.set_tooltip_text(Some(label));
        if let Some(first) = buttons.first() {
            button.set_group(Some(first));
        }
        button.set_active(wordcraft_gnome::preferences::theme() == mode as u32);
        button.connect_toggled(move |b| {
            if b.is_active() && wordcraft_gnome::preferences::theme() != mode as u32 {
                if wordcraft_gnome::preferences::set_theme(mode as u32).is_ok() {
                    apply_theme(mode as u32);
                }
            }
        });
        row.append(&button);
        buttons.push(button);
    }
    let sync = |row: &gtk::Box| {
        if let Some(button) = find_widget(
            row,
            &format!("theme-{}", wordcraft_gnome::preferences::theme()),
        )
        .and_then(|w| w.downcast::<gtk::CheckButton>().ok())
        {
            button.set_active(true);
        }
    };
    row.connect_map(sync);
    let weak = row.downgrade();
    adw::StyleManager::default().connect_color_scheme_notify(move |_| {
        if let Some(row) = weak.upgrade() {
            sync(&row);
        }
    });
    row
}
pub(super) fn install_css() {
    if CSS_INSTALLED.with(|s| s.replace(true)) {
        return;
    }
    let theme_provider = gtk::CssProvider::new();
    theme_provider.load_from_string(include_str!("../../data/theme-selector.css"));
    let provider = gtk::CssProvider::new();
    // Adwaita suministra color, espaciados y estados; CSS mínimo para papel.
    provider.load_from_string("
      .leaf-ruler { background: white; color: #292929; border-radius: 2px; box-shadow: 0 0 0 1px alpha(black, 0.12); }
      .document-scroll { background: @window_bg_color; }
      .document-sheet { background: white; border-radius: 6px; box-shadow: 0 1px 4px alpha(black, 0.15); }
      .leaf-dark-document .document-sheet, .leaf-dark-document .thumbnail-sheet { background: #333333; }
      .thumbnail-sheet { background: white; border-radius: 6px; border: 1px solid alpha(black, 0.12); background-clip: content-box; }
      .thumbnail-button { padding: 8px 12px; min-width: 0; min-height: 0; }
      .thumbnail-button.selected-thumb { background: alpha(@window_fg_color, 0.09); }
      .sidebar-selector button { border-radius: 10px; }
      .comment-thread { background: alpha(@window_fg_color, 0.05); border-radius: 8px; }
      .comment-thread.selected { background: alpha(@window_fg_color, 0.09); }
      .comment-thread .comment-body { font-weight: normal; }
      .comment-thread separator.comment-divider { background: alpha(@window_fg_color, 0.16); min-height: 1px; }
      button.mixed { background: alpha(@accent_bg_color, 0.18); }
      .sidebar-selector button:checked { background: alpha(@window_fg_color, 0.12); }
      button.outline-expander { min-width: 18px; min-height: 24px; padding: 0; }
      .search-options > button { background: transparent; box-shadow: none; }
      button.search-replace { min-width: 24px; min-height: 24px; padding: 0; }
      .style-popover button, .format-popover button { font-weight: normal; }
      .bookmark-name { font-weight: normal; }
      .style-row.current-style, .format-popover button.current-style { background: alpha(@window_fg_color, 0.14); border-radius: 6px; }
      .format-popover button:checked:not(.split-action) { background: alpha(@window_fg_color, 0.14); }
      .page-indicator, .zoom-value { font-feature-settings: 'tnum'; }
    ");
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &theme_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
