//! Relocated Windows package check: no resources from the developer prefix.
use super::*;

pub(super) fn package_test() {
    gtk::init().unwrap();
    let executable = std::env::current_exe().unwrap();
    let prefix = executable.parent().unwrap().parent().unwrap();
    assert_eq!(wordcraft_gnome::dictionaries::bundled(), prefix.join("share/hunspell"));
    gtk::gdk_pixbuf::Pixbuf::from_file(
        prefix.join("share/icons/hicolor/scalable/apps/leaf.svg"),
    )
    .expect("the relocated package must decode SVG icons");
    wordcraft_gnome::dictionaries::refresh(true).unwrap();
    let dir = std::env::temp_dir().join(format!("Leaf documentos ñ {}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for ext in ["docx", "odt"] {
        let mut session = Session::new(wordcraft_doc::Document::from_text("Prueba ñ — Windows"));
        session.run("text.insert", &serde_json::json!({"text":"Edición "})).unwrap();
        let text = session.doc.plain_text(wordcraft_doc::StoryRef::Body);
        let path = dir.join(format!("Documento con espacios ñ.{ext}"));
        wordcraft_gnome::saving::save(&mut session, &path).unwrap();
        let reopened = open_document(&path).unwrap();
        assert_eq!(reopened.doc.plain_text(wordcraft_doc::StoryRef::Body), text);
        session.run("text.insert", &serde_json::json!({"text":"Segunda "})).unwrap();
        wordcraft_gnome::saving::save(&mut session, &path).unwrap();
        assert_eq!(
            open_document(&path).unwrap().doc.plain_text(wordcraft_doc::StoryRef::Body),
            session.doc.plain_text(wordcraft_doc::StoryRef::Body)
        );
        let pdf = dir.join(format!("Exportación ñ {ext}.pdf"));
        pdf_export::export(&mut session, &pdf).unwrap();
        assert!(std::fs::read(pdf).unwrap().starts_with(b"%PDF-"));
    }
    wordcraft_gnome::preferences::set_user_name("Prueba Windows").unwrap();
    wordcraft_gnome::preferences::set_theme(2).unwrap();
    wordcraft_gnome::preferences::set_theme(1).unwrap();
    assert_eq!(wordcraft_gnome::preferences::theme(), 1);
    assert_eq!(wordcraft_gnome::preferences::user_name(), "Prueba Windows");
    #[cfg(windows)]
    {
        let roaming = PathBuf::from(std::env::var_os("APPDATA").unwrap());
        let local = PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap());
        assert_eq!(wordcraft_gnome::preferences::config_root(), roaming.join("Leaf"));
        assert_eq!(wordcraft_gnome::preferences::recovery_root(), local.join("Leaf/recovery"));
    }
    std::fs::remove_dir_all(dir).unwrap();
    wordcraft_gnome::dictionaries::shutdown();
    eprintln!("WINDOWS_PACKAGE_RESOURCES_OK");
}
