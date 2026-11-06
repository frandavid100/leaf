//! Native inline picture controls and cancellable file selection.
use super::*;

pub(super) struct PicturePopover {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) pop: gtk::Popover,
    pub(super) context: gtk::Box,
    pub(super) hint: gtk::Label,
    pub(super) width: gtk::SpinButton,
    pub(super) height: gtk::SpinButton,
    pub(super) lock: gtk::CheckButton,
    spacing: gtk::SpinButton,
    move_with_text: gtk::CheckButton,
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
        self.syncing.set(true);
        if let Some((_, w, h)) = &target {
            self.width.set_value(*w as f64 / PT_PER_MM);
            self.height.set_value(*h as f64 / PT_PER_MM);
        }
        if let Some(ui) = self.ui.upgrade() {
            if let Some(float) = ui
                .current
                .borrow()
                .as_ref()
                .and_then(|d| pictures::layout(&d.session))
            {
                self.spacing.set_value(float.dist as f64 / PT_PER_MM);
                self.move_with_text
                    .set_active(wordcraft_engine::cmd::objects::moves_with_text(&float));
                self.move_with_text
                    .set_sensitive(float.wrap != wordcraft_doc::para::Wrap::Inline);
            }
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
    let selected = gtk::Label::new(Some(wordcraft_gnome::t!("Selected Image")));
    selected.set_xalign(0.);
    selected.add_css_class("heading");
    context.append(&selected);
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
    context.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let spacing = mm_input(2.0, 0.0, 144.0 / PT_PER_MM);
    let spacing_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::new(Some(wordcraft_gnome::t!("Space Around Image (mm)")));
    label.set_hexpand(true);
    label.set_xalign(0.0);
    spacing_row.append(&label);
    spacing_row.append(&spacing);
    context.append(&spacing_row);
    let apply_spacing = style_menu_button(wordcraft_gnome::t!("Apply Spacing"));
    context.append(&apply_spacing);
    let mut placement_buttons = Vec::new();
    for (label, mode) in [
        (wordcraft_gnome::t!("In Line with Text"), "inline"),
        (wordcraft_gnome::t!("Image on Left, Text on Right"), "left"),
        (wordcraft_gnome::t!("Image on Right, Text on Left"), "right"),
        (wordcraft_gnome::t!("Text Above and Below"), "topAndBottom"),
    ] {
        let button = style_menu_button(label);
        context.append(&button);
        button.set_widget_name(&format!("picture-placement-{mode}"));
        placement_buttons.push((button, mode));
    }
    let move_with_text = gtk::CheckButton::with_label(wordcraft_gnome::t!("Move with Text"));
    context.append(&move_with_text);
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
        spacing,
        move_with_text,
        syncing: Cell::new(false),
        width_changed: Cell::new(true),
        target: RefCell::new(None),
    });
    for (button, mode) in placement_buttons {
        let weak = Rc::downgrade(&state);
        button.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                p.spacing.update();
                let distance = p.spacing.value() as f32 * PT_PER_MM as f32;
                p.execute(move |s| pictures::placement(s, Some(mode), distance));
            }
        });
    }
    {
        let weak = Rc::downgrade(&state);
        apply_spacing.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                p.spacing.update();
                let distance = p.spacing.value() as f32 * PT_PER_MM as f32;
                p.execute(move |s| pictures::placement(s, None, distance));
            }
        });
    }
    {
        let weak = Rc::downgrade(&state);
        state.move_with_text.connect_toggled(move |button| {
            let Some(p) = weak.upgrade() else { return };
            if p.syncing.get() {
                return;
            }
            let active = button.is_active();
            p.execute(move |s| {
                s.run("arrange.moveWithText", &serde_json::json!({"value":active}))
                    .map_err(|e| e.to_string())?;
                Ok(())
            });
        });
    }
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
