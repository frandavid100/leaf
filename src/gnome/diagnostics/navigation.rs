use super::*;

pub(in crate::gnome) fn navigation_ui_test(
    ui: &Rc<Ui>,
    s: &Rc<OutlinePanel>,
    titles: &gtk::ToggleButton,
    host: &Rc<Workspace>,
    app: &Application,
) {
    let rev = ui.current.borrow().as_ref().unwrap().session.rev();
    titles.set_active(true);
    s.refresh();
    assert_eq!(
        ui.navigation_pages.visible_child_name().as_deref(),
        Some("outline")
    );
    assert!(!s.positions.borrow().is_empty());
    let before = s.positions.borrow().len();
    let expand = || {
        find_widget(&s.list, "outline-heading-0")
            .unwrap()
            .downcast::<gtk::ListBoxRow>()
            .unwrap()
            .child()
            .unwrap()
            .first_child()
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked()
    };
    expand();
    assert!(s.positions.borrow().len() < before);
    expand();
    assert_eq!(s.positions.borrow().len(), before);
    let row = s.list.row_at_index(0).unwrap();
    row.emit_by_name::<()>("activate", &[]);
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.sel.focus,
        s.positions.borrow()[0]
    );
    assert_eq!(
        ui.current.borrow().as_ref().unwrap().session.rev(),
        rev,
        "navigation must not edit"
    );
    let search = ui.search.borrow().as_ref().unwrap().clone();
    search.show(false);
    search.toggle.set_active(false);
    assert_eq!(
        ui.sidebar_pages.visible_child_name().as_deref(),
        Some("navigation")
    );
    assert_eq!(
        ui.navigation_pages.visible_child_name().as_deref(),
        Some("outline")
    );
    assert_eq!(
        ui.sidebar_pages.transition_type(),
        gtk::StackTransitionType::Crossfade
    );
    apply_user_edit(ui, Edit::Insert("x".into()));
    refresh_main_actions(host);
    assert!(host.window.lookup_action("undo").unwrap().is_enabled());
    gio::prelude::ActionGroupExt::activate_action(&host.window, "undo", None);
    refresh_main_actions(host);
    assert!(host.window.lookup_action("redo").unwrap().is_enabled());
    let count = host.tabs.n_pages();
    gio::prelude::ActionGroupExt::activate_action(&host.window, "new-tab", None);
    assert_eq!(host.tabs.n_pages(), count + 1);
    gio::prelude::ActionGroupExt::activate_action(&host.window, "tab-overview", None);
    assert!(host.overview.is_open());
    host.overview.set_open(false);
    show_keyboard_shortcuts(&host.window);
    show_editor_help(&host.window);
    eprintln!("SIDEBAR_NAVIGATION_OK");
    app.quit();
}

/// Check the deferred scroll using allocated native geometry, including two-page rows.
pub(in crate::gnome) fn heading_top_ui_test(
    ui: &Rc<Ui>,
    panel: &Rc<OutlinePanel>,
    app: &Application,
) {
    let text = (0..100).map(|n| format!("Heading {n}. Following section content fills the page. More text for this section. Further text remains visible after navigation.")).collect::<Vec<_>>().join("\n");
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(&text)),
        "Heading alignment",
    );
    heading_top_case(ui.clone(), panel.clone(), app.clone(), 0);
}
fn heading_top_case(ui: Rc<Ui>, panel: Rc<OutlinePanel>, app: Application, case: usize) {
    if case == 12 {
        eprintln!(
            "HEADING_TOP_UI_OK downward=true upward=true visible=true clamped=true zoom=true context_lines=3"
        );
        app.quit();
        return;
    }
    if case == 8 {
        if let Some(host) = ui.host.upgrade() {
            host.window.set_default_size(900, 520);
        }
    }
    zoom_to(
        &ui,
        if case < 4 {
            1.0
        } else if case < 8 {
            0.5
        } else {
            3.0
        },
    );
    let pos = Pos::body([50, 2, 50, 99][case % 4], 0);
    let next_ui = ui.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
        panel.goto(&pos);
        glib::timeout_add_local_once(std::time::Duration::from_millis(550), move || {
            let mut v = next_ui.current.borrow_mut();
            let d = v.as_mut().unwrap();
            assert_eq!(d.session.sel.focus, pos);
            let caret = d
                .session
                .layout()
                .caret_on(&pos, d.session.page_hint)
                .unwrap();
            let frame = d.page_frames[caret.page]
                .compute_bounds(&next_ui.scroll)
                .unwrap();
            let top = frame.y() as f64 + caret.top as f64 * d.zoom as f64;
            let adj = next_ui.scroll.vadjustment();
            let max = (adj.upper() - adj.page_size()).max(adj.lower());
            if adj.value() > adj.lower() + 1.0 && adj.value() < max - 1.0 {
                assert!(
                    top >= -2.0 && top <= adj.page_size() * 0.25 + 3.0,
                    "heading remains near top with following section visible: {top}"
                );
                let layout = d.session.layout();
                let mut preceding = Vec::new();
                for item in &layout.pages[caret.page].items {
                    if let wordcraft_engine::layout::Placed::Lines {
                        path,
                        para,
                        l0,
                        l1,
                        y,
                        ..
                    } = item
                    {
                        if path < &pos.path {
                            for line in &para.lines[*l0..*l1] {
                                let rendered = frame.y() as f64
                                    + (*y + line.top - para.lines[*l0].top) as f64 * d.zoom as f64;
                                if rendered < top {
                                    preceding.push(rendered);
                                }
                            }
                        }
                    }
                }
                if let Some(first) = preceding.iter().rev().take(3).last() {
                    assert!(
                        (*first >= -3.0 && *first < 3.0)
                            || (top - adj.page_size() * 0.25).abs() < 3.0,
                        "real preceding text is visible, except when large blank space caps context: {first}"
                    );
                }
            } else {
                assert!(
                    top >= 0.0 && top < adj.page_size(),
                    "clamped heading visible: {top}"
                );
            }
            assert_eq!(d.session.undo_depth(), 0);
            assert!(!d.session.dirty);
            drop(v);
            heading_top_case(next_ui, panel, app, case + 1);
        });
    });
}
