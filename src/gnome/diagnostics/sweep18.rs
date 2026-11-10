use super::*;
use serde_json::json;

fn select_paragraph(ui: &Rc<Ui>, n: usize) {
    {
        let mut c = ui.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        let end = s.doc.para_at(&Pos::body(n, 0)).unwrap().len();
        selection::select_range(s, &Pos::body(n, 0), &Pos::body(n, end)).unwrap();
    }
    refresh_selection(ui);
}
fn button(root: &impl IsA<gtk::Widget>, name: &str) {
    find_widget(root, name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
}
pub(super) fn sweep18_ui_test(ui: &Rc<Ui>, app: &Application) {
    let display = gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    for name in ["headings-allcap-symbolic", "font-symbolic"] {
        assert!(theme.has_icon(name), "missing exact icon {name}");
    }
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "First word\nSecond word\nThird word",
        )),
        "Sweep18",
    );
    select_paragraph(ui, 0);
    let menu = format_bar::open_group(ui, "text", None);
    menu.popup();
    let tools = ui.text_tools.borrow().as_ref().unwrap().clone();
    let palette = tools.palette.borrow().as_ref().unwrap().clone();
    assert!(ui.format_bold.is::<gtk::CheckButton>());
    assert!(palette.underline_check.is::<gtk::CheckButton>());
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    for page in ["underline", "strike", "ink", "highlight"] {
        tools.page(page);
        assert_eq!(tools.stack.visible_child_name().as_deref(), Some("detail"));
        tools.home();
    }
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
    tools.page("ink");
    test_choose_color(&menu.popover().unwrap(), "text-ink-chooser", "1C71D8");
    tools.home();
    assert_eq!(*palette.last_ink.borrow(), "1C71D8");
    for n in [1, 2] {
        select_paragraph(ui, n);
        palette.ink_apply.emit_clicked();
        assert_eq!(
            selection_character(ui)["color"],
            json!({"Rgb":[28,113,216]})
        );
        assert_eq!(tools.stack.visible_child_name().as_deref(), Some("main"));
    }
    tools.page("highlight");
    test_choose_color(&menu.popover().unwrap(), "text-highlight-chooser", "FFFF00");
    tools.home();
    select_paragraph(ui, 0);
    palette.highlight_apply.emit_clicked();
    assert_eq!(selection_character(ui)["highlight"], json!([255, 255, 0]));
    assert_eq!(*palette.last_ink.borrow(), "1C71D8");
    button(
        &palette.options_page("underline").unwrap(),
        "underline-choice-double",
    );
    assert_eq!(selection_character(ui)["underline"], "double");
    palette
        .underline_check
        .set_active(!palette.underline_check.is_active());
    assert_eq!(selection_character(ui)["underline"], "none");
    palette
        .underline_check
        .set_active(!palette.underline_check.is_active());
    assert_eq!(selection_character(ui)["underline"], "double");
    button(&palette.options_page("strike").unwrap(), "strike-double");
    assert_eq!(selection_character(ui)["doubleStrike"], true);
    palette
        .strike_check
        .set_active(!palette.strike_check.is_active());
    assert_eq!(selection_character(ui)["doubleStrike"], false);
    palette.superscript.set_active(true);
    assert_eq!(selection_character(ui)["vertAlign"], "superscript");
    tools.page("language");
    let off = find_widget(&menu.popover().unwrap(), "language-choice-off").unwrap();
    assert_eq!(
        off.next_sibling().unwrap().widget_name(),
        "language-choice-default"
    );
    off.downcast::<gtk::Button>().unwrap().emit_clicked();
    assert_eq!(
        wordcraft_gnome::proofing_defaults::choice(&ui.current.borrow().as_ref().unwrap().session),
        "off"
    );
    tools.page("language");
    button(&menu.popover().unwrap(), "language-choice-default");
    assert_eq!(
        wordcraft_gnome::proofing_defaults::choice(&ui.current.borrow().as_ref().unwrap().session),
        "default"
    );
    tools.home();
    button(&menu.popover().unwrap(), "text-more-options");
    assert!(find_widget(&menu.popover().unwrap(), "format-row-text-case").is_some());
    let clear = find_widget(&menu.popover().unwrap(), "text-clear-formatting").unwrap();
    assert!(clear.parent().unwrap().next_sibling().is_none());
    menu.popdown();
    // Document defaults preserve explicit yes and disabled text, independent of app defaults.
    {
        let mut c = ui.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        wordcraft_gnome::proofing_defaults::apply(s, "es-ES", false).unwrap();
    }
    select_paragraph(ui, 1);
    format_execute(ui, "leaf.proofText", json!({"choice":"en-US"})).unwrap();
    {
        let c = ui.current.borrow();
        let s = &c.as_ref().unwrap().session;
        let p = s.doc.para_at(&Pos::body(1, 0)).unwrap();
        assert!(
            !s.doc
                .styles
                .resolve_char(p.props.style.as_deref(), p.props_of_char(0))
                .no_proof
        );
    }
    // Native theme values, including unfocused treatment and live changes.
    let css = gtk::CssProvider::new();
    let w = &ui.document_box;
    gtk::style_context_add_provider_for_display(&display, &css, gtk::STYLE_PROVIDER_PRIORITY_USER);
    css.load_from_string("@define-color theme_selected_bg_color #123456; @define-color theme_selected_fg_color #fedcba; @define-color theme_unfocused_selected_bg_color #654321; @define-color theme_unfocused_selected_fg_color #abcdef;");
    let (bg, fg) = selection_colors(w, true);
    assert!((bg.red() - 0x12 as f32 / 255.0).abs() < 0.01);
    assert!((fg.red() - 0xfe as f32 / 255.0).abs() < 0.01);
    let (bg, fg) = selection_colors(w, false);
    assert!((bg.red() - 0x65 as f32 / 255.0).abs() < 0.01);
    assert!((fg.red() - 0xab as f32 / 255.0).abs() < 0.01);
    css.load_from_string("@define-color theme_selected_bg_color #abcdef; @define-color theme_selected_fg_color #012345;");
    let (bg, fg) = selection_colors(w, true);
    assert!((bg.red() - 0xab as f32 / 255.0).abs() < 0.01);
    assert!((fg.red() - 1.0 / 255.0).abs() < 0.01);
    gtk::style_context_remove_provider_for_display(&display, &css);
    // Caret-driven navigation selects the owning heading without navigation side effects.
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        "Front\nTop\nBody\nNested\nText\nNext",
    ));
    for (p, cmd) in [
        (1, "para.heading1"),
        (3, "para.heading2"),
        (5, "para.heading1"),
    ] {
        s.run("caret.set", &json!({"pos":Pos::body(p,0)})).unwrap();
        s.run(cmd, &json!({})).unwrap();
    }
    opened(ui, s, "Navigation18");
    let panel = ui.outline.borrow().as_ref().unwrap().clone();
    panel.show();
    ui.document_box.grab_focus();
    for (p, index) in [
        (0, None),
        (1, Some(0)),
        (2, Some(0)),
        (3, Some(1)),
        (4, Some(1)),
        (5, Some(2)),
    ] {
        let (rev, depth, scroll) = {
            let mut c = ui.current.borrow_mut();
            let s = &mut c.as_mut().unwrap().session;
            s.run("caret.set", &json!({"pos":Pos::body(p,0)})).unwrap();
            (s.rev(), s.undo_depth(), ui.scroll.vadjustment().value())
        };
        refresh_selection(ui);
        assert_eq!(
            panel
                .list
                .selected_row()
                .map(|r| r.widget_name().to_string()),
            index
                .map(|i| format!("outline-heading-{i}"))
                .or_else(|| (p == 0).then(|| "outline-section-1".into()))
        );
        let c = ui.current.borrow();
        let s = &c.as_ref().unwrap().session;
        assert_eq!(s.sel.focus, Pos::body(p, 0));
        assert_eq!(s.rev(), rev);
        assert_eq!(s.undo_depth(), depth);
        assert_eq!(ui.scroll.vadjustment().value(), scroll);
    }
    eprintln!(
        "SWEEP18_UI_OK icons=true compact=true reapply=true language=true theme=true navigation=true"
    );
    app.quit();
}

