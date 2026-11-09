use super::*;

// One popover with list, detail and name-entry pages; no auxiliary windows.
pub(super) fn style_menu_button(label: &str) -> gtk::Button {
    let b = gtk::Button::with_label(label);
    b.add_css_class("flat");
    if let Some(l) = b.child().and_then(|w| w.downcast::<gtk::Label>().ok()) {
        l.set_xalign(0.0);
    }
    b
}
pub(super) struct StylePopover {
    pub(super) ui: std::rc::Weak<Ui>,
    pub(super) popover: glib::WeakRef<gtk::Popover>,
    pub(super) stack: gtk::Stack,
    pub(super) list: gtk::Box,
    pub(super) search: gtk::SearchEntry,
    show_all: Cell<bool>,
    pub(super) detail: gtk::Box,
    pub(super) form: gtk::Box,
    pub(super) message: gtk::Label,
    pub(super) header: Rc<FormatHeader>,
}
impl StylePopover {
    pub(super) fn sync_selection(self: &Rc<Self>) {
        // Reopening rebuilds the list from the current selection. The actual
        // list also covers its borrowed home inside a contextual menu: checking
        // only the original popover would miss that visible presentation.
        if !self.list.is_mapped() {
            return;
        }
        let Some(ui) = self.ui.upgrade() else { return };
        let current = ui
            .current
            .borrow()
            .as_ref()
            .and_then(|d| wordcraft_gnome::styles::current(&d.session));
        if !self.show_all.get()
            && self.search.text().is_empty()
            && current
                .as_ref()
                .is_some_and(|id| find_widget(&self.list, &format!("style-apply-{id}")).is_none())
        {
            self.refresh_list();
            return;
        }
        let mut row = self.list.first_child();
        while let Some(w) = row {
            perf_count("style_rows_visited", 1);
            row = w.next_sibling();
            let active = current
                .as_ref()
                .is_some_and(|id| find_widget(&w, &format!("style-apply-{id}")).is_some());
            if active {
                w.add_css_class("current-style");
            } else {
                w.remove_css_class("current-style");
            }
        }
    }
    pub(super) fn report(&self, text: &str) {
        self.message.set_text(text);
        self.message.set_visible(!text.is_empty());
    }
    pub(super) fn run(
        &self,
        edit: wordcraft_gnome::styles::Edit,
    ) -> Result<Option<String>, String> {
        let ui = self
            .ui
            .upgrade()
            .ok_or(wordcraft_gnome::t!("The document is closed"))?;
        let (result, changed) = {
            let mut s = ui.current.borrow_mut();
            let d = s.as_mut().ok_or(wordcraft_gnome::t!("No document"))?;
            let rev = d.session.rev();
            let result = wordcraft_gnome::styles::edit(&mut d.session, edit)?;
            (result, d.session.rev() != rev)
        };
        if changed {
            after_content_change(&ui);
        } else {
            refresh_selection(&ui);
        }
        Ok(result)
    }
    pub(super) fn refresh_list(self: &Rc<Self>) {
        clear_box(&self.list);
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let (styles, current, expandable) = {
            let s = ui.current.borrow();
            let Some(d) = s.as_ref() else {
                return;
            };
            let all = wordcraft_gnome::styles::list(&d.session);
            let current = wordcraft_gnome::styles::current(&d.session);
            let mut gallery = d.session.doc.styles.gallery();
            let has_gallery = !gallery.is_empty();
            if let Some(id) = current.as_deref() {
                if let Some(style) = all.iter().find(|style| style.id == id) {
                    if !gallery.iter().any(|shown| shown.id == style.id) {
                        gallery.push(style);
                    }
                }
            }
            let expandable = !self.show_all.get()
                && self.search.text().is_empty()
                && has_gallery
                && gallery.len() < all.len();
            let styles: Vec<_> = if expandable {
                gallery.into_iter().cloned().collect()
            } else {
                all
            };
            let styles = styles
                .into_iter()
                .map(|style| {
                    use wordcraft_doc::{props::CharProps, styles::StyleKind};
                    let sheet = &d.session.doc.styles;
                    let resolved = if style.kind == StyleKind::Paragraph {
                        sheet.resolve_char(Some(&style.id), &CharProps::default())
                    } else {
                        let direct = CharProps {
                            style: Some(style.id.clone()),
                            ..Default::default()
                        };
                        sheet.resolve_char(None, &direct)
                    };
                    (style, resolved)
                })
                .collect::<Vec<_>>();
            (styles, current, expandable)
        };
        let query = self.search.text().to_lowercase();
        let mut count = 0;
        for (st, resolved) in styles.into_iter().filter(|(s, _)| {
            s.name.to_lowercase().contains(&query) || s.id.to_lowercase().contains(&query)
        }) {
            count += 1;
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            row.add_css_class("style-row");
            if current.as_deref() == Some(st.id.as_str()) {
                row.add_css_class("current-style");
            }
            let apply = gtk::Button::new();
            apply.add_css_class("flat");
            apply.set_hexpand(true);
            apply.set_widget_name(&format!("style-apply-{}", st.id));
            let label = gtk::Label::new(Some(&st.name));
            let attributes = gtk::pango::AttrList::new();
            attributes.insert(gtk::pango::AttrString::new_family(&resolved.font));
            attributes.insert(gtk::pango::AttrSize::new(
                (resolved.size.clamp(8., 28.) * gtk::pango::SCALE as f32).round() as i32,
            ));
            if resolved.bold {
                attributes.insert(gtk::pango::AttrInt::new_weight(gtk::pango::Weight::Bold));
            }
            if resolved.italic {
                attributes.insert(gtk::pango::AttrInt::new_style(gtk::pango::Style::Italic));
            }
            if resolved.underline != wordcraft_doc::props::Underline::None {
                attributes.insert(gtk::pango::AttrInt::new_underline(
                    gtk::pango::Underline::Single,
                ));
            }
            if resolved.strike || resolved.double_strike {
                attributes.insert(gtk::pango::AttrInt::new_strikethrough(true));
            }
            if let wordcraft_doc::props::TextColor::Rgb(color) = resolved.color {
                let luminance = [color.0, color.1, color.2].map(|channel| {
                    let value = f64::from(channel) / 255.0;
                    if value <= 0.04045 {
                        value / 12.92
                    } else {
                        ((value + 0.055) / 1.055).powf(2.4)
                    }
                });
                let light = luminance[0] * 0.2126 + luminance[1] * 0.7152 + luminance[2] * 0.0722;
                let legible = if adw::StyleManager::default().is_dark() {
                    light > 0.22
                } else {
                    light < 0.18
                };
                if legible {
                    attributes.insert(gtk::pango::AttrColor::new_foreground(
                        u16::from(color.0) * 257,
                        u16::from(color.1) * 257,
                        u16::from(color.2) * 257,
                    ));
                }
            }
            label.set_attributes(Some(&attributes));
            label.set_xalign(0.0);
            label.set_hexpand(true);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(28);
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            content.append(&label);
            if current.as_deref() == Some(st.id.as_str()) {
                apply.set_tooltip_text(Some(wordcraft_gnome::t!("Current Style")));
            }
            accessible_name(
                &apply,
                &wordcraft_gnome::i18n::interpolate(
                    wordcraft_gnome::t!("Apply Style {}"),
                    &[("0", format!("{}", st.name))],
                ),
            );
            apply.set_child(Some(&content));
            row.append(&apply);
            let manage = gtk::Button::new();
            manage.set_icon_name("go-next-symbolic");
            manage.add_css_class("flat");
            let manage_name = wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Manage “{style}”"),
                &[("style", st.name.clone())],
            );
            manage.set_tooltip_text(Some(&manage_name));
            manage.set_widget_name(&format!("style-manage-{}", st.id));
            accessible_name(&manage, &manage_name);
            row.append(&manage);
            let weak = Rc::downgrade(self);
            let id = st.id.clone();
            apply.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    match s.run(wordcraft_gnome::styles::Edit::Apply(id.clone())) {
                        Ok(_) => {
                            s.refresh_list();
                            if let Some(p) = s.popover.upgrade() {
                                close_format_popover(&p);
                            }
                            if let Some(ui) = s.ui.upgrade() {
                                ui.document_box.grab_focus();
                                caret_ensure_visible(&ui);
                            }
                        }
                        Err(e) => s.report(&e),
                    }
                }
            });
            let weak = Rc::downgrade(self);
            manage.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.show_detail(&st.id);
                }
            });
            self.list.append(&row);
        }
        if expandable {
            let more = style_menu_button(wordcraft_gnome::t!("More Styles…"));
            let weak = Rc::downgrade(self);
            more.connect_clicked(move |_| {
                if let Some(panel) = weak.upgrade() {
                    panel.show_all.set(true);
                    panel.refresh_list();
                }
            });
            self.list.append(&more);
        }
        if count == 0 {
            let label = gtk::Label::new(Some(wordcraft_gnome::t!("No Styles Found")));
            label.add_css_class("dim-label");
            label.set_margin_top(18);
            label.set_margin_bottom(18);
            self.list.append(&label);
        }
    }
    pub(super) fn main(self: &Rc<Self>) {
        self.header.home();
        self.report("");
        self.refresh_list();
        self.stack.set_visible_child_name("list");
    }
    pub(super) fn back(self: &Rc<Self>, container: &gtk::Box, label: &str, id: Option<String>) {
        let _ = container;
        let weak = Rc::downgrade(self);
        self.header.back(label, move || {
            if let Some(s) = weak.upgrade() {
                if let Some(id) = &id {
                    s.show_detail(id);
                } else {
                    s.main();
                }
            }
        });
    }

    pub(super) fn style(self: &Rc<Self>, id: &str) -> Option<wordcraft_doc::styles::Style> {
        let ui = self.ui.upgrade()?;
        let state = ui.current.borrow();
        state.as_ref()?.session.doc.styles.get(id).cloned()
    }
    pub(super) fn action(
        self: &Rc<Self>,
        label: &str,
        name: &str,
        edit: wordcraft_gnome::styles::Edit,
        stay: Option<String>,
    ) -> gtk::Button {
        let b = style_menu_button(label);
        b.set_widget_name(name);
        let weak = Rc::downgrade(self);
        b.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                match s.run(edit.clone()) {
                    Ok(_) => {
                        if let Some(id) = &stay {
                            s.show_detail(id);
                            s.report(wordcraft_gnome::t!("Style Updated"));
                        } else {
                            s.main();
                            s.report(wordcraft_gnome::t!("Style Deleted"));
                        }
                    }
                    Err(e) => s.report(&e),
                }
            }
        });
        b
    }
    pub(super) fn show_detail(self: &Rc<Self>, id: &str) {
        let Some(st) = self.style(id) else {
            self.main();
            return;
        };
        self.report("");
        clear_box(&self.detail);
        self.back(&self.detail, &st.name, None);
        let title = gtk::Label::new(Some(&st.name));
        title.set_xalign(0.0);
        title.set_wrap(true);
        title.set_max_width_chars(28);
        title.add_css_class("heading");
        title.set_margin_top(6);
        self.detail.append(&title);
        if let Some(ui) = self.ui.upgrade() {
            let preview = {
                let state = ui.current.borrow();
                state
                    .as_ref()
                    .map(|d| wordcraft_gnome::styles::preview(&d.session, id))
            };
            if let Some(Ok(image)) = preview {
                let picture = gtk::Picture::for_paintable(&texture(image));
                picture.set_content_fit(gtk::ContentFit::ScaleDown);
                picture.set_size_request(290, 85);
                picture.add_css_class("document-sheet");
                picture.set_margin_top(8);
                picture.set_margin_bottom(8);
                picture.set_widget_name("style-preview");
                self.detail.append(&picture);
            }
        }
        self.detail
            .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        if st.kind == wordcraft_doc::styles::StyleKind::Paragraph {
            let candidates = self
                .ui
                .upgrade()
                .and_then(|u| {
                    u.current
                        .borrow()
                        .as_ref()
                        .map(|d| wordcraft_gnome::styles::list(&d.session))
                })
                .unwrap_or_default()
                .into_iter()
                .filter(|v| v.kind == wordcraft_doc::styles::StyleKind::Paragraph)
                .collect::<Vec<_>>();
            let label = gtk::Label::new(Some(wordcraft_gnome::t!("Following Paragraph Style")));
            label.set_xalign(0.0);
            self.detail.append(&label);
            let names = candidates
                .iter()
                .map(|v| v.name.as_str())
                .collect::<Vec<_>>();
            let next = gtk::DropDown::from_strings(&names);
            next.set_widget_name("style-next");
            next.set_tooltip_text(Some(wordcraft_gnome::t!(
                "Use DOCX to preserve changes to the following paragraph style."
            )));
            next.set_selected(
                candidates
                    .iter()
                    .position(|v| Some(&v.id) == st.next.as_ref().or(Some(&st.id)))
                    .unwrap_or(0) as u32,
            );
            self.detail.append(&next);
            let id = st.id.clone();
            let weak = Rc::downgrade(self);
            next.connect_selected_notify(move |choice| {
                if let (Some(s), Some(target)) =
                    (weak.upgrade(), candidates.get(choice.selected() as usize))
                {
                    if let Err(error) = s.run(wordcraft_gnome::styles::Edit::Next(
                        id.clone(),
                        target.id.clone(),
                    )) {
                        s.report(&error);
                    }
                }
            });
        }
        let update = self.action(
            wordcraft_gnome::t!("Update from Selection"),
            "style-update",
            wordcraft_gnome::styles::Edit::Update(st.id.clone()),
            Some(st.id.clone()),
        );
        self.detail.append(&update);
        let rename = style_menu_button(wordcraft_gnome::t!("Rename…"));
        rename.set_widget_name("style-rename");
        self.detail.append(&rename);
        let weak = Rc::downgrade(self);
        let style = st.clone();
        rename.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                s.show_form(Some(&style.id));
            }
        });
        let delete = self.action(
            wordcraft_gnome::t!("Delete Style"),
            "style-delete",
            wordcraft_gnome::styles::Edit::Delete(st.id.clone()),
            None,
        );
        delete.set_sensitive(!st.builtin);
        if st.builtin {
            delete.set_tooltip_text(Some(wordcraft_gnome::t!(
                "Built-in styles cannot be deleted"
            )));
        }
        self.detail.append(&delete);
        self.stack.set_visible_child_name("detail");
    }
    pub(super) fn show_form(self: &Rc<Self>, id: Option<&str>) {
        self.report("");
        clear_box(&self.form);
        let style = id.and_then(|id| self.style(id));
        self.back(
            &self.form,
            if style.is_some() {
                wordcraft_gnome::t!("Rename Style")
            } else {
                wordcraft_gnome::t!("New Style")
            },
            style.as_ref().map(|s| s.id.clone()),
        );
        let title = gtk::Label::new(Some(if style.is_some() {
            wordcraft_gnome::t!("Rename Style")
        } else {
            wordcraft_gnome::t!("New Style")
        }));
        title.set_xalign(0.0);
        title.add_css_class("heading");
        title.set_margin_top(8);
        self.form.append(&title);
        let entry = gtk::Entry::new();
        entry.set_placeholder_text(Some(wordcraft_gnome::t!("Style Name")));
        entry.set_widget_name("style-name");
        entry.set_hexpand(true);
        if let Some(st) = &style {
            entry.set_text(&st.name);
        }
        self.form.append(&entry);
        if style.is_none() {
            let hint = gtk::Label::new(Some(wordcraft_gnome::t!(
                "The style will use the formatting of the current selection."
            )));
            hint.set_wrap(true);
            hint.set_xalign(0.0);
            hint.add_css_class("dim-label");
            self.form.append(&hint);
        }
        let commit = gtk::Button::with_label(if style.is_some() {
            wordcraft_gnome::t!("Rename")
        } else {
            wordcraft_gnome::t!("Create Style")
        });
        commit.add_css_class("suggested-action");
        commit.set_widget_name("style-commit");
        self.form.append(&commit);
        let weak = Rc::downgrade(self);
        let input = entry.clone();
        commit.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                let edit = if let Some(st) = &style {
                    wordcraft_gnome::styles::Edit::Rename(st.id.clone(), input.text().to_string())
                } else {
                    wordcraft_gnome::styles::Edit::Create(input.text().to_string())
                };
                match s.run(edit) {
                    Ok(_) => {
                        s.search.set_text("");
                        s.main();
                        s.report(wordcraft_gnome::t!("Style Saved"));
                    }
                    Err(e) => s.report(&e),
                }
            }
        });
        let b = commit.downgrade();
        entry.connect_activate(move |_| {
            if let Some(b) = b.upgrade() {
                b.emit_clicked();
            }
        });
        self.stack.set_visible_child_name("form");
        entry.grab_focus();
        entry.select_region(0, -1);
    }
}
pub(super) fn install_style_popover(ui: &Rc<Ui>, menu: &gtk::MenuButton) -> Rc<StylePopover> {
    let pop = gtk::Popover::new();
    pop.add_css_class("style-popover");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    root.set_width_request(330);
    root.set_margin_top(8);
    root.set_margin_bottom(8);
    root.set_margin_start(8);
    root.set_margin_end(8);
    let header = format_popover_header(&pop, &root, wordcraft_gnome::t!("Style"), &ui.format_group);
    header.button.set_widget_name("style-back");
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    stack.set_vhomogeneous(false);
    stack.set_hhomogeneous(false);
    stack.set_interpolate_size(true);
    let main = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(wordcraft_gnome::t!("Search Styles")));
    search.set_widget_name("style-search");
    main.append(&search);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_min_content_height(240);
    scroll.set_max_content_height(340);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&list));
    main.append(&scroll);
    main.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    let create = style_menu_button(wordcraft_gnome::t!("New Style…"));
    create.set_widget_name("style-new");
    create.add_css_class("flat");
    main.append(&create);
    let detail = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let form = gtk::Box::new(gtk::Orientation::Vertical, 8);
    for (body, name) in [(&main, "list"), (&detail, "detail"), (&form, "form")] {
        body.set_margin_bottom(8);
        let page = gtk::ScrolledWindow::new();
        page.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        page.set_propagate_natural_height(true);
        page.set_max_content_height(420);
        page.set_child(Some(body));
        stack.add_named(&page, Some(name));
    }
    root.append(&stack);
    let message = gtk::Label::new(None);
    message.set_wrap(true);
    message.set_max_width_chars(36);
    message.set_xalign(0.0);
    message.set_visible(false);
    root.append(&message);
    pop.set_child(Some(&root));
    let state = Rc::new(StylePopover {
        ui: Rc::downgrade(ui),
        popover: pop.downgrade(),
        stack,
        list,
        search: search.clone(),
        show_all: Cell::new(false),
        detail,
        form,
        message,
        header,
    });
    let weak = Rc::downgrade(&state);
    search.connect_search_changed(move |_| {
        if let Some(s) = weak.upgrade() {
            s.refresh_list();
        }
    });
    let weak = Rc::downgrade(&state);
    create.connect_clicked(move |_| {
        if let Some(s) = weak.upgrade() {
            s.show_form(None);
        }
    });
    let s = state.clone();
    pop.connect_show(glib::clone!(
        #[weak]
        s,
        move |_| {
            if !s.ui.upgrade().is_some_and(|u| u.format_reflowing.get()) {
                s.main();
            }
        }
    ));
    menu.set_popover(Some(&pop));
    state
}
