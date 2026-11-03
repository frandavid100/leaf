use super::*;

// Papers-like bottom selector and a heading hierarchy sourced from WordCraft.
pub(super) fn sidebar_icon(kind: &str) -> gtk::Widget {
    let area = gtk::DrawingArea::new();
    area.set_content_width(18);
    area.set_content_height(18);
    area.set_halign(gtk::Align::Center);
    area.set_valign(gtk::Align::Center);
    let kind = kind.to_string();
    area.set_draw_func(move |w, cr, _, _| {
        let c = w.color();
        cr.set_source_rgba(
            c.red() as f64,
            c.green() as f64,
            c.blue() as f64,
            c.alpha() as f64,
        );
        cr.set_line_width(1.4);
        if kind == "pages" {
            for (x, y) in [(2.0, 2.0), (10.0, 2.0), (2.0, 10.0), (10.0, 10.0)] {
                cr.rectangle(x, y, 5.0, 5.0);
            }
            let _ = cr.stroke();
        } else {
            for (x, y, end) in [(2.0, 3.0, 16.0), (6.0, 8.0, 16.0), (6.0, 13.0, 13.0)] {
                cr.move_to(x, y);
                cr.line_to(end, y);
            }
            cr.move_to(2.0, 6.0);
            cr.line_to(2.0, 15.0);
            let _ = cr.stroke();
        }
    });
    area.upcast()
}
pub(super) struct OutlinePanel {
    pub(super) ui: std::rc::Weak<Ui>,
    toggle: gtk::ToggleButton,
    pub(super) list: gtk::ListBox,
    pub(super) status: gtk::Label,
    pub(super) collapsed: RefCell<std::collections::HashSet<String>>,
    pub(super) positions: RefCell<Vec<Pos>>,
    pub(super) pending: Cell<bool>,
    headings: RefCell<Vec<wordcraft_gnome::navigation::Heading>>,
    caret: RefCell<Option<(u64, u64, Pos)>>,
}
impl OutlinePanel {
    pub(super) fn show(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if let Some(search) = ui.search.borrow().as_ref() {
            search.toggle.set_active(false);
        }
        self.toggle.set_active(true);
        ui.sidebar_pages.set_visible_child_name("navigation");
        ui.navigation_pages.set_visible_child_name("outline");
        ui.sidebar_button.set_active(true);
        self.refresh();
    }
    pub(super) fn schedule(self: &Rc<Self>) {
        if !self
            .ui
            .upgrade()
            .is_some_and(|u| u.navigation_pages.visible_child_name().as_deref() == Some("outline"))
        {
            return;
        }
        if self.pending.replace(true) {
            return;
        }
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(160), move || {
            if let Some(s) = weak.upgrade() {
                s.pending.set(false);
                s.refresh();
            }
        });
    }
    pub(super) fn refresh(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        let (headings, bookmarks, sections, focus) = {
            let value = ui.current.borrow();
            let Some(d) = value.as_ref() else {
                return;
            };
            let headings = wordcraft_gnome::navigation::headings(&d.session.doc);
            let bookmarks = wordcraft_gnome::navigation::bookmarks(&d.session.doc, &headings);
            let mut sections = wordcraft_gnome::navigation::section_starts(&d.session.doc);
            sections.insert(
                0,
                (1, d.session.doc.start_of(wordcraft_doc::StoryRef::Body)),
            );
            (headings, bookmarks, sections, d.session.sel.focus.clone())
        };
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        self.status
            .set_visible(headings.is_empty() && bookmarks.is_empty() && sections.is_empty());
        self.positions.borrow_mut().clear();
        let mut hidden: Option<u8> = None;
        let current = wordcraft_gnome::navigation::containing_heading(&headings, &focus);
        self.headings.replace(headings.clone());
        let mut selected = None;
        for mark in bookmarks.iter().filter(|b| b.heading.is_none()) {
            self.append_bookmark(&ui, mark, 0);
        }
        for (i, h) in headings.iter().enumerate() {
            if let Some(level) = hidden {
                if h.level > level {
                    continue;
                }
                hidden = None;
            }
            let has_children = headings.get(i + 1).is_some_and(|n| n.level > h.level)
                || bookmarks.iter().any(|b| b.heading == Some(i));
            let key = format!("{:?}:{}", h.pos.path, h.text);
            let collapsed = self.collapsed.borrow().contains(&key);
            if collapsed && has_children {
                hidden = Some(h.level);
            }
            let row = gtk::ListBoxRow::new();
            row.set_widget_name(&format!("outline-heading-{i}"));
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 2);
            content.set_margin_start(6 + i32::from(h.level.min(8)) * 12);
            content.set_margin_end(8);
            content.set_margin_top(3);
            content.set_margin_bottom(3);
            let arrow = gtk::Button::with_label(if collapsed { "›" } else { "⌄" });
            arrow.add_css_class("flat");
            arrow.add_css_class("outline-expander");
            arrow.set_opacity(if has_children { 1.0 } else { 0.0 });
            arrow.set_can_target(has_children);
            arrow.set_focusable(has_children);
            arrow.set_tooltip_text(Some(if collapsed {
                wordcraft_gnome::t!("Expand Section")
            } else {
                wordcraft_gnome::t!("Collapse Section")
            }));
            accessible_name(
                &arrow,
                &format!(
                    "{} {}",
                    if collapsed {
                        wordcraft_gnome::t!("Expand")
                    } else {
                        wordcraft_gnome::t!("Collapse")
                    },
                    h.text
                ),
            );
            content.append(&arrow);
            let label = gtk::Label::new(Some(&h.text));
            label.set_xalign(0.0);
            label.set_hexpand(true);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(1);
            label.set_tooltip_text(Some(&h.text));
            if h.level == 0 {
                label.add_css_class("heading");
            }
            content.append(&label);
            row.set_child(Some(&content));
            self.list.append(&row);
            let weak = Rc::downgrade(self);
            arrow.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    let mut keys = s.collapsed.borrow_mut();
                    if !keys.remove(&key) {
                        keys.insert(key.clone());
                    }
                    drop(keys);
                    s.refresh();
                }
            });
            self.positions.borrow_mut().push(h.pos.clone());
            if current == Some(i) {
                selected = Some(row);
            }
            if !collapsed {
                for mark in bookmarks.iter().filter(|b| b.heading == Some(i)) {
                    self.append_bookmark(&ui, mark, h.level + 1);
                }
            }
        }
        for (number, pos) in sections {
            let index = self
                .positions
                .borrow()
                .iter()
                .position(|other| *other >= pos)
                .unwrap_or_else(|| self.positions.borrow().len());
            let label = wordcraft_gnome::i18n::interpolate(
                wordcraft_gnome::t!("Section {number}"),
                &[("number", number.to_string())],
            );
            let row = gtk::ListBoxRow::new();
            let text = gtk::Label::new(Some(&label));
            text.set_xalign(0.0);
            text.set_margin_start(12);
            text.set_margin_top(4);
            text.set_margin_bottom(4);
            text.add_css_class("heading");
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            text.set_hexpand(true);
            line.append(&text);
            let menu = gtk::MenuButton::new();
            menu.set_icon_name("view-more-symbolic");
            menu.add_css_class("flat");
            menu.set_valign(gtk::Align::Center);
            menu.set_margin_end(8);
            menu.set_margin_top(8);
            menu.set_margin_bottom(8);
            accessible_name(&menu, wordcraft_gnome::t!("Section Properties"));
            let actions = gio::SimpleActionGroup::new();
            let action = gio::SimpleAction::new("properties", None);
            let weak = Rc::downgrade(&ui);
            let identity = document_identity(&ui);
            action.connect_activate(move |_, _| {
                let Some(ui) = weak.upgrade().filter(|u| document_identity(u) == identity) else {
                    return;
                };
                let Some(host) = ui.host.upgrade() else {
                    return;
                };
                let valid = ui
                    .current
                    .borrow()
                    .as_ref()
                    .is_some_and(|doc| number <= doc.session.doc.sections().len());
                if valid {
                    super::document_properties_dialog::show_section_properties(
                        &ui,
                        &host.window,
                        number - 1,
                    );
                }
            });
            actions.add_action(&action);
            menu.insert_action_group("section", Some(&actions));
            let model = gio::Menu::new();
            model.append(
                Some(wordcraft_gnome::t!("Section Properties…")),
                Some("section.properties"),
            );
            menu.set_menu_model(Some(&model));
            line.append(&menu);
            row.set_child(Some(&line));
            self.list.insert(&row, index as i32);
            self.positions.borrow_mut().insert(index, pos);
        }
        if let Some(row) = selected {
            self.list.select_row(Some(&row));
        } else {
            self.list.unselect_all();
        }
        self.sync_caret();
    }
    pub(super) fn sync_caret(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if ui.sidebar_pages.visible_child_name().as_deref() != Some("navigation")
            || ui.navigation_pages.visible_child_name().as_deref() != Some("outline")
        {
            return;
        }
        let Some(key) = ui.current.borrow().as_ref().map(|d| {
            (
                d.session.document_id(),
                d.session.rev(),
                d.session.sel.focus.clone(),
            )
        }) else {
            return;
        };
        let moved = self.caret.borrow().as_ref() != Some(&key);
        self.caret.replace(Some(key.clone()));
        let headings = self.headings.borrow();
        let owner = wordcraft_gnome::navigation::containing_heading(&headings, &key.2);
        let Some(owner) = owner else {
            self.list.unselect_all();
            return;
        };
        let find = |i: usize| {
            find_widget(&self.list, &format!("outline-heading-{i}"))
                .and_then(|w| w.downcast::<gtk::ListBoxRow>().ok())
        };
        let mut row = find(owner);
        if row.is_none() && moved {
            let mut level = headings[owner].level;
            let mut collapsed = self.collapsed.borrow_mut();
            for h in headings[..owner].iter().rev() {
                if h.level < level {
                    collapsed.remove(&format!("{:?}:{}", h.pos.path, h.text));
                    level = h.level;
                }
            }
            drop(collapsed);
            drop(headings);
            self.refresh();
            return;
        }
        if row.is_none() {
            row = (0..owner)
                .rev()
                .filter(|i| headings[*i].level < headings[owner].level)
                .find_map(find);
        }
        drop(headings);
        if let Some(row) = row {
            self.list.select_row(Some(&row));
            let weak = row.downgrade();
            glib::idle_add_local_once(move || {
                let Some(row) = weak.upgrade() else { return };
                let Some(scroll) = row
                    .ancestor(gtk::ScrolledWindow::static_type())
                    .and_then(|w| w.downcast::<gtk::ScrolledWindow>().ok())
                else {
                    return;
                };
                let Some(bounds) = scroll.child().and_then(|c| row.compute_bounds(&c)) else {
                    return;
                };
                let adj = scroll.vadjustment();
                let top = bounds.y() as f64;
                let bottom = top + bounds.height() as f64;
                if top < adj.value() {
                    adj.set_value(top);
                } else if bottom > adj.value() + adj.page_size() {
                    adj.set_value(bottom - adj.page_size());
                }
            });
        } else {
            self.list.unselect_all();
        }
    }
    fn append_bookmark(
        &self,
        ui: &Rc<Ui>,
        mark: &wordcraft_gnome::navigation::Bookmark,
        level: u8,
    ) {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(&format!("outline-bookmark-{}", mark.name));
        // The nested buttons provide keyboard navigation and separate deletion.
        row.set_activatable(false);
        row.set_selectable(false);
        let content = bookmark_row(ui, &mark.name);
        content.set_margin_start(12 + i32::from(level.min(9)) * 12);
        content.set_margin_end(6);
        row.set_child(Some(&content));
        self.list.append(&row);
        self.positions.borrow_mut().push(mark.pos.clone());
    }
    pub(super) fn goto(&self, pos: &Pos) {
        let Some(ui) = self.ui.upgrade() else {
            return;
        };
        let result = {
            let mut value = ui.current.borrow_mut();
            value
                .as_mut()
                .map(|d| selection::set_caret(&mut d.session, pos, false))
        };
        if let Some(Err(e)) = result {
            ui.toasts.add_toast(Toast::new(&e));
            return;
        }
        if ui.sidebar.is_collapsed() {
            ui.sidebar.set_show_sidebar(false);
        }
        refresh_selection(&ui);
        ui.document_box.grab_focus();
        let weak = Rc::downgrade(&ui);
        let target = pos.clone();
        let identity = document_identity(&ui);
        glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
            let Some(ui) = weak
                .upgrade()
                .filter(|u| !u.closed.get() && document_identity(u) == identity)
            else {
                return;
            };
            let y = {
                let mut v = ui.current.borrow_mut();
                let d = v.as_mut().unwrap();
                if d.session.sel.focus != target {
                    return;
                }
                wordcraft_gnome::navigation::heading_view_target(&mut d.session, &target).and_then(
                    |(page, top)| {
                        let caret = d.session.layout().caret_on(&target, d.session.page_hint)?;
                        let max_context =
                            (ui.scroll.vadjustment().page_size() * 0.25 / d.zoom as f64) as f32;
                        let top = top.max(caret.top - max_context);
                        d.page_frames
                            .get(page)
                            .and_then(|f| f.compute_bounds(&ui.document_box))
                            .map(|r| r.y() as f64 + top as f64 * d.zoom as f64)
                    },
                )
            };
            if let Some(y) = y {
                let adj = ui.scroll.vadjustment();
                adj.set_value(y.clamp(
                    adj.lower(),
                    (adj.upper() - adj.page_size()).max(adj.lower()),
                ));
            }
        });
    }
}
pub(super) fn install_outline(
    ui: &Rc<Ui>,
    pages: &gtk::ToggleButton,
    titles: &gtk::ToggleButton,
) -> Rc<OutlinePanel> {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let status = gtk::Label::new(Some(wordcraft_gnome::t!("No Headings in This Document")));
    status.set_wrap(true);
    status.set_margin_start(12);
    status.set_margin_end(12);
    status.set_margin_top(18);
    status.add_css_class("dim-label");
    box_.append(&status);
    let list = gtk::ListBox::new();
    list.set_activate_on_single_click(true);
    list.add_css_class("navigation-sidebar");
    box_.append(&list);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_vexpand(true);
    scroll.set_child(Some(&box_));
    ui.navigation_pages.add_named(&scroll, Some("outline"));
    let s = Rc::new(OutlinePanel {
        ui: Rc::downgrade(ui),
        toggle: titles.clone(),
        list,
        status,
        collapsed: RefCell::new(Default::default()),
        positions: RefCell::new(Vec::new()),
        pending: Cell::new(false),
        headings: RefCell::new(Vec::new()),
        caret: RefCell::new(None),
    });
    let weak = Rc::downgrade(&s);
    s.list.connect_row_activated(move |_, row| {
        if let Some(s) = weak.upgrade() {
            let pos = s.positions.borrow().get(row.index() as usize).cloned();
            if let Some(pos) = pos {
                s.goto(&pos);
            }
        }
    });
    let weak = Rc::downgrade(&s);
    titles.connect_toggled(move |b| {
        if b.is_active() {
            if let Some(s) = weak.upgrade() {
                if let Some(ui) = s.ui.upgrade() {
                    ui.navigation_pages.set_visible_child_name("outline");
                }
                s.refresh();
            }
        }
    });
    let weak = Rc::downgrade(ui);
    pages.connect_toggled(move |b| {
        if b.is_active() {
            if let Some(ui) = weak.upgrade() {
                ui.navigation_pages.set_visible_child_name("thumbnails");
                refresh_thumbnails(&ui);
            }
        }
    });
    s
}
