//! Actual folder chooser, live refresh and persistent access controls for #84.
use super::*;
use wordcraft_gnome::dictionaries as sources;

async fn pause(ms: u64) {
    glib::timeout_future(std::time::Duration::from_millis(ms)).await;
}
async fn ready(panel: &Rc<DictionaryTools>, stage: usize, role: &str, name: &str) {
    let button = find_widget(&panel.window, name).unwrap();
    button.grab_focus();
    pause(120).await;
    let r = button.compute_bounds(&panel.window).unwrap();
    let (tx, ty) = panel.window.surface_transform();
    let scale = panel.window.scale_factor() as f64;
    let x = ((r.x() as f64 + r.width() as f64 / 2. + tx) * scale).round() as i32;
    let y = ((r.y() as f64 + r.height() as f64 / 2. + ty) * scale).round() as i32;
    eprintln!("DICTIONARY84_READY {stage} {role} {x} {y}");
    let ack = PathBuf::from(std::env::var_os("LEAF84_ACK").unwrap()).join(stage.to_string());
    for _ in 0..12000 {
        if ack.exists() {
            pause(250).await;
            return;
        }
        pause(10).await;
    }
    panic!("dictionary driver did not acknowledge stage {stage}");
}
pub(super) fn dictionaries84_ui_test(ui: &Rc<Ui>, app: &Application, mode: &'static str) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous(info);
        std::process::exit(1);
    }));
    let (ui, app) = (ui.clone(), app.clone());
    glib::spawn_future_local(async move {
        let tag = if mode == "native" { "nl-NL" } else { "it-IT" };
        let mut s = Session::new(wordcraft_doc::Document::from_text(
            "foglio foglioprovaextra qqqleaferrore",
        ));
        wordcraft_gnome::proofing_defaults::apply(&mut s, tag, true).unwrap();
        opened(&ui, s, "Dictionary verification");
        pause(200).await;
        let original = {
            let c = ui.current.borrow();
            let s = &c.as_ref().unwrap().session;
            (
                s.doc.clone(),
                s.sel.clone(),
                s.undo_depth(),
                s.dirty,
                wordcraft_gnome::preferences::default_language(),
            )
        };
        let panel = show_dictionary_tools(&ui).unwrap();
        panel.window.set_title(Some("Leaf Dictionary Probe"));
        pause(180).await;
        if std::env::var_os("FLATPAK_ID").is_some() {
            assert!(find_widget(&panel.window, "dictionary-system-settings").is_none());
        }
        match mode {
            "native" => {
                eprintln!(
                    "DICTIONARY84_NATIVE_BEFORE {:?}",
                    sources::dictionary_path(tag)
                );
                ready(&panel, 0, "install-system", "dictionary-refresh").await;
                assert!(
                    sources::dictionary_path(tag)
                        .is_some_and(|p| p.starts_with("/usr/share/hunspell")
                            || p.starts_with("/usr/share/myspell"))
                );
                assert!(sources::available(tag));
                let text = ui.text_tools.borrow().as_ref().unwrap().clone();
                text.page("language");
                assert!(
                    find_widget(&text.detail, "language-choice-nl-NL")
                        .or_else(|| find_widget(&text.detail, "language-choice-nl"))
                        .is_some()
                );
                eprintln!("DICTIONARY84_NATIVE_SYSTEM_OK dynamic_language=true");
            }
            "reopen" => {
                let source = sources::folder().expect("selected folder must survive restart");
                assert!(sources::dictionary_path(tag).is_some_and(|p| p.starts_with(&source)));
                assert!(wordcraft_engine::proof::it::is_correct("foglioprovanuova"));
                assert!(!wordcraft_engine::proof::it::is_correct("qqqleaferrore"));
                eprintln!("DICTIONARY84_REOPEN_OK persisted_access=true");
                ready(&panel, 0, "forget", "dictionary-folder-forget").await;
                assert!(sources::folder().is_none());
                assert!(!sources::dictionary_path(tag).is_some_and(|p| p.starts_with(&source)));
                assert!(sources::available("es-ES"));
                eprintln!("DICTIONARY84_FORGET_OK fallback=true");
            }
            _ => {
                assert!(sources::folder().is_none());
                ready(&panel, 0, "choose", "dictionary-folder-choose").await;
                for _ in 0..300 {
                    if sources::folder().is_some() {
                        break;
                    }
                    pause(10).await;
                }
                let source =
                    sources::folder().expect("real chooser must install a dictionary folder");
                assert!(sources::dictionary_path(tag).is_some_and(|p| p.starts_with(&source)));
                assert!(sources::available(tag));
                assert!(wordcraft_engine::proof::it::is_correct("foglioprovaextra"));
                assert!(!wordcraft_engine::proof::it::is_correct("qqqleaferrore"));
                assert!(!wordcraft_engine::proof::it::is_correct("foglioprovanuova"));
                assert!(sources::available("es-ES"));
                let text = ui.text_tools.borrow().as_ref().unwrap().clone();
                text.page("language");
                assert!(
                    !find_widget(&text.detail, "language-choice-it-IT")
                        .unwrap()
                        .has_css_class("dim-label")
                );
                eprintln!("DICTIONARY84_FOLDER_OK actual_source={source:?}");
                ready(&panel, 1, "update-and-refresh", "dictionary-refresh").await;
                assert!(wordcraft_engine::proof::it::is_correct("foglioprovanuova"));
                assert!(!wordcraft_engine::proof::it::is_correct("qqqleaferrore"));
                eprintln!("DICTIONARY84_REFRESH_OK new_word=true");
                ready(&panel, 2, "cancel", "dictionary-folder-choose").await;
                assert_eq!(sources::folder(), Some(source.clone()));
                eprintln!("DICTIONARY84_CANCEL_OK");
                ready(&panel, 3, "close-owner", "dictionary-folder-choose").await;
                panel.window.close();
                pause(200).await;
                assert!(ui.dictionary_tools.borrow().is_none());
                assert!(ui.dictionary_chooser.borrow().is_none());
                assert_eq!(sources::folder(), Some(source));
                eprintln!("DICTIONARY84_OWNER_CLEANUP_OK");
            }
        }
        {
            let c = ui.current.borrow();
            let s = &c.as_ref().unwrap().session;
            assert_eq!(s.doc, original.0);
            assert_eq!(s.sel, original.1);
            assert_eq!(s.undo_depth(), original.2);
            assert_eq!(s.dirty, original.3);
            assert_eq!(wordcraft_gnome::preferences::default_language(), original.4);
        }
        eprintln!("DICTIONARY84_UI_OK mode={mode} document_unchanged=true");
        app.quit();
    });
}
