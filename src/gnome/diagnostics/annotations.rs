use super::*;
use wordcraft_gnome::annotations as model;

fn form(ui: &Rc<Ui>) -> AnnotationDialog {
    let window = ui.annotation_dialog.borrow().as_ref().unwrap().clone();
    AnnotationDialog {
        name: find_widget(&window, "bookmark-name")
            .and_then(|w| w.downcast().ok())
            .unwrap_or_default(),
        text: find_widget(&window, "comment-text")
            .and_then(|w| w.downcast().ok())
            .unwrap_or_default(),
        submit: find_widget(&window, "annotation-submit")
            .unwrap()
            .downcast()
            .unwrap(),
        error: find_widget(&window, "annotation-error")
            .unwrap()
            .downcast()
            .unwrap(),
        window,
    }
}

fn composer(ui: &Rc<Ui>) -> (gtk::Widget, gtk::TextView, gtk::Button) {
    let root = find_widget(&ui.navigation_pages, "comment-composer").unwrap();
    let text = find_widget(&root, "comment-text")
        .unwrap()
        .downcast()
        .unwrap();
    let submit = find_widget(&root, "annotation-submit")
        .unwrap()
        .downcast()
        .unwrap();
    (root, text, submit)
}

pub(super) fn annotations_ui_test(ui: &Rc<Ui>, parent: &ApplicationWindow, app: &Application) {
    let toggle = find_widget(&ui.sidebar_pages, "sidebar-annotations")
        .unwrap()
        .downcast::<gtk::ToggleButton>()
        .unwrap();
    let theme = gtk::IconTheme::for_display(&ui.document_box.display());
    assert!(theme.has_icon(toggle.icon_name().as_deref().unwrap()));
    opened(
        ui,
        Session::new(wordcraft_doc::Document::from_text(
            "Café con emoji 🦊.\n\u{c}Segundo párrafo para revisar.",
        )),
        "Annotations",
    );
    {
        let mut value = ui.current.borrow_mut();
        let d = value.as_mut().unwrap();
        d.session
            .run("para.heading1", &serde_json::json!({}))
            .unwrap();
        selection::select_range(&mut d.session, &Pos::body(0, 0), &Pos::body(0, 5)).unwrap();
    }
    gio::prelude::ActionGroupExt::activate_action(parent, "insert-bookmark", None);
    let bookmark = form(ui);
    assert!(!bookmark.submit.is_sensitive());
    bookmark.name.set_text("Café_1");
    bookmark.submit.emit_clicked();
    assert!(!bookmark.window.is_visible());
    assert_eq!(
        model::bookmarks(&ui.current.borrow().as_ref().unwrap().session),
        ["Café_1"]
    );
    assert_eq!(
        ui.navigation_pages.visible_child_name().as_deref(),
        Some("outline")
    );
    assert!(find_widget(&ui.navigation_pages, "bookmark-go-Café_1").is_some());
    let go = find_widget(&ui.navigation_pages, "bookmark-go-Café_1")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
    let label = go
        .child()
        .unwrap()
        .last_child()
        .unwrap()
        .downcast::<gtk::Label>()
        .unwrap();
    assert!(label.has_css_class("bookmark-name"));
    for state in [
        gtk::StateFlags::NORMAL,
        gtk::StateFlags::PRELIGHT,
        gtk::StateFlags::FOCUSED,
        gtk::StateFlags::SELECTED,
    ] {
        label.set_state_flags(state, true);
        assert_eq!(
            label.pango_context().font_description().unwrap().weight(),
            gtk::pango::Weight::Normal
        );
    }
    label.unset_state_flags(
        gtk::StateFlags::PRELIGHT | gtk::StateFlags::FOCUSED | gtk::StateFlags::SELECTED,
    );
    let heading = find_widget(&ui.navigation_pages, "outline-heading-0").unwrap();
    assert_eq!(
        heading.next_sibling(),
        find_widget(&ui.navigation_pages, "outline-bookmark-Café_1")
    );
    let expander = heading
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
    expander.emit_clicked();
    assert!(find_widget(&ui.navigation_pages, "bookmark-go-Café_1").is_none());
    find_widget(&ui.navigation_pages, "outline-heading-0")
        .unwrap()
        .first_child()
        .unwrap()
        .first_child()
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    let rev = ui.current.borrow().as_ref().unwrap().session.rev();
    find_widget(&ui.navigation_pages, "bookmark-go-Café_1")
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(), rev);
    let duplicate = show_annotation_dialog(ui, AnnotationEdit::Bookmark).unwrap();
    duplicate.name.set_text("Café_1");
    duplicate.submit.emit_clicked();
    assert!(!duplicate.error.text().is_empty());
    duplicate.window.close();

    let pop = show_document_context(ui, parent, 100, 150, DocumentContext::Text);
    assert!(context_model_row(&pop, wordcraft_gnome::t!("Add Bookmark…")).is_some());
    context_activate_row(&pop, wordcraft_gnome::t!("Add Comment…"));
    let (_, text, submit) = composer(ui);
    assert!(!submit.is_sensitive());
    // Navigating while composing must keep the original first-page anchor.
    {
        let mut current = ui.current.borrow_mut();
        selection::set_caret(
            &mut current.as_mut().unwrap().session,
            &Pos::body(1, 1),
            false,
        )
        .unwrap();
    }
    text.buffer().set_text("Review this café 🦊");
    submit.emit_clicked();
    let root = model::comments(&ui.current.borrow().as_ref().unwrap().session)[0].id;
    let panel = ui.annotations.borrow().as_ref().unwrap().clone();
    panel.refresh();
    let reply = find_widget(&ui.navigation_pages, &format!("comment-reply-{root}"))
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
    reply.emit_clicked();
    let (draft, text, submit) = composer(ui);
    assert_eq!(
        draft.prev_sibling().unwrap().widget_name(),
        format!("comment-{root}")
    );
    text.buffer().set_text("Checked the wording.");
    submit.emit_clicked();
    let reply_id = model::comments(&ui.current.borrow().as_ref().unwrap().session)
        .iter()
        .find(|c| c.parent == Some(root))
        .unwrap()
        .id;
    panel.refresh();
    let comments_view = find_widget(&ui.navigation_pages, "comments-panel").unwrap();
    assert!(find_widget(&comments_view, "bookmark-go-Café_1").is_none());
    let thread = find_widget(&ui.navigation_pages, &format!("comment-thread-{root}")).unwrap();
    assert!(thread.has_css_class("comment-thread"));
    assert!(!thread.has_css_class("card"));
    for id in [root, reply_id] {
        let message = find_widget(&ui.navigation_pages, &format!("comment-{id}")).unwrap();
        assert!(!message.has_css_class("card"));
        assert_eq!(
            message.parent(),
            Some(thread.clone()),
            "one bubble per thread"
        );
    }
    assert!(
        thread
            .first_child()
            .unwrap()
            .next_sibling()
            .unwrap()
            .is::<gtk::Separator>()
    );
    let options =
        find_widget(&ui.navigation_pages, &format!("comment-options-{reply_id}")).unwrap();
    options.activate_action("annotation.edit", None).unwrap();
    let edit = form(ui);
    edit.text
        .buffer()
        .set_text("Checked the wording.\nSecond line with an accent: á.");
    edit.submit.emit_clicked();
    panel.refresh();
    let options = find_widget(&ui.navigation_pages, &format!("comment-options-{root}")).unwrap();
    options.activate_action("annotation.resolve", None).unwrap();
    assert!(model::comments(&ui.current.borrow().as_ref().unwrap().session)[0].resolved);
    apply_user_edit(ui, Edit::Undo);
    assert!(!model::comments(&ui.current.borrow().as_ref().unwrap().session)[0].resolved);
    apply_user_edit(ui, Edit::Redo);
    panel.refresh();
    let rev = ui.current.borrow().as_ref().unwrap().session.rev();
    find_widget(&ui.navigation_pages, &format!("comment-go-{reply_id}"))
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.rev(), rev);

    // Switching to search must not lose the third navigation tab or its content.
    let search = ui.search.borrow().as_ref().unwrap().clone();
    search.show(false);
    panel.show();
    assert!(!search.toggle.is_active());
    assert_eq!(
        ui.navigation_pages.visible_child_name().as_deref(),
        Some("annotations")
    );

    // Replies have their own inline position and retain their parent.
    find_widget(&ui.navigation_pages, &format!("comment-reply-{reply_id}"))
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap()
        .emit_clicked();
    let (draft, text, submit) = composer(ui);
    assert_eq!(
        draft.prev_sibling().unwrap().widget_name(),
        format!("comment-{reply_id}")
    );
    text.buffer().set_text("Nested reply");
    submit.emit_clicked();
    assert!(
        model::comments(&ui.current.borrow().as_ref().unwrap().session)
            .iter()
            .any(|comment| comment.parent == Some(reply_id))
    );

    // Add a thread on the second page and select each thread through its content.
    {
        let mut current = ui.current.borrow_mut();
        selection::select_range(
            &mut current.as_mut().unwrap().session,
            &Pos::body(1, 1),
            &Pos::body(1, 8),
        )
        .unwrap();
    }
    gio::prelude::ActionGroupExt::activate_action(parent, "insert-comment", None);
    let (_, text, submit) = composer(ui);
    text.buffer().set_text("Second page");
    submit.emit_clicked();
    let second = model::comments(&ui.current.borrow().as_ref().unwrap().session)
        .into_iter()
        .find(|comment| comment.text == "Second page")
        .unwrap()
        .id;
    for (id, page) in [(second, 1), (root, 0)] {
        find_widget(&ui.navigation_pages, &format!("comment-go-{id}"))
            .unwrap()
            .downcast::<gtk::Button>()
            .unwrap()
            .emit_clicked();
        assert_eq!(*panel.active.borrow(), HashSet::from([id]));
        let mut current = ui.current.borrow_mut();
        let session = &mut current.as_mut().unwrap().session;
        let focus = session.sel.focus.clone();
        let hint = session.page_hint;
        assert_eq!(session.layout().caret_on(&focus, hint).unwrap().page, page);
    }

    // Replacing a document invalidates its pending composer.
    compose_comment(ui, None);
    let (_, text, stale_submit) = composer(ui);
    text.buffer()
        .set_text("Must not be attached to another document");
    let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    opened(ui, Session::new(original.clone()), "Replacement");
    panel.show();
    stale_submit.emit_clicked();
    assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, original);
    assert!(find_widget(&ui.navigation_pages, "comment-composer").is_none());
    if std::env::args().any(|a| a == "--bookmarks-demo") {
        ui.outline.borrow().as_ref().unwrap().show();
        eprintln!("ANNOTATIONS_DEMO_READY");
        return;
    }
    if std::env::args().any(|a| a == "--annotations-demo") {
        eprintln!("ANNOTATIONS_DEMO_READY");
        return;
    }
    let dialog = show_annotation_dialog(ui, AnnotationEdit::Bookmark).unwrap();
    release_document(ui);
    assert!(
        !dialog.window.is_visible(),
        "closing a tab closes its annotation editor"
    );
    eprintln!("ANNOTATIONS_UI_OK");
    app.quit();
}