pub(super) fn table_pointer_ui_test(ui: &Rc<Ui>, app: &Application, case: String) {
    ui.host
        .upgrade()
        .unwrap()
        .window
        .set_title(Some("Leaf Table Pointer Regression"));
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        "Table fixture\nFollowing text",
    ));
    s.run("caret.set", &json!({"pos":Pos::body(1,0)})).unwrap();
    s.run(
        "insert.table",
        &json!({"rows":if case=="later"{90}else{3},"cols":3}),
    )
    .unwrap();
    if case == "merged" {
        let a = s.sel.focus.clone();
        let mut b = a.clone();
        b.path.0[2] = 1;
        selection::select_range(&mut s, &a, &b).unwrap();
        s.run("table.merge", &json!({})).unwrap();
        s.run("table.selectTable", &json!({})).unwrap();
    }
    if case == "later" {
        s.run("table.repeatHeader", &json!({"value":true})).unwrap();
        s.run("table.selectTable", &json!({})).unwrap();
        s.run("table.rowHeight", &json!({"height":24})).unwrap();
        let path = s
            .doc
            .para_paths(wordcraft_doc::StoryRef::Body)
            .into_iter()
            .filter(|p| p.cell().is_some())
            .last()
            .unwrap();
        s.run(
            "caret.set",
            &json!({"pos":Pos{story:wordcraft_doc::StoryRef::Body,path,off:0}}),
        )
        .unwrap();
    }
    opened(ui, s, "Table pointer18");
    if case == "later" {
        ui.host
            .upgrade()
            .unwrap()
            .window
            .set_default_size(1800, 900);
    }
    ui.current.borrow_mut().as_mut().unwrap().fit_to_width = false;
    zoom_to(ui, if case == "later" { 0.6 } else { 0.8 });
    caret_ensure_visible(ui);
    ui.document_box.grab_focus();
    let u = ui.clone();
    let a = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || {
        use wordcraft_engine::cmd::{table_mouse as m, table_resize as r};
        let (g, page, original, depth, rev) = {
            let mut c = u.current.borrow_mut();
            let d = c.as_mut().unwrap();
            let l = d.session.layout();
            let caret = l
                .caret_on(&d.session.sel.focus, d.session.page_hint)
                .unwrap();
            let g =
                r::table_under(&l, &d.session.doc, caret.page, caret.x, caret.top, 3.0).unwrap();
            (
                g,
                caret.page,
                d.session.doc.clone(),
                d.session.undo_depth(),
                d.session.rev(),
            )
        };
        let (x, y, dx, dy) = match case.as_str() {
            "merged" => (
                g.cols[2],
                (g.rows[1].top + g.rows[1].bottom) / 2.0,
                25.0,
                0.0,
            ),
            "row" => (g.left() + 20.0, g.rows[0].bottom, 0.0, 25.0),
            "corner" => {
                let r = g.handle();
                (r.x + 3.0, r.y + 3.0, 30.0, 25.0)
            }
            "column-select" => (
                (g.cols[0] + g.cols[1]) / 2.0,
                g.top() - 4.0,
                g.cols[1] - g.cols[0],
                0.0,
            ),
            "cells" => (
                g.left() + 5.0,
                (g.rows[0].top + g.rows[0].bottom) / 2.0,
                g.cols[1] - g.left() + 6.0,
                g.rows[1].bottom - g.rows[0].bottom,
            ),
            "table-select" => {
                let r = m::handle(&g);
                (r.x + 4.0, r.y + 4.0, 0.0, 0.0)
            }
            "insert-row" => (g.left() - 8.0, g.rows[0].bottom, 0.0, 0.0),
            "select" => (
                g.left() - 18.0,
                (g.rows[0].top + g.rows[0].bottom) / 2.0,
                0.0,
                g.rows[1].bottom - g.rows[0].bottom,
            ),
            "insert" => (g.cols[1], g.top() - 8.0, 0.0, 0.0),
            "move" => {
                let r = m::handle(&g);
                (r.x + 4.0, r.y + 4.0, 25.0, 20.0)
            }
            _ => (g.cols[1], (g.top() + g.rows[0].bottom) / 2.0, 30.0, 0.0),
        };
        let (sx, sy, ex, ey) = {
            let c = u.current.borrow();
            let d = c.as_ref().unwrap();
            let f = d.page_frames[page]
                .compute_bounds(&u.host.upgrade().unwrap().window)
                .unwrap();
            let z = d.zoom;
            let (tx, ty) = u.host.upgrade().unwrap().window.surface_transform();
            eprintln!("TABLE18_TRANSFORM {tx} {ty} frame={f:?}");
            (
                (f.x() as f64 + x as f64 * z as f64 + tx).round() as i32,
                (f.y() as f64 + y as f64 * z as f64 + ty).round() as i32,
                (f.x() as f64 + (x + dx) as f64 * z as f64 + tx).round() as i32,
                (f.y() as f64 + (y + dy) as f64 * z as f64 + ty).round() as i32,
            )
        };
        eprintln!("TABLE18_READY {sx} {sy} {ex} {ey}");
        glib::timeout_add_local_once(std::time::Duration::from_secs(4), move || {
            let mut c = u.current.borrow_mut();
            let s = &mut c.as_mut().unwrap().session;
            match case.as_str() {
                "cancel" => {
                    assert!(s.doc == original, "document unchanged");
                    assert_eq!(s.undo_depth(), depth);
                }
                "select" | "column-select" | "cells" | "table-select" => {
                    assert!(s.cell_range().is_some());
                    assert_eq!(
                        wordcraft_engine::cmd::table::selected_cells(s).len(),
                        if case == "cells" {
                            4
                        } else if case == "table-select" {
                            9
                        } else {
                            6
                        }
                    );
                    assert!(s.doc == original, "document unchanged");
                    assert_eq!(s.rev(), rev);
                    assert_eq!(s.undo_depth(), depth);
                }
                _ => {
                    assert!(
                        s.doc != original,
                        "pointer operation changed document: {case}"
                    );
                    assert_eq!(s.undo_depth(), depth + 1, "one edit per drag");
                    s.run("edit.undo", &json!({})).unwrap();
                    assert!(s.doc == original, "undo restores table");
                }
            }
            eprintln!("TABLE18_OK case={case}");
            a.quit();
        });
    });
}

