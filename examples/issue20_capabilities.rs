//! Focused, synthetic public-API audit for the fifth issue sweep.
use serde_json::json;
use std::path::Path;
use wordcraft_doc::{Document, InlineObject, PartKind, Pos, StoryRef};
use wordcraft_engine::Session;
fn fields(s: &Session) -> usize {
    let mut n = 0;
    s.doc.objects_in_reading_order(&mut |o|if matches!(o,InlineObject::FieldStart{instr,..}|InlineObject::Field{instr,..} if instr.trim_start().starts_with("TOC")){n+=1;});
    n
}
fn save(s: &mut Session, name: &str, ext: &str) -> Session {
    let path = format!("/tmp/leaf20-capabilities/{name}.{ext}");
    s.run("file.saveAs", &json!({"path":path})).unwrap();
    wordcraft_gnome::open_document(Path::new(&path)).unwrap()
}
fn main() {
    std::fs::create_dir_all("/tmp/leaf20-capabilities").unwrap();
    // Real equation structures, never rendered-image substitution.
    let mut s = Session::new(Document::from_text("Before after"));
    s.run("caret.set", &json!({"pos":Pos::body(0,7)})).unwrap();
    s.run(
        "insert.equation",
        &json!({"linear":"(a+b)/c + x^2 + √(x)","display":false}),
    )
    .unwrap();
    let before = s.run("equation.get", &json!({})).unwrap();
    println!("MATH_BEFORE {before}");
    s.run("equation.exit", &json!({})).unwrap();
    for ext in ["docx", "odt"] {
        let mut copy = save(&mut s, "math", ext);
        copy.run("caret.set", &json!({"pos":Pos::body(0,7)}))
            .unwrap();
        let mut equations = Vec::new();
        for path in copy.doc.para_paths(StoryRef::Body) {
            let para = copy.doc.para(StoryRef::Body, &path).unwrap();
            for (off, o) in para.objects_with_offsets() {
                if matches!(o, InlineObject::Equation { .. }) {
                    equations.push((path.clone(), off, serde_json::to_value(o).unwrap()));
                }
            }
        }
        println!(
            "MATH_{ext}_OBJECTS {}",
            serde_json::to_string(&equations).unwrap()
        );
        let result = copy.run("equation.get", &json!({}));
        println!("MATH_{ext} {result:?}");
        wordcraft_gnome::pdf_export::export(
            &mut copy,
            Path::new(&format!("/tmp/leaf20-capabilities/math-{ext}.pdf")),
        )
        .unwrap();
    }
    // Notes have distinct engine-owned text stories and automatic marks.
    let mut s = Session::new(Document::from_text("Body paragraph"));
    let result = s
        .run("references.footnote", &json!({"text":"Footnote body"}))
        .unwrap();
    println!("NOTE_INSERT {result} caret={:?}", s.sel.focus);
    for ext in ["docx", "odt"] {
        let mut copy = save(&mut s, "note", ext);
        let notes = copy.run("references.notes", &json!({}));
        let count = copy
            .doc
            .parts
            .values()
            .filter(|p| p.kind == PartKind::Footnote)
            .count();
        println!("NOTE_{ext} count={count} notes={notes:?}");
        wordcraft_gnome::pdf_export::export(
            &mut copy,
            Path::new(&format!("/tmp/leaf20-capabilities/note-{ext}.pdf")),
        )
        .unwrap();
    }
    // A multipage automatic TOC must survive as a TOC, not only as its text.
    let text = (0..100)
        .map(|n| format!("Paragraph {n}. {}", "Ordinary body text. ".repeat(12)))
        .collect::<Vec<_>>()
        .join("\n");
    let mut s = Session::new(Document::from_text(&text));
    for n in [0, 40, 80] {
        s.run("caret.set", &json!({"pos":Pos::body(n,0)})).unwrap();
        s.run("para.heading1", &json!({})).unwrap();
    }
    s.run("caret.set", &json!({"pos":Pos::body(0,0)})).unwrap();
    s.run(
        "references.toc",
        &json!({"levels":3,"pageNumbers":true,"hyperlinks":true}),
    )
    .unwrap();
    println!(
        "TOC_BEFORE fields={} pages={}",
        fields(&s),
        s.layout().pages.len()
    );
    for ext in ["docx", "odt"] {
        let mut copy = save(&mut s, "toc", ext);
        println!(
            "TOC_{ext} fields={} pages={}",
            fields(&copy),
            copy.layout().pages.len()
        );
        let update = copy.run("references.updateToc", &json!({}));
        println!("TOC_{ext}_UPDATE {update:?}");
    }
    // Runtime image wrapping and format retention are distinct from flags alone.
    let mut s = Session::new(Document::from_text(
        &"Words flowing beside an image. ".repeat(80),
    ));
    wordcraft_gnome::pictures::insert(&mut s, Path::new("/tmp/leaf20-capabilities/image.png"))
        .unwrap();
    s.run("arrange.wrap", &json!({"wrap":"square"})).unwrap();
    s.run("arrange.position", &json!({"preset":"topLeft"}))
        .unwrap();
    let mut original = Vec::new();
    s.doc.objects_in_reading_order(&mut |o| {
        if matches!(o, InlineObject::Image { .. }) {
            original.push(serde_json::to_value(o).unwrap());
        }
    });
    println!("IMAGE_BEFORE {}", serde_json::to_string(&original).unwrap());
    for ext in ["docx", "odt"] {
        let copy = save(&mut s, "image", ext);
        let mut objects = Vec::new();
        copy.doc.objects_in_reading_order(&mut |o| {
            if matches!(o, InlineObject::Image { .. }) {
                objects.push(serde_json::to_value(o).unwrap());
            }
        });
        println!("IMAGE_{ext} {}", serde_json::to_string(&objects).unwrap());
    }
    println!("AUDIT_DONE body_story={:?}", StoryRef::Body);
}
