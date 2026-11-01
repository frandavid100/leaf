use super::*;

thread_local! {static CSS_INSTALLED:Cell<bool>=const{Cell::new(false)};}
pub(super) fn install_css() {
    if CSS_INSTALLED.with(|s| s.replace(true)) {
        return;
    }
    let provider = gtk::CssProvider::new();
    // Adwaita suministra color, espaciados y estados; CSS mínimo para papel.
    provider.load_from_data("
      .document-scroll { background: @window_bg_color; }
      .document-sheet { background: white; box-shadow: 0 1px 4px alpha(black, 0.15); }
      .leaf-dark-document .document-sheet, .leaf-dark-document .thumbnail-sheet { background: #333333; }
      .thumbnail-sheet { background: white; border-radius: 6px; border: 1px solid alpha(black, 0.12); background-clip: content-box; }
      .thumbnail-button { padding: 12px; min-width: 0; min-height: 0; }
      .thumbnail-button.selected-thumb { background: alpha(@window_fg_color, 0.09); }
      .sidebar-selector button { border-radius: 10px; }
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
