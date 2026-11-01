use super::*;

pub(super) fn paragraph_ui_test(s:&Rc<ParagraphPopover>,menu:&gtk::MenuButton,app:&Application){
    menu.popup();let ui=s.ui.upgrade().unwrap();
    let click=|name:&str|find_widget(&s.popover.upgrade().unwrap().child().unwrap(),name).unwrap_or_else(||panic!("missing {name}")).downcast::<gtk::Button>().unwrap().emit_clicked();
    let body=find_widget(&s.main,"paragraph-indent").unwrap();
    let split=body.parent().unwrap().downcast::<adw::SplitButton>().expect("native split button");
    assert!(split.has_css_class("flat"));
    let arrow=find_widget(&s.main,"paragraph-more-indents").unwrap();
    assert!(arrow.parent().unwrap().is::<gtk::MenuButton>());
    assert_eq!(arrow.parent().unwrap().parent().unwrap(),split.clone().upcast::<gtk::Widget>());
    click("paragraph-align-center");assert_eq!(selection_paragraph(&ui)["align"],"center");
    click("paragraph-indent");assert!(selection_paragraph(&ui)["indentLeft"].as_f64().unwrap()>0.0);
    click("paragraph-more-bullets");assert_eq!(s.stack.visible_child_name().as_deref(),Some("detail"));
    click("paragraph-list-•");assert!(selection_paragraph(&ui)["numbering"].is_object());click("paragraph-back");
    click("paragraph-more-spacing");click("paragraph-line-1.5");assert_eq!(selection_paragraph(&ui)["lineSpacing"]["value"],1.5);click("paragraph-back");
    click("paragraph-more-shading");click("paragraph-shade-FFF2CC");assert!(selection_paragraph(&ui)["shading"].is_array());click("paragraph-back");
    click("paragraph-more-borders");click("paragraph-border-outside");assert!(selection_paragraph(&ui)["borders"].is_object());click("paragraph-back");
    click("paragraph-more-options");assert!(find_widget(&s.detail,"paragraph-sort-asc").is_some());click("paragraph-back");
    assert_eq!(s.stack.visible_child_name().as_deref(),Some("main"));assert_eq!(app.windows().len(),1);
    assert!(ui.current.borrow().as_ref().unwrap().session.can_undo());eprintln!("PARAGRAPH_POPOVER_OK");app.quit();
}