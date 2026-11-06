use super::*;
use serde_json::json;

fn click(root: &impl IsA<gtk::Widget>, name: &str) {
    find_widget(root, name)
        .unwrap_or_else(|| panic!("missing {name}"))
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
}
pub(super) fn review29_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    window.set_default_size(1250, 950);
    window.set_title(Some("Leaf Alpha29 Review Regression"));
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        "First passage.\nComment passage.\nI recieve qqqreviewzz and qqqreviewzz.",
    ));
    wordcraft_gnome::proofing_defaults::apply(&mut s, "en-US", true).unwrap();
    s.author = "David".into();
    s.run("review.trackChanges", &json!({"value":true}))
        .unwrap();
    selection::set_caret(&mut s, &Pos::body(0, 6), false).unwrap();
    s.run("text.insert", &json!({"text":"proposed "})).unwrap();
    s.run("review.trackChanges", &json!({"value":false}))
        .unwrap();
    selection::select_range(&mut s, &Pos::body(1, 0), &Pos::body(1, 7)).unwrap();
    wordcraft_gnome::annotations::add_comment(&mut s, "Check this passage.").unwrap();
    opened(ui, s, "Unified review");
    let review = ui.review.borrow().as_ref().unwrap().clone();
    review.filter.set_selected(0);
    review.show(None);
    assert_eq!(ui.sidebar_title.text(), wordcraft_gnome::t!("Review"));
    assert!(ui.navigation_pages.child_by_name("annotations").is_none());
    assert!(ui.navigation_pages.child_by_name("proofing").is_none());
    let before = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let depth = ui.current.borrow().as_ref().unwrap().session.undo_depth();
    let selected = ui.current.borrow().as_ref().unwrap().session.sel.clone();
    let (u, a) = (ui.clone(), app.clone());
    let attempts = Cell::new(0);
    let phase = Cell::new(0);
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        attempts.set(attempts.get() + 1);
        assert!(
            attempts.get() < 400,
            "expanded review UI stalled in phase {}",
            phase.get()
        );
        let spell = u.spelling.borrow().as_ref().unwrap().clone();
        if spell.source.borrow().is_some() || spell.debounce.borrow().is_some() {
            return glib::ControlFlow::Continue;
        }
        match phase.get() {
            0 => {
                review.refresh();
                assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, before);
                assert_eq!(
                    u.current.borrow().as_ref().unwrap().session.undo_depth(),
                    depth
                );
                assert_eq!(u.current.borrow().as_ref().unwrap().session.sel, selected);
                let first = review.content.first_child().unwrap();
                assert_eq!(first.widget_name(), "review-change-0");
                assert!(find_widget(&review.content, "proofing-apply-0").is_some());
                assert!(find_widget(&review.content, "comment-thread-0").is_some());
                for filter in [1, 2, 3, 0] {
                    review.filter.set_selected(filter);
                    review.refresh();
                    assert_eq!(
                        find_widget(&review.content, "review-change-0").is_some(),
                        filter == 0 || filter == 2
                    );
                    assert_eq!(
                        find_widget(&review.content, "proofing-apply-0").is_some(),
                        filter == 0 || filter == 1
                    );
                    assert_eq!(
                        find_widget(&review.content, "comment-thread-0").is_some(),
                        filter == 0 || filter == 3
                    );
                }
                let n = spell
                    .issues
                    .borrow()
                    .iter()
                    .position(|i| i.text == "qqqreviewzz")
                    .unwrap();
                let old_count = spell
                    .issues
                    .borrow()
                    .iter()
                    .filter(|i| i.text == "qqqreviewzz")
                    .count();
                assert_eq!(old_count, 2);
                click(&review.content, &format!("proofing-ignore-{n}"));
                review.refresh();
                assert_eq!(
                    spell
                        .issues
                        .borrow()
                        .iter()
                        .filter(|i| i.text == "qqqreviewzz")
                        .count(),
                    1
                );
                assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, before);
                // Resolve in Edit mode while comments remain independent.
                click(&review.content, "review-accept-0");
                assert!(
                    wordcraft_gnome::review::changes(
                        &mut u.current.borrow_mut().as_mut().unwrap().session
                    )
                    .unwrap()
                    .is_empty()
                );
                assert_eq!(
                    wordcraft_gnome::annotations::comments(
                        &u.current.borrow().as_ref().unwrap().session
                    )
                    .len(),
                    1
                );
                apply_user_edit(&u, Edit::Undo);
                phase.set(1);
            }
            1 => {
                review.refresh();
                assert!(find_widget(&review.content, "review-change-0").is_some());
                let n = spell
                    .issues
                    .borrow()
                    .iter()
                    .position(|i| i.text == "recieve")
                    .unwrap();
                let button = find_widget(&review.content, &format!("proofing-apply-{n}")).unwrap();
                assert!(button.is_sensitive());
                button.downcast::<gtk::Button>().unwrap().emit_clicked();
                assert!(
                    !u.current
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .session
                        .doc
                        .plain_text(wordcraft_doc::StoryRef::Body)
                        .contains("recieve")
                );
                phase.set(2);
            }
            _ => {
                review.refresh();
                format_rows_and_elements(&u);
                review.show(None);
                review.filter.set_selected(0);
                review.refresh();
                eprintln!(
                    "REVIEW29_UI_OK filters=true mixed=true occurrence=true apply=true revisions=true shared-elements=true"
                );
                if !std::env::args().any(|a| a == "--review29-demo") {
                    a.quit();
                } else if let Some(panel) = std::env::args()
                    .find_map(|a| a.strip_prefix("--review29-show=").map(str::to_owned))
                {
                    if panel == "navigation" {
                        u.navigation_button.set_active(true);
                    } else if panel == "structure" {
                        u.outline.borrow().as_ref().unwrap().show();
                    } else if panel == "elements" {
                        u.elements.borrow().as_ref().unwrap().menu.popup();
                    } else if let Some(menu) = format_button(&u, &format!("{panel}-menu")) {
                        menu.popup();
                    }
                }
                return glib::ControlFlow::Break;
            }
        }
        glib::ControlFlow::Continue
    });
}
fn format_rows_and_elements(ui: &Rc<Ui>) {
    let text = format_button(ui, "text-menu").unwrap();
    text.popup();
    let tools = ui.text_tools.borrow().as_ref().unwrap().clone();
    let palette = tools.palette.borrow().as_ref().unwrap().clone();
    assert!(ui.format_bold.is::<gtk::CheckButton>());
    assert!(palette.underline_check.is::<gtk::CheckButton>());
    assert!(!tools.language_value_text().is_empty());
    let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    {
        let mut c = ui.current.borrow_mut();
        let s = &mut c.as_mut().unwrap().session;
        selection::select_range(s, &Pos::body(2, 0), &Pos::body(2, 1)).unwrap();
        s.run("format.bold", &json!({"value":true})).unwrap();
        selection::select_range(s, &Pos::body(2, 0), &Pos::body(2, 5)).unwrap();
    }
    refresh_selection(ui);
    assert!(ui.format_bold.is_inconsistent());
    ui.format_bold.set_active(true);
    assert_eq!(selection_character(ui)["bold"], true);
    palette.choose_ink("1C71D8");
    {
        let mut c = ui.current.borrow_mut();
        selection::select_range(
            &mut c.as_mut().unwrap().session,
            &Pos::body(1, 0),
            &Pos::body(1, 4),
        )
        .unwrap();
    }
    refresh_selection(ui);
    assert_eq!(*palette.last_ink.borrow(), "1C71D8");
    text.popdown();
    let paragraph = ui.paragraph.borrow().as_ref().unwrap().clone();
    format_button(ui, "paragraph-menu").unwrap().popup();
    paragraph.home();
    assert!(
        find_widget(&paragraph.main, "paragraph-list")
            .unwrap()
            .is::<gtk::Button>()
    );
    assert!(
        find_widget(&paragraph.main, "paragraph-more-borders")
            .unwrap()
            .is::<gtk::Button>()
    );
    paragraph.popover.upgrade().unwrap().popdown();
    let elements = ui.elements.borrow().as_ref().unwrap().clone();
    elements.show("table");
    let table = ui.table_popover.borrow().as_ref().unwrap().clone();
    let body = elements.body_for(&table).unwrap();
    assert!(find_widget(&body, "table-insert").unwrap().is_visible());
    assert!(!find_widget(&body, "table-context").unwrap().is_sensitive());
    elements.pop.popdown();
    assert!(table.child().is_some());
    elements.show("image");
    assert!(
        find_widget(&elements.pop, "picture-insert")
            .unwrap()
            .is_visible()
    );
    assert!(
        !find_widget(&elements.pop, "picture-context")
            .unwrap()
            .is_sensitive()
    );
    elements.pop.popdown();
    elements.show("toc");
    assert!(find_widget(&elements.pop, "elements-insert-toc").is_some());
    assert!(
        !find_widget(&elements.pop, "elements-toc-apply")
            .unwrap()
            .is_sensitive()
    );
    elements.pop.popdown();
    // Restore the review fixture with canonical document replacement for the final capture.
    opened(ui, Session::new(original), "Unified review");
}
