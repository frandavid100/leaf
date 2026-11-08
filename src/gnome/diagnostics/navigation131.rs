use super::*;

/// Exercise persistent native representations without conflating a sidebar
/// scroll position with the document's editing target or viewport.
pub(super) fn navigation131_test(ui: &Rc<Ui>, finish: impl FnOnce() + 'static) {
    let u = ui.clone();
    let mut original = {
        let c = u.current.borrow();
        let d = c.as_ref().unwrap();
        (
            d.session.doc.clone(),
            d.session.sel.clone(),
            d.session.undo_depth(),
        )
    };
    let main = (
        u.scroll.hadjustment().value(),
        u.scroll.vadjustment().value(),
    );
    let phase = Cell::new(0);
    let thumbs = Cell::new(0.);
    let structure = Cell::new(0.);
    let finish = RefCell::new(Some(Box::new(finish) as Box<dyn FnOnce()>));
    let other: RefCell<Option<(adw::TabPage, Rc<Ui>)>> = RefCell::new(None);
    glib::timeout_add_local(std::time::Duration::from_millis(400), move || {
        let outline = u
            .navigation_pages
            .child_by_name("outline")
            .unwrap()
            .downcast::<gtk::ScrolledWindow>()
            .unwrap();
        let filter = find_widget(&outline, "navigation-filter")
            .unwrap()
            .downcast::<gtk::DropDown>()
            .unwrap();
        if std::env::var_os("LEAF_NAVIGATION_TRACE").is_some() {
            eprintln!(
                "NAVIGATION131_PHASE {} time={:?} value={}",
                phase.get(),
                std::time::SystemTime::now(),
                outline.vadjustment().value()
            );
        }
        match phase.get() {
            0 => {
                u.navigation_button.set_active(true);
                u.representation.set_selected(0);
                let a = u.thumbnails_scroll.vadjustment();
                a.set_value((a.upper() - a.page_size()) * 0.35);
                thumbs.set(a.value());
                assert!(thumbs.get() > 20.);
                u.representation.set_selected(1);
            }
            1 => {
                filter.set_selected(2);
            }
            2 => {
                let a = outline.vadjustment();
                a.set_value((a.upper() - a.page_size()) * 0.45);
                structure.set(a.value());
                assert!(structure.get() > 20.);
                u.representation.set_selected(0);
            }
            3 => {
                assert!(
                    (u.thumbnails_scroll.vadjustment().value() - thumbs.get()).abs() < 1.,
                    "Pages preserves manual browsing"
                );
                u.review.borrow().as_ref().unwrap().show(None);
                assert_eq!(u.sidebar_title.text(), wordcraft_gnome::t!("Review"));
                assert!(!u.navigation_selector.is_visible());
            }
            4 => {
                u.navigation_button.set_active(true);
                assert_eq!(
                    u.navigation_pages.visible_child_name().as_deref(),
                    Some("thumbnails")
                );
                assert_eq!(u.sidebar_title.text(), wordcraft_gnome::t!("Navigation"));
                assert!(u.navigation_selector.is_visible());
                assert!(u.representation.is_mapped());
                assert_eq!(u.representation.model().unwrap().n_items(), 2);
                u.representation.set_selected(1);
            }
            5 => {
                assert_eq!(filter.selected(), 2);
                assert!(
                    (outline.vadjustment().value() - structure.get()).abs() < 1.,
                    "Structure preserves independent scrolling: actual={} expected={}",
                    outline.vadjustment().value(),
                    structure.get()
                );
                let row = u.sidebar_selector.first_child().unwrap();
                let first = row.first_child().unwrap();
                let last = row.last_child().unwrap();
                assert_eq!(first.widget_name(), "sidebar-navigation");
                assert_eq!(last.widget_name(), "sidebar-review");
                assert_eq!(first.next_sibling(), Some(last));
                assert!(
                    (u.scroll.hadjustment().value() - main.0).abs() < 1.
                        && (u.scroll.vadjustment().value() - main.1).abs() < 1.,
                    "representations do not move the main viewport"
                );
                u.search.borrow().as_ref().unwrap().show(false);
            }
            6 => {
                assert!(
                    !u.navigation_pages.is_mapped(),
                    "Search does not show Navigation/Review filters"
                );
                u.search.borrow().as_ref().unwrap().toggle.set_active(false);
                assert!(u.navigation_pages.is_mapped());
                assert_eq!(
                    u.navigation_pages.visible_child_name().as_deref(),
                    Some("outline")
                );
                assert_eq!(filter.selected(), 2);
                u.sidebar.set_show_sidebar(false);
            }
            7 => u.sidebar.set_show_sidebar(true),
            8 => {
                assert_eq!(
                    u.navigation_pages.visible_child_name().as_deref(),
                    Some("outline")
                );
                assert!(
                    (outline.vadjustment().value() - structure.get()).abs() < 1.,
                    "reopening preserves browsing: actual={} expected={}",
                    outline.vadjustment().value(),
                    structure.get()
                );
                let h = u.host.upgrade().unwrap();
                gio::prelude::ActionGroupExt::activate_action(&h.window, "new-tab", None);
                let new = selected_ui(&h).unwrap();
                assert!(!Rc::ptr_eq(&new, &u));
                assert_eq!(new.last_navigation.get(), "thumbnails");
                let page = new.tab_page.borrow().as_ref().unwrap().clone();
                other.replace(Some((page, new)));
            }
            9 => {
                let h = u.host.upgrade().unwrap();
                check_headers(&selected_ui(&h).unwrap(), &h.window);
                h.tabs
                    .set_selected_page(u.tab_page.borrow().as_ref().unwrap());
            }
            10 => {
                let h = u.host.upgrade().unwrap();
                check_headers(&u, &h.window);
                if let Some((page, _)) = other.borrow_mut().take() {
                    h.tabs.close_page(&page);
                }
                assert_eq!(filter.selected(), 2);
                assert_eq!(u.last_navigation.get(), "outline");
                assert!(
                    (outline.vadjustment().value() - structure.get()).abs() < 1.,
                    "document tabs retain their own sidebar state: actual={} expected={}",
                    outline.vadjustment().value(),
                    structure.get()
                );
                assert!(
                    u.pages_button
                        .tooltip_text()
                        .unwrap()
                        .contains(&u.current.borrow().as_ref().unwrap().sizes.len().to_string())
                );
                filter.set_selected(0);
            }
            11 => {
                check_headers(&u, &u.host.upgrade().unwrap().window);
                u.review.borrow().as_ref().unwrap().show(None);
                {
                    let mut c = u.current.borrow_mut();
                    selection::set_caret(&mut c.as_mut().unwrap().session, &Pos::body(0, 0), false)
                        .unwrap();
                }
                apply_user_edit(&u, Edit::Insert("x".into()));
                let c = u.current.borrow();
                let d = c.as_ref().unwrap();
                assert_eq!(d.session.undo_depth(), original.2 + 1);
                original = (
                    d.session.doc.clone(),
                    d.session.sel.clone(),
                    d.session.undo_depth(),
                );
            }
            12 => u.navigation_button.set_active(true),
            _ => {
                let row = u
                    .outline
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .list
                    .selected_row()
                    .unwrap();
                assert_eq!(row.widget_name(), "outline-heading-0");
                // Adjustments use content coordinates. ScrolledWindow's child
                // may be a GtkViewport, whose bounds already subtract scroll.
                let bounds = row
                    .parent()
                    .and_then(|list| list.parent())
                    .and_then(|content| row.compute_bounds(&content))
                    .unwrap();
                let a = outline.vadjustment();
                assert!(row.is_mapped() && bounds.height() > 0.);
                eprintln!(
                    "NAVIGATION131_REVEAL content_y={} height={} scroll={} page={}",
                    bounds.y(),
                    bounds.height(),
                    a.value(),
                    a.page_size()
                );
                assert!(
                    bounds.y() as f64 >= a.value() - 1.
                        && (bounds.y() + bounds.height()) as f64 <= a.value() + a.page_size() + 1.,
                    "return reveals a genuinely changed target"
                );
                eprintln!(
                    "NAVIGATION131_UI_OK panels=2 representations=true independent-scroll=true review-return=true search-return=true sidebar-return=true tab-state=true target-change=true readonly=true"
                );
                finish.borrow_mut().take().unwrap()();
                return glib::ControlFlow::Break;
            }
        }
        {
            let c = u.current.borrow();
            let d = c.as_ref().unwrap();
            assert_eq!(d.session.doc, original.0);
            assert_eq!(d.session.sel, original.1);
            assert_eq!(d.session.undo_depth(), original.2);
        }
        phase.set(phase.get() + 1);
        glib::ControlFlow::Continue
    });
}
