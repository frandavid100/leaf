//! Legacy synchronous probes wait for the adaptive dialog's real close lifecycle.
//! GtkWidget::visible stays true after an AdwDialog is unparented; it cannot
//! describe whether the modal surface is open. New workflow probes use async waits.
use super::*;

/// Await GTK frames without recursively polling GLib's future executor.
/// Async probes must not call the legacy nested-main-loop helpers below.
pub(in crate::gnome) async fn dialog_presented_async(dialog: &impl IsA<adw::Dialog>) {
    let dialog = dialog.as_ref();
    let frames = Rc::new(Cell::new(0));
    let count = frames.clone();
    dialog.add_tick_callback(move |_, _| {
        count.set(count.get() + 1);
        if count.get() >= 4 {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
    for _ in 0..150 {
        if dialog.is_mapped() && dialog.root().is_some() && frames.get() >= 4 {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(20)).await;
    }
    eprintln!(
        "DIALOG_PRESENT_TIMEOUT mapped={} root={} frames={} visible={} size={}x{}",
        dialog.is_mapped(),
        dialog.root().is_some(),
        frames.get(),
        dialog.is_visible(),
        dialog.width(),
        dialog.height()
    );
    panic!(
        "adaptive dialog did not become interactive: {:?}",
        dialog.title()
    );
}

pub(in crate::gnome) async fn dialog_closed_async(dialog: &impl IsA<adw::Dialog>) {
    let dialog = dialog.as_ref();
    for _ in 0..150 {
        if dialog.root().is_none() && !dialog.is_mapped() {
            return;
        }
        glib::timeout_future(std::time::Duration::from_millis(20)).await;
    }
    panic!(
        "adaptive dialog did not finish closing: {:?}",
        dialog.title()
    );
}

pub(in crate::gnome) fn wait_dialog_presented(dialog: &impl IsA<adw::Dialog>) {
    let dialog = dialog.as_ref();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    let context = glib::MainContext::default();
    // libadwaita 1.5 opens its sheet after the first mapping ticks. A mapped
    // widget alone is not yet a dialog a user can interact with.
    let frames = Rc::new(Cell::new(0));
    let count = frames.clone();
    dialog.add_tick_callback(move |_, _| {
        count.set(count.get() + 1);
        if count.get() >= 4 {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
    while (!dialog.is_mapped() || frames.get() < 4) && std::time::Instant::now() < deadline {
        while context.pending() {
            context.iteration(false);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        dialog.is_mapped() && dialog.root().is_some() && frames.get() >= 4,
        "adaptive dialog did not become interactive: {:?}",
        dialog.title()
    );
}

pub(in crate::gnome) fn assert_dialog_closed(dialog: &impl IsA<adw::Dialog>) {
    let dialog = dialog.as_ref();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    let context = glib::MainContext::default();
    while dialog.root().is_some() && std::time::Instant::now() < deadline {
        while context.pending() {
            context.iteration(false);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(
        dialog.root().is_none() && !dialog.is_mapped(),
        "adaptive dialog did not finish closing: title={:?} mapped={} visible={} parent={:?} root={:?}",
        dialog.title(),
        dialog.is_mapped(),
        dialog.is_visible(),
        dialog.parent().map(|p| p.type_().name()),
        dialog.root().map(|r| r.type_().name())
    );
}
