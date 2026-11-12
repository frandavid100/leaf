//! Trailing debounce for views derived from the document.
//!
//! Panels such as Review, the outline and page thumbnails do not need to be
//! rebuilt on every keystroke. `Debounce::schedule` runs the refresh once no
//! further request has arrived for a full `delay`, so continuous typing costs
//! one refresh after the pause instead of one per character.
use std::cell::Cell;
use std::rc::{Rc, Weak};
use std::time::Duration;

use gtk::glib;

#[derive(Default)]
pub(super) struct Debounce {
    armed: Cell<bool>,
    requested_again: Cell<bool>,
}

impl Debounce {
    /// Requests `run(owner)` after `delay` without further requests.
    /// `field` returns this `Debounce` from its owner, so the timer only keeps a
    /// weak reference and stops if the owner is dropped.
    pub(super) fn schedule<T: 'static>(
        &self,
        owner: &Rc<T>,
        delay: Duration,
        field: fn(&T) -> &Debounce,
        run: fn(&Rc<T>),
    ) {
        if self.armed.replace(true) {
            self.requested_again.set(true);
            return;
        }
        arm(Rc::downgrade(owner), delay, field, run);
    }
}

fn arm<T: 'static>(owner: Weak<T>, delay: Duration, field: fn(&T) -> &Debounce, run: fn(&Rc<T>)) {
    glib::timeout_add_local_once(delay, move || {
        let Some(strong) = owner.upgrade() else {
            return;
        };
        let debounce = field(&strong);
        if debounce.requested_again.replace(false) {
            arm(owner, delay, field, run);
            return;
        }
        debounce.armed.set(false);
        run(&strong);
    });
}
