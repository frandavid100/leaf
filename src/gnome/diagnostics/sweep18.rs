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
    let menu = format_button(ui, "text-menu").unwrap();
    menu.popup();
    let tools = ui.text_tools.borrow().as_ref().unwrap().clone();
    let palette = tools.palette.borrow().as_ref().unwrap().clone();
    for (n, count) in [(1, 3), (2, 2), (3, 2)] {
        let row = find_widget(&palette.root, &format!("text-tools-row-{n}")).unwrap();
        let mut c = row.first_child();
        let mut actual = 0;
        while let Some(w) = c {
            actual += 1;
            c = w.next_sibling();
        }
        assert_eq!(actual, count);
    }
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    for b in [
        &palette.underline,
        &palette.strike,
        &palette.ink,
        &palette.highlight,
    ] {
        assert!(b.popover().is_some());
        b.popover().unwrap().popup();
        b.popover().unwrap().popdown();
    }
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, before);
    button(&palette.ink.popover().unwrap(), "ink-choice-1C71D8");
    assert_eq!(*palette.last_ink.borrow(), "1C71D8");
    for n in [1, 2] {
        select_paragraph(ui, n);
        palette.ink.emit_by_name::<()>("clicked", &[]);
        assert_eq!(selection_character(ui)["color"], "1C71D8");
        assert!(!palette.ink.popover().unwrap().is_visible());
    }
    button(
        &palette.highlight.popover().unwrap(),
        "highlight-choice-yellow",
    );
    select_paragraph(ui, 0);
    palette.highlight.emit_by_name::<()>("clicked", &[]);
    assert_eq!(selection_character(ui)["highlight"], "yellow");
    assert_eq!(*palette.last_ink.borrow(), "1C71D8");
    button(
        &palette.underline.popover().unwrap(),
        "underline-choice-double",
    );
    assert_eq!(selection_character(ui)["underline"], "double");
    palette.underline.emit_by_name::<()>("clicked", &[]);
    assert_eq!(selection_character(ui)["underline"], "none");
    palette.underline.emit_by_name::<()>("clicked", &[]);
    assert_eq!(selection_character(ui)["underline"], "double");
    button(&palette.strike.popover().unwrap(), "strike-double");
    assert_eq!(selection_character(ui)["doubleStrike"], true);
    palette.strike.emit_by_name::<()>("clicked", &[]);
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
    let clear = find_widget(&menu.popover().unwrap(), "text-clear-formatting").unwrap();
    assert!(clear.next_sibling().is_none());
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
    css.load_from_data("@define-color theme_selected_bg_color #123456; @define-color theme_selected_fg_color #fedcba; @define-color theme_unfocused_selected_bg_color #654321; @define-color theme_unfocused_selected_fg_color #abcdef;");
    let (bg, fg) = selection_colors(w, true);
    assert!((bg.red() - 0x12 as f32 / 255.0).abs() < 0.01);
    assert!((fg.red() - 0xfe as f32 / 255.0).abs() < 0.01);
    let (bg, fg) = selection_colors(w, false);
    assert!((bg.red() - 0x65 as f32 / 255.0).abs() < 0.01);
    assert!((fg.red() - 0xab as f32 / 255.0).abs() < 0.01);
    css.load_from_data("@define-color theme_selected_bg_color #abcdef; @define-color theme_selected_fg_color #012345;");
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
            index.map(|i| format!("outline-heading-{i}"))
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
            (
                (f.x() + x * z).round() as i32,
                (f.y() + y * z).round() as i32,
                (f.x() + (x + dx) * z).round() as i32,
                (f.y() + (y + dy) * z).round() as i32,
            )
        };
        eprintln!("TABLE18_READY {sx} {sy} {ex} {ey}");
        glib::timeout_add_local_once(std::time::Duration::from_secs(4), move || {
            let mut c = u.current.borrow_mut();
            let s = &mut c.as_mut().unwrap().session;
            match case.as_str() {
                "cancel" => {
                    assert_eq!(s.doc, original);
                    assert_eq!(s.undo_depth(), depth);
                }
                "select" => {
                    assert!(s.cell_range().is_some());
                    assert_eq!(wordcraft_engine::cmd::table::selected_cells(s).len(), 6);
                    assert_eq!(s.doc, original);
                    assert_eq!(s.rev(), rev);
                    assert_eq!(s.undo_depth(), depth);
                }
                _ => {
                    assert_ne!(
                        s.doc, original,
                        "pointer operation changed document: {case}"
                    );
                    assert_eq!(s.undo_depth(), depth + 1, "one edit per drag");
                    s.run("edit.undo", &json!({})).unwrap();
                    assert_eq!(s.doc, original, "undo restores table");
                }
            }
            eprintln!("TABLE18_OK case={case}");
            a.quit();
        });
    });
}
