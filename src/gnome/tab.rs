use super::*;

pub(super) fn build_tab(
    app: &Application,
    host: &Rc<Workspace>,
    sample_on_start: bool,
    sidebar_on_start: bool,
    narrow_on_start: bool,
    dark_on_start: bool,
    selection_demo: bool,
    editing_demo: bool,
    formatting_demo: bool,
    initial_document: Option<(Session, String)>,
) {
    let initial_loaded = initial_document.is_some();
    if dark_on_start {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    }
    install_css();
    let format_group = Rc::new(FormatPopoverGroup::default());
    let window = host.window.clone();
    // Composición oficial: panel adaptable y dos páginas de barras nativas.
    let side_header = HeaderBar::new();
    let side_title = gtk::Label::new(Some(wordcraft_gnome::t!("Pages")));
    side_title.set_widget_name("sidebar-title");
    side_title.add_css_class("heading");
    side_title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    side_title.set_max_width_chars(16);
    side_header.set_title_widget(Some(&side_title));
    let search_toggle = gtk::ToggleButton::new();
    accessible_name(&search_toggle, wordcraft_gnome::t!("Search Document"));
    search_toggle.set_icon_name("edit-find-symbolic");
    search_toggle.set_tooltip_text(Some(wordcraft_gnome::t!("Search Document (Ctrl+F)")));
    side_header.pack_start(&search_toggle);

    let open_button = gtk::Button::with_label(wordcraft_gnome::t!("Open…"));
    let app_menu = gtk::MenuButton::new();
    app_menu.set_icon_name("open-menu-symbolic");
    app_menu.set_tooltip_text(Some(wordcraft_gnome::t!("Main Menu")));
    side_header.pack_end(&app_menu);
    let header = HeaderBar::new();
    let title = WindowTitle::new("Leaf", "");
    header.set_title_widget(Some(&title));

    let sidebar_button = gtk::ToggleButton::new();
    sidebar_button.set_icon_name("sidebar-show-symbolic");
    sidebar_button.set_tooltip_text(Some(wordcraft_gnome::t!("Show or Hide Sidebar (F9)")));
    accessible_name(
        &sidebar_button,
        wordcraft_gnome::t!("Show or Hide Sidebar (F9)"),
    );
    header.pack_start(&sidebar_button);
    let compact_search = gtk::Button::from_icon_name("edit-find-symbolic");
    accessible_name(&compact_search, wordcraft_gnome::t!("Search Document"));
    compact_search.set_tooltip_text(Some(wordcraft_gnome::t!("Search (Ctrl+F)")));
    header.pack_start(&compact_search);
    let zoom_out = gtk::Button::with_label("−");
    zoom_out.set_tooltip_text(Some(wordcraft_gnome::t!("Zoom Out (Ctrl+-)")));
    let zoom_in = gtk::Button::with_label("+");
    zoom_in.set_tooltip_text(Some(wordcraft_gnome::t!("Zoom In (Ctrl++)")));
    let zoom_label = gtk::Label::new(Some("100 %"));
    zoom_label.add_css_class("zoom-value");
    let fit_button = gtk::Button::with_label(wordcraft_gnome::t!("Fit Width"));
    let normal_zoom = gtk::Button::with_label(wordcraft_gnome::t!("Actual Size (100%)"));
    // Si el panel lateral está cerrado, su menú de aplicación debe seguir
    // accesible desde la cabecera principal.
    let compact_app_menu = gtk::MenuButton::new();
    compact_app_menu.set_icon_name("open-menu-symbolic");
    compact_app_menu.set_tooltip_text(Some(wordcraft_gnome::t!("Main Menu")));
    header.pack_end(&compact_app_menu);

    let menu_button = gtk::MenuButton::new();
    menu_button.set_icon_name("view-more-symbolic");
    menu_button.set_tooltip_text(Some(wordcraft_gnome::t!("Document Options")));
    accessible_name(&menu_button, wordcraft_gnome::t!("Document Options"));
    header.pack_end(&menu_button);
    let top_history = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header.pack_start(&top_history);
    let top_formats = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header.pack_end(&top_formats);

    let stack = gtk::Stack::new();
    let empty = StatusPage::builder()
        .icon_name("x-office-document-symbolic")
        .title(wordcraft_gnome::t!("Open a Document"))
        .description(wordcraft_gnome::t!("Edit DOCX and ODT documents"))
        .build();
    stack.add_named(&empty, Some("empty"));

    let document_box = DocumentCanvas::new();
    document_box.set_valign(gtk::Align::Start);
    document_box.set_halign(gtk::Align::Center);
    document_box.set_focusable(true);
    document_box.set_cursor_from_name(Some("text"));
    document_box.set_has_tooltip(true);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.add_css_class("document-scroll");
    // The canvas is taller than the viewport. Restoring focus must not scroll
    // its whole allocation into view before pointer hit testing runs.
    let viewport = gtk::Viewport::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    viewport.set_scroll_to_focus(false);
    let document_content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let template_slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let template_revealer = gtk::Revealer::new();
    template_revealer.set_transition_type(gtk::RevealerTransitionType::SlideUp);
    template_revealer.set_transition_duration(180);
    template_slot.append(&template_revealer);
    template_slot.set_visible(false);
    document_content.append(&document_box);
    viewport.set_child(Some(&document_content));
    scroll.set_child(Some(&viewport));

    let thumbnails_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    thumbnails_box.set_valign(gtk::Align::Start);
    let thumbnails_scroll = gtk::ScrolledWindow::new();
    thumbnails_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    thumbnails_scroll.set_vexpand(true);
    let thumbnail_column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    thumbnail_column.append(&thumbnails_box);
    let empty_height = gtk::Box::new(gtk::Orientation::Vertical, 0);
    empty_height.set_vexpand(true);
    thumbnail_column.append(&empty_height);
    thumbnails_scroll.set_child(Some(&thumbnail_column));

    let sidebar_toolbar = ToolbarView::new();
    side_header.set_widget_name("sidebar-primary-header");
    // One bar container avoids ToolbarView's multi-bar collapse-spacing inset:
    // the sidebar selector remains visible when the document tab row disappears.
    let sidebar_bars = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar_bars.append(&side_header);
    sidebar_toolbar.add_top_bar(&sidebar_bars);
    let sidebar_pages = gtk::Stack::new();
    sidebar_pages.set_transition_type(gtk::StackTransitionType::Crossfade);
    sidebar_pages.set_transition_duration(180);
    let start_scroll = gtk::ScrolledWindow::new();
    start_scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    start_scroll.set_child(Some(&template_slot));
    sidebar_pages.add_named(&start_scroll, Some("start"));
    let navigation_pages = gtk::Stack::new();
    navigation_pages.set_vexpand(true);
    navigation_pages.add_named(&thumbnails_scroll, Some("thumbnails"));
    let navigation = ToolbarView::new();
    navigation.set_content(Some(&navigation_pages));
    navigation.set_bottom_bar_style(adw::ToolbarStyle::Flat);
    let selector = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    selector.add_css_class("sidebar-selector");
    selector.set_margin_start(6);
    selector.set_margin_end(6);
    selector.set_margin_top(5);
    selector.set_margin_bottom(5);
    let pages_toggle = gtk::ToggleButton::with_label(wordcraft_gnome::t!("Pages"));
    accessible_name(&pages_toggle, wordcraft_gnome::t!("Pages"));
    // Pages is the internal, labeled Navigation representation selector.
    pages_toggle.set_tooltip_text(Some(wordcraft_gnome::t!("Pages")));
    pages_toggle.set_hexpand(true);

    pages_toggle.set_widget_name("sidebar-pages");
    let titles_toggle = gtk::ToggleButton::with_label(wordcraft_gnome::t!("Structure"));
    accessible_name(&titles_toggle, wordcraft_gnome::t!("Structure"));
    titles_toggle.set_tooltip_text(Some(wordcraft_gnome::t!("Structure")));
    titles_toggle.set_hexpand(true);

    titles_toggle.set_widget_name("sidebar-outline");
    titles_toggle.set_group(Some(&pages_toggle));
    let navigation_toggle = gtk::ToggleButton::new();
    navigation_toggle.set_widget_name("sidebar-navigation");
    navigation_toggle.set_icon_name("compass-symbolic");
    navigation_toggle.set_hexpand(true);
    navigation_toggle.add_css_class("flat");
    navigation_toggle.set_tooltip_text(Some(wordcraft_gnome::t!("Navigation")));
    accessible_name(&navigation_toggle, wordcraft_gnome::t!("Navigation"));
    navigation_toggle.set_active(true);
    let navigation_selector = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    navigation_selector.set_widget_name("navigation-representation");
    navigation_selector.set_margin_start(6);
    navigation_selector.set_margin_end(6);
    navigation_selector.set_margin_top(6);
    navigation_selector.set_margin_bottom(6);
    // The old toggle state remains private to the existing outline controller;
    // only this native dropdown is presented, including keyboard/a11y access.
    let representation = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("Pages"),
        wordcraft_gnome::t!("Structure"),
    ]);
    representation.set_widget_name("navigation-representation-choice");
    representation.set_hexpand(true);
    accessible_name(&representation, wordcraft_gnome::t!("Navigation"));
    navigation_selector.append(&representation);
    representation.add_css_class("sidebar-choice");
    representation.add_css_class("flat");
    let pages = pages_toggle.clone();
    let titles = titles_toggle.clone();
    representation.connect_selected_notify(move |d| {
        if d.selected() == 0 {
            pages.set_active(true);
        } else {
            titles.set_active(true);
        }
    });
    let annotations_toggle = gtk::ToggleButton::new();
    accessible_name(&annotations_toggle, wordcraft_gnome::t!("Review"));
    annotations_toggle.set_icon_name("document-edit-symbolic");
    annotations_toggle.set_tooltip_text(Some(wordcraft_gnome::t!("Review")));
    annotations_toggle.set_hexpand(true);
    annotations_toggle.add_css_class("flat");
    annotations_toggle.set_widget_name("sidebar-review");
    annotations_toggle.set_group(Some(&navigation_toggle));
    pages_toggle.set_active(true);
    selector.append(&navigation_toggle);
    selector.append(&annotations_toggle);
    let selector_bar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    selector_bar.add_css_class("toolbar");
    selector_bar.append(&selector);
    thumbnail_column.prepend(&navigation_selector);
    // Mode tabs belong to the sidebar, not to any representation's scroll.
    // Keep the same controls reachable in Pages, Structure, Review and Search.
    sidebar_toolbar.set_bottom_bar_style(adw::ToolbarStyle::Flat);
    sidebar_toolbar.add_bottom_bar(&selector_bar);
    sidebar_pages.add_named(&navigation, Some("navigation"));
    sidebar_pages.set_visible_child_name("navigation");
    sidebar_toolbar.set_content(Some(&sidebar_pages));

    let canvas_overlay = gtk::Overlay::new();
    canvas_overlay.set_child(Some(&scroll));
    let editor = gtk::Box::new(gtk::Orientation::Vertical, 0);
    canvas_overlay.set_vexpand(true);
    editor.append(&canvas_overlay);
    stack.add_named(&editor, Some("pages"));
    stack.set_visible_child_name("empty");

    let toasts = ToastOverlay::new();
    toasts.set_child(Some(&stack));
    // A bottom native toolbar for document editing, horizontally scrollable
    // on small windows. It lives in AdwToolbarView, never over the pages.
    let format_bar = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    format_bar.set_margin_start(10);
    format_bar.set_margin_end(10);
    format_bar.set_margin_top(5);
    format_bar.set_margin_bottom(5);
    let (undo_split, undo_button, _) = history_button(false);
    let (redo_split, redo_button, _) = history_button(true);
    undo_button.update_property(&[gtk::accessible::Property::Label(wordcraft_gnome::t!(
        "Undo"
    ))]);
    redo_button.update_property(&[gtk::accessible::Property::Label(wordcraft_gnome::t!(
        "Redo"
    ))]);
    undo_button.set_tooltip_text(Some(wordcraft_gnome::t!("Undo (Ctrl+Z)")));
    redo_button.set_tooltip_text(Some(wordcraft_gnome::t!("Redo (Ctrl+Shift+Z)")));
    undo_button.set_sensitive(false);
    redo_button.set_sensitive(false);
    let history_tools = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    history_tools.append(&undo_split);
    history_tools.append(&redo_split);
    format_bar.append(&history_tools);
    let history_separator = gtk::Separator::new(gtk::Orientation::Vertical);
    format_bar.append(&history_separator);
    let format_tools = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    format_bar.append(&format_tools);
    let bold_button = gtk::CheckButton::with_label(wordcraft_gnome::t!("Bold"));
    bold_button.set_tooltip_text(Some(wordcraft_gnome::t!("Bold (Ctrl+B)")));
    bold_button.add_css_class("flat");
    let italic_button = gtk::CheckButton::with_label(wordcraft_gnome::t!("Italic"));
    italic_button.set_tooltip_text(Some(wordcraft_gnome::t!("Italic (Ctrl+I)")));
    italic_button.add_css_class("flat");
    let text_popover = gtk::Popover::new();
    text_popover.add_css_class("format-popover");
    let text_options = gtk::Box::new(gtk::Orientation::Vertical, 12);
    text_options.set_margin_top(12);
    text_options.set_margin_bottom(12);
    text_options.set_margin_start(12);
    text_options.set_margin_end(12);
    let inline_buttons = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    inline_buttons.append(&bold_button);
    inline_buttons.append(&italic_button);
    let text_header = format_popover_header(
        &text_popover,
        &text_options,
        wordcraft_gnome::t!("Text"),
        &format_group,
    );
    text_options.append(&inline_buttons);
    let selectors = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let font_input = gtk::Entry::new();
    font_input.set_widget_name("text-font");
    font_input.set_placeholder_text(Some(wordcraft_gnome::t!("Font")));
    font_input.set_width_chars(12);
    font_input.set_hexpand(true);
    accessible_name(&font_input, wordcraft_gnome::t!("Font"));
    let (font_selector, font_choices_menu, font_choices) =
        text_selector(&font_input, "text-font-choices");
    selectors.append(&font_selector);
    let size_input = gtk::SpinButton::with_range(1.0, 1638.0, 0.5);
    size_input.set_hexpand(false);
    size_input.set_digits(2);
    size_input.set_value(12.0);
    size_input.set_width_chars(7);
    size_input.set_widget_name("text-size");
    size_input.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
    accessible_name(&size_input, wordcraft_gnome::t!("Font Size"));
    let (size_selector, size_choices_menu, size_choices) =
        text_selector(&size_input, "text-size-choices");
    size_selector.set_hexpand(false);
    size_selector.set_halign(gtk::Align::Start);
    selectors.append(&size_selector);
    text_options.append(&selectors);
    let more_text = style_menu_button(wordcraft_gnome::t!("More Text Options…"));
    let language_button = gtk::Button::with_label(wordcraft_gnome::t!("Language"));
    language_button.set_widget_name("text-language");
    text_options.append(&language_button);
    text_options.append(&more_text);
    text_popover.set_child(Some(&text_options));
    let text_menu = gtk::MenuButton::new();
    format_menu_label(&text_menu, wordcraft_gnome::t!("Text"));
    text_menu.set_tooltip_text(Some(wordcraft_gnome::t!(
        "Bold, Italic, Underline and Font Size"
    )));
    text_menu.set_popover(Some(&text_popover));
    text_menu.set_direction(gtk::ArrowType::Up);

    let format_actions: Vec<(gtk::Button, Format, gtk::Popover)> = Vec::new();
    let style_menu = gtk::MenuButton::new();
    format_menu_label(&style_menu, wordcraft_gnome::t!("Style"));
    style_menu.set_direction(gtk::ArrowType::Up);
    style_menu.set_tooltip_text(Some(wordcraft_gnome::t!("Apply and Manage Styles")));
    let align_menu = gtk::MenuButton::new();
    format_menu_label(&align_menu, wordcraft_gnome::t!("Paragraph"));
    align_menu.set_direction(gtk::ArrowType::Up);
    align_menu.set_tooltip_text(Some(wordcraft_gnome::t!("Paragraph Formatting")));
    format_tools.append(&style_menu);
    format_tools.append(&align_menu);
    format_tools.append(&text_menu);
    let format_toolbar = gtk::ScrolledWindow::new();
    format_toolbar.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    format_toolbar.set_child(Some(&format_bar));
    format_toolbar.add_css_class("toolbar");
    format_toolbar.set_visible(false);

    let document_toolbar = ToolbarView::new();
    document_toolbar.set_bottom_bar_style(adw::ToolbarStyle::Flat);
    header.set_widget_name("document-primary-header");
    let document_bars = gtk::Box::new(gtk::Orientation::Vertical, 0);
    document_bars.append(&header);
    document_toolbar.add_top_bar(&document_bars);
    let tab_bar = adw::TabBar::new();
    tab_bar.set_view(Some(&host.tabs));
    tab_bar.set_autohide(true);
    let tab_row = gtk::Box::new(gtk::Orientation::Vertical, 0);
    tab_row.set_widget_name("document-header-secondary");
    tab_row.append(&tab_bar);
    document_bars.append(&tab_row);
    let header_heights = gtk::SizeGroup::new(gtk::SizeGroupMode::Vertical);
    header_heights.add_widget(&side_header);
    header_heights.add_widget(&header);
    // The tab row follows TabBar autohide. Sidebar controls belong to each
    // scrolling page, so no secondary header reserves their previous height.
    tab_row.set_visible(tab_bar.is_tabs_revealed());
    let weak_row = tab_row.downgrade();
    tab_bar.connect_tabs_revealed_notify(move |bar| {
        if let Some(row) = weak_row.upgrade() {
            row.set_visible(bar.is_tabs_revealed());
        }
    });
    document_toolbar.add_bottom_bar(&format_toolbar);
    document_toolbar.set_content(Some(&toasts));

    let sidebar = adw::OverlaySplitView::new();
    sidebar.set_sidebar(Some(&sidebar_toolbar));
    sidebar.set_content(Some(&document_toolbar));
    sidebar.set_collapsed(narrow_on_start);
    sidebar.set_show_sidebar(sidebar_on_start || !narrow_on_start);
    sidebar_button.set_active(sidebar_on_start || !narrow_on_start);
    compact_app_menu.set_visible(!(sidebar_on_start || !narrow_on_start));
    compact_search.set_visible(!(sidebar_on_start || !narrow_on_start));
    sidebar.set_min_sidebar_width(190.0);
    sidebar.set_max_sidebar_width(240.0);
    sidebar.set_sidebar_width_fraction(0.21);
    sidebar.set_pin_sidebar(true);
    let tab_page = host.tabs.append(&sidebar);
    host.tabs.set_selected_page(&tab_page);

    // En estrecho la barra lateral se superpone sobre la página.
    let narrow = adw::Breakpoint::new(
        adw::BreakpointCondition::parse("max-width: 760sp").expect("condición de ancho válida"),
    );
    narrow.add_setter(&sidebar, "collapsed", Some(&true.to_value()));
    window.add_breakpoint(narrow);
    window.set_size_request(360, 420);

    let ui = Rc::new(Ui {
        table_pointer: RefCell::new(TablePointer::default()),
        picture_drag: RefCell::new(None),
        image_drag_active: Cell::new(false),
        image_drop: RefCell::new(None),
        image_read: RefCell::new(None),
        reference_hover: RefCell::new(None),
        caption_hover: RefCell::new(None),
        caption_cues: RefCell::new(None),
        current: RefCell::new(None),
        input_context: gtk::IMMulticontext::new(),
        selection_rasters: RefCell::new(Vec::new()),
        printing: PrintState::default(),
        selection_data: Rc::new(RefCell::new(SelectionGeometry::default())),
        selection_zoom: Rc::new(Cell::new(1.0)),
        caret_navigation_generation: Cell::new(0),
        caret_motion: Rc::new(RefCell::new(Default::default())),
        caret_tick_active: Cell::new(false),
        drag_scroll: RefCell::new(None),
        drag_point: Cell::new(None),
        word_drag: RefCell::new(None),
        paragraph_drag: RefCell::new(None),
        drag_anchor: RefCell::new(None),
        pointer_moved: Cell::new(false),
        dirty: Cell::new(false),
        thumbnail_refresh_pending: Cell::new(false),
        thumbnail_bounds_pending: Cell::new(false),
        closing_confirmed: Cell::new(false),
        tab_page: RefCell::new(Some(tab_page.clone())),
        closed: Cell::new(false),
        derived_source: RefCell::new(None),
        toc_cache: RefCell::new(TocCache::default()),
        clipboard_read: RefCell::new(None),
        autosave_source: RefCell::new(None),
        theme_signal: RefCell::new(None),
        restoring: Cell::new(false),
        autosave_error: RefCell::new(None),
        pending_tab_close: RefCell::new(None),
        save_path: RefCell::new(None),
        source_path: RefCell::new(None),
        spelling: RefCell::new(None),
        format_toolbar,
        adaptive: RefCell::new(None),
        quick_tools: RefCell::new(None),
        format_buttons: RefCell::new(Vec::new()),
        style: RefCell::new(None),
        text_tools: RefCell::new(None),
        format_reflowing: Cell::new(false),
        pages_button: pages_toggle.clone(),
        sidebar_selector: selector_bar.clone(),
        _header_heights: header_heights,
        format_bold: bold_button.clone(),
        format_italic: italic_button.clone(),
        format_syncing: Cell::new(false),
        ruler: RefCell::new(None),
        history_controls: RefCell::new(None),
        repeat_action: RefCell::new(None),
        document_menu_open: Cell::new(false),
        document_menu: RefCell::new(None),
        undo_button: undo_button.clone(),
        redo_button: redo_button.clone(),
        font_input: font_input.clone(),
        text_controls: RefCell::new(None),
        format_sequence: RefCell::new(None),
        size_input: size_input.clone(),
        document_box,
        template_slot,
        template_revealer,
        template_eligible: Cell::new(false),
        template_document: Cell::new(None),
        template_gallery: RefCell::new(None),
        thumbnails_box,
        scroll,
        thumbnails_scroll,
        sidebar,
        sidebar_button,
        sidebar_title: side_title,
        sidebar_pages,
        navigation_pages,
        navigation_button: navigation_toggle.clone(),
        navigation_selector,
        representation,
        last_navigation: Cell::new("thumbnails"),
        navigation_restore_generation: Cell::new(0),
        hidden_navigation_scroll: RefCell::new(None),
        thumbnail_reveal_pending: Cell::new(true),
        outline: RefCell::new(None),
        annotations: RefCell::new(None),
        elements: RefCell::new(None),
        review: RefCell::new(None),
        annotation_dialog: RefCell::new(None),
        link_dialog: RefCell::new(None),
        dictionary_tools: RefCell::new(None),
        dictionary_chooser: RefCell::new(None),
        host: Rc::downgrade(host),
        open_button: open_button.clone(),
        search: RefCell::new(None),
        paragraph: RefCell::new(None),
        picture: RefCell::new(None),
        table_popover: RefCell::new(None),
        context_popovers: RefCell::new(Vec::new()),
        outside_controller: RefCell::new(None),
        tab_bar: tab_bar.clone(),
        format_group: format_group.clone(),
        search_highlights: Rc::new(RefCell::new(Vec::new())),
        document_name: RefCell::new(String::new()),
        layout_pending: Cell::new(false),
        viewport_width: Cell::new(0),
        zoom_label,
        zoom_out,
        zoom_in,
        stack,
        title,
        toasts,
    });
    for stack in [&ui.sidebar_pages, &ui.navigation_pages] {
        let weak = Rc::downgrade(&ui);
        stack.connect_visible_child_name_notify(move |_| {
            if let Some(ui) = weak.upgrade() {
                refresh_sidebar_title(&ui);
            }
        });
    }
    install_navigation_switching(&ui);
    let weak = Rc::downgrade(&ui);
    ui.sidebar_pages
        .connect_transition_running_notify(move |_| {
            if let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) {
                templates::finish_transition(&ui);
            }
        });
    refresh_sidebar_title(&ui);
    ui.history_controls
        .replace(Some(install_history_controls(&ui, &history_tools)));
    ui.ruler.replace(Some(install_ruler(&ui, &canvas_overlay)));
    install_document_menu(&menu_button, &ui);

    {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.set_margin_top(6);
        row.set_margin_bottom(6);
        row.set_widget_name("zoom-controls");
        for b in [&ui.zoom_out, &normal_zoom, &fit_button, &ui.zoom_in] {
            b.add_css_class("flat");
        }
        ui.zoom_out
            .set_child(Some(&gtk::Image::from_icon_name("list-remove-symbolic")));
        ui.zoom_in
            .set_child(Some(&gtk::Image::from_icon_name("list-add-symbolic")));
        row.set_halign(gtk::Align::Center);
        ui.zoom_out.set_widget_name("zoom-out");
        ui.zoom_in.set_widget_name("zoom-in");
        normal_zoom.set_widget_name("zoom-reset");
        normal_zoom.set_child(Some(&ui.zoom_label));
        accessible_name(&normal_zoom, wordcraft_gnome::t!("Actual Size (100%)"));
        normal_zoom.set_tooltip_text(Some(wordcraft_gnome::t!("Actual Size (100%)")));
        fit_button.set_widget_name("zoom-fit-width");
        fit_button.set_child(Some(&gtk::Image::from_icon_name("zoom-fit-best-symbolic")));
        accessible_name(&fit_button, wordcraft_gnome::t!("Fit Width"));
        fit_button.set_tooltip_text(Some(wordcraft_gnome::t!("Fit Width")));
        row.append(&ui.zoom_out);
        row.append(&normal_zoom);
        row.append(&fit_button);
        row.append(&ui.zoom_in);
        if let Some(pop) = menu_button
            .popover()
            .and_then(|p| p.downcast::<gtk::PopoverMenu>().ok())
        {
            pop.add_child(&row, "zoom-controls");
        }
    }
    install_accessibility(&ui, &app_menu, &compact_app_menu);
    let text_tools = install_text_pages(
        &ui,
        &text_popover,
        &text_options,
        &inline_buttons,
        &text_header,
        &more_text,
    );
    ui.text_tools.replace(Some(text_tools));
    let paragraph_popover = install_paragraph_popover(&ui, &align_menu);
    ui.paragraph.replace(Some(paragraph_popover.clone()));
    install_motion_check(&ui, app);
    {
        let g = Rc::downgrade(&format_group);
        let w = window.downgrade();
        let outside = gtk::GestureClick::new();
        outside.set_propagation_phase(gtk::PropagationPhase::Capture);
        outside.set_button(0);
        outside.connect_pressed(move |c, _, x, y| {
            if let (Some(g), Some(w)) = (g.upgrade(), w.upgrade()) {
                g.outside(c, &w, x, y);
            }
        });
        window.add_controller(outside.clone());
        ui.outside_controller.replace(Some(outside));
    }
    let outline_panel = install_outline(&ui, &pages_toggle, &titles_toggle);
    ui.outline.replace(Some(outline_panel.clone()));
    ui.annotations
        .replace(Some(install_annotations(&ui, &annotations_toggle)));
    ui.spelling
        .replace(Some(install_spelling(&ui, &annotations_toggle)));
    ui.review
        .replace(Some(install_review(&ui, &annotations_toggle)));
    let search_panel = install_search_panel(&ui, &search_toggle, &compact_search);
    ui.search.replace(Some(search_panel.clone()));
    style_menu.set_widget_name("style-menu");
    align_menu.set_widget_name("paragraph-menu");
    text_menu.set_widget_name("text-menu");
    let style_popover = install_style_popover(&ui, &style_menu);
    ui.style.replace(Some(style_popover.clone()));
    ui.format_buttons.replace(vec![
        style_menu.clone(),
        align_menu.clone(),
        text_menu.clone(),
    ]);
    let adaptive = Rc::new(AdaptiveTools {
        ui: Rc::downgrade(&ui),
        history: history_tools,
        formats: format_tools,
        top_history,
        top_formats,
        bottom: format_bar.clone(),
        separator: history_separator,
        narrow: Cell::new(true),
        width_band: Cell::new(u8::MAX),
        generation: Cell::new(0),
        pending: RefCell::new(Vec::new()),
    });
    for (menu, label, kind) in [
        (&style_menu, wordcraft_gnome::t!("Style"), "style"),
        (&align_menu, wordcraft_gnome::t!("Paragraph"), "paragraph"),
        (&text_menu, wordcraft_gnome::t!("Text"), "text"),
    ] {
        format_icon_button(menu, label, kind);
    }
    ui.adaptive.replace(Some(adaptive.clone()));
    adaptive.sync_width(if narrow_on_start { 420 } else { 900 });
    let table_menu = install_table_controls(&ui, &window, &format_bar);
    let table_pop = table_menu.popover().unwrap();
    ui.table_popover.replace(Some(table_pop.clone()));
    if !tool_diagnostics() {
        table_menu.set_popover(None::<&gtk::Popover>);
        format_bar.remove(&table_menu);
        table_pop.set_parent(&ui.document_box);
    }
    let (picture_menu, picture_panel) = install_picture_controls(&ui, &window, &format_bar);
    picture_menu.set_widget_name("picture-menu");
    ui.picture.replace(Some(picture_panel.clone()));
    if !tool_diagnostics() {
        picture_menu.set_popover(None::<&gtk::Popover>);
        format_bar.remove(&picture_menu);
        picture_panel.pop.set_parent(&ui.document_box);
    }
    let elements = install_elements(&ui, &adaptive.formats);
    ui.format_buttons.borrow_mut().push(elements.menu.clone());
    ui.quick_tools.replace(Some(format_bar::install_quick_tools(
        &ui,
        &adaptive.formats,
        &style_menu,
        &align_menu,
        &text_menu,
        &elements,
    )));
    ui.elements.replace(Some(elements));
    adaptive.sync_width(if narrow_on_start { 420 } else { 900 });
    host.docs.borrow_mut().push((tab_page, ui.clone()));
    install_main_menu(&app_menu);
    install_main_menu(&compact_app_menu);
    refresh_main_actions(host);
    install_autosave(&ui);
    {
        let weak = Rc::downgrade(&ui);
        let manager = adw::StyleManager::default();
        let id = manager.connect_dark_notify(move |_| {
            if let Some(ui) = weak.upgrade() {
                if !ui.closed.get() {
                    refresh_document_appearance(&ui);
                }
            }
        });
        ui.theme_signal.replace(Some(id));
        refresh_document_appearance(&ui);
    }
    for (button, edit) in [(undo_button, Edit::Undo), (redo_button, Edit::Redo)] {
        let ui = ui.clone();
        button.connect_clicked(glib::clone!(
            #[weak]
            ui,
            move |_| {
                if edit == Edit::Redo {
                    redo_or_repeat(&ui);
                } else {
                    apply_user_edit(&ui, edit.clone());
                }
                ui.document_box.grab_focus();
            }
        ));
    }
    ui.text_controls.replace(Some(install_text_controls(
        &ui,
        font_choices_menu,
        size_choices_menu,
        font_choices,
        size_choices,
    )));
    // All formatting commands preserve the selection and return keyboard
    // focus to the editable document.
    for (button, action) in [(bold_button, Format::Bold), (italic_button, Format::Italic)] {
        let ui = ui.clone();
        let popover = text_popover.clone();
        button.connect_toggled(glib::clone!(
            #[weak]
            ui,
            #[weak]
            popover,
            move |button| {
                if !ui.format_syncing.get() {
                    let on = button.is_active() || button.is_inconsistent();
                    button.set_inconsistent(false);
                    close_format_popover(&popover);
                    let command = match action {
                        Format::Bold => "format.bold",
                        Format::Italic => "format.italic",
                        _ => "format.underline",
                    };
                    if let Err(e) = format_execute(&ui, command, serde_json::json!({"value":on})) {
                        ui.toasts.add_toast(Toast::new(&e));
                    }
                }
            }
        ));
    }
    for (button, action, popover) in format_actions {
        let ui = ui.clone();
        button.connect_clicked(glib::clone!(
            #[weak]
            ui,
            #[weak]
            popover,
            move |_| {
                close_format_popover(&popover);
                apply_user_format(&ui, action);
            }
        ));
    }

    install_pointer_and_view_input(&ui, &compact_app_menu, &compact_search);
    {
        let ui = ui.clone();
        fit_button.connect_clicked(glib::clone!(
            #[weak]
            ui,
            move |_| {
                fit_width(&ui);
            }
        ));
    }
    {
        let ui = ui.clone();
        normal_zoom.connect_clicked(glib::clone!(
            #[weak]
            ui,
            move |_| {
                if let Some(doc) = ui.current.borrow_mut().as_mut() {
                    doc.fit_to_width = false;
                }
                zoom_to(&ui, 1.0);
            }
        ));
    }
    install_keyboard_input(&ui, &window, host, app, &open_button);
    {
        let ui = ui.clone();
        let parent = window.clone();
        let app = app.clone();
        let host = host.clone();
        open_button.connect_clicked(glib::clone!(
            #[weak]
            ui,
            #[weak]
            parent,
            #[weak]
            app,
            #[weak]
            host,
            move |_| {
                let dialog = gtk::FileDialog::new();
                dialog.set_title(wordcraft_gnome::t!("Open Document"));
                let filter = gtk::FileFilter::new();
                filter.set_name(Some(wordcraft_gnome::t!("DOCX and ODT Documents")));
                filter.add_pattern("*.docx");
                filter.add_pattern("*.odt");
                let filters = gio::ListStore::new::<gtk::FileFilter>();
                filters.append(&filter);
                dialog.set_filters(Some(&filters));
                let ui = ui.clone();
                let app = app.clone();
                let host = host.clone();
                dialog.open(
                    Some(&parent),
                    None::<&gio::Cancellable>,
                    glib::clone!(
                        #[weak]
                        ui,
                        #[weak]
                        host,
                        #[weak]
                        app,
                        move |result| {
                            if ui.closed.get() {
                                return;
                            }
                            let Ok(file) = result else { return };
                            let Some(path) = file.path() else {
                                ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                                    "Only local files are supported"
                                )));
                                return;
                            };
                            match open_document(Path::new(&path)) {
                                Ok(session) => {
                                    let label = path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().into_owned())
                                        .unwrap_or_else(|| {
                                            wordcraft_gnome::t!("Document").to_string()
                                        });
                                    if ui
                                        .current
                                        .borrow()
                                        .as_ref()
                                        .is_none_or(|doc| doc.session.is_untouched_blank())
                                    {
                                        opened(&ui, session, &label);
                                    } else {
                                        build_tab(
                                            &app,
                                            &host,
                                            false,
                                            true,
                                            false,
                                            false,
                                            false,
                                            false,
                                            false,
                                            Some((session, label)),
                                        );
                                    }
                                }
                                Err(err) => ui.toasts.add_toast(Toast::new(&err)),
                            }
                        }
                    ),
                );
            }
        ));
    }

    {
        let weak = Rc::downgrade(&ui);
        ui.template_slot.connect_map(move |_| {
            if let Some(ui) = weak.upgrade() {
                templates::refresh_if_visible(&ui);
            }
        });
        let weak = Rc::downgrade(&ui);
        window.connect_is_active_notify(move |window| {
            if window.is_active() {
                if let Some(ui) = weak.upgrade() {
                    templates::refresh_if_visible(&ui);
                }
            }
        });
    }
    if let Some((session, label)) = initial_document {
        opened(&ui, session, &label);
    } else if sample_on_start {
        opened(&ui, sample_session(), "Prueba de interfaz");
    } else {
        opened(
            &ui,
            wordcraft_gnome::proofing_defaults::new_document(),
            wordcraft_gnome::t!("New Document"),
        );
        templates::offer(&ui);
    }
    {
        let weak = Rc::downgrade(&ui);
        ui.document_box.connect_scale_factor_notify(move |_| {
            let weak = weak.clone();
            glib::idle_add_local_once(move || {
                if let Some(ui) = weak.upgrade() {
                    if !ui.closed.get() {
                        refresh_pages(&ui);
                        refresh_thumbnails(&ui);
                    }
                }
            });
        });
    }
    window.present();
    install_table_context_menu(&ui, &window);
    image_drop::install(&ui);
    install_diagnostics(
        &ui,
        &window,
        host,
        app,
        DiagnosticControls {
            app_menu: &app_menu,
            style_menu: &style_menu,
            align_menu: &align_menu,
            text_menu: &text_menu,
            table_menu: &table_menu,
            menu_button: &menu_button,
            picture_menu: &picture_menu,
            style_popover: &style_popover,
            paragraph_popover: &paragraph_popover,
            text_popover: &text_popover,
            picture_panel: &picture_panel,
            outline_panel: &outline_panel,
            titles_toggle: &titles_toggle,
            search_panel: &search_panel,
        },
    );
    install_menu_demos(&text_menu, &menu_button);
    if sample_on_start || initial_loaded {
        let ui = ui.clone();
        glib::idle_add_local_once(move || fit_width(&ui));
    }
    install_content_demos(&ui, formatting_demo, editing_demo, selection_demo);
    capture_ready::install(&ui, &window);
}
