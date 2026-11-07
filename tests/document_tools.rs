use serde_json::json;
use wordcraft_doc::{Document, Pos};
use wordcraft_engine::Session;
use wordcraft_gnome::{
    document_properties::{self, PageSetup},
    open_document, pdf_export, pictures, saving,
};
fn folder() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "wordcraft-tools-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}
fn fixture(p: &std::path::Path) -> std::path::PathBuf {
    let f = p.join("imagen.png");
    std::fs::write(
        &f,
        wordcraft_engine::cmd::insert::base64_decode(include_str!("fixtures/picture.b64")).unwrap(),
    )
    .unwrap();
    f
}
#[test]
fn inline_picture_history_selection_and_roundtrips() {
    let dir = folder();
    let file = fixture(&dir);
    let mut s = Session::new(Document::from_text("Antes y después"));
    s.run("caret.set", &json!({"pos":Pos::body(0,6)})).unwrap();
    let depth = s.undo_depth();
    pictures::insert(&mut s, &file).unwrap();
    let (pos, w, h) = pictures::selected(&s).unwrap();
    assert!((w / h - 2.0).abs() < 0.01);
    assert!(
        matches!(wordcraft_engine::cmd::objects::object_selection(&s).unwrap().1,wordcraft_doc::para::InlineObject::Image{float,..}if float.wrap==wordcraft_doc::para::Wrap::Inline)
    );
    assert_eq!(s.undo_depth(), depth + 1);
    let rev = s.rev();
    pictures::select(&mut s, &pos).unwrap();
    assert_eq!(s.rev(), rev);
    pictures::resize(&mut s, w * 2.0, h, true, true).unwrap();
    let (_, nw, nh) = pictures::selected(&s).unwrap();
    assert!((nw / nh - 2.0).abs() < 0.01);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(pictures::selected(&s).unwrap().1, w);
    s.run("edit.redo", &json!({})).unwrap();
    pictures::original(&mut s).unwrap();
    assert_eq!(pictures::selected(&s).unwrap().1, 120.0);
    pictures::resize(&mut s, 90.0, 60.0, false, true).unwrap();
    pictures::replace(&mut s, &file).unwrap();
    assert_eq!(pictures::selected(&s).unwrap().1, 90.0);
    for ext in ["docx", "odt"] {
        let path = dir.join(format!("copia.{ext}"));
        saving::save(&mut s, &path).unwrap();
        let mut reopened = open_document(&path).unwrap();
        reopened.run("select.objects", &json!({"index":0})).unwrap();
        let (_, rw, rh) = pictures::selected(&reopened).unwrap();
        assert!((rw - 90.0).abs() < 0.2 && (rh - 60.0).abs() < 0.2);
        assert!(!reopened.doc.media.is_empty());
    }
    pictures::delete(&mut s).unwrap();
    assert!(pictures::selected(&s).is_none());
    s.run("edit.undo", &json!({})).unwrap();
    assert!(pictures::selected(&s).is_some());
    s.run("select.all", &json!({})).unwrap();
    let doc = s.doc.clone();
    assert!(pictures::delete(&mut s).is_err());
    assert_eq!(s.doc, doc);
    let bad = dir.join("rota.png");
    std::fs::write(&bad, b"not an image").unwrap();
    let depth = s.undo_depth();
    assert!(pictures::insert(&mut s, &bad).is_err());
    assert_eq!(s.doc, doc);
    assert_eq!(s.undo_depth(), depth);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn page_setup_preserves_sections_and_groups_undo() {
    let mut s = Session::new(Document::from_text("Primera\nSegunda"));
    s.run("caret.set", &json!({"pos":Pos::body(1,0)})).unwrap();
    s.run("layout.break", &json!({"kind":"nextPage"})).unwrap();
    s.run("layout.columns", &json!({"count":2,"space":20}))
        .unwrap();
    assert!(s.doc.sections().len() > 1);
    let sections: Vec<_> = s
        .doc
        .sections()
        .into_iter()
        .map(|(_, p)| p.clone())
        .collect();
    let sel = s.sel.clone();
    let doc = s.doc.clone();
    let depth = s.undo_depth();
    let setup = PageSetup {
        width: 841.89,
        height: 595.28,
        top: 36.0,
        bottom: 42.0,
        left: 45.0,
        right: 48.0,
    };
    assert!(document_properties::apply(&mut s, setup, true).unwrap());
    assert_eq!(s.sel, sel);
    assert_eq!(s.undo_depth(), depth + 1);
    for ((_, p), old) in s.doc.sections().into_iter().zip(&sections) {
        assert_eq!(p.columns, old.columns);
        assert_eq!(p.header, old.header);
        assert!(p.landscape);
        assert_eq!(p.margin_top, 36.0);
    }
    let applied = s.doc.clone();
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, doc);
    assert_eq!(s.sel, sel);
    s.run("edit.redo", &json!({})).unwrap();
    assert_eq!(s.doc, applied);
    let invalid = PageSetup {
        left: 900.0,
        ..setup
    };
    let depth = s.undo_depth();
    assert!(document_properties::apply(&mut s, invalid, true).is_err());
    assert_eq!(s.doc, applied);
    assert_eq!(s.undo_depth(), depth);
    assert!(!document_properties::apply(&mut s, setup, true).unwrap());
    assert_eq!(s.undo_depth(), depth);
    let before = s.doc.sections()[0].1.clone();
    let changed = PageSetup { top: 60.0, ..setup };
    document_properties::apply(&mut s, changed, false).unwrap();
    assert_eq!(s.doc.sections()[0].1, &before);
}
#[test]
fn pdf_export_preserves_editable_destination_dirty_selection_and_history() {
    let dir = folder();
    let img = fixture(&dir);
    let mut s = Session::new(Document::from_text("Documento mínimo con imagen"));
    pictures::insert(&mut s, &img).unwrap();
    let path = dir.join("original.docx");
    saving::save(&mut s, &path).unwrap();
    pictures::resize(&mut s, 90.0, 60.0, false, true).unwrap();
    let (destination, doc, sel, dirty, depth, rev) = (
        s.path.clone(),
        s.doc.clone(),
        s.sel.clone(),
        s.dirty,
        s.undo_depth(),
        s.rev(),
    );
    let pdf = dir.join("salida.pdf");
    pdf_export::export(&mut s, &pdf).unwrap();
    let data = std::fs::read(&pdf).unwrap();
    assert!(data.starts_with(b"%PDF-"));
    assert!(data.len() > 1000);
    assert_eq!(s.path, destination);
    assert_eq!(s.doc, doc);
    assert_eq!(s.sel, sel);
    assert_eq!(s.dirty, dirty);
    assert_eq!(s.undo_depth(), depth);
    assert_eq!(s.rev(), rev);
    assert!(pdf_export::export(&mut s, &dir.join("no.docx")).is_err());
    assert!(!dir.join("no.docx").exists());
    assert!(pdf_export::export(&mut s, &dir.join("missing/out.pdf")).is_err());
    for ext in ["docx", "odt"] {
        let setup = PageSetup {
            width: 841.89,
            height: 595.28,
            top: 36.0,
            bottom: 36.0,
            left: 40.0,
            right: 40.0,
        };
        document_properties::apply(&mut s, setup, true).unwrap();
        let file = dir.join(format!("pagina.{ext}"));
        saving::save(&mut s, &file).unwrap();
        let reopened = open_document(&file).unwrap();
        let p = PageSetup::current(&reopened);
        assert!((p.width - setup.width).abs() < 0.2 && (p.left - 40.0).abs() < 0.2);
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn page_setup_preserves_pending_character_format() {
    let mut s = Session::new(Document::from_text("Texto"));
    s.run("format.bold", &json!({"value":true})).unwrap();
    let pending = s.pending.clone();
    assert!(pending.is_some());
    let p = PageSetup::current(&s);
    document_properties::apply(&mut s, PageSetup { top: 50.0, ..p }, true).unwrap();
    assert_eq!(s.pending, pending);
}

#[test]
fn floating_picture_wrap_spacing_history_and_docx() {
    let dir = folder();
    let image = fixture(&dir);
    let mut s = Session::new(Document::from_text(
        &"A paragraph flowing around the image. ".repeat(80),
    ));
    pictures::insert(&mut s, &image).unwrap();
    let pos = pictures::selected(&s).unwrap().0;
    let before = s.doc.clone();
    let depth = s.undo_depth();
    pictures::placement(&mut s, Some("right"), 8.).unwrap();
    assert_eq!(s.undo_depth(), depth + 1);
    let float = pictures::layout(&s).unwrap();
    assert_eq!(float.wrap, wordcraft_doc::para::Wrap::Square);
    assert_eq!(float.dist, 8.);
    assert!(wordcraft_engine::cmd::objects::moves_with_text(&float));
    let hit = s.layout().object(&pos, 0).unwrap();
    assert!(hit.rect.x > 200.);
    let path = dir.join("floating.docx");
    saving::save(&mut s, &path).unwrap();
    let mut copy = open_document(&path).unwrap();
    copy.run("select.objects", &json!({"index":0})).unwrap();
    assert_eq!(pictures::layout(&copy).unwrap().wrap, float.wrap);
    assert!((pictures::layout(&copy).unwrap().dist - 8.).abs() < 0.1);
    let odt = path.with_extension("odt");
    saving::save(&mut copy, &odt).unwrap();
    let mut odt_copy = open_document(&odt).unwrap();
    odt_copy.run("select.objects", &json!({"index":0})).unwrap();
    assert_eq!(pictures::layout(&odt_copy).unwrap().wrap, float.wrap);
    assert!((pictures::layout(&odt_copy).unwrap().dist - 8.).abs() < 0.1);
    s.run("edit.undo", &json!({})).unwrap();
    assert_eq!(s.doc, before);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn changing_a_laid_out_floating_picture_back_to_inline_reflows_text() {
    fn lines(s: &mut Session) -> Vec<(usize, usize, u32, u32, u32)> {
        s.layout()
            .pages
            .iter()
            .enumerate()
            .flat_map(|(page, p)| {
                p.items.iter().flat_map(move |item| match item {
                    wordcraft_engine::layout::Placed::Lines { para, l0, l1, .. } => para.lines
                        [*l0..*l1]
                        .iter()
                        .map(|l| {
                            (
                                page,
                                l.start,
                                l.height.to_bits(),
                                l.left.to_bits(),
                                l.right.to_bits(),
                            )
                        })
                        .collect::<Vec<_>>(),
                    _ => Vec::new(),
                })
            })
            .collect()
    }
    let dir = folder();
    let image = fixture(&dir);
    let mut s = Session::new(Document::from_text(
        &"Text around an imported floating image. ".repeat(60),
    ));
    pictures::insert(&mut s, &image).unwrap();
    s.run("arrange.wrap", &json!({"wrap":"behindText"}))
        .unwrap();
    let _ = lines(&mut s);
    pictures::placement(&mut s, Some("inline"), 0.).unwrap();
    let mut fresh = Session::new(s.doc.clone());
    assert_eq!(
        lines(&mut s),
        lines(&mut fresh),
        "cached layout must match a fresh layout after placement changes"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn inline_and_floating_pictures_move_to_the_destination_page_with_one_undo() {
    for floating in [false, true] {
        let dir = folder();
        let image = fixture(&dir);
        let mut s = Session::new(Document::from_text(
            "Source paragraph.\nDestination paragraph.",
        ));
        s.run("caret.set", &json!({"pos":Pos::body(1,0)})).unwrap();
        s.run("layout.break", &json!({"kind":"nextPage"})).unwrap();
        s.run("caret.set", &json!({"pos":Pos::body(0,0)})).unwrap();
        pictures::insert(&mut s, &image).unwrap();
        if floating {
            pictures::placement(&mut s, Some("left"), 8.).unwrap();
        }
        let pos = pictures::selected(&s).unwrap().0;
        let before = s.doc.clone();
        let depth = s.undo_depth();
        let layout = s.layout();
        assert!(layout.pages.len() >= 2);
        let page = &layout.pages[1];
        let at = layout
            .hit(
                1,
                page.body.x + 50.,
                page.body.y + 30.,
                wordcraft_doc::StoryRef::Body,
            )
            .unwrap();
        let args = if floating {
            json!({"page":1,"x":100.,"y":110.})
        } else {
            json!({"to":at.clone()})
        };
        pictures::drop_at(&mut s, 1, Some(&at), 100., 110.).unwrap();
        let moved = pictures::selected(&s).unwrap().0;
        assert_eq!(
            s.layout().object(&moved, 1).unwrap().page,
            1,
            "floating={floating} args={args} moved={moved:?} target_caret={:?}",
            layout.caret_on(&at, 1)
        );
        assert_eq!(
            pictures::layout(&s).unwrap().wrap,
            if floating {
                wordcraft_doc::para::Wrap::Square
            } else {
                wordcraft_doc::para::Wrap::Inline
            }
        );
        assert_eq!(s.undo_depth(), depth + 1);
        let path = dir.join("moved.docx");
        saving::save(&mut s, &path).unwrap();
        let mut copy = open_document(&path).unwrap();
        copy.run("select.objects", &json!({"index":0})).unwrap();
        let cp = pictures::selected(&copy).unwrap().0;
        assert_eq!(copy.layout().object(&cp, 1).unwrap().page, 1);
        pdf_export::export(&mut copy, &dir.join("moved.pdf")).unwrap();
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc, before);
        assert_eq!(s.layout().object(&pos, 0).unwrap().page, 0);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[test]
fn a_rejected_cross_page_anchor_restores_the_original_picture() {
    let dir = folder();
    let image = fixture(&dir);
    let mut s = Session::new(Document::from_text("Source\nDestination"));
    s.run("caret.set", &json!({"pos":Pos::body(1,0)})).unwrap();
    s.run("layout.break", &json!({"kind":"page"})).unwrap();
    s.run("caret.set", &json!({"pos":Pos::body(0,0)})).unwrap();
    pictures::insert(&mut s, &image).unwrap();
    pictures::placement(&mut s, Some("left"), 6.).unwrap();
    let before = s.doc.clone();
    let depth = s.undo_depth();
    assert!(pictures::drop_at(&mut s, 1, None, 100., 110.).is_err());
    assert_eq!(s.doc, before);
    assert_eq!(s.undo_depth(), depth);
    std::fs::remove_dir_all(dir).unwrap();
}
