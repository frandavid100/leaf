use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{open_document, proofing_defaults as d, saving, selection};

fn para(s: &mut Session, p: usize) {
    let len = s.doc.para_at(&Pos::body(p, 0)).unwrap().len();
    selection::select_range(s, &Pos::body(p, 0), &Pos::body(p, len)).unwrap();
}
#[test]
fn defaults_preserve_explicit_fragments_and_restore_inheritance_with_undo() {
    let mut s = Session::new(Document::from_text("heredado\nexception\nsin revisar"));
    d::apply(&mut s, "es-ES", true).unwrap();
    para(&mut s, 1);
    d::text(&mut s, "en-GB").unwrap();
    para(&mut s, 2);
    d::text(&mut s, "off").unwrap();
    let selection = s.sel.clone();
    let mark = s.undo_depth();
    d::apply(&mut s, "fr-FR", false).unwrap();
    assert_eq!(s.undo_depth(), mark + 1);
    assert_eq!(s.sel, selection);
    let props = |s: &Session, n| {
        let p = s.doc.para_at(&Pos::body(n, 0)).unwrap();
        s.doc
            .styles
            .resolve_char(p.props.style.as_deref(), p.props_of_char(0))
    };
    assert_eq!(props(&s, 0).lang.as_deref(), Some("fr-FR"));
    assert!(props(&s, 0).no_proof);
    assert_eq!(props(&s, 1).lang.as_deref(), Some("en-GB"));
    assert!(!props(&s, 1).no_proof);
    assert!(props(&s, 2).no_proof);
    para(&mut s, 1);
    d::text(&mut s, "default").unwrap();
    assert_eq!(d::choice(&s), "default");
    assert!(props(&s, 1).no_proof);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(props(&s, 1).lang.as_deref(), Some("en-GB"));
    assert!(!props(&s, 1).no_proof);
}
#[test]
fn docx_keeps_defaults_exceptions_and_disabled_review_while_odt_is_guarded() {
    let mut s = Session::new(Document::from_text("heredado\nEnglish exception"));
    d::apply(&mut s, "es-ES", false).unwrap();
    para(&mut s, 1);
    d::text(&mut s, "en-US").unwrap();
    let path = std::env::temp_dir().join(format!(
        "leaf-proof-inheritance-{}.docx",
        std::process::id()
    ));
    saving::save(&mut s, &path).unwrap();
    let mut reopened = open_document(&path).unwrap();
    assert_eq!(d::language(&reopened), "es-ES");
    assert!(!d::review(&reopened));
    para(&mut reopened, 0);
    assert_eq!(d::choice(&reopened), "default");
    para(&mut reopened, 1);
    assert_eq!(d::choice(&reopened), "en-US");
    assert!(saving::format_preservation_error(&reopened, &path.with_extension("odt")).is_some());
    std::fs::remove_file(path).unwrap();
}
#[test]
fn changing_defaults_is_protected_and_caret_default_uses_inheritance() {
    let mut s = Session::new(Document::from_text("text"));
    d::apply(&mut s, "es-ES", true).unwrap();
    d::text(&mut s, "en-US").unwrap();
    d::text(&mut s, "default").unwrap();
    assert!(s.pending.as_ref().unwrap().lang.is_none());
    assert!(s.pending.as_ref().unwrap().lang_bidi.is_none());
    assert!(s.pending.as_ref().unwrap().lang_ea.is_none());
    assert!(s.pending.as_ref().unwrap().no_proof.is_none());
    s.doc.settings.protection = Some("readOnly".into());
    let before = s.doc.clone();
    assert!(d::apply(&mut s, "fr-FR", false).is_err());
    assert_eq!(before, s.doc);
}
#[test]
fn effective_review_follows_inheritance_and_explicit_yes_in_body_and_headers() {
    use wordcraft_gnome::{
        header_footer::{self as hf, Region},
        proofing,
    };
    proofing::initialize();
    let mut s = Session::new(Document::from_text(
        "El qqqinheritedzz texto.\nThe qqqexplicityeszz word.\nEl qqqoffzz texto.",
    ));
    d::apply(&mut s, "es-ES", false).unwrap();
    para(&mut s, 1);
    d::text(&mut s, "en-US").unwrap();
    para(&mut s, 2);
    d::text(&mut s, "off").unwrap();
    hf::enter(&mut s, 0, Region::Header).unwrap();
    s.run("text.insert", &json!({"text":"El qqqheaderzz texto."}))
        .unwrap();
    let header = s.sel.focus.story;
    hf::close(&mut s).unwrap();
    let issues = proofing::issues(&s);
    assert!(issues.iter().any(|i| i.text == "qqqexplicityeszz"));
    assert!(
        !issues
            .iter()
            .any(|i| ["qqqinheritedzz", "qqqoffzz", "qqqheaderzz"].contains(&i.text.as_str()))
    );
    d::apply(&mut s, "es-ES", true).unwrap();
    let issues = proofing::issues(&s);
    assert!(issues.iter().any(|i| i.text == "qqqinheritedzz"));
    assert!(
        issues
            .iter()
            .any(|i| i.text == "qqqheaderzz" && i.start.story == header)
    );
    assert!(!issues.iter().any(|i| i.text == "qqqoffzz"));
    s.run("edit.undo", &json!({})).unwrap();
    assert!(!d::review(&s));
    assert!(!proofing::issues(&s).iter().any(|i| i.text == "qqqheaderzz"));
}
