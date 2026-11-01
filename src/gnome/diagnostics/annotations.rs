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
            "Café con emoji 🦊.\nSegundo párrafo para revisar.",
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
    let comment = form(ui);
    comment.text.buffer().set_text("Review this café 🦊");
    comment.submit.emit_clicked();
    let root = model::comments(&ui.current.borrow().as_ref().unwrap().session)[0].id;
    let panel = ui.annotations.borrow().as_ref().unwrap().clone();
    panel.refresh();
    let reply = find_widget(&ui.navigation_pages, &format!("comment-reply-{root}"))
        .unwrap()
        .downcast::<gtk::Button>()
        .unwrap();
    reply.emit_clicked();
    let response = form(ui);
    response.text.buffer().set_text("Checked the wording.");
    response.submit.emit_clicked();
    let reply_id = model::comments(&ui.current.borrow().as_ref().unwrap().session)
        .iter()
        .find(|c| c.parent == Some(root))
        .unwrap()
        .id;
    panel.refresh();
    let comments_view = find_widget(&ui.navigation_pages, "comments-panel").unwrap();
    assert!(find_widget(&comments_view, "bookmark-go-Café_1").is_none());
    let thread = find_widget(&ui.navigation_pages, &format!("comment-thread-{root}")).unwrap();
    assert!(thread.has_css_class("card"));
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

    // A stale modal cannot attach its comment to a replacement document.
    let stale = show_annotation_dialog(ui, AnnotationEdit::NewComment).unwrap();
    stale
        .text
        .buffer()
        .set_text("Must not be attached to another document");
    let original = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    opened(ui, Session::new(original), "Replacement");
    stale.submit.emit_clicked();
    assert!(!stale.error.text().is_empty());
    assert!(stale.window.is_visible());
    stale.window.close();
    panel.show();
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
