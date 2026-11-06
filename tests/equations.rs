use serde_json::json;
use wordcraft_doc::{Document, InlineObject, StoryRef};
use wordcraft_engine::Session;
use wordcraft_gnome::{equations, open_document, pdf_export, saving};
#[test]
fn native_equation_subset_edits_structured_math_and_preserves_docx_history() {
    let mut s = Session::new(Document::from_text("An equation: "));
    let before = s.doc.clone();
    let depth = s.undo_depth();
    equations::apply(&mut s, "(a+b)/(c+d)+x^2+x_1+√(x)", true).unwrap();
    let (pos, text, display, editable) = equations::target(&s).unwrap();
    assert!(display && editable);
    assert!(text.contains('/'));
    assert!(
        matches!(s.doc.para_at(&pos).unwrap().object_at(pos.off),Some(InlineObject::Equation{math,..})if !math.nodes.is_empty())
    );
    assert_eq!(s.undo_depth(), depth + 1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    s.run("edit.redo", &json!({})).unwrap();
    let old = s.doc.clone();
    assert!(equations::apply(&mut s, "(a+b", false).is_err());
    assert_eq!(s.doc, old);
    equations::apply(&mut s, "x^3+√(y)", false).unwrap();
    let path = std::env::temp_dir().join(format!("leaf29-equation-{}.docx", std::process::id()));
    saving::save(&mut s, &path).unwrap();
    let mut copy = open_document(&path).unwrap();
    let p = copy
        .doc
        .para_paths(StoryRef::Body)
        .into_iter()
        .find_map(|path| {
            let p = copy.doc.para(StoryRef::Body, &path)?;
            p.object_offsets()
                .into_iter()
                .find(|o| matches!(p.object_at(*o), Some(InlineObject::Equation { .. })))
                .map(|off| wordcraft_doc::Pos {
                    story: StoryRef::Body,
                    path,
                    off,
                })
        })
        .unwrap();
    wordcraft_gnome::selection::set_caret(&mut copy, &p, false).unwrap();
    let (_, text, _, _) = equations::target(&copy).unwrap();
    assert!(text.contains("x"));
    pdf_export::export(&mut copy, &path.with_extension("pdf")).unwrap();
    assert!(saving::format_preservation_error(&copy, &path.with_extension("odt")).is_some());
    std::fs::remove_file(path.with_extension("pdf")).unwrap();
    std::fs::remove_file(path).unwrap();
}

#[test]
fn imported_math_with_unrepresentable_properties_is_protected_from_simplification() {
    use wordcraft_doc::math::{MNode, MRun, MScr};
    let mut s = Session::new(Document::from_text(
        "Original equation and surrounding text",
    ));
    equations::apply(&mut s, "x", false).unwrap();
    let (pos, _, _, _) = equations::target(&s).unwrap();
    let p = s.doc.para_mut(pos.story, &pos.path).unwrap();
    if let InlineObject::Equation { math, .. } = p.object_at_mut(pos.off).unwrap() {
        let mut run = MRun::new("x");
        run.scr = MScr::DoubleStruck;
        math.nodes = vec![MNode::Run(run)];
    }
    p.touch();
    s.touch();
    assert!(!equations::target(&s).unwrap().3);
    let original = s.doc.clone();
    assert!(equations::apply(&mut s, "x^2", false).is_err());
    assert_eq!(s.doc, original);
}
