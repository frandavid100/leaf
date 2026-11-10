//! Native inline picture controls and cancellable file selection.
use super::*;

pub(super) fn picture_properties_target(s: &Session) -> Option<(Pos, f32, f32)> {
    pictures::selected(s).or_else(|| {
        let caption = wordcraft_gnome::captions::at(&s.doc, &s.sel.focus)?;
        if caption.pos.story != s.sel.focus.story || caption.pos.path != s.sel.focus.path {
            return None;
        }
        let wordcraft_gnome::captions::Target::Image(pos) = caption.owner else {
            return None;
        };
        match s.doc.para_at(&pos)?.object_at(pos.off)? {
            wordcraft_doc::para::InlineObject::Image { w, h, .. } => Some((pos, *w, *h)),
            _ => None,
        }
    })
}

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
    caption: gtk::CheckButton,
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
                .and_then(|d| picture_properties_target(&d.session))
        });
        self.context.set_sensitive(target.is_some());
        self.hint.set_visible(false);
        if let Some(parent) = self.context.parent() {
            parent.set_tooltip_text(target.is_none().then_some(wordcraft_gnome::t!(
                "Select an image in the document to edit it."
            )));
        }
        self.syncing.set(true);
        let shown = self
            .ui
            .upgrade()
            .and_then(|ui| {
                let current = ui.current.borrow();
                let s = &current.as_ref()?.session;
                wordcraft_gnome::captions::for_target(
                    &s.doc,
                    &wordcraft_gnome::captions::Target::Image(target.as_ref()?.0.clone()),
                )
                .map(|c| c.visible)
            })
            .unwrap_or(false);
        self.caption.set_active(shown);
        let body = target
            .as_ref()
            .is_some_and(|(p, _, _)| p.story == wordcraft_doc::StoryRef::Body);
        let writable = self.ui.upgrade().is_some_and(|u| {
            u.current
                .borrow()
                .as_ref()
                .is_some_and(|d| !wordcraft_engine::cmd::protect::enforced(&d.session.doc))
        });
        self.caption.set_sensitive(body && writable);
        self.caption
            .set_tooltip_text((target.is_some() && !body).then_some(wordcraft_gnome::t!(
                "Captions are available in the document body."
            )));
        if let Some((_, w, h)) = &target {
            self.width.set_value(*w as f64 / (72. / 2.54));
            self.height.set_value(*h as f64 / (72. / 2.54));
        } else {
            self.width.set_text("");
            self.height.set_text("");
            self.spacing.set_text("");
            self.move_with_text.set_active(false);
        }
        if let Some(ui) = self.ui.upgrade() {
            if let Some(float) = ui.current.borrow().as_ref().and_then(|d| {
                let (p, _, _) = target.as_ref()?;
                match d.session.doc.para_at(p)?.object_at(p.off)? {
                    wordcraft_doc::para::InlineObject::Image { float, .. } => Some(float.clone()),
                    _ => None,
                }
            }) {
                self.spacing.set_value(float.dist as f64 / (72. / 2.54));
                self.move_with_text
                    .set_active(wordcraft_engine::cmd::objects::moves_with_text(&float));
                self.move_with_text
                    .set_sensitive(float.wrap != wordcraft_doc::para::Wrap::Inline);
                self.move_with_text.set_tooltip_text(
                    (float.wrap == wordcraft_doc::para::Wrap::Inline).then_some(
                        wordcraft_gnome::t!("Inline images move with the text automatically."),
                    ),
                );
            }
        }
        self.syncing.set(false);
        self.target.replace(target);
    }
    fn number_edit(
        &self,
        f: impl FnOnce(&mut Session) -> Result<(), String>,
    ) -> Result<(), String> {
        if self.syncing.get() {
            return Ok(());
        }
        let ui = self.ui.upgrade().ok_or("Document closed")?;
        let mut failure = None;
        tools_change(&ui, |session| {
            if pictures::selected(session).is_none() {
                let (pos, _, _) = picture_properties_target(session)
                    .ok_or(wordcraft_gnome::t!("Select an Image"))?;
                pictures::select(session, &pos)?;
            }
            let result = f(session);
            if let Err(e) = &result {
                failure = Some(e.clone());
            }
            result
        });
        if let Some(e) = failure {
            return Err(e);
        }
        self.refresh();
        Ok(())
    }
    pub(super) fn execute(&self, f: impl FnOnce(&mut Session) -> Result<(), String>) {
        if let Some(ui) = self.ui.upgrade() {
            if tools_change(&ui, |s| {
                if pictures::selected(s).is_none() {
                    let (pos, _, _) = picture_properties_target(s)
                        .ok_or(wordcraft_gnome::t!("Select an Image"))?;
                    pictures::select(s, &pos)?;
                }
                f(s)
            }) {
                self.refresh();
                close_format_popover(&self.pop);
                if !ui.format_group.pinned.get() {
                    ui.document_box.grab_focus();
                    caret_ensure_visible(&ui);
                }
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
    let caption = gtk::CheckButton::with_label(wordcraft_gnome::t!("Show Caption"));
    caption.set_widget_name("picture-show-caption");
    context.append(&caption);
    context.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let width = mm_input(3.0, 1.0 / (72. / 2.54), 4000.0 / (72. / 2.54));
    width.set_widget_name("picture-width");
    width.set_digits(3);
    let height = mm_input(1.5, 1.0 / (72. / 2.54), 4000.0 / (72. / 2.54));
    height.set_widget_name("picture-height");
    height.set_digits(3);
    for (label, n) in [
        (wordcraft_gnome::t!("Width (cm)"), &width),
        (wordcraft_gnome::t!("Height (cm)"), &height),
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

    context.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let spacing = mm_input(0.2, 0.0, 144.0 / (72. / 2.54));
    spacing.set_digits(3);
    let spacing_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let label = gtk::Label::new(Some(wordcraft_gnome::t!("Space Around Image (cm)")));
    label.set_hexpand(true);
    label.set_xalign(0.0);
    spacing_row.append(&label);
    spacing_row.append(&spacing);
    context.append(&spacing_row);
    let apply_spacing = style_menu_button(wordcraft_gnome::t!("Apply Spacing"));

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
    move_with_text.set_widget_name("picture-move-with-text");
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
    // Stable logical groups are leased, rather than duplicated, by individual tools.
    let size_body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    for control in [
        width.parent().unwrap(),
        height.parent().unwrap(),
        lock.clone().upcast(),
    ] {
        context.remove(&control);
        size_body.append(&control);
    }
    let size_slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    size_slot.set_widget_name("format-row-elements-image-size");
    size_slot.append(&size_body);
    context.insert_child_after(&size_slot, Some(&caption));
    let spacing_body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    context.remove(&spacing_row);

    spacing_body.append(&spacing_row);

    let spacing_slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    spacing_slot.set_widget_name("format-row-elements-image-spacing");
    spacing_slot.append(&spacing_body);
    context.insert_child_after(&spacing_slot, Some(&size_slot));
    let wrap_body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    for (button, _) in &placement_buttons {
        context.remove(button);
        wrap_body.append(button);
    }
    let wrap_slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    wrap_slot.set_widget_name("format-row-elements-image-wrap");
    wrap_slot.append(&wrap_body);
    context.insert_child_after(&wrap_slot, Some(&spacing_slot));
    for (widget, id) in [
        (caption.clone().upcast::<gtk::Widget>(), "image-caption"),
        (move_with_text.clone().upcast(), "image-move"),
        (original.clone().upcast(), "image-original"),
        (replace.clone().upcast(), "image-replace"),
        (delete.clone().upcast(), "image-delete"),
    ] {
        let prev = widget.prev_sibling();
        context.remove(&widget);
        let slot = gtk::Box::new(gtk::Orientation::Vertical, 0);
        slot.set_widget_name(&format!("format-row-elements-{id}"));
        slot.append(&widget);
        context.insert_child_after(&slot, prev.as_ref());
    }
    // Keep the header fixed when the window is short.
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(440);
    scroll.set_propagate_natural_height(true);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.remove(&insert);
    root.remove(&hint);
    root.remove(&context);
    body.append(&context);
    scroll.set_child(Some(&body));
    root.append(&scroll);
    root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    root.append(&insert);
    pop.set_child(Some(&root));
    menu.set_popover(Some(&pop));
    apply.set_visible(false);
    apply_spacing.set_visible(false);
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
        caption,
        syncing: Cell::new(false),
        width_changed: Cell::new(true),
        target: RefCell::new(None),
    });
    {
        let weak = Rc::downgrade(&state);
        state.caption.connect_toggled(move |button| {
            let Some(p) = weak.upgrade().filter(|p| !p.syncing.get()) else {
                return;
            };
            let show = button.is_active();
            p.execute(move |s| wordcraft_gnome::captions::set_visible(s, show));
        });
    }
    for (button, mode) in placement_buttons {
        let weak = Rc::downgrade(&state);
        button.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                p.spacing.update();
                let distance = p.spacing.value() as f32 * (72. / 2.54) as f32;
                p.execute(move |s| pictures::placement(s, Some(mode), distance));
            }
        });
    }
    {
        let weak = Rc::downgrade(&state);
        apply_spacing.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                p.spacing.update();
                let distance = p.spacing.value() as f32 * (72. / 2.54) as f32;
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
    for (is_width, n) in [(true, &state.width), (false, &state.height)] {
        let weak = Rc::downgrade(&state);
        bind_numeric(ui, n, move |value| {
            let p = weak.upgrade().ok_or("Document closed")?;
            if p.syncing.get() {
                return Ok(());
            }
            let lock = p.lock.is_active();
            p.number_edit(move |session| {
                let mut args = serde_json::json!({"lockAspect":lock});
                args[if is_width { "width" } else { "height" }] =
                    serde_json::json!(value * (72. / 2.54));
                session
                    .run("picture.size", &args)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
        });
    }
    {
        let weak = Rc::downgrade(&state);
        bind_numeric(ui, &state.spacing, move |value| {
            let p = weak.upgrade().ok_or("Document closed")?;
            p.number_edit(move |session| {
                pictures::placement(session, None, (value * (72. / 2.54)) as f32)
            })
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
                        s.width.value() as f32 * (72. / 2.54) as f32,
                        s.height.value() as f32 * (72. / 2.54) as f32,
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
