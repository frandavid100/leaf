use super::*;

pub(super) fn navigation_ui_test(ui:&Rc<Ui>,s:&Rc<OutlinePanel>,titles:&gtk::ToggleButton,host:&Rc<Workspace>,app:&Application){
    let rev=ui.current.borrow().as_ref().unwrap().session.rev();titles.set_active(true);s.refresh();
    assert_eq!(ui.navigation_pages.visible_child_name().as_deref(),Some("outline"));assert!(!s.positions.borrow().is_empty());
    let before=s.positions.borrow().len();
    let expand=||s.list.row_at_index(0).unwrap().child().unwrap().first_child().unwrap().downcast::<gtk::Button>().unwrap().emit_clicked();
    expand();assert!(s.positions.borrow().len()<before);expand();assert_eq!(s.positions.borrow().len(),before);
    let row=s.list.row_at_index(0).unwrap();row.emit_by_name::<()>("activate",&[]);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.sel.focus,s.positions.borrow()[0]);
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(),rev,"navigation must not edit");
    let search=ui.search.borrow().as_ref().unwrap().clone();search.show(false);search.toggle.set_active(false);
    assert_eq!(ui.sidebar_pages.visible_child_name().as_deref(),Some("navigation"));assert_eq!(ui.navigation_pages.visible_child_name().as_deref(),Some("outline"));
    assert_eq!(ui.sidebar_pages.transition_type(),gtk::StackTransitionType::Crossfade);
    apply_user_edit(ui,Edit::Insert("x".into()));refresh_main_actions(host);assert!(host.window.lookup_action("undo").unwrap().is_enabled());
    gio::prelude::ActionGroupExt::activate_action(&host.window,"undo",None);refresh_main_actions(host);assert!(host.window.lookup_action("redo").unwrap().is_enabled());
    let count=host.tabs.n_pages();gio::prelude::ActionGroupExt::activate_action(&host.window,"new-tab",None);assert_eq!(host.tabs.n_pages(),count+1);
    gio::prelude::ActionGroupExt::activate_action(&host.window,"tab-overview",None);assert!(host.overview.is_open());host.overview.set_open(false);
    show_keyboard_shortcuts(&host.window);show_editor_help(&host.window);
    eprintln!("SIDEBAR_NAVIGATION_OK");app.quit();
}