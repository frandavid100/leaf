//! Native clipboard ownership, format negotiation and cancellation for one tab.
use super::{Ui, caret_ensure_visible, document_identity, tools_change};
use adw::{Toast, prelude::*};
use gtk::glib::value::ToValue;
use gtk::{gdk, glib};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

const RICH_MIME_TYPES: &[&str] = &["text/html", "text/rtf", "application/rtf"];

pub(super) fn clipboard_can_paste(ui: &Ui) -> bool {
    let formats = ui.document_box.clipboard().formats();
    formats.contains_type(String::static_type())
        || ["text/plain", "text/plain;charset=utf-8"]
            .iter()
            .chain(RICH_MIME_TYPES)
            .any(|mime| formats.contain_mime_type(mime))
}

// Share the engine fragment between tabs only while GTK still owns this exact
// clipboard provider. Never paste an old fragment after another app copies.
thread_local! {
    static CLIPBOARD_WATCHED: Cell<bool> = const { Cell::new(false) };
    pub(super) static RICH_CLIPBOARD: RefCell<Option<(gdk::ContentProvider, wordcraft_doc::edit::Fragment)>> = const { RefCell::new(None) };
}
pub(super) fn context_copy(ui: &Rc<Ui>) -> bool {
    if !CLIPBOARD_WATCHED.with(|v| v.replace(true)) {
        ui.document_box.clipboard().connect_changed(|clipboard| {
            RICH_CLIPBOARD.with(|c| {
                let mut c = c.borrow_mut();
                if c.as_ref()
                    .is_some_and(|(provider, _)| clipboard.content().as_ref() != Some(provider))
                {
                    c.take();
                }
            });
        });
    }
    let result = {
        let mut value = ui.current.borrow_mut();
        let Some(d) = value.as_mut() else {
            return false;
        };
        if d.session.sel.is_collapsed() {
            return false;
        }
        d.session
            .run("edit.copy", &serde_json::json!({}))
            .map(|v| (v, d.session.clipboard.clone()))
    };
    let (value, fragment) = match result {
        Ok(v) => v,
        Err(e) => {
            ui.toasts.add_toast(Toast::new(&e.to_string()));
            return false;
        }
    };
    let text = value["text"].as_str().unwrap_or_default();
    let mut providers = vec![gdk::ContentProvider::for_value(&text.to_value())];
    for (key, mime) in [
        ("html", "text/html"),
        ("rtf", "text/rtf"),
        ("mathml", "application/mathml+xml"),
    ] {
        if let Some(data) = value[key].as_str().filter(|data| !data.is_empty()) {
            providers.push(gdk::ContentProvider::for_bytes(
                mime,
                &glib::Bytes::from_owned(data.as_bytes().to_vec()),
            ));
        }
    }
    let provider = gdk::ContentProvider::new_union(&providers);
    if let Err(e) = ui.document_box.clipboard().set_content(Some(&provider)) {
        ui.toasts.add_toast(Toast::new(&e.to_string()));
        return false;
    }
    RICH_CLIPBOARD.with(|c| c.replace(fragment.map(|f| (provider, f))));
    true
}
pub(super) fn context_paste(ui: &Rc<Ui>) {
    cancel_clipboard_read(ui);
    let clipboard = ui.document_box.clipboard();
    let fragment = RICH_CLIPBOARD.with(|c| {
        c.borrow()
            .as_ref()
            .filter(|(p, _)| clipboard.content().as_ref() == Some(p))
            .map(|(_, f)| f.clone())
    });
    if let Some(fragment) = fragment {
        tools_change(ui, move |s| {
            s.clipboard_text = fragment.clipboard_text();
            s.clipboard = Some(fragment);
            s.run("edit.paste", &serde_json::json!({}))
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        ui.document_box.grab_focus();
        caret_ensure_visible(ui);
        return;
    }
    let identity = document_identity(ui);
    let selection = ui.current.borrow().as_ref().map(|d| d.session.sel.clone());
    let weak = Rc::downgrade(ui);
    let task = glib::spawn_future_local(async move {
        let result = external_clipboard(&clipboard).await;
        let Some(ui) = weak.upgrade() else { return };
        ui.clipboard_read.borrow_mut().take();
        if ui.closed.get()
            || document_identity(&ui) != identity
            || ui.current.borrow().as_ref().map(|d| d.session.sel.clone()) != selection
        {
            return;
        }
        match result {
            Ok(Some(args)) => {
                tools_change(&ui, move |session| {
                    session
                        .run("edit.paste", &args)
                        .map(|_| ())
                        .map_err(|error| error.to_string())
                });
            }
            Ok(None) => {}
            Err(error) => ui.toasts.add_toast(Toast::new(&error.to_string())),
        }
        ui.document_box.grab_focus();
        caret_ensure_visible(&ui);
    });
    ui.clipboard_read.replace(Some(task));
}

pub(super) fn cancel_clipboard_read(ui: &Ui) {
    if let Some(task) = ui.clipboard_read.borrow_mut().take() {
        task.abort();
    }
}

/// Prefer supported rich formats; fall back to GDK's native text decoding.
async fn external_clipboard(
    clipboard: &gdk::Clipboard,
) -> Result<Option<serde_json::Value>, glib::Error> {
    if let Some(args) = rich_clipboard(clipboard).await {
        return Ok(Some(args));
    }
    Ok(clipboard
        .read_text_future()
        .await?
        .filter(|text| !text.is_empty())
        .map(|text| serde_json::json!({"text":text.as_str()})))
}

async fn rich_clipboard(clipboard: &gdk::Clipboard) -> Option<serde_json::Value> {
    const MAX_RICH_BYTES: usize = 8 * 1024 * 1024;
    let formats = clipboard.formats();
    if !RICH_MIME_TYPES
        .iter()
        .any(|mime| formats.contain_mime_type(mime))
    {
        return None;
    }
    let (stream, mime) = clipboard
        .read_future(RICH_MIME_TYPES, glib::Priority::DEFAULT)
        .await
        .ok()?;
    let mut data = Vec::new();
    loop {
        let chunk = stream
            .read_bytes_future(64 * 1024, glib::Priority::DEFAULT)
            .await
            .ok()?;
        if chunk.is_empty() {
            break;
        }
        if data.len() + chunk.len() > MAX_RICH_BYTES {
            return None;
        }
        data.extend_from_slice(&chunk);
    }
    let _ = stream.close_future(glib::Priority::DEFAULT).await;
    let text = String::from_utf8(data).ok()?;
    if text.is_empty() {
        return None;
    }
    Some(if mime == "text/html" {
        serde_json::json!({"html":text})
    } else {
        serde_json::json!({"rtf":text})
    })
}
