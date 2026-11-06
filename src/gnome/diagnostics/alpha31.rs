use super::*;
use serde_json::json;

pub(super) fn alpha31_ui_test(ui: &Rc<Ui>, window: &ApplicationWindow, app: &Application) {
    window.set_default_size(1250, 950);
    window.set_title(Some("Leaf Alpha31 Regression"));
    let mut s = Session::new(wordcraft_doc::Document::from_text(
        "Comment passage.
Change passage.
I recieve text and recieve more.
I recieve text.
I recieve text.",
    ));
    wordcraft_gnome::proofing_defaults::apply(&mut s, "en-US", true).unwrap();
    selection::select_range(&mut s, &Pos::body(0, 0), &Pos::body(0, 7)).unwrap();
    wordcraft_gnome::annotations::add_comment(&mut s, "Retain this conversation.").unwrap();
    s.run("review.trackChanges", &json!({"value":true}))
        .unwrap();
    selection::set_caret(&mut s, &Pos::body(1, 0), false).unwrap();
    s.run("text.insert", &json!({"text":"Suggested "})).unwrap();
    s.run("review.trackChanges", &json!({"value":false}))
        .unwrap();
    opened(ui, s, "Incremental review");
    let review = ui.review.borrow().as_ref().unwrap().clone();
    review.show(None);
    let (u, w, a) = (ui.clone(), window.clone(), app.clone());
    let visited = Rc::new(Cell::new(0u8));
    let visited_signal = visited.clone();
    u.representation.connect_selected_notify(move |d| {
        visited_signal.set(visited_signal.get() | (1 << d.selected()))
    });
    let phase = Cell::new(0u32);
    let attempts = Cell::new(0u32);
    let comment: RefCell<Option<gtk::Widget>> = RefCell::new(None);
    let change: RefCell<Option<gtk::Widget>> = RefCell::new(None);
    let retained: RefCell<Option<gtk::Widget>> = RefCell::new(None);
    let original: RefCell<Option<wordcraft_doc::Document>> = RefCell::new(None);
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        attempts.set(attempts.get() + 1);
        assert!(attempts.get() < 300, "Alpha31 stalled at {}", phase.get());
        let spell = u.spelling.borrow().as_ref().unwrap().clone();
        match phase.get() {
            0 => {
                if spell.source.borrow().is_some() || spell.debounce.borrow().is_some() {
                    return glib::ControlFlow::Continue;
                }
                review.refresh();
                assert!(
                    spell
                        .issues
                        .borrow()
                        .iter()
                        .filter(|i| i.text == "recieve")
                        .count()
                        >= 4
                );
                comment.replace(find_widget(&review.content, "comment-thread-0"));
                change.replace(find_widget(&review.content, "review-change-0"));
                assert!(comment.borrow().is_some() && change.borrow().is_some());
                let card = spell.review_cards().last().unwrap().1.clone();
                retained.replace(Some(card));
                original.replace(Some(
                    u.current.borrow().as_ref().unwrap().session.doc.clone(),
                ));
                let index = spell
                    .issues
                    .borrow()
                    .iter()
                    .position(|i| i.text == "recieve")
                    .unwrap();
                let apply = find_widget(&review.content, &format!("proofing-apply-{index}"))
                    .unwrap()
                    .downcast::<gtk::Button>()
                    .unwrap();
                assert!(apply.is_sensitive());
                apply.grab_focus();
                apply.emit_clicked();
                // Resolved entry is removed before the background rescan publishes.
                assert!(!spell.issues.borrow().is_empty());
                review.refresh();
                assert!(
                    !find_widget(&u.navigation_pages, "review-status")
                        .is_some_and(|w| w.is_visible())
                );
                assert_eq!(
                    find_widget(&review.content, "comment-thread-0"),
                    *comment.borrow()
                );
                assert_eq!(
                    find_widget(&review.content, "review-change-0"),
                    *change.borrow()
                );
                assert!(
                    spell
                        .review_cards()
                        .iter()
                        .any(|(_, w)| Some(w) == retained.borrow().as_ref())
                );
                eprintln!("ALPHA31_INCREMENTAL_PENDING_OK");
            }
            1 => {
                review.refresh();
                assert_eq!(
                    find_widget(&review.content, "comment-thread-0"),
                    *comment.borrow()
                );
                let target = spell
                    .issues
                    .borrow()
                    .iter()
                    .find(|i| i.text == "recieve")
                    .unwrap()
                    .clone();
                proof_replace(&u, &target, "🙂 receive a longer phrase");
                review.refresh();
                assert!(
                    spell
                        .issues
                        .borrow()
                        .iter()
                        .filter(|i| i.text == "recieve")
                        .count()
                        >= 2
                );
                // Every retained destination points to exactly its current text.
                for issue in spell.issues.borrow().iter() {
                    let c = u.current.borrow();
                    assert_eq!(
                        c.as_ref()
                            .unwrap()
                            .session
                            .doc
                            .para_at(&issue.start)
                            .unwrap()
                            .text
                            .get(issue.start.off..issue.end.off),
                        Some(issue.text.as_str())
                    );
                }
                apply_user_edit(&u, Edit::Undo);
                apply_user_edit(&u, Edit::Undo);
            }
            2 => {
                if spell.source.borrow().is_some() || spell.debounce.borrow().is_some() {
                    return glib::ControlFlow::Continue;
                }
                assert_eq!(
                    u.current.borrow().as_ref().unwrap().session.doc,
                    *original.borrow().as_ref().unwrap()
                );
                assert!(
                    spell
                        .issues
                        .borrow()
                        .iter()
                        .filter(|i| i.text == "recieve")
                        .count()
                        >= 4
                );
                u.navigation_button.set_active(true);
                u.representation.set_selected(1);
                let before = u.current.borrow().as_ref().unwrap().session.sel.clone();
                u.representation.set_selected(0);
                assert_eq!(u.current.borrow().as_ref().unwrap().session.sel, before);
            }
            3 => {
                check_headers(&u, &w);
                let end = u
                    .current
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .session
                    .doc
                    .end_of(wordcraft_doc::StoryRef::Body);
                selection::set_caret(
                    &mut u.current.borrow_mut().as_mut().unwrap().session,
                    &end,
                    false,
                )
                .unwrap();
                apply_user_edit(&u, Edit::Insert(" qqqhiddenreviewzz".into()));
                assert!(spell.checked.get().is_none());
                visited.set(0);
                let r = u.representation.compute_bounds(&w).unwrap();
                let (tx, ty) = w.surface_transform();
                let scale = w.scale_factor() as f64;
                eprintln!(
                    "ALPHA31_NAV_READY {} {}",
                    ((r.x() as f64 + r.width() as f64 / 2. + tx) * scale) as i32,
                    ((r.y() as f64 + r.height() as f64 / 2. + ty) * scale) as i32
                );
            }
            4 => {
                let ack = std::env::var_os("LEAF29_ACK_DIR")
                    .map(PathBuf::from)
                    .unwrap()
                    .join("alpha31-navigation");
                if !ack.exists() {
                    return glib::ControlFlow::Continue;
                }
                assert_eq!(
                    visited.get(),
                    3,
                    "mouse/keyboard visited both representations"
                );
                assert_eq!(u.representation.selected(), 0);
                u.review.borrow().as_ref().unwrap().show(None);
            }
            5 => {
                if spell.checked.get() != document_identity(&u) {
                    return glib::ControlFlow::Continue;
                }
                assert!(
                    spell
                        .issues
                        .borrow()
                        .iter()
                        .any(|i| i.text == "qqqhiddenreviewzz"),
                    "reopening Review must scan edits made while hidden"
                );
                check_headers(&u, &w);
                let text = format_button(&u, "text-menu").unwrap();
                text.popup();
                let tools = u.text_tools.borrow().as_ref().unwrap().clone();
                let before = u.current.borrow().as_ref().unwrap().session.doc.clone();
                for page in [
                    "underline",
                    "strike",
                    "ink",
                    "highlight",
                    "font",
                    "language",
                ] {
                    tools.page(page);
                    assert!(tools.header.arrow.is_visible());
                    tools.header.button.emit_clicked();
                    assert_eq!(
                        tools.stack.visible_child_name().as_deref(),
                        Some("main"),
                        "Back from {page}"
                    );
                }
                assert_eq!(u.current.borrow().as_ref().unwrap().session.doc, before);
                tools.home();
                assert!(
                    find_widget(&text.popover().unwrap(), "text-font-property")
                        .unwrap()
                        .ancestor(adw::SplitButton::static_type())
                        .is_some()
                );
                eprintln!("ALPHA31_FORMAT_READY");
            }
            6..=25 => {} // Leave time for diagnostic capture/input driver.
            _ => {
                eprintln!("ALPHA31_UI_OK");
                a.quit();
                return glib::ControlFlow::Break;
            }
        }
        phase.set(phase.get() + 1);
        glib::ControlFlow::Continue
    });
}
fn check_headers(ui: &Rc<Ui>, window: &ApplicationWindow) {
    let a = find_widget(&ui.sidebar, "sidebar-header-region")
        .unwrap()
        .compute_bounds(window)
        .unwrap();
    let b = find_widget(&ui.sidebar, "document-header-region")
        .unwrap()
        .compute_bounds(window)
        .unwrap();
    assert!(
        (a.y() + a.height() - b.y() - b.height()).abs() < 1.1,
        "header bounds: {a:?} vs {b:?}"
    );
}