thread_local! { static TEXT_POINTER_GEOMETRY: RefCell<std::collections::HashMap<String, ([f64;4],u8)>> = RefCell::new(std::collections::HashMap::new()); }
/// Opt-in test handshake: stable geometry does not prove that GTK delivered a
/// native pointer event to the newly mapped popup control. Consume no events.
pub(super) fn observe_pointer_target(
    marker: &str,
    pop: &gtk::Popover,
    widget: &impl IsA<gtk::Widget>,
) {
    if std::env::args().any(|a| a == "--text18-pointer-test") {
        let events = gtk::EventControllerLegacy::new();
        events.set_propagation_phase(gtk::PropagationPhase::Capture);
        let marker = marker.to_owned();
        events.connect_event(move |_, event| {
            if matches!(
                event.event_type(),
                gdk::EventType::ButtonPress | gdk::EventType::ButtonRelease
            ) {
                eprintln!(
                    "POINTER_POPUP_BUTTON {marker} {:?} xy={:?}",
                    event.event_type(),
                    event.position()
                );
            }
            glib::Propagation::Proceed
        });
        pop.add_controller(events);
    }
    let motion = gtk::EventControllerMotion::new();
    motion.set_propagation_phase(gtk::PropagationPhase::Capture);
    let pop = pop.downgrade();
    let widget_weak = widget.as_ref().downgrade();
    let marker = marker.to_owned();
    let button_marker = marker.clone();
    let seen = Cell::new(false);
    let notify = Rc::new(
        move |controller: &gtk::EventControllerMotion, x: f64, y: f64| {
            if seen.get() {
                return;
            }
            let Some(pop) = pop.upgrade().filter(|p| p.is_mapped()) else {
                return;
            };
            let Some(widget) = widget_weak.upgrade().filter(|w| w.is_sensitive()) else {
                return;
            };
            if controller.current_event().and_then(|e| e.surface()) == pop.surface()
                && widget.contains(x, y)
                && widget.is_mapped()
            {
                seen.set(true);
                eprintln!("POINTER_TARGET_ENTERED {marker}");
            }
        },
    );
    let entered = notify.clone();
    motion.connect_enter(move |m, x, y| entered(m, x, y));
    motion.connect_motion(move |m, x, y| notify(m, x, y));
    widget.add_controller(motion);
    if std::env::args().any(|a| a == "--text18-pointer-test") {
        let events = gtk::EventControllerLegacy::new();
        events.set_propagation_phase(gtk::PropagationPhase::Capture);
        let marker = button_marker;
        events.connect_event(move |_, event| {
            if matches!(
                event.event_type(),
                gdk::EventType::ButtonPress | gdk::EventType::ButtonRelease
            ) {
                eprintln!(
                    "POINTER_BUTTON {marker} {:?} xy={:?}",
                    event.event_type(),
                    event.position()
                );
            }
            glib::Propagation::Proceed
        });
        widget.add_controller(events);
    }
}
pub(super) fn text_pointer_target(
    marker: &str,
    pop: &gtk::Popover,
    widget: &impl IsA<gtk::Widget>,
) -> bool {
    use gdk::prelude::PopupExt;
    if !widget.as_ref().is_mapped() {
        return false;
    }
    let r = widget.as_ref().compute_bounds(pop).unwrap();
    let (tx, ty) = pop.surface_transform();
    let (mut x, mut y) = (-tx, -ty);
    let mut surface = pop.surface();
    while let Some(s) = surface {
        let Ok(p) = s.downcast::<gdk::Popup>() else {
            break;
        };
        x += p.position_x() as f64;
        y += p.position_y() as f64;
        surface = gdk::prelude::PopupExt::parent(&p);
    }
    let scale = pop.scale_factor() as f64;
    let geometry = [
        (x + r.x() as f64 + r.width() as f64 / 2.) * scale,
        (y + r.y() as f64 + r.height() as f64 / 2.) * scale,
        r.width() as f64,
        r.height() as f64,
    ];
    let stable = TEXT_POINTER_GEOMETRY.with(|cache| {
        let mut c = cache.borrow_mut();
        let last = c.entry(marker.into()).or_insert((geometry, 0));
        if last.0 == geometry {
            last.1 += 1;
        } else {
            *last = (geometry, 0);
        }
        last.1 >= 3
    });
    if !stable {
        return false;
    }
    if std::env::args().any(|a| a == "--format136-ui-test" || a == "--text18-pointer-test") {
        observe_pointer_target(marker, pop, widget);
    }
    eprintln!(
        "{marker} {} {}",
        geometry[0].round() as i32,
        geometry[1].round() as i32
    );
    true
}
pub(super) fn text18_pointer_test(ui: &Rc<Ui>, app: &Application) {
    ui.host
        .upgrade()
        .unwrap()
        .window
        .set_title(Some("Leaf Text Pointer Regression"));
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "First target\nSecond target\nThird target",
        )),
        "Text pointer",
    );
    select_paragraph(ui, 0);
    let menu = format_bar::open_group(ui, "text", None);
    let pop = format_bar::group_pop(ui, "text");
    menu.popup();
    let tools = ui.text_tools.borrow().as_ref().unwrap().clone();
    let palette = tools.palette.borrow().as_ref().unwrap().clone();

    let (u, a) = (ui.clone(), app.clone());
    let phase = Cell::new(0);
    let ticks = Cell::new(0);
    let settled = Cell::new(0);
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        ticks.set(ticks.get() + 1);
        assert!(
            ticks.get() < 300,
            "text pointer phase {} timed out; state={:?}, check={}, mixed={}, last={}",
            phase.get(),
            selection_character(&u),
            palette.underline_check.is_active(),
            palette.underline_check.is_inconsistent(),
            palette.last_underline.borrow()
        );
        match phase.get() {
            0 if pop.is_mapped() => {
                let arrow = find_widget(&pop, "text-ink").unwrap();
                if !text_pointer_target("TEXT18_INK_ARROW", &pop, &arrow) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(1);
            }
            1 if tools.stack.visible_child_name().as_deref() == Some("detail") => {
                settled.set(settled.get() + 1);
                if settled.get() < 4 {
                    return glib::ControlFlow::Continue;
                }
                settled.set(0);
                let child = pop.clone();
                let picker = find_widget(&child, "text-ink-chooser").unwrap();
                let blue =
                    native_color_swatch(&picker, &gdk::RGBA::parse("#1C71D8").unwrap()).unwrap();
                if !text_pointer_target("TEXT18_BLUE", &child, &blue) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(2);
            }
            2 if palette.last_ink.borrow().as_str() == "1C71D8" => {
                assert_eq!(
                    selection_character(&u)["color"],
                    json!({"Rgb":[28,113,216]})
                );
                if !text_pointer_target("TEXT18_INK_BACK", &pop, &tools.header.button) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(11);
            }
            11 if tools.stack.visible_child_name().as_deref() == Some("main") => {
                select_paragraph(&u, 1);
                settled.set(0);
                phase.set(8);
            }
            8 => {
                settled.set(settled.get() + 1);
                if settled.get() < 5 {
                    return glib::ControlFlow::Continue;
                }
                settled.set(0);
                let body = find_widget(&pop, "text-ink").unwrap();
                if !text_pointer_target("TEXT18_INK_REOPEN", &pop, &body) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(12);
            }
            12 if tools.stack.visible_child_name().as_deref() == Some("detail") => {
                // The chooser has no redundant Apply row. Return to its genuine
                // split body, then reapply the last color on the current target.
                tools.home();
                settled.set(0);
                phase.set(13);
            }
            13 if tools.stack.visible_child_name().as_deref() == Some("main") => {
                settled.set(settled.get() + 1);
                if settled.get() < 5 {
                    return glib::ControlFlow::Continue;
                }
                settled.set(0);
                let body = find_widget(&pop, "text-ink-apply").unwrap();
                if !text_pointer_target("TEXT18_INK_BODY", &pop, &body) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(3);
            }
            3 if selection_character(&u)["color"] == json!({"Rgb":[28,113,216]}) => {
                assert_eq!(tools.stack.visible_child_name().as_deref(), Some("main"));
                select_paragraph(&u, 2);
                settled.set(0);
                phase.set(9);
            }
            9 => {
                settled.set(settled.get() + 1);
                if settled.get() < 5 {
                    return glib::ControlFlow::Continue;
                }
                let repeat = find_widget(&pop, "text-ink-apply").unwrap();
                if !text_pointer_target("TEXT18_INK_AGAIN", &pop, &repeat) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(10);
            }
            10 if selection_character(&u)["color"] == json!({"Rgb":[28,113,216]}) => {
                tools.home();
                let arrow = palette.underline.clone().upcast::<gtk::Widget>();
                if !text_pointer_target("TEXT18_UNDERLINE_ARROW", &pop, &arrow) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(4);
            }
            4 if tools.stack.visible_child_name().as_deref() == Some("detail") => {
                settled.set(settled.get() + 1);
                if settled.get() < 4 {
                    return glib::ControlFlow::Continue;
                }
                settled.set(0);
                let child = pop.clone();
                let double = find_widget(&child, "underline-choice-double").unwrap();
                if !text_pointer_target("TEXT18_DOUBLE", &child, &double) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(5);
            }
            5 if palette.last_underline.borrow().as_str() == "double" => {
                if !pop.is_visible() || tools.stack.visible_child_name().as_deref() != Some("main")
                {
                    format_bar::open_group(&u, "text", None);
                    settled.set(0);
                    return glib::ControlFlow::Continue;
                }
                settled.set(settled.get() + 1);
                if settled.get() < 5 {
                    return glib::ControlFlow::Continue;
                }
                settled.set(0);
                assert_eq!(selection_character(&u)["underline"], "double");
                let body = palette.underline_check.clone().upcast::<gtk::Widget>();
                if !text_pointer_target("TEXT18_UNDERLINE_OFF", &pop, &body) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(6);
            }
            6 if selection_character(&u)["underline"] == "none" => {
                // A second click must be a separate native checkbox activation,
                // not the second press in GTK's double-click gesture.
                settled.set(settled.get() + 1);
                if settled.get() < 5 {
                    return glib::ControlFlow::Continue;
                }
                settled.set(0);
                let body = palette.underline_check.clone().upcast::<gtk::Widget>();
                if !text_pointer_target("TEXT18_UNDERLINE_ON", &pop, &body) {
                    return glib::ControlFlow::Continue;
                }
                phase.set(7);
            }
            7 if selection_character(&u)["underline"] == "double" => {
                assert_eq!(tools.stack.visible_child_name().as_deref(), Some("main"));
                assert_eq!(*palette.last_ink.borrow(), "1C71D8");
                eprintln!("TEXT18_POINTER_OK chevron=true choice=true reapply=true toggle=true");
                a.quit();
                return glib::ControlFlow::Break;
            }
            _ => {}
        }
        glib::ControlFlow::Continue
    });
}

#[allow(deprecated)]
pub(super) fn test_choose_color(root: &impl IsA<gtk::Widget>, name: &str, hex: &str) {
    let picker = find_widget(root, name).unwrap();
    assert!(picker.is_mapped(), "shared chooser must be shown: {name}");
    let editor = find_widget(&picker, "color-custom-editor")
        .unwrap()
        .downcast::<gtk::ColorChooserWidget>()
        .unwrap();
    let rgba = gdk::RGBA::parse(format!("#{hex}")).unwrap();
    editor.set_rgba(&rgba);
    editor.emit_by_name::<()>("color-activated", &[&rgba]);
}
pub(super) fn native_color_swatch(root: &gtk::Widget, color: &gdk::RGBA) -> Option<gtk::Widget> {
    if root.type_().name() == "GtkColorSwatch"
        && root.find_property("rgba").is_some()
        && root.property::<Option<gdk::RGBA>>("rgba").as_ref() == Some(color)
    {
        return Some(root.clone());
    }
    let mut child = root.first_child();
    while let Some(w) = child {
        child = w.next_sibling();
        if let Some(found) = native_color_swatch(&w, color) {
            return Some(found);
        }
    }
    None
}
