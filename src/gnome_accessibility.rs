// Native controls expose their purpose independently of tooltip timing/icons.
fn accessible_name(widget: &impl IsA<gtk::Accessible>, name: &str) {
    widget.update_property(&[gtk::accessible::Property::Label(name)]);
}
fn contains_focus(container: &impl IsA<gtk::Widget>, focus: &gtk::Widget) -> bool {
    focus == container.as_ref() || focus.is_ancestor(container)
}
fn focus_zone(ui: &Rc<Ui>, backwards: bool) {
    let Some(host) = ui.host.upgrade() else { return; };
    let focus = gtk::prelude::GtkWindowExt::focus(&host.window);
    let zone = focus.as_ref().map(|f| {
        if ui.format_buttons.borrow().iter().any(|m| contains_focus(m, f)) || contains_focus(&ui.undo_button,f) || contains_focus(&ui.redo_button,f) { 2 }
        else if ui.sidebar.sidebar().is_some_and(|side| contains_focus(&side,f)) { 1 }
        else { 0 }
    }).unwrap_or(0);
    match (zone + if backwards { 2 } else { 1 }) % 3 {
        0 => { ui.document_box.grab_focus(); },
        1 => {
            ui.sidebar_button.set_active(true);
            let weak=Rc::downgrade(ui);
            glib::idle_add_local_once(move || {
                let Some(ui)=weak.upgrade().filter(|u|!u.closed.get())else{return;};
                if ui.search.borrow().as_ref().is_some_and(|s| s.toggle.is_active()) {
                    if let Some(s) = ui.search.borrow().as_ref() { s.query.grab_focus(); }
                } else if let Some(side) = ui.sidebar.sidebar() { side.child_focus(gtk::DirectionType::TabForward); }
            });
        },
        _ => { if let Some(button) = ui.format_buttons.borrow().first() { button.grab_focus(); } }
    }
}
fn install_accessibility(ui: &Rc<Ui>, main: &gtk::MenuButton, compact: &gtk::MenuButton) {
    ui.document_box.bind(ui);
    accessible_name(&ui.document_box,wordcraft_gnome::t!("Document Content"));
    ui.document_box.update_property(&[gtk::accessible::Property::MultiLine(true),gtk::accessible::Property::ReadOnly(false)]);
    ui.document_box.update_property(&[gtk::accessible::Property::Description(wordcraft_gnome::t!("F6 switches between the document, sidebar and formatting controls. Tab moves between table cells. Shift+F10 opens the context menu."))]);
    for (w,name) in [(ui.sidebar_button.clone().upcast::<gtk::Widget>(),wordcraft_gnome::t!("Show or Hide Sidebar")),
        (ui.zoom_in.clone().upcast(),wordcraft_gnome::t!("Zoom In")),(ui.zoom_out.clone().upcast(),wordcraft_gnome::t!("Zoom Out")),
        (ui.font_button.clone().upcast(),wordcraft_gnome::t!("Font Family")),(ui.size_input.clone().upcast(),wordcraft_gnome::t!("Font Size in Points")),
        (main.clone().upcast(),wordcraft_gnome::t!("Main Menu")),(compact.clone().upcast(),wordcraft_gnome::t!("Main Menu"))] { accessible_name(&w,name); }
    for (w,name) in [(&ui.format_bold,wordcraft_gnome::t!("Bold")),(&ui.format_italic,wordcraft_gnome::t!("Italic")),(&ui.format_underline,wordcraft_gnome::t!("Underline"))] { accessible_name(w,name); }
    let keys=gtk::EventControllerKey::new();keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    let weak=Rc::downgrade(ui);let main=main.downgrade();let compact=compact.downgrade();
    keys.connect_key_pressed(move |_,key,_,mods| {
        if mods.intersects(gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK | gdk::ModifierType::SUPER_MASK) { return glib::Propagation::Proceed; }
        let Some(ui)=weak.upgrade() else { return glib::Propagation::Proceed; };
        let shift=mods.contains(gdk::ModifierType::SHIFT_MASK);
        if key==gdk::Key::F6 { focus_zone(&ui,shift); return glib::Propagation::Stop; }
        if key==gdk::Key::F10 && !shift {
            if let Some(menu)=main.upgrade().filter(|m|m.is_mapped()).or_else(||compact.upgrade()) { menu.popup(); }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    ui.sidebar.add_controller(keys);
}

// Staged assertions driven by real xdotool keys in CI, not direct callbacks.
fn keyboard_ui_test(ui:&Rc<Ui>, parent:&ApplicationWindow, main:&gtk::MenuButton, app:&Application) {
    let ui=ui.clone();let parent=parent.clone();let main=main.clone();let app=app.clone();
    opened(&ui, Session::new(wordcraft_doc::Document::from_text("Keyboard test")), "Teclado");
    let document=ui.current.borrow().as_ref().unwrap().session.doc.clone();
    ui.document_box.grab_focus(); eprintln!("KEYBOARD_READY 0");
    let stage=Rc::new(Cell::new(0));let ticks=Rc::new(Cell::new(0));
    glib::timeout_add_local(std::time::Duration::from_millis(150),move || {
        ticks.set(ticks.get()+1);assert!(ticks.get()<250,"keyboard test timed out");
        let marker=format!("keyboard-stage-{}",stage.get());
        if !Path::new(&marker).exists(){return glib::ControlFlow::Continue;}
        std::fs::remove_file(marker).unwrap();
        match stage.get() {
            0=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.sidebar.sidebar().is_some_and(|s|contains_focus(&s,&f)));},
            1=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.format_buttons.borrow().iter().any(|m|contains_focus(m,&f)));},
            2=>{assert!(ui.document_box.has_focus());},
            3=>{assert!(main.popover().unwrap().is_visible());},
            4=>{assert!(!main.popover().unwrap().is_visible());show_search(&ui,&parent,false);},
            5=>{assert!(ui.document_box.has_focus());assert!(!ui.search.borrow().as_ref().unwrap().toggle.is_active());},
            6=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.format_buttons.borrow().iter().any(|m|contains_focus(m,&f)));},
            7=>{let f=gtk::prelude::GtkWindowExt::focus(&parent).unwrap();assert!(ui.sidebar.sidebar().is_some_and(|side|contains_focus(&side,&f)));},
            8=>{assert!(ui.document_box.has_focus());},
            _=>{assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc,document);eprintln!("KEYBOARD_UI_OK");app.quit();return glib::ControlFlow::Break;}
        }
        stage.set(stage.get()+1);eprintln!("KEYBOARD_READY {}",stage.get());glib::ControlFlow::Continue
    });
}
