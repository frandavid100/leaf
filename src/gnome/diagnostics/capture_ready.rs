//! Opt-in screenshot handshake. Ordinary application and test timing is unchanged.
use super::*;
thread_local! { static PENDING: Cell<usize> = const { Cell::new(0) }; }
fn enabled() -> bool {
    std::env::var_os("LEAF_CAPTURE_READY").is_some()
}
pub(super) fn after(delay: std::time::Duration, callback: impl FnOnce() + 'static) {
    let capture = enabled();
    if capture {
        PENDING.with(|v| v.set(v.get() + 1));
    }
    glib::timeout_add_local_once(delay, move || {
        callback();
        if capture {
            PENDING.with(|v| v.set(v.get() - 1));
        }
    });
}
fn fingerprint(
    widget: &gtk::Widget,
    renderer: &gtk::gsk::Renderer,
    hash: &mut impl std::hash::Hasher,
) {
    if !widget.is_mapped() {
        return;
    }
    if widget.is::<ApplicationWindow>() || widget.is::<gtk::Popover>() || widget.is::<adw::Dialog>()
    {
        let snapshot = gtk::Snapshot::new();
        gtk::WidgetPaintable::new(Some(widget)).snapshot(
            &snapshot,
            widget.width() as f64,
            widget.height() as f64,
        );
        if let Some(node) = snapshot.to_node() {
            let texture = renderer.render_texture(&node, None);
            let stride = texture.width() as usize * 4;
            let mut pixels = vec![0; stride * texture.height() as usize];
            texture.download(&mut pixels, stride);
            hash.write(&pixels);
        }
    }
    let mut child = widget.first_child();
    while let Some(widget) = child {
        fingerprint(&widget, renderer, hash);
        child = widget.next_sibling();
    }
}
pub(super) fn install(ui: &Rc<Ui>, window: &ApplicationWindow) {
    if !enabled() {
        return;
    }
    let done = Rc::new(Cell::new(false));
    let finished = done.clone();
    let weak = Rc::downgrade(ui);
    window.add_tick_callback(move |window, _| {
        if finished.get() || weak.upgrade().is_none_or(|u| u.closed.get()) {
            return glib::ControlFlow::Break;
        }
        window.queue_draw();
        glib::ControlFlow::Continue
    });
    let weak = Rc::downgrade(ui);
    let clock = window.frame_clock().unwrap();
    let signal = Rc::new(RefCell::new(None));
    let cleanup = signal.clone();
    let previous = Cell::new(None);
    let stable = Cell::new(0);
    let id = clock.connect_after_paint(move |clock| {
        use std::hash::Hasher;
        let ui = weak.upgrade().filter(|u| !u.closed.get());
        let finish = if let Some(ui) = ui {
            if PENDING.with(Cell::get) != 0 || ui.layout_pending.get() {
                return;
            }
            let current = ui.current.borrow();
            if current.as_ref().is_none_or(|c| c.painted_pages.is_empty()) {
                return;
            }
            drop(current);
            let Some(window) = ui.host.upgrade().map(|h| h.window.clone()) else {
                return;
            };
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            fingerprint(window.upcast_ref(), &window.renderer().unwrap(), &mut hash);
            let value = hash.finish();
            stable.set(if previous.get() == Some(value) {
                stable.get() + 1
            } else {
                0
            });
            previous.set(Some(value));
            if stable.get() < 3 {
                return;
            }
            eprintln!("LEAF_CAPTURE_READY");
            true
        } else {
            true
        };
        if finish {
            done.set(true);
            if let Some(id) = cleanup.borrow_mut().take() {
                clock.disconnect(id);
            }
        }
    });
    signal.replace(Some(id));
}
