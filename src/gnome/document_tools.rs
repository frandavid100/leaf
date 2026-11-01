use super::*;

// Native document tools; engine adapters own all mutations.
use wordcraft_gnome::{
    document_properties::{DocumentInfo, PageSetup},
    pdf_export, pictures,
};
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
pub(super) struct PicturePopover {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) pop: gtk::Popover,
    pub(super) context: gtk::Box,
    pub(super) hint: gtk::Label,
    pub(super) width: gtk::SpinButton,
    pub(super) height: gtk::SpinButton,
    pub(super) lock: gtk::CheckButton,
    pub(super) syncing: Cell<bool>,
    pub(super) width_changed: Cell<bool>,
    pub(super) target: RefCell<Option<(Pos, f32, f32)>>,
}
impl PicturePopover {
    pub(super) fn refresh(&self) {
        let target = self.ui.upgrade().and_then(|ui| {
            ui.current
                .borrow()
                .as_ref()
                .and_then(|d| pictures::selected(&d.session))
        });
        self.context.set_visible(target.is_some());
        self.hint.set_visible(target.is_none());
        if *self.target.borrow() == target {
            return;
        }
        self.syncing.set(true);
        if let Some((_, w, h)) = &target {
            self.width.set_value(*w as f64 / PT_PER_MM);
            self.height.set_value(*h as f64 / PT_PER_MM);
        }
        self.syncing.set(false);
        self.target.replace(target);
    }
    pub(super) fn execute(&self, f: impl FnOnce(&mut Session) -> Result<(), String>) {
        if let Some(ui) = self.ui.upgrade() {
            if tools_change(&ui, f) {
                self.refresh();
                close_format_popover(&self.pop);
                ui.document_box.grab_focus();
                caret_ensure_visible(&ui);
            }
        }
    }
}
pub(super) fn choose_picture(ui: &Rc<Ui>, parent: &ApplicationWindow, replace: bool) {
    let identity = document_identity(ui);
    let target = ui
        .current
        .borrow()
        .as_ref()
        .and_then(|d| pictures::selected(&d.session));
    let dialog = gtk::FileDialog::new();
    dialog.set_modal(true);
    dialog.set_title(if replace {
        wordcraft_gnome::t!("Replace Image")
    } else {
        wordcraft_gnome::t!("Insert Image")
    });
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(wordcraft_gnome::t!(
        "Images (PNG, JPEG, GIF, WebP, BMP, TIFF)"
    )));
    for mime in [
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
        "image/bmp",
        "image/tiff",
    ] {
        filter.add_mime_type(mime);
    }
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));
    let ui = ui.clone();
    dialog.open(
        Some(parent),
        None::<&gio::Cancellable>,
        glib::clone!(
            #[weak]
            ui,
            move |result| match result {
                Ok(file) => {
                    if ui.closed.get() || document_identity(&ui) != identity {
                        ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                            "The document has changed; choose the image again"
                        )));
                        return;
                    }
                    let Some(path) = file.path() else {
                        ui.toasts
                            .add_toast(Toast::new(wordcraft_gnome::t!("Choose a Local File")));
                        return;
                    };
                    if replace
                        && ui
                            .current
                            .borrow()
                            .as_ref()
                            .and_then(|d| pictures::selected(&d.session))
                            != target
                    {
                        return;
                    }
                    if tools_change(&ui, |s| {
                        if replace {
                            pictures::replace(s, &path)
                        } else {
                            pictures::insert(s, &path)
                        }
                    }) {
                        if let Some(p) = ui.picture.borrow().as_ref() {
                            p.refresh();
                            close_format_popover(&p.pop);
                        }
                        ui.document_box.grab_focus();
                        caret_ensure_visible(&ui);
                    }
                }
                Err(e)
                    if e.matches(gtk::DialogError::Cancelled)
                        || e.matches(gtk::DialogError::Dismissed) => {}
                Err(e) => ui.toasts.add_toast(Toast::new(&e.to_string())),
            }
        ),
    );
}
pub(super) fn install_picture_controls(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
    bar: &gtk::Box,
) -> (gtk::MenuButton, Rc<PicturePopover>) {
    let menu = gtk::MenuButton::new();
    format_menu_label(&menu, wordcraft_gnome::t!("Image"));
    menu.set_direction(gtk::ArrowType::Up);
    bar.append(&menu);
    let pop = gtk::Popover::new();
    pop.add_css_class("format-popover");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.set_width_request(300);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);
    format_popover_header(&pop, &root, wordcraft_gnome::t!("Image"), &ui.format_group);
    let insert = style_menu_button(wordcraft_gnome::t!("Insert Image…"));
    insert.set_widget_name("picture-insert");
    insert.set_visible(tool_diagnostics());
    root.append(&insert);
    let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
        "Select an image in the document to edit it."
    )));
    hint.set_wrap(true);
    hint.set_max_width_chars(30);
    hint.add_css_class("dim-label");
    root.append(&hint);
    let context = gtk::Box::new(gtk::Orientation::Vertical, 8);
    context.set_widget_name("picture-context");
    root.append(&context);
    context.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let width = mm_input(30.0, 1.0 / PT_PER_MM, 4000.0 / PT_PER_MM);
    width.set_widget_name("picture-width");
    let height = mm_input(15.0, 1.0 / PT_PER_MM, 4000.0 / PT_PER_MM);
    height.set_widget_name("picture-height");
    for (label, n) in [
        (wordcraft_gnome::t!("Width (mm)"), &width),
        (wordcraft_gnome::t!("Height (mm)"), &height),
    ] {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let l = gtk::Label::new(Some(label));
        l.set_xalign(0.0);
        l.set_hexpand(true);
        row.append(&l);
        row.append(n);
        context.append(&row);
    }
    let lock = gtk::CheckButton::with_label(wordcraft_gnome::t!("Keep Aspect Ratio"));
    lock.set_active(true);
    context.append(&lock);
    let apply = style_menu_button(wordcraft_gnome::t!("Apply Size"));
    apply.set_widget_name("picture-apply");
    context.append(&apply);
    let original = style_menu_button(wordcraft_gnome::t!("Restore Original Size"));
    original.set_widget_name("picture-original");
    context.append(&original);
    let replace = style_menu_button(wordcraft_gnome::t!("Replace Image…"));
    replace.set_widget_name("picture-replace");
    context.append(&replace);
    let delete = style_menu_button(wordcraft_gnome::t!("Delete Image"));
    delete.set_widget_name("picture-delete");
    context.append(&delete);
    // Keep the header fixed when the window is short.
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(440);
    scroll.set_propagate_natural_height(true);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.remove(&insert);
    root.remove(&hint);
    root.remove(&context);
    body.append(&insert);
    body.append(&hint);
    body.append(&context);
    scroll.set_child(Some(&body));
    root.append(&scroll);
    pop.set_child(Some(&root));
    menu.set_popover(Some(&pop));
    let state = Rc::new(PicturePopover {
        ui: Rc::downgrade(ui),
        pop: pop.clone(),
        context,
        hint,
        width,
        height,
        lock,
        syncing: Cell::new(false),
        width_changed: Cell::new(true),
        target: RefCell::new(None),
    });
    for (is_width, n) in [(true, &state.width), (false, &state.height)] {
        let weak = Rc::downgrade(&state);
        n.connect_value_changed(move |n| {
            let Some(s) = weak.upgrade() else {
                return;
            };
            if s.syncing.get() {
                return;
            }
            s.width_changed.set(is_width);
            if s.lock.is_active() {
                if let Some((_, w, h)) = s.target.borrow().as_ref() {
                    s.syncing.set(true);
                    if is_width {
                        s.height.set_value(n.value() * *h as f64 / *w as f64);
                    } else {
                        s.width.set_value(n.value() * *w as f64 / *h as f64);
                    }
                    s.syncing.set(false);
                }
            }
        });
    }
    {
        let weak = Rc::downgrade(&state);
        apply.connect_clicked(glib::clone!(
            #[weak]
            ui,
            move |_| {
                // Preserve the weak tab-lifetime guard before touching controls.
                let _tab_guard = &ui;
                if let Some(s) = weak.upgrade() {
                    s.width.update();
                    s.height.update();
                    let (w, h, lock, by_width) = (
                        s.width.value() as f32 * PT_PER_MM as f32,
                        s.height.value() as f32 * PT_PER_MM as f32,
                        s.lock.is_active(),
                        s.width_changed.get(),
                    );
                    s.execute(move |session| pictures::resize(session, w, h, lock, by_width));
                }
            }
        ));
    }
    {
        let weak = Rc::downgrade(&state);
        original.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.execute(pictures::original);
            }
        });
    }
    {
        let weak = Rc::downgrade(&state);
        delete.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.execute(pictures::delete);
            }
        });
    }
    for (b, replacing) in [(insert, false), (replace, true)] {
        let weak = Rc::downgrade(ui);
        let parent = parent.downgrade();
        b.connect_clicked(move |_| {
            if let (Some(ui), Some(parent)) = (weak.upgrade(), parent.upgrade()) {
                choose_picture(&ui, &parent, replacing);
            }
        });
    }
    {
        let weak = Rc::downgrade(&state);
        pop.connect_show(move |_| {
            if let Some(s) = weak.upgrade() {
                s.refresh();
            }
        });
    }
    state.refresh();
    (menu, state)
}

