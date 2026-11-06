use super::*;
use wordcraft_gnome::review as revisions;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReviewKind {
    Correction,
    Change,
    Comment,
}
pub(super) fn review_type_icon(kind: ReviewKind) -> gtk::Image {
    let (name, label) = match kind {
        ReviewKind::Correction => (
            "tools-check-spelling-symbolic",
            wordcraft_gnome::t!("Correction"),
        ),
        ReviewKind::Change => (
            "document-edit-symbolic",
            wordcraft_gnome::t!("Tracked Change"),
        ),
        ReviewKind::Comment => ("comment-symbolic", wordcraft_gnome::t!("Comment")),
    };
    let icon = gtk::Image::from_icon_name(name);
    icon.set_pixel_size(16);
    icon.set_valign(gtk::Align::Start);
    icon.set_tooltip_text(Some(label));
    accessible_name(&icon, label);
    icon
}
pub(super) struct ReviewPanel {
    ui: std::rc::Weak<Ui>,
    toggle: gtk::ToggleButton,
    pub(super) filter: gtk::DropDown,
    pub(super) content: gtk::Box,
    status: gtk::Label,
    scroll: gtk::ScrolledWindow,
    pub(super) rebuilding: Cell<bool>,
    pending: Cell<bool>,
    document: Cell<Option<u64>>,
    changes: RefCell<Vec<(revisions::Change, gtk::Widget)>>,
}
impl ReviewPanel {
    fn includes(&self, kind: ReviewKind) -> bool {
        self.filter.selected() == 0
            || self.filter.selected()
                == match kind {
                    ReviewKind::Correction => 1,
                    ReviewKind::Change => 2,
                    ReviewKind::Comment => 3,
                }
    }
    pub(super) fn show(self: &Rc<Self>, kind: Option<ReviewKind>) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        if let Some(kind) = kind.filter(|k| !self.includes(*k)) {
            self.filter.set_selected(match kind {
                ReviewKind::Correction => 1,
                ReviewKind::Change => 2,
                ReviewKind::Comment => 3,
            });
        }
        if !self.toggle.is_active() {
            self.toggle.set_active(true);
            return;
        }
        if let Some(search) = ui.search.borrow().as_ref() {
            search.toggle.set_active(false);
        }
        ui.sidebar_pages.set_visible_child_name("navigation");
        ui.navigation_pages.set_visible_child_name("review");
        ui.sidebar_button.set_active(true);
        if let Some(spell) = ui.spelling.borrow().as_ref() {
            if spell.checked.get() != document_identity(&ui)
                && spell.source.borrow().is_none()
                && spell.debounce.borrow().is_none()
            {
                spell.scan();
            }
        }
        self.refresh();
    }
    pub(super) fn schedule(self: &Rc<Self>) {
        if self.rebuilding.get() || !self.toggle.is_active() || self.pending.replace(true) {
            return;
        }
        let weak = Rc::downgrade(self);
        glib::idle_add_local_once(move || {
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
        if !self.toggle.is_active() || self.rebuilding.replace(true) {
            return;
        }
        let id = ui
            .current
            .borrow()
            .as_ref()
            .map(|d| d.session.document_id());
        let old_y = if self.document.replace(id) == id {
            self.scroll.vadjustment().value()
        } else {
            self.changes.borrow_mut().clear();
            0.
        };
        let Some(annotations) = ui.annotations.borrow().as_ref().cloned() else {
            self.rebuilding.set(false);
            return;
        };
        if self.includes(ReviewKind::Comment) {
            annotations.refresh();
        }
        let (comments, changes) = {
            let mut c = ui.current.borrow_mut();
            if let Some(d) = c.as_mut() {
                (
                    wordcraft_gnome::annotations::comments(&d.session),
                    revisions::changes(&mut d.session).unwrap_or_default(),
                )
            } else {
                (Vec::new(), Vec::new())
            }
        };
        let mut rows: Vec<(Pos, ReviewKind, gtk::Widget)> = Vec::new();
        let mut child = self.content.first_child();
        while let Some(w) = child {
            child = w.next_sibling();
            let name = w.widget_name();
            let comment = name
                .strip_prefix("comment-thread-")
                .and_then(|id| id.parse::<u32>().ok())
                .and_then(|id| comments.iter().find(|c| c.id == id));
            let is_draft = name == "comment-composer";
            if self.includes(ReviewKind::Comment) && (comment.is_some() || is_draft) {
                let pos = comment
                    .and_then(|c| c.anchor.clone())
                    .or_else(|| {
                        ui.current
                            .borrow()
                            .as_ref()
                            .map(|d| d.session.sel.focus.clone())
                    })
                    .unwrap_or_else(|| Pos::body(0, 0));
                rows.push((pos, ReviewKind::Comment, w));
            } else if !name.starts_with("review-correction-") && !name.starts_with("review-change-")
            {
                self.content.remove(&w);
            }
        }
        if self.includes(ReviewKind::Correction) {
            if let Some(spell) = ui.spelling.borrow().as_ref() {
                spell.render_list();
                for (n, (pos, card)) in spell.review_cards().into_iter().enumerate() {
                    if let Some(row) = card
                        .parent()
                        .and_then(|p| p.downcast::<gtk::ListBoxRow>().ok())
                    {
                        row.set_child(None::<&gtk::Widget>);
                    }
                    card.set_widget_name(&format!("review-correction-{n}"));
                    rows.push((pos, ReviewKind::Correction, card));
                }
            }
        }
        if self.includes(ReviewKind::Change) {
            let identity = document_identity(&ui);
            let previous = self.changes.take();
            let mut cache = Vec::new();
            for (n, change) in changes.into_iter().enumerate() {
                let card = previous
                    .iter()
                    .find(|(c, _)| c == &change)
                    .map(|(_, w)| w.clone())
                    .unwrap_or_else(|| revision_card(&ui, &change, n, identity).upcast());
                rows.push((change.start.clone(), ReviewKind::Change, card.clone()));
                cache.push((change, card));
            }
            self.changes.replace(cache);
        } else {
            self.changes.borrow_mut().clear();
        }

        let mut removed_focus = None;
        let mut old_index = 0usize;
        let mut child = self.content.first_child();
        while let Some(w) = child {
            child = w.next_sibling();
            if (w.widget_name().starts_with("review-correction-")
                || w.widget_name().starts_with("review-change-"))
                && !rows.iter().any(|(_, _, card)| card == &w)
            {
                if control_has_focus(&w) {
                    removed_focus = Some(old_index);
                }
                self.content.remove(&w);
            }
            old_index += 1;
        }
        rows.sort_by_cached_key(|(at, kind, _)| {
            let mut c = ui.current.borrow_mut();
            let page = c
                .as_mut()
                .and_then(|d| d.session.layout().page_of(at))
                .unwrap_or(usize::MAX);
            let rank = match at.story {
                wordcraft_doc::StoryRef::Body => 1u8,
                wordcraft_doc::StoryRef::Part(id) => c
                    .as_ref()
                    .and_then(|d| d.session.doc.parts.get(&id))
                    .map_or(2, |p| match p.kind {
                        wordcraft_doc::PartKind::Header => 0,
                        wordcraft_doc::PartKind::Footer => 3,
                        _ => 2,
                    }),
            };
            (
                page,
                rank,
                at.path.0.clone(),
                at.off,
                match kind {
                    ReviewKind::Correction => 0u8,
                    ReviewKind::Change => 1,
                    ReviewKind::Comment => 2,
                },
            )
        });
        let mut previous: Option<gtk::Widget> = None;
        for (_, _, widget) in &rows {
            if widget.parent().is_none() {
                self.content.append(widget);
            }
            self.content.reorder_child_after(widget, previous.as_ref());
            previous = Some(widget.clone());
        }
        if let Some(index) = removed_focus {
            if let Some((_, _, card)) = rows.get(index.min(rows.len().saturating_sub(1))) {
                card.child_focus(gtk::DirectionType::TabForward);
            } else {
                self.filter.grab_focus();
            }
        }
        self.status.set_visible(rows.is_empty());
        let checking = ui
            .spelling
            .borrow()
            .as_ref()
            .is_some_and(|s| s.checked.get() != document_identity(&ui));
        self.status
            .set_text(if checking && self.includes(ReviewKind::Correction) {
                wordcraft_gnome::t!("Checking…")
            } else {
                match self.filter.selected() {
                    1 => wordcraft_gnome::t!(
                        "No corrections found in text with available proofing tools."
                    ),
                    2 => wordcraft_gnome::t!("No Tracked Changes"),
                    3 => wordcraft_gnome::t!("No Comments"),
                    _ => wordcraft_gnome::t!("No Review Items"),
                }
            });
        accessible_name(&self.content, wordcraft_gnome::t!("Review"));
        self.rebuilding.set(false);
        let weak = Rc::downgrade(self);
        glib::idle_add_local_once(move || {
            if let Some(s) = weak.upgrade() {
                let a = s.scroll.vadjustment();
                a.set_value(old_y.clamp(a.lower(), (a.upper() - a.page_size()).max(a.lower())));
            }
        });
    }
}
fn revision_card(
    ui: &Rc<Ui>,
    change: &revisions::Change,
    n: usize,
    identity: Option<(u64, u64)>,
) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 5);
    card.set_widget_name(&format!("review-change-{n}"));
    card.add_css_class("comment-thread");
    card.set_overflow(gtk::Overflow::Hidden);
    card.set_margin_top(2);
    card.set_margin_bottom(2);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 5);
    body.set_margin_top(8);
    body.set_margin_bottom(8);
    body.set_margin_start(6);
    body.set_margin_end(6);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    header.append(&review_type_icon(ReviewKind::Change));
    let kind = match change.kind.as_str() {
        "insert" => wordcraft_gnome::t!("Insertion"),
        "delete" => wordcraft_gnome::t!("Deletion"),
        "moveFrom" => wordcraft_gnome::t!("Moved From"),
        "moveTo" => wordcraft_gnome::t!("Moved To"),
        _ => wordcraft_gnome::t!("Formatting"),
    };
    let title = gtk::Label::new(Some(&wordcraft_gnome::i18n::interpolate(
        wordcraft_gnome::t!("{kind} — {author}"),
        &[
            ("kind", kind.into()),
            (
                "author",
                if change.author.is_empty() {
                    wordcraft_gnome::t!("Unknown Author").into()
                } else {
                    change.author.clone()
                },
            ),
        ],
    )));
    title.set_wrap(true);
    title.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    title.set_xalign(0.);
    title.set_max_width_chars(1);
    title.set_hexpand(true);
    title.add_css_class("heading");
    header.append(&title);
    body.append(&header);
    let go = gtk::Button::new();
    go.add_css_class("flat");
    go.set_widget_name(&format!("review-change-go-{n}"));
    let text = if change.text.is_empty() {
        if change.description.is_empty() {
            kind
        } else {
            &change.description
        }
    } else {
        &change.text
    };
    let passage = gtk::Label::new(Some(text));
    passage.set_wrap(true);
    passage.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    passage.set_xalign(0.);
    passage.set_max_width_chars(1);
    passage.set_lines(3);
    passage.set_ellipsize(gtk::pango::EllipsizeMode::End);
    go.set_child(Some(&passage));
    accessible_name(&go, wordcraft_gnome::t!("Go to Change"));
    let weak = Rc::downgrade(ui);
    let target = change.clone();
    go.connect_clicked(move |_| {
        let Some(ui) = weak.upgrade().filter(|u| {
            !u.closed.get() && document_identity(u).map(|id| id.0) == identity.map(|id| id.0)
        }) else {
            return;
        };
        let result = ui
            .current
            .borrow_mut()
            .as_mut()
            .map(|d| revisions::select(&mut d.session, &target));
        if let Some(Err(e)) = result {
            ui.toasts.add_toast(Toast::new(&e));
            return;
        }
        refresh_selection(&ui);
        caret_ensure_visible(&ui);
        ui.document_box.grab_focus();
    });
    body.append(&go);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    for (accept, label) in [
        (true, wordcraft_gnome::t!("Accept")),
        (false, wordcraft_gnome::t!("Reject")),
    ] {
        let button = gtk::Button::with_label(label);
        button.set_hexpand(true);
        button.set_widget_name(&format!(
            "review-{}-{n}",
            if accept { "accept" } else { "reject" }
        ));
        button.set_sensitive(
            ui.current
                .borrow()
                .as_ref()
                .is_some_and(|d| !wordcraft_engine::cmd::protect::enforced(&d.session.doc)),
        );
        let weak = Rc::downgrade(ui);
        let target = change.clone();
        button.connect_clicked(move |_| {
            let Some(ui) = weak.upgrade().filter(|u| {
                !u.closed.get() && document_identity(u).map(|id| id.0) == identity.map(|id| id.0)
            }) else {
                return;
            };
            let result = ui
                .current
                .borrow_mut()
                .as_mut()
                .map(|d| revisions::resolve(&mut d.session, &target, accept));
            match result {
                Some(Ok(())) => after_content_change(&ui),
                Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
                _ => {}
            }
            if let Some(review) = ui.review.borrow().as_ref() {
                review.schedule();
            }
        });
        actions.append(&button);
    }
    body.append(&actions);
    card.append(&body);
    card
}
pub(super) fn install_review(ui: &Rc<Ui>, toggle: &gtk::ToggleButton) -> Rc<ReviewPanel> {
    let annotations = ui.annotations.borrow().as_ref().unwrap().clone();
    ui.navigation_pages.remove(&annotations.scroll);
    if let Some(old) = ui.navigation_pages.child_by_name("proofing") {
        ui.navigation_pages.remove(&old);
    }
    let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    body.set_widget_name("review-panel");
    let filter = gtk::DropDown::from_strings(&[
        wordcraft_gnome::t!("All"),
        wordcraft_gnome::t!("Corrections"),
        wordcraft_gnome::t!("Changes"),
        wordcraft_gnome::t!("Comments"),
    ]);
    filter.set_widget_name("review-filter");
    filter.set_margin_start(6);
    filter.set_margin_end(6);
    filter.set_margin_top(6);
    filter.set_margin_bottom(6);
    accessible_name(&filter, wordcraft_gnome::t!("Review Filter"));
    ui.sidebar_filters.add_named(&filter, Some("review"));
    let status = gtk::Label::new(None);
    status.set_widget_name("review-status");
    status.set_wrap(true);
    status.set_margin_start(8);
    status.set_margin_end(8);
    body.append(&status);
    annotations.scroll.set_vexpand(true);
    body.append(&annotations.scroll);
    ui.navigation_pages.add_named(&body, Some("review"));
    let s = Rc::new(ReviewPanel {
        ui: Rc::downgrade(ui),
        toggle: toggle.clone(),
        filter,
        content: annotations.content.clone(),
        status,
        scroll: annotations.scroll.clone(),
        rebuilding: Cell::new(false),
        pending: Cell::new(false),
        document: Cell::new(None),
        changes: RefCell::new(Vec::new()),
    });
    let weak = Rc::downgrade(&s);
    toggle.connect_toggled(move |b| {
        if b.is_active() {
            if let Some(s) = weak.upgrade() {
                s.show(None);
            }
        }
    });
    let weak = Rc::downgrade(&s);
    s.filter.connect_selected_notify(move |_| {
        if let Some(s) = weak.upgrade() {
            s.scroll.vadjustment().set_value(0.);
            s.schedule();
        }
    });
    s
}
