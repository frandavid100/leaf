use super::*;

pub(super) fn run() {
    wordcraft_gnome::i18n::init();
    if diagnostics::startup_commands::run_command() {
        return;
    }
    if diagnostics::flag("--version") {
        println!("Leaf {APP_VERSION}");
        return;
    }
    if diagnostics::flag("--language-data-licenses") {
        assert!(
            wordcraft_engine::proof::packs::embedded_locales().contains(&"en-us"),
            "release must embed its pinned Language data"
        );
        for (title, text) in wordcraft_engine::proof::packs::licence_texts() {
            println!("{title}\n\n{text}\n");
        }
        return;
    }
    glib::set_application_name("Leaf");
    let input_files: Vec<_> = std::env::args_os()
        .skip(1)
        .filter(|a| !a.to_string_lossy().starts_with("--"))
        .collect();
    let sample_on_start = diagnostics::flag("--test-document");
    let sidebar_on_start = diagnostics::flag("--sidebar");
    let narrow_on_start = diagnostics::flag("--narrow");
    let dark_on_start = diagnostics::flag("--dark");
    let selection_demo = diagnostics::flag("--select-demo");
    let editing_demo = diagnostics::flag("--edit-demo");
    let formatting_demo = diagnostics::flag("--format-demo");
    // Permite probar versiones simultáneas sin activar una ventana anterior por D-Bus.
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_startup(|_| {
        if let Some(display) = gdk::Display::default() {
            let icons = gtk::IconTheme::for_display(&display);
            if let Ok(exe) = std::env::current_exe() {
                if let Some(root) = exe.parent().and_then(Path::parent) {
                    icons.add_search_path(root.join("share/icons"));
                }
            }
            icons.add_search_path(Path::new(env!("CARGO_MANIFEST_DIR")).join("data/icons"));
        }
        gtk::Window::set_default_icon_name(APP_ID);
        apply_theme(wordcraft_gnome::preferences::theme());
    });
    app.connect_activate(move |app| {
        if !input_files.is_empty() {
            for argument in &input_files {
                let file = gio::File::for_commandline_arg(argument);
                let result = file
                    .path()
                    .ok_or_else(|| {
                        wordcraft_gnome::t!("Only local files are supported").to_string()
                    })
                    .and_then(|path| {
                        open_document(&path).map(|session| {
                            (
                                session,
                                path.file_name()
                                    .map(|s| s.to_string_lossy().into_owned())
                                    .unwrap_or_else(|| wordcraft_gnome::t!("Document").into()),
                            )
                        })
                    });
                match result {
                    Ok(initial) => {
                        if diagnostics::flag("--launch-file-test") {
                            assert!(initial.1.contains("launch-test"));
                            assert!(
                                initial
                                    .0
                                    .doc
                                    .plain_text(wordcraft_doc::StoryRef::Body)
                                    .contains("Leaf launch test")
                            );
                        }
                        let window = make_window(
                            app,
                            false,
                            true,
                            false,
                            false,
                            false,
                            false,
                            false,
                            Some(initial),
                        );
                        if diagnostics::flag("--launch-file-test") {
                            let app = app.clone();
                            glib::timeout_add_local_once(
                                std::time::Duration::from_millis(1600),
                                move || {
                                    assert!(
                                        window.is_visible(),
                                        "external document window is visible"
                                    );
                                    eprintln!("LAUNCH_FILE_OK");
                                    app.quit();
                                },
                            );
                        }
                    }
                    Err(error) => {
                        if diagnostics::flag("--launch-file-test") {
                            panic!("external file failed: {error}");
                        }
                        let window =
                            make_window(app, false, true, false, false, false, false, false, None);
                        let dialog = adw::AlertDialog::new(
                            Some(wordcraft_gnome::t!("Could Not Open Document")),
                            Some(&error),
                        );
                        dialog.add_response("close", wordcraft_gnome::t!("Close"));
                        dialog.present(Some(&window));
                    }
                }
            }
            return;
        }
        if diagnostics::flag("--multiwindow-test") {
            diagnostics::startup_commands::multiwindow_test(app);
            return;
        }
        make_window(
            app,
            sample_on_start,
            sidebar_on_start,
            narrow_on_start,
            dark_on_start,
            selection_demo,
            editing_demo,
            formatting_demo,
            None,
        );
    });
    app.connect_shutdown(|_| {
        shutdown_workspaces();
        wordcraft_gnome::dictionaries::shutdown();
    });
    app.run_with_args(&["wordcraft-gnome"]);
}