pub(super) struct DocumentProperties {
    pub(super) window: adw::Window,
    pub(super) metadata: [adw::EntryRow; 5],
    pub(super) width: gtk::SpinButton,
    pub(super) height: gtk::SpinButton,
    pub(super) top: gtk::SpinButton,
    pub(super) bottom: gtk::SpinButton,
    pub(super) left: gtk::SpinButton,
    pub(super) right: gtk::SpinButton,
    pub(super) apply: gtk::Button,
    pub(super) headers: Rc<HeaderControls>,
}
pub(super) fn properties_number(
    group: &adw::PreferencesGroup,
    label: &str,
    value: f32,
) -> gtk::SpinButton {
    let row = adw::ActionRow::new();
    row.set_title(label);
    let n = mm_input(value as f64 / PT_PER_MM, 0.0, 1584.0 / PT_PER_MM);
    row.add_suffix(&n);
    row.set_activatable_widget(Some(&n));
    accessible_name(&n, label);
    group.add(&row);
    n
}
pub(super) fn show_document_properties(
    ui: &Rc<Ui>,
    parent: &ApplicationWindow,
) -> DocumentProperties {
    let (state, section, info) = {
        let mut value = ui.current.borrow_mut();
        let s = &mut value.as_mut().unwrap().session;
        let i = page_options::current_section(s);
        let p = s.doc.sections()[i].1;
        (
            PageSetup {
                width: p.page_w,
                height: p.page_h,
                top: p.margin_top,
                bottom: p.margin_bottom,
                left: p.margin_left,
                right: p.margin_right,
            },
            i,
            DocumentInfo::current(s),
        )
    };
    let identity = document_identity(ui);
    let window = adw::Window::builder()
        .title(wordcraft_gnome::t!("Document Properties"))
        .transient_for(parent)
        .modal(true)
        .default_width(480)
        .default_height(650)
        .build();
    let toolbar = ToolbarView::new();
    let header = HeaderBar::new();
    header.set_show_end_title_buttons(false);
    header.set_show_start_title_buttons(false);
    let cancel = gtk::Button::with_label(wordcraft_gnome::t!("Cancel"));
    header.pack_start(&cancel);
    let apply = gtk::Button::with_label(wordcraft_gnome::t!("Apply"));
    apply.set_widget_name("document-properties-apply");
    apply.add_css_class("suggested-action");
    header.pack_end(&apply);
    toolbar.add_top_bar(&header);
    let page = adw::PreferencesPage::new();
    let paper = adw::PreferencesGroup::new();
    paper.set_title(wordcraft_gnome::t!("Page"));
    page.add(&paper);
    let sizes = adw::ComboRow::new();
    sizes.set_title(wordcraft_gnome::t!("Paper Size"));
    sizes.set_model(Some(&gtk::StringList::new(&[
        wordcraft_gnome::t!("Custom"),
        "A4",
        "A5",
        "A3",
        wordcraft_gnome::t!("Letter"),
        "Legal",
    ])));
    paper.add(&sizes);
    let orientation = adw::ComboRow::new();
    orientation.set_title(wordcraft_gnome::t!("Orientation"));
    orientation.set_model(Some(&gtk::StringList::new(&[
        wordcraft_gnome::t!("Portrait"),
        wordcraft_gnome::t!("Landscape"),
    ])));
    orientation.set_selected(if state.width > state.height { 1 } else { 0 });
    paper.add(&orientation);
    let width = properties_number(&paper, wordcraft_gnome::t!("Width (mm)"), state.width);
    let height = properties_number(&paper, wordcraft_gnome::t!("Height (mm)"), state.height);
    width.set_widget_name("paper-width");
    height.set_widget_name("paper-height");
    sizes.set_widget_name("paper-size");
    orientation.set_widget_name("paper-orientation");
    let presets = [
        (210.0, 297.0),
        (148.0, 210.0),
        (297.0, 420.0),
        (215.9, 279.4),
        (215.9, 355.6),
    ];
    for (i, (w, h)) in presets.iter().enumerate() {
        let (a, b) = if state.width > state.height {
            (*h, *w)
        } else {
            (*w, *h)
        };
        if (width.value() - a).abs() < 0.1 && (height.value() - b).abs() < 0.1 {
            sizes.set_selected(i as u32 + 1);
        }
    }
    width.set_sensitive(sizes.selected() == 0);
    height.set_sensitive(sizes.selected() == 0);
    let margins = adw::PreferencesGroup::new();
    margins.set_title(wordcraft_gnome::t!("Margins"));
    page.add(&margins);
    let top = properties_number(&margins, wordcraft_gnome::t!("Top (mm)"), state.top);
    let bottom = properties_number(&margins, wordcraft_gnome::t!("Bottom (mm)"), state.bottom);
    let left = properties_number(&margins, wordcraft_gnome::t!("Left (mm)"), state.left);
    let right = properties_number(&margins, wordcraft_gnome::t!("Right (mm)"), state.right);
    let scope = adw::ComboRow::new();
    scope.set_title(wordcraft_gnome::t!("Apply To"));
    scope.set_model(Some(&gtk::StringList::new(&[
        wordcraft_gnome::t!("Whole Document"),
        wordcraft_gnome::t!("Current Section"),
    ])));
    let multi = ui
        .current
        .borrow()
        .as_ref()
        .is_some_and(|d| d.session.doc.sections().len() > 1);
    scope.set_selected(if multi { 1 } else { 0 });
    scope.set_visible(multi);
    margins.add(&scope);
    let message = gtk::Label::new(None);
    message.set_wrap(true);
    message.add_css_class("error");
    message.set_margin_start(12);
    message.set_margin_end(12);
    message.set_margin_bottom(12);
    message.set_visible(false);
    let stack = adw::ViewStack::new();
    stack.set_widget_name("document-properties-pages");
    stack.set_vexpand(true);
    let information = adw::PreferencesPage::new();
    let properties = adw::PreferencesGroup::new();
    properties.set_description(Some(wordcraft_gnome::t!(
        "Save as DOCX to preserve the category."
    )));
    information.add(&properties);
    let metadata = [
        ("title", wordcraft_gnome::t!("Title"), info.title),
        ("subject", wordcraft_gnome::t!("Subject"), info.subject),
        ("author", wordcraft_gnome::t!("Author"), info.author),
        ("keywords", wordcraft_gnome::t!("Keywords"), info.keywords),
        ("category", wordcraft_gnome::t!("Category"), info.category),
    ]
    .map(|(id, title, text)| {
        let row = adw::EntryRow::new();
        row.set_widget_name(&format!("document-{id}"));
        row.set_title(title);
        row.set_text(&text);
        properties.add(&row);
        row
    });
    stack.add_titled(
        &information,
        Some("information"),
        wordcraft_gnome::t!("Information"),
    );
    stack.add_titled(&page, Some("page"), wordcraft_gnome::t!("Page"));
    let header_page = adw::PreferencesPage::new();
    let headers = HeaderControls::new(ui, &header_page, section);
    stack.add_titled(
        &header_page,
        Some("headers"),
        wordcraft_gnome::t!("Headers and Footers"),
    );
    for (child, icon) in [
        (
            information.upcast_ref::<gtk::Widget>(),
            "dialog-information-symbolic",
        ),
        (
            page.upcast_ref::<gtk::Widget>(),
            "document-properties-symbolic",
        ),
        (
            header_page.upcast_ref::<gtk::Widget>(),
            "view-paged-symbolic",
        ),
    ] {
        stack.page(child).set_icon_name(Some(icon));
    }
    let switcher = adw::ViewSwitcher::new();
    switcher.set_stack(Some(&stack));
    switcher.set_policy(adw::ViewSwitcherPolicy::Narrow);
    switcher.set_halign(gtk::Align::Center);
    switcher.set_margin_top(6);
    switcher.set_margin_bottom(6);
    toolbar.add_top_bar(&switcher);
    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    content.append(&stack);
    content.append(&message);
    toolbar.set_content(Some(&content));
    window.set_content(Some(&toolbar));
    let syncing = Rc::new(Cell::new(false));
    {
        let width = width.downgrade();
        let height = height.downgrade();
        let orientation = orientation.downgrade();
        let syncing = syncing.clone();
        sizes.connect_selected_notify(move |r| {
            if let (Some(width), Some(height)) = (width.upgrade(), height.upgrade()) {
                width.set_sensitive(r.selected() == 0);
                height.set_sensitive(r.selected() == 0);
            }
            let Some(i) = r.selected().checked_sub(1) else {
                return;
            };
            let Some(&(a, b)) = presets.get(i as usize) else {
                return;
            };
            let (Some(width), Some(height), Some(orientation)) =
                (width.upgrade(), height.upgrade(), orientation.upgrade())
            else {
                return;
            };
            syncing.set(true);
            let (a, b) = if orientation.selected() == 1 {
                (b, a)
            } else {
                (a, b)
            };
            width.set_value(a);
            height.set_value(b);
            syncing.set(false);
        });
    }
    {
        let width = width.downgrade();
        let height = height.downgrade();
        let syncing = syncing.clone();
        orientation.connect_selected_notify(move |r| {
            let (Some(width), Some(height)) = (width.upgrade(), height.upgrade()) else {
                return;
            };
            let (w, h) = (width.value(), height.value());
            if (r.selected() == 1 && w < h) || (r.selected() == 0 && w > h) {
                syncing.set(true);
                width.set_value(h);
                height.set_value(w);
                syncing.set(false);
            }
        });
    }
    for n in [&width, &height] {
        let sizes = sizes.downgrade();
        let orientation = orientation.downgrade();
        let width = width.downgrade();
        let height = height.downgrade();
        let syncing = syncing.clone();
        n.connect_value_changed(move |_| {
            if !syncing.get() {
                let (Some(sizes), Some(orientation), Some(width), Some(height)) = (
                    sizes.upgrade(),
                    orientation.upgrade(),
                    width.upgrade(),
                    height.upgrade(),
                ) else {
                    return;
                };
                sizes.set_selected(0);
                orientation.set_selected(if width.value() > height.value() { 1 } else { 0 });
            }
        });
    }
    {
        let weak = window.downgrade();
        cancel.connect_clicked(move |_| {
            if let Some(w) = weak.upgrade() {
                w.close();
            }
        });
    }
    {
        let ui = ui.clone();
        let window = window.downgrade();
        let fields = [
            width.clone(),
            height.clone(),
            top.clone(),
            bottom.clone(),
            left.clone(),
            right.clone(),
        ];
        let headers = headers.clone();
        let metadata = metadata.clone();
        apply.connect_clicked(glib::clone!(
            #[weak]
            ui,
            move |_| {
                if ui.closed.get() || document_identity(&ui) != identity {
                    message.set_text(wordcraft_gnome::t!(
                        "The document has changed. Close and reopen its properties."
                    ));
                    message.set_visible(true);
                    return;
                }
                for n in &fields {
                    n.update();
                }
                let values = fields
                    .each_ref()
                    .map(|n| n.value() as f32 * PT_PER_MM as f32);
                let setup = PageSetup {
                    width: values[0],
                    height: values[1],
                    top: values[2],
                    bottom: values[3],
                    left: values[4],
                    right: values[5],
                };
                let result = {
                    let mut value = ui.current.borrow_mut();
                    let [title, subject, author, keywords, category] =
                        metadata.each_ref().map(|r| r.text().to_string());
                    apply_properties(
                        &mut value.as_mut().unwrap().session,
                        setup,
                        scope.selected() == 0,
                        section,
                        &headers.values(),
                        &DocumentInfo {
                            title,
                            subject,
                            author,
                            keywords,
                            category,
                        },
                    )
                };
                match result {
                    Ok(changed) => {
                        if changed {
                            after_content_change(&ui);
                        }
                        if let Some(w) = window.upgrade() {
                            w.close();
                        }
                    }
                    Err(e) => {
                        message.set_text(&e);
                        message.set_visible(true);
                    }
                }
            }
        ));
    }
    window.present();
    DocumentProperties {
        window,
        metadata,
        width,
        height,
        top,
        bottom,
        left,
        right,
        apply,
        headers,
    }
}
pub(super) fn export_pdf_dialog(ui: &Rc<Ui>, parent: &ApplicationWindow) {
    let dialog = gtk::FileDialog::new();
    dialog.set_title(wordcraft_gnome::t!("Export to PDF"));
    dialog.set_modal(true);
    let filter = gtk::FileFilter::new();
    filter.set_name(Some(wordcraft_gnome::t!("PDF Document (.pdf)")));
    filter.add_pattern("*.pdf");
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));
    let name = ui.document_name.borrow().clone();
    let stem = Path::new(&name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    dialog.set_initial_name(Some(&format!("{stem}.pdf")));
    let identity = document_identity(ui);
    let ui = ui.clone();
    dialog.save(
        Some(parent),
        None::<&gio::Cancellable>,
        glib::clone!(
            #[weak]
            ui,
            move |result| match result {
                Ok(file) => {
                    if ui.closed.get() || document_identity(&ui) != identity {
                        ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                            "The document has changed; export it again"
                        )));
                        return;
                    }
                    let Some(path) = file.path() else {
                        ui.toasts
                            .add_toast(Toast::new(wordcraft_gnome::t!("Choose a Local File")));
                        return;
                    };
                    let doc = ui.current.borrow().as_ref().unwrap().session.doc.clone();
                    ui.toasts
                        .add_toast(Toast::new(wordcraft_gnome::t!("Exporting PDF…")));
                    let weak = Rc::downgrade(&ui);
                    glib::MainContext::default().spawn_local(async move {
                        let result = gio::spawn_blocking(move || {
                            pdf_export::export(&mut Session::new(doc), &path)
                        })
                        .await;
                        if let Some(ui) = weak.upgrade() {
                            if !ui.closed.get() {
                                let text = match result {
                                    Ok(Ok(())) => wordcraft_gnome::t!("PDF Exported").to_string(),
                                    Ok(Err(e)) => e,
                                    Err(_) => {
                                        wordcraft_gnome::t!("Could Not Export PDF").to_string()
                                    }
                                };
                                ui.toasts.add_toast(Toast::new(&text));
                            }
                        }
                    });
                }
                Err(e)
                    if e.matches(gtk::DialogError::Cancelled)
                        || e.matches(gtk::DialogError::Dismissed) => {}
                Err(e) => ui.toasts.add_toast(Toast::new(&e.to_string())),
            }
        ),
    );
}
