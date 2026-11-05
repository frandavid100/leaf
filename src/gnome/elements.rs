use super::*;

// Temporarily borrow the same format bodies used by context actions. The
// original popovers still own their callbacks; close restores every body.
pub(super) struct ElementsPanel {
    ui: std::rc::Weak<Ui>,
    pub(super) menu: gtk::MenuButton,
    pub(super) pop: gtk::Popover,
    stack: gtk::Stack,
    pub(super) header: Rc<FormatHeader>,
    borrowed: RefCell<Vec<(gtk::Popover, gtk::Widget, Rc<FormatHeader>)>>,
    toc: gtk::Box,
}
impl ElementsPanel {
    pub(super) fn body_for(&self, origin: &gtk::Popover) -> Option<gtk::Widget> {
        self.borrowed
            .borrow()
            .iter()
            .find(|(p, _, _)| p == origin)
            .map(|(_, w, _)| w.clone())
    }
    pub(super) fn restore(&self) {
        for (origin, body, header) in self.borrowed.borrow_mut().drain(..) {
            self.stack.remove(&body);
            origin.set_child(Some(&body));
            header.parent_back.borrow_mut().take();
            header.home();
            EMBEDDED_FORMAT.with(|items| {
                items
                    .borrow_mut()
                    .retain(|(p, _, _)| p.upgrade().as_ref() != Some(&origin))
            });
        }
    }
    fn home(&self) {
        self.header.home();
        self.stack.set_visible_child_name("home");
    }
    pub(super) fn show(self: &Rc<Self>, page: &str) {
        let Some(ui) = self.ui.upgrade().filter(|u| !u.closed.get()) else {
            return;
        };
        let origin = match page {
            "table" => ui.table_popover.borrow().clone(),
            "image" => ui.picture.borrow().as_ref().map(|p| p.pop.clone()),
            _ => None,
        };
        if let Some(origin) = origin {
            if page == "table" {
                refresh_table_controls(&ui, &origin);
            }
            if let Some(picture) = ui.picture.borrow().as_ref().filter(|_| page == "image") {
                picture.refresh();
            }
            if self.body_for(&origin).is_none() {
                let Some(body) = origin.child() else { return };
                let Some(header) = ui
                    .format_group
                    .entries
                    .borrow()
                    .iter()
                    .find(|(p, _)| p.upgrade().as_ref() == Some(&origin))
                    .map(|(_, h)| h.clone())
                else {
                    return;
                };
                origin.popdown();
                origin.set_child(None::<&gtk::Widget>);
                let weak = Rc::downgrade(self);
                header.parent_back.replace(Some(Rc::new(move || {
                    if let Some(s) = weak.upgrade() {
                        s.home();
                    }
                })));
                header.home();
                self.stack.add_named(&body, Some(page));
                EMBEDDED_FORMAT.with(|items| {
                    items.borrow_mut().push((
                        origin.downgrade(),
                        self.pop.downgrade(),
                        self.header.pin.downgrade(),
                    ))
                });
                self.borrowed.borrow_mut().push((origin, body, header));
            }
        } else if page == "toc" {
            self.toc_page(&ui);
        }
        self.stack.set_visible_child_name(page);
        let weak = Rc::downgrade(self);
        self.header.back(wordcraft_gnome::t!("Elements"), move || {
            if let Some(s) = weak.upgrade() {
                s.home();
            }
        });
        self.menu.popup();
    }
    fn toc_page(self: &Rc<Self>, ui: &Rc<Ui>) {
        clear_box(&self.toc);
        let insert = style_menu_button(wordcraft_gnome::t!("Insert Table of Contents…"));
        insert.set_widget_name("elements-insert-toc");
        self.toc.append(&insert);
        let weak = Rc::downgrade(ui);
        let pop = self.pop.downgrade();
        insert.connect_clicked(move |_| {
            if let Some(ui) = weak.upgrade() {
                if let Some(p) = pop.upgrade() {
                    close_format_popover(&p);
                }
                if let Some(host) = ui.host.upgrade() {
                    show_insert_contents_dialog(&ui, &host.window);
                }
            }
        });
        let existing = ui.current.borrow().as_ref().and_then(|d| {
            wordcraft_engine::cmd::references::caret_in_toc(&d.session)
                .then(|| wordcraft_gnome::references::toc_field(&d.session))
                .flatten()
        });
        let Some(existing) = existing else { return };
        let label = gtk::Label::new(Some(wordcraft_gnome::t!("Current Table of Contents")));
        label.set_xalign(0.);
        label.add_css_class("heading");
        self.toc.append(&label);
        let controls = gtk::Box::new(gtk::Orientation::Vertical, 6);
        self.toc.append(&controls);
        let apply = style_menu_button(wordcraft_gnome::t!("Apply"));
        apply.set_widget_name("elements-toc-apply");
        let action = contents_controls(ui, &controls, Some(existing));
        self.toc.append(&apply);
        let identity = document_identity(ui);
        let selection = ui.current.borrow().as_ref().map(|d| d.session.sel.clone());
        let weak = Rc::downgrade(ui);
        let panel = Rc::downgrade(self);
        apply.connect_clicked(move |_| {
            let Some(ui) = weak.upgrade().filter(|u| !u.closed.get()) else {
                return;
            };
            if document_identity(&ui) != identity
                || ui.current.borrow().as_ref().map(|d| d.session.sel.clone()) != selection
            {
                ui.toasts.add_toast(Toast::new(wordcraft_gnome::t!(
                    "The document has changed; open this dialog again."
                )));
                return;
            }
            let result = ui
                .current
                .borrow_mut()
                .as_mut()
                .map(|d| action(&mut d.session));
            match result {
                Some(Ok(())) => {
                    after_content_change(&ui);
                    if let Some(panel) = panel.upgrade() {
                        close_format_popover(&panel.pop);
                        panel.toc_page(&ui);
                    }
                }
                Some(Err(e)) => ui.toasts.add_toast(Toast::new(&e)),
                None => {}
            }
        });
    }
}
pub(super) fn install_elements(ui: &Rc<Ui>, bar: &gtk::Box) -> Rc<ElementsPanel> {
    let menu = gtk::MenuButton::new();
    menu.set_widget_name("elements-menu");
    menu.set_icon_name("list-add-symbolic");
    menu.add_css_class("flat");
    menu.set_tooltip_text(Some(wordcraft_gnome::t!("Elements")));
    accessible_name(&menu, wordcraft_gnome::t!("Elements"));
    bar.append(&menu);
    let pop = gtk::Popover::new();
    pop.add_css_class("format-popover");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    for f in [
        gtk::Widget::set_margin_top,
        gtk::Widget::set_margin_bottom,
        gtk::Widget::set_margin_start,
        gtk::Widget::set_margin_end,
    ] {
        f(root.upcast_ref(), 8);
    }
    root.set_width_request(290);
    let header = format_popover_header(
        &pop,
        &root,
        wordcraft_gnome::t!("Elements"),
        &ui.format_group,
    );
    let stack = gtk::Stack::new();
    stack.set_vhomogeneous(false);
    stack.set_hhomogeneous(false);
    let home = gtk::Box::new(gtk::Orientation::Vertical, 2);
    elements_home(&home, &elements_menu().upcast::<gio::MenuModel>());
    stack.add_named(&home, Some("home"));
    let toc = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_max_content_height(480);
    scroll.set_propagate_natural_height(true);
    scroll.set_child(Some(&toc));
    stack.add_named(&scroll, Some("toc"));
    root.append(&stack);
    pop.set_child(Some(&root));
    menu.set_popover(Some(&pop));
    let s = Rc::new(ElementsPanel {
        ui: Rc::downgrade(ui),
        menu,
        pop,
        stack,
        header,
        borrowed: RefCell::new(Vec::new()),
        toc,
    });
    s.home();
    let weak = Rc::downgrade(&s);
    s.pop.connect_closed(move |_| {
        if let Some(s) = weak.upgrade() {
            s.restore();
            s.home();
        }
    });
    s
}

fn elements_home(root: &gtk::Box, model: &gio::MenuModel) {
    for n in 0..model.n_items() {
        if let Some(section) = model.item_link(n, "section") {
            if root.first_child().is_some() {
                root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
            }
            elements_home(root, &section);
            continue;
        }
        let label = model
            .item_attribute_value(n, "label", None)
            .and_then(|v| v.str().map(str::to_owned))
            .unwrap_or_default();
        if let Some(submenu) = model.item_link(n, "submenu") {
            let button = gtk::MenuButton::new();
            button.add_css_class("flat");
            button.set_label(&label);
            button.set_menu_model(Some(&submenu));
            accessible_name(&button, &label);
            root.append(&button);
        } else {
            let button = style_menu_button(&label);
            if let Some(action) = model
                .item_attribute_value(n, "action", None)
                .and_then(|v| v.str().map(str::to_owned))
            {
                button.set_action_name(Some(&action));
            }
            root.append(&button);
        }
    }
}
