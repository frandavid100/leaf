use serde_json::json;
use wordcraft_doc::{Document, Pos, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{proofing, selection};

#[test]
fn explicit_issue_list_and_navigation_work_with_live_checking_disabled() {
    proofing::initialize();
    let mut session = Session::new(Document::from_text("El qqqmanualzz examen."));
    proofing::language(&mut session, "es-ES", false, true).unwrap();
    session.view.proofing = false;
    session.prefs.check_spelling = false;
    let before = (
        session.doc.clone(),
        session.rev(),
        session.undo_depth(),
        session.dirty,
    );
    let issues = proofing::issues(&session);
    let issue = issues.iter().find(|i| i.text == "qqqmanualzz").unwrap();
    proofing::select(&mut session, issue).unwrap();
    assert_eq!(selection::copy_text(&mut session).unwrap(), "qqqmanualzz");
    assert_eq!(
        (
            session.doc.clone(),
            session.rev(),
            session.undo_depth(),
            session.dirty
        ),
        before
    );
    assert!(!session.view.proofing);
}

#[test]
fn english_suggestions_replacement_and_custom_dictionary() {
    proofing::initialize();
    let mut s = Session::new(Document::from_text("The qqqspelingzzz word."));
    proofing::language(&mut s, "en-US", false, true).unwrap();
    let issue = proofing::issues(&s)
        .into_iter()
        .find(|i| i.text == "qqqspelingzzz")
        .unwrap();
    let original = s.doc.clone();
    let depth = s.undo_depth();
    proofing::replace(&mut s, &issue, "example").unwrap();
    assert_eq!(s.doc.plain_text(StoryRef::Body), "The example word.");
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, original);
    let folder = std::env::temp_dir().join(format!("leaf-dictionary-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("custom.txt");
    s.user_dict_path = Some(path.clone());
    let (doc, rev, depth, dirty) = (s.doc.clone(), s.rev(), s.undo_depth(), s.dirty);
    proofing::add(&mut s, &issue).unwrap();
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("qqqspelingzzz")
    );
    assert!(
        proofing::issues(&s)
            .iter()
            .all(|i| i.text != "qqqspelingzzz")
    );
    assert_eq!(s.doc, doc);
    assert_eq!(s.rev(), rev);
    assert_eq!(s.undo_depth(), depth);
    assert_eq!(s.dirty, dirty);
    std::fs::remove_dir_all(folder).unwrap();
}
#[test]
fn real_spanish_dictionary_and_language_scopes() {
    proofing::initialize();
    assert!(
        wordcraft_engine::proof::has_dictionary(Some("es-ES")),
        "Spanish dictionary must be included"
    );
    let mut s = Session::new(Document::from_text("El profesor prepara un examen."));
    proofing::language(&mut s, "es-ES", false, true).unwrap();
    assert!(!proofing::issues(&s).iter().any(|i| i.spelling));
    let mut s = Session::new(Document::from_text("El qqqpalabrazxq examen."));
    proofing::language(&mut s, "es-ES", false, true).unwrap();
    assert!(
        proofing::issues(&s)
            .iter()
            .any(|i| i.text == "qqqpalabrazxq" && i.spelling)
    );
    proofing::language(&mut s, "es-ES", true, true).unwrap();
    assert!(proofing::issues(&s).is_empty());
}
#[test]
fn mixed_languages_and_no_proof_survive_docx_and_odt() {
    proofing::initialize();
    let mut s = Session::new(Document::from_text("Hello mundo."));
    selection::select_range(&mut s, &Pos::body(0, 6), &Pos::body(0, 11)).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    proofing::language(&mut s, "es-ES", false, false).unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    assert_eq!(proofing::state(&mut s)["lang"], "es-ES");
    for ext in ["docx", "odt"] {
        let name = format!("languages.{ext}");
        let bytes = wordcraft_engine::io::save_bytes(&name, &s.doc).unwrap();
        let out = wordcraft_engine::io::open_bytes(&name, &bytes).unwrap();
        let p = out.para_at(&Pos::body(0, 6)).unwrap();
        let c = out
            .styles
            .resolve_char(p.props.style.as_deref(), p.props_of_char(6));
        assert_eq!(c.lang.as_deref(), Some("es-ES"), "{ext}");
    }
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    proofing::language(&mut s, "en-GB", true, true).unwrap();
    for ext in ["docx", "odt"] {
        let name = format!("unchecked.{ext}");
        let bytes = wordcraft_engine::io::save_bytes(&name, &s.doc).unwrap();
        let out = wordcraft_engine::io::open_bytes(&name, &bytes).unwrap();
        let p = out.para_at(&Pos::body(0, 0)).unwrap();
        let c = out
            .styles
            .resolve_char(p.props.style.as_deref(), p.props_of_char(0));
        assert!(c.no_proof, "{ext}");
        assert_eq!(c.lang.as_deref(), Some("en-GB"), "{ext}");
        assert_eq!(out.styles.default_chr.lang.as_deref(), Some("en-GB"));
    }
}
#[test]
fn protected_and_stale_text_are_not_changed() {
    let mut s = Session::new(Document::from_text("Stale qqqwordzz."));
    let issue = proofing::issues(&s)
        .into_iter()
        .find(|i| i.text == "qqqwordzz")
        .unwrap();
    s.run("text.insert", &json!({"text":"prefix "})).unwrap();
    let doc = s.doc.clone();
    assert!(proofing::replace(&mut s, &issue, "changed").is_err());
    assert_eq!(s.doc, doc);
    s.run("review.restrict", &json!({"mode":"readOnly"}))
        .unwrap();
    let doc = s.doc.clone();
    let depth = s.undo_depth();
    assert!(proofing::language(&mut s, "es-ES", false, true).is_err());
    assert_eq!(s.doc, doc);
    assert_eq!(s.undo_depth(), depth);
}

#[test]
fn whole_document_language_updates_typing_without_losing_other_pending_format() {
    let mut s = Session::new(Document::from_text("Hello"));
    s.pending = Some(wordcraft_doc::CharProps {
        lang: Some("es-ES".into()),
        bold: Some(true),
        ..Default::default()
    });
    proofing::language(&mut s, "en-GB", false, true).unwrap();
    assert_eq!(s.pending.as_ref().unwrap().lang.as_deref(), Some("en-GB"));
    assert_eq!(s.pending.as_ref().unwrap().bold, Some(true));
    assert_eq!(s.pending.as_ref().unwrap().no_proof, Some(false));
}
