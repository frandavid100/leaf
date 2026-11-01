use super::*;

pub(super) fn print_ui_test(ui: &Rc<Ui>, parent: &ApplicationWindow, app: &Application) {
    let document = ui.current.borrow().as_ref().unwrap().session.doc.clone();
    let ui = ui.clone(); let parent = parent.clone(); let app = app.clone();
    gtk::prelude::WidgetExt::activate_action(&parent,"win.print", None).unwrap();
    // Duplicate activation must not create a second dialog/job.
    print_document(&ui, &parent);
    let attempts = Rc::new(Cell::new(0));
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        attempts.set(attempts.get()+1); assert!(attempts.get()<300, "print callback timed out");
        if let Some(outcome) = ui.printing.outcome.get() {
            assert_eq!(ui.current.borrow().as_ref().unwrap().session.doc, document);
            let expected = match std::env::var("LEAF_PRINT_TEST").as_deref() { Ok("cancel") => PrintOutcome::Dismissed, Ok("error") => PrintOutcome::Failed, _ => PrintOutcome::Submitted };
            assert_eq!(outcome, expected); assert!(ui.printing.pending.borrow().is_none());
            eprintln!("PRINT_UI_OK {outcome:?}"); app.quit(); return glib::ControlFlow::Break;
        }
        glib::ControlFlow::Continue
    });
}