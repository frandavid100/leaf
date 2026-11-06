use super::*;

// Shared native Navigation and Review sidebar.
pub(super) fn refresh_sidebar_title(ui: &Ui) {
    let label = if ui.sidebar_pages.visible_child_name().as_deref() == Some("search") {
        wordcraft_gnome::t!("Search")
    } else {
        if ui.navigation_pages.visible_child_name().as_deref() == Some("review") {
            wordcraft_gnome::t!("Review")
        } else {
            wordcraft_gnome::t!("Navigation")
        }
    };
    ui.sidebar_title.set_text(label);
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
    rendered_identity: Cell<Option<(u64, u64)>>,
    reveal_pending: Cell<bool>,
    filter: gtk::DropDown,
    elements: RefCell<Vec<wordcraft_gnome::navigation::Element>>,
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
        self.ensure_fresh();
    }
    pub(super) fn document_opened(self: &Rc<Self>) {
        let identity = self.ui.upgrade().and_then(|u| document_identity(&u));
        if self.rendered_identity.get().map(|(id, _)| id) != identity.map(|(id, _)| id) {
            self.collapsed.borrow_mut().clear();
            self.reveal_pending.set(true);
            if self.ui.upgrade().is_some_and(|u| {
                u.navigation_pages.visible_child_name().as_deref() == Some("outline")
            }) {
                self.ensure_fresh();
            }
        }
    }
    pub(super) fn ensure_fresh(self: &Rc<Self>) {
        let identity = self.ui.upgrade().and_then(|u| document_identity(&u));
        if self.rendered_identity.get() != identity {
            self.refresh();
        } else {
            self.sync_caret();
        }
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
                s.ensure_fresh();
            }
        });
    }
    pub(super) fn refresh(self: &Rc<Self>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        self.rendered_identity.set(document_identity(&ui));
        let (headings, mut bookmarks, mut sections, focus, elements) = {
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
            (
                headings,
                bookmarks,
                sections,
                d.session.sel.focus.clone(),
                wordcraft_gnome::navigation::elements(&d.session.doc),
            )
        };
        let section_of = |p: &Pos| {
            sections
                .iter()
                .rposition(|(_, start)| start <= p)
                .unwrap_or(0)
        };
        let owner_of = |p: &Pos| {
            headings
                .iter()
                .rposition(|h| h.pos <= *p && section_of(&h.pos) == section_of(p))
        };
        for mark in &mut bookmarks {
            mark.heading = owner_of(&mark.pos);
        }
        let filter = self.filter.selected();
        let previous_target = self
            .list
            .selected_row()
            .and_then(|row| self.positions.borrow().get(row.index() as usize).cloned());
        let active_target = {
            let current = ui.current.borrow();
            let d = current.as_ref().unwrap();
            elements
                .iter()
                .filter(|e| {
                    (filter == 0 || filter == e.kind)
                        && wordcraft_gnome::navigation::element_contains(&d.session, e, &focus)
                })
                .max_by_key(|e| if e.kind == 4 { 0 } else { 1 })
                .map(|e| e.pos.clone())
                .or_else(|| owner_of(&focus).map(|i| headings[i].pos.clone()))
                .or_else(|| sections.get(section_of(&focus)).map(|(_, p)| p.clone()))
        };
        if previous_target != active_target {
            self.reveal_pending.set(true);
        }
        let matches = if filter == 1 {
            headings.iter().map(|h| h.pos.clone()).collect::<Vec<_>>()
        } else if filter == 2 {
            bookmarks.iter().map(|m| m.pos.clone()).collect()
        } else {
            elements
                .iter()
                .filter(|e| filter == 0 || filter == e.kind)
                .map(|e| e.pos.clone())
                .collect()
        };
        let mut context = HashSet::new();
        for pos in &matches {
            if let Some(owner) = owner_of(pos) {
                let mut level = 10;
                for (i, h) in headings[..=owner].iter().enumerate().rev() {
                    if section_of(&h.pos) != section_of(pos) {
                        break;
                    }
                    if h.level < level {
                        context.insert(i);
                        level = h.level;
                    }
                }
            }
        }
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
        for mark in bookmarks
            .iter()
            .filter(|b| (filter == 0 || filter == 2) && b.heading.is_none())
        {
            self.append_bookmark(&ui, mark, 0);
        }
        let mut previous_section = None;
        for (i, h) in headings.iter().enumerate() {
            let section = section_of(&h.pos);
            if previous_section != Some(section) {
                hidden = None;
                previous_section = Some(section);
            }
            if filter > 1 && !context.contains(&i) {
                continue;
            }
            if let Some(level) = hidden {
                if h.level > level {
                    continue;
                }
                hidden = None;
            }
            let has_children = headings.get(i + 1).is_some_and(|n| n.level > h.level)
                || bookmarks.iter().any(|b| b.heading == Some(i))
                || elements.iter().any(|e| {
                    wordcraft_gnome::navigation::containing_heading(&headings, &e.pos) == Some(i)
                });
            let key = format!("{:?}:{}", h.pos.path, h.text);
            let collapsed = filter == 0 && self.collapsed.borrow().contains(&key);
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
                for mark in bookmarks
                    .iter()
                    .filter(|b| (filter == 0 || filter == 2) && b.heading == Some(i))
                {
                    self.append_bookmark(&ui, mark, h.level + 1);
                }
            }
        }
        let mut count = 0;
        for (n, element) in elements
            .iter()
            .enumerate()
            .filter(|(_, e)| filter == 0 || filter == e.kind)
        {
            let owner = owner_of(&element.pos);
            if filter == 0
                && owner.is_some_and(|i| {
                    let mut level = 10;
                    for h in headings[..=i].iter().rev() {
                        if section_of(&h.pos) != section_of(&element.pos) {
                            break;
                        }
                        if h.level < level {
                            level = h.level;
                            if self
                                .collapsed
                                .borrow()
                                .contains(&format!("{:?}:{}", h.pos.path, h.text))
                            {
                                return true;
                            }
                        }
                    }
                    false
                })
            {
                continue;
            }
            let index = self
                .positions
                .borrow()
                .iter()
                .position(|p| *p > element.pos)
                .unwrap_or_else(|| self.positions.borrow().len());
            let row = gtk::ListBoxRow::new();
            row.set_widget_name(&format!("outline-element-{n}"));
            let label = gtk::Label::new(Some(&element.label));
            label.set_xalign(0.);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(1);
            label.set_tooltip_text(Some(&element.label));
            label.set_margin_start(24 + owner.map_or(0, |i| i32::from(headings[i].level + 1) * 12));
            label.set_margin_top(8);
            label.set_margin_bottom(8);
            label.set_margin_end(8);
            row.set_child(Some(&label));
            self.list.insert(&row, index as i32);
            self.positions
                .borrow_mut()
                .insert(index, element.pos.clone());
            count += 1;
        }
        self.elements.replace(elements);
        if filter > 0 {
            self.status.set_text(wordcraft_gnome::t!(
                "No elements of this type in the document."
            ));
            self.status.set_visible(if filter < 3 {
                matches.is_empty()
            } else {
                count == 0
            });
        }
        let wanted_sections = matches.iter().map(section_of).collect::<HashSet<_>>();
        if filter > 0 {
            sections = sections
                .into_iter()
                .enumerate()
                .filter(|(i, _)| wanted_sections.contains(i))
                .map(|(_, v)| v)
                .collect();
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
            row.set_widget_name(&format!("outline-section-{number}"));
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
                Some(wordcraft_gnome::t!("Section Properties")),
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
        let element = {
            let current = ui.current.borrow();
            let d = current.as_ref().unwrap();
            self.elements
                .borrow()
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    wordcraft_gnome::navigation::element_contains(&d.session, e, &key.2)
                })
                .max_by_key(|(_, e)| if e.kind == 4 { 0 } else { 1 })
                .and_then(|(n, _)| {
                    find_widget(&self.list, &format!("outline-element-{n}"))
                        .and_then(|w| w.downcast::<gtk::ListBoxRow>().ok())
                })
        };
        if let Some(row) = element {
            self.select_outline_row(&row);
            return;
        }
        let headings = self.headings.borrow();
        let boundary = ui.current.borrow().as_ref().and_then(|d| {
            wordcraft_gnome::navigation::section_starts(&d.session.doc)
                .into_iter()
                .filter(|(_, p)| p <= &key.2)
                .last()
        });
        let owner =
            wordcraft_gnome::navigation::containing_heading(&headings, &key.2).filter(|i| {
                boundary
                    .as_ref()
                    .is_none_or(|(_, start)| headings[*i].pos >= *start)
            });
        let Some(owner) = owner else {
            let number = boundary.map_or(1, |(n, _)| n);
            if let Some(row) = find_widget(&self.list, &format!("outline-section-{number}"))
                .and_then(|w| w.downcast::<gtk::ListBoxRow>().ok())
            {
                self.select_outline_row(&row);
            } else {
                self.list.unselect_all();
            }
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
            self.select_outline_row(&row);
        } else {
            self.list.unselect_all();
        }
    }
    fn select_outline_row(&self, row: &gtk::ListBoxRow) {
        let changed = self.list.selected_row().as_ref() != Some(row);
        self.list.select_row(Some(row));
        if !changed && !self.reveal_pending.get() {
            return;
        }
        let visible = self.ui.upgrade().is_some_and(|u| {
            u.sidebar.shows_sidebar()
                && u.sidebar_pages.visible_child_name().as_deref() == Some("navigation")
                && u.navigation_pages.visible_child_name().as_deref() == Some("outline")
        });
        self.reveal_pending.set(!visible);
        if !visible {
            return;
        }
        let weak = row.downgrade();
        let weak_ui = self.ui.clone();
        let document = self
            .ui
            .upgrade()
            .and_then(|u| u.current.borrow().as_ref().map(|d| d.session.document_id()));
        let scroll_value = row
            .ancestor(gtk::ScrolledWindow::static_type())
            .and_then(|w| w.downcast::<gtk::ScrolledWindow>().ok())
            .map(|s| s.vadjustment().value());
        glib::idle_add_local_once(move || {
            let Some(row) = weak.upgrade() else { return };
            let Some(ui) = weak_ui.upgrade().filter(|u| !u.closed.get()) else {
                return;
            };
            if ui
                .current
                .borrow()
                .as_ref()
                .map(|d| d.session.document_id())
                != document
            {
                return;
            }
            if ui
                .outline
                .borrow()
                .as_ref()
                .and_then(|p| p.list.selected_row())
                .as_ref()
                != Some(&row)
            {
                return;
            }
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
            // A queued reveal must not override subsequent manual browsing or
            // operate on a hidden representation after a native allocation.
            if scroll_value.is_some_and(|v| (adj.value() - v).abs() > 1.) {
                return;
            }
            if !ui.sidebar.is_mapped()
                || !ui.sidebar.shows_sidebar()
                || ui.sidebar_pages.visible_child_name().as_deref() != Some("navigation")
                || ui.navigation_pages.visible_child_name().as_deref() != Some("outline")
            {
                if let Some(panel) = ui.outline.borrow().as_ref() {
                    panel.reveal_pending.set(true);
                }
                return;
            }
            let top = bounds.y() as f64;
            let bottom = top + bounds.height() as f64;
            if top < adj.value() {
                adj.set_value(top);
            } else if bottom > adj.value() + adj.page_size() {
                adj.set_value(bottom - adj.page_size());
            }
        });
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
            value.as_mut().map(|d| {
                if let Some(_e) = self
                    .elements
                    .borrow()
                    .iter()
                    .find(|e| e.pos == *pos && e.object)
                {
                    if let Some(wordcraft_doc::para::InlineObject::NoteRef { id, .. }) = d
                        .session
                        .doc
                        .para_at(pos)
                        .and_then(|p| p.object_at(pos.off))
                    {
                        let target = d.session.doc.start_of(wordcraft_doc::StoryRef::Part(*id));
                        return selection::set_caret(&mut d.session, &target, false);
                    }
                    let mut end = pos.clone();
                    end.off += wordcraft_doc::para::OBJ.len_utf8();
                    return selection::select_range(&mut d.session, pos, &end);
                }
                selection::set_caret(&mut d.session, pos, false)
            })
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
                        let max_context = (ui.scroll.vadjustment().page_size() * 0.25
                            / d.page_zoom(page) as f64)
                            as f32;
                        let top = top.max(caret.top - max_context);
                        d.page_frames
                            .get(page)
                            .and_then(|f| f.compute_bounds(&ui.document_box))
                            .map(|r| r.y() as f64 + top as f64 * d.page_zoom(page) as f64)
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
    let filter = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("All"),
        wordcraft_gnome::t!("Headings"),
        wordcraft_gnome::t!("Bookmarks"),
        wordcraft_gnome::t!("Images"),
        wordcraft_gnome::t!("Tables"),
        wordcraft_gnome::t!("Table of Contents"),
        wordcraft_gnome::t!("Captions"),
        wordcraft_gnome::t!("Footnotes"),
        wordcraft_gnome::t!("Endnotes"),
        wordcraft_gnome::t!("Equations"),
        wordcraft_gnome::t!("Links"),
        wordcraft_gnome::t!("Automatic Fields"),
    ]);
    filter.set_widget_name("navigation-filter");
    accessible_name(&filter, wordcraft_gnome::t!("Filter Navigation"));
    filter.set_margin_start(8);
    filter.set_margin_end(8);
    filter.set_margin_top(8);
    box_.append(&filter);
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
        rendered_identity: Cell::new(None),
        reveal_pending: Cell::new(true),
        filter,
        elements: RefCell::new(Vec::new()),
    });
    let weak = Rc::downgrade(&s);
    s.filter.connect_selected_notify(move |_| {
        if let Some(s) = weak.upgrade() {
            s.refresh();
        }
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
                s.ensure_fresh();
            }
        }
    });
    let weak = Rc::downgrade(ui);
    pages.connect_toggled(move |b| {
        if b.is_active() {
            if let Some(ui) = weak.upgrade() {
                ui.navigation_pages.set_visible_child_name("thumbnails");
                refresh_thumbnails(&ui);
                if ui.thumbnail_reveal_pending.get() {
                    reveal_active_thumbnail(&ui);
                }
            }
        }
    });
    s
}

