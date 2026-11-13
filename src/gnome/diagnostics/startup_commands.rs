//! Test commands handled before the application starts, and startup checks.
use super::*;

/// Runs a diagnostic command given on the command line, if any.
/// Returns true when one ran and Leaf should exit without opening a window.
pub(in crate::gnome) fn run_command() -> bool {
    if super::flag("--workflows38-prepare-startup") {
        super::workflows38_checks::prepare_startup();
        return true;
    }
    if super::flag("--workflows35-template-restart-test") {
        super::workflows35_checks::template_restart();
        return true;
    }
    if super::flag("--workflows36-template-create") {
        let folder = PathBuf::from(std::env::var_os("LEAF36_TEMPLATE_SOURCE").unwrap());
        std::fs::create_dir_all(&folder).unwrap();
        let mut session = Session::new(wordcraft_doc::Document::from_text("Read-only template"));
        wordcraft_gnome::saving::save(&mut session, &folder.join("Personal.docx")).unwrap();
        return true;
    }
    if super::flag("--workflows36-template-restart-test") {
        let folder = wordcraft_gnome::templates::folder().unwrap();
        let session = wordcraft_gnome::templates::open(&folder.join("Personal.docx")).unwrap();
        assert_eq!(
            session.doc.plain_text(wordcraft_doc::StoryRef::Body),
            "Read-only template"
        );
        assert!(session.file_name.is_none());
        if std::env::var_os("FLATPAK_ID").is_some() {
            assert!(
                std::fs::OpenOptions::new()
                    .write(true)
                    .open(folder.join("Personal.docx"))
                    .is_err(),
                "portal must deny writing to the source"
            );
        }
        println!(
            "WORKFLOWS36_TEMPLATE_RESTART_OK sandbox_read_only={}",
            std::env::var_os("FLATPAK_ID").is_some()
        );
        return true;
    }
    if super::flag("--windows-package-test") {
        super::windows_checks::package_test();
        return true;
    }
    if super::flag("--package-storage-test") {
        let root = wordcraft_gnome::preferences::recovery_root();
        let data = std::env::var_os("XDG_DATA_HOME").expect("sandbox data home");
        assert!(
            root.starts_with(Path::new(&data)),
            "recovery stays in persistent sandbox data"
        );
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("package-test"), b"persistent").unwrap();
        wordcraft_gnome::preferences::set_autosave_mode(0).unwrap();
        assert_eq!(wordcraft_gnome::preferences::autosave_mode(), 0);
        println!("PACKAGE_STORAGE_OK");
        return true;
    }
    if super::flag("--issue-preferences-read-test") {
        assert_eq!(wordcraft_gnome::preferences::user_name(), "Leaf QA");
        assert_eq!(wordcraft_gnome::preferences::theme(), 2);
        println!("ISSUE_PREFERENCES_RESTART_OK");
        return true;
    }
    if super::flag("--workflows35-drag-source") {
        super::workflows35_checks::drag_source();
        return true;
    }
    false
}

/// Opens two windows, closes one and checks that the other survives.
pub(in crate::gnome) fn multiwindow_test(app: &Application) {
    let first = make_window(
        app,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        Some((sample_session(), "Primero".into())),
    );
    let _second = make_window(
        app,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        Some((sample_session(), "Segundo".into())),
    );
    let app = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(1000), move || {
        if app.windows().len() != 2 {
            std::process::exit(1);
        }
        first.close();
        glib::timeout_add_local_once(std::time::Duration::from_millis(150), move || {
            if app.windows().len() != 1 {
                std::process::exit(1);
            }
            eprintln!("MULTIWINDOW_OK");
            app.quit();
        });
    });
}
