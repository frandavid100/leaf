use super::*;

pub(super) fn run() {
    wordcraft_gnome::i18n::init();
    if std::env::args().any(|arg| arg == "--version") {
        println!("Leaf {APP_VERSION}");
        return;
    }
    if std::env::args().any(|a|a=="--package-storage-test") {
        let root=wordcraft_gnome::preferences::recovery_root();
        let data=std::env::var_os("XDG_DATA_HOME").expect("sandbox data home");
        assert!(root.starts_with(Path::new(&data)),"recovery stays in persistent sandbox data");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("package-test"),b"persistent").unwrap();
        wordcraft_gnome::preferences::set_autosave_mode(0).unwrap();
        assert_eq!(wordcraft_gnome::preferences::autosave_mode(),0);
        println!("PACKAGE_STORAGE_OK");return;
    }
    glib::set_application_name("Leaf");
    let input_files: Vec<_> = std::env::args_os().skip(1).filter(|a| !a.to_string_lossy().starts_with("--")).collect();
    let sample_on_start = std::env::args().any(|arg| arg == "--test-document");
    let sidebar_on_start = std::env::args().any(|arg| arg == "--sidebar");
    let narrow_on_start = std::env::args().any(|arg| arg == "--narrow");
    let dark_on_start = std::env::args().any(|arg| arg == "--dark");
    let selection_demo = std::env::args().any(|arg| arg == "--select-demo");
    let editing_demo = std::env::args().any(|arg| arg == "--edit-demo");
    let formatting_demo = std::env::args().any(|arg| arg == "--format-demo");
    // Permite probar versiones simultáneas sin activar una ventana anterior por D-Bus.
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_startup(|_|gtk::Window::set_default_icon_name(APP_ID));
    app.connect_activate(move |app| {
        if !input_files.is_empty() {
            for argument in &input_files {
                let file=gio::File::for_commandline_arg(argument);
                let result=file.path().ok_or_else(||wordcraft_gnome::t!("Only local files are supported").to_string())
                    .and_then(|path|open_document(&path).map(|session|(session,path.file_name().map(|s|s.to_string_lossy().into_owned()).unwrap_or_else(||wordcraft_gnome::t!("Document").into()))));
                match result {
                    Ok(initial)=>{
                        if std::env::args().any(|a|a=="--launch-file-test") {
                            assert!(initial.1.contains("launch-test"));
                            assert!(initial.0.doc.plain_text(wordcraft_doc::StoryRef::Body).contains("Leaf launch test"));
                        }
                        let window=make_window(app,false,true,false,false,false,false,false,Some(initial));
                        if std::env::args().any(|a|a=="--launch-file-test") {
                            let app=app.clone();
                            glib::timeout_add_local_once(std::time::Duration::from_millis(1600),move||{
                                assert!(window.is_visible(),"external document window is visible");
                                eprintln!("LAUNCH_FILE_OK");app.quit();
                            });
                        }
                    }
                    Err(error)=>{
                        if std::env::args().any(|a|a=="--launch-file-test"){panic!("external file failed: {error}");}
                        let window=make_window(app,false,true,false,false,false,false,false,None);
                        let dialog=adw::MessageDialog::new(Some(&window),Some(wordcraft_gnome::t!("Could Not Open Document")),Some(&error));
                        dialog.add_response("close",wordcraft_gnome::t!("Close"));dialog.present();
                    }
                }
            }
            return;
        }
        if std::env::args().any(|a| a == "--multiwindow-test") {
            let first = make_window(app, false, false, false, false, false, false, false,
                Some((sample_session(), "Primero".into())));
            let _second = make_window(app, false, false, false, false, false, false, false,
                Some((sample_session(), "Segundo".into())));
            let app = app.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(1000), move || {
                if app.windows().len() != 2 { std::process::exit(1); }
                first.close();
                glib::timeout_add_local_once(std::time::Duration::from_millis(150),move || {
                    if app.windows().len() != 1 { std::process::exit(1); }
                    eprintln!("MULTIWINDOW_OK");app.quit();
                });
            });
            return;
        }
        make_window(app, sample_on_start, sidebar_on_start, narrow_on_start, dark_on_start, selection_demo, editing_demo, formatting_demo, None);
    });
    app.connect_shutdown(|_|shutdown_workspaces());
    app.run_with_args(&["wordcraft-gnome"]);
}