pub(super) fn install_navigation_switching(ui: &Rc<Ui>) {
    let weak = Rc::downgrade(ui);
    ui.navigation_pages
        .connect_visible_child_name_notify(move |stack| {
            let Some(ui) = weak.upgrade() else { return };
            match stack.visible_child_name().as_deref() {
                Some("thumbnails") => {
                    ui.last_navigation.set("thumbnails");
                    ui.navigation_button.set_active(true);
                    ui.navigation_selector.set_visible(true);
                }
                Some("outline") => {
                    ui.last_navigation.set("outline");
                    ui.navigation_button.set_active(true);
                    ui.navigation_selector.set_visible(true);
                }
                Some("review") => ui.navigation_selector.set_visible(false),
                _ => {}
            }
            refresh_sidebar_title(&ui);
        });
    let weak = Rc::downgrade(ui);
    ui.navigation_button.connect_toggled(move |b| {
        if b.is_active() {
            if let Some(ui) = weak.upgrade() {
                show_navigation(&ui);
            }
        }
    });
    let weak = Rc::downgrade(ui);
    ui.navigation_button.connect_clicked(move |_| {
        if let Some(ui) = weak.upgrade() {
            show_navigation(&ui);
        }
    });
    let weak = Rc::downgrade(ui);
    ui.sidebar.connect_show_sidebar_notify(move |s| {
        if s.shows_sidebar() {
            if let Some(ui) = weak.upgrade() {
                sync_navigation_representation(&ui);
            }
        }
    });
}
fn show_navigation(ui: &Rc<Ui>) {
    if let Some(search) = ui.search.borrow().as_ref() {
        search.toggle.set_active(false);
    }
    ui.sidebar_pages.set_visible_child_name("navigation");
    ui.navigation_pages
        .set_visible_child_name(ui.last_navigation.get());
    sync_navigation_representation(ui);
}
fn sync_navigation_representation(ui: &Rc<Ui>) {
    if ui.navigation_pages.visible_child_name().as_deref() == Some("outline") {
        if let Some(outline) = ui.outline.borrow().as_ref() {
            outline.ensure_fresh();
        }
    } else if ui.navigation_pages.visible_child_name().as_deref() == Some("thumbnails") {
        refresh_thumbnails(ui);
        if ui.thumbnail_reveal_pending.get() {
            reveal_active_thumbnail(ui);
        }
    }
}
