//! Exercise incremental note layout and compare it with an isolated cold layout.
//! Optional local input is never copied, rendered or written by this audit.
use serde_json::json;
use wordcraft_doc::{Document, PartKind, Pos, StoryRef};
use wordcraft_engine::{Session, layout::Placed};
use wordcraft_gnome::{
    document_properties::{self, PageSetup},
    header_footer::{self, Region},
    references, selection,
};
fn bands(s: &mut Session) -> Vec<(usize, f32, f32, f32)> {
    let doc = s.doc.clone();
    let layout = s.layout();
    let mut result = Vec::new();
    for (i, p) in layout.pages.iter().enumerate() {
        let mut bottom: f32 = 0.;
        let mut notes = 0;
        for item in &p.items {
            if let Placed::Lines {
                story: StoryRef::Part(id),
                y,
                para,
                l0,
                l1,
                ..
            } = item
            {
                if doc
                    .parts
                    .get(id)
                    .is_some_and(|p| p.kind == PartKind::Footnote)
                    && l1 > l0
                {
                    bottom = bottom.max(
                        y + para.lines[l1 - 1].top - para.lines[*l0].top
                            + para.lines[l1 - 1].height,
                    );
                    notes += 1;
                }
            }
        }
        let footer = p
            .footer
            .iter()
            .filter_map(|it| {
                if let Placed::Lines { y, l0, l1, .. } = it {
                    (l1 > l0).then_some(*y)
                } else {
                    None
                }
            })
            .fold(p.h, f32::min);
        if notes > 0 {
            result.push((i, bottom, footer, p.h));
        }
    }
    result
}
fn check(s: &mut Session, label: &str) {
    let warm = bands(s);
    let mut cold = Session::new(s.doc.clone());
    cold.sel = s.sel.clone();
    cold.view = s.view.clone();
    cold.prefs = s.prefs.clone();
    cold.path = s.path.clone();
    let fresh = bands(&mut cold);
    let overlap = warm
        .iter()
        .any(|(_, b, f, h)| *b > *f + 0.05 || *b > *h + 0.05);
    let same = warm.len() == fresh.len()
        && warm
            .iter()
            .zip(&fresh)
            .all(|(a, b)| a.0 == b.0 && (a.1 - b.1).abs() < 0.05 && (a.2 - b.2).abs() < 0.05);
    println!(
        "NOTE_LAYOUT {label} overlap={overlap} cold_matches={same} warm={warm:?} cold={fresh:?}"
    );
}
fn fixture(distance: f32, single: bool, footer_lines: usize) -> Session {
    let text = (0..65)
        .map(|i| {
            format!(
                "Paragraph {i}: {}\n",
                "ordinary words for two-column note layout. ".repeat(4)
            )
        })
        .collect::<String>();
    let text = if single {
        text.replace('\n', " ")
    } else {
        text
    };
    let mut s = Session::new(Document::from_text(&text));
    document_properties::apply(
        &mut s,
        PageSetup {
            width: 595.3,
            height: 841.9,
            top: 28.35,
            bottom: 28.35,
            left: 28.35,
            right: 28.35,
        },
        true,
    )
    .unwrap();
    s.run("layout.columns", &json!({"count":2,"space":14.15}))
        .unwrap();
    header_footer::enter(&mut s, 0, Region::Footer).unwrap();
    s.run(
        "text.insert",
        &json!({"text":"An ordinary footer.\n".repeat(footer_lines)}),
    )
    .unwrap();
    header_footer::close(&mut s).unwrap();
    let mut opt = wordcraft_gnome::page_options::Options::current(&s, 0);
    opt.footer = distance;
    wordcraft_gnome::page_options::apply(&mut s, &opt).unwrap();
    selection::set_caret(&mut s, &Pos::body(if single { 0 } else { 8 }, 12), false).unwrap();
    references::insert_footnote(&mut s).unwrap();
    s.run("text.insert", &json!({"text":"A short footnote."}))
        .unwrap();
    s
}
fn table_fixture(distance: f32) -> Session {
    let mut s = Session::new(Document::new());
    document_properties::apply(
        &mut s,
        PageSetup {
            width: 595.3,
            height: 841.9,
            top: 28.35,
            bottom: 28.35,
            left: 28.35,
            right: 28.35,
        },
        true,
    )
    .unwrap();
    s.run("layout.columns", &json!({"count":2,"space":14.15}))
        .unwrap();
    s.run("insert.table", &json!({"rows":24,"cols":2})).unwrap();
    let root = s.sel.focus.path.0[0];
    for row in 0..24 {
        for cell in 0..2 {
            let p = Pos {
                story: StoryRef::Body,
                path: wordcraft_doc::Path(vec![root, row, cell, 0]),
                off: 0,
            };
            selection::set_caret(&mut s, &p, false).unwrap();
            s.run(
                "text.insert",
                &json!({"text":"Ordinary table cell content. ".repeat(5)}),
            )
            .unwrap();
        }
    }
    header_footer::enter(&mut s, 0, Region::Footer).unwrap();
    s.run("text.insert", &json!({"text":"An ordinary footer."}))
        .unwrap();
    header_footer::close(&mut s).unwrap();
    let mut opt = wordcraft_gnome::page_options::Options::current(&s, 0);
    opt.footer = distance;
    wordcraft_gnome::page_options::apply(&mut s, &opt).unwrap();
    selection::set_caret(
        &mut s,
        &Pos {
            story: StoryRef::Body,
            path: wordcraft_doc::Path(vec![root, 12, 0, 0]),
            off: 20,
        },
        false,
    )
    .unwrap();
    references::insert_footnote(&mut s).unwrap();
    s.run("text.insert", &json!({"text":"A short footnote."}))
        .unwrap();
    s
}
fn exercise(mut s: Session, label: &str) {
    check(&mut s, &format!("{label}:open"));
    let notes = references::footnote_refs(&s);
    if notes.is_empty() {
        println!("NO_NOTES {label}");
        return;
    }
    let id = notes[0].1;
    let end = s.doc.end_of(StoryRef::Part(id));
    selection::set_caret(&mut s, &end, false).unwrap();
    for repeat in [1, 10, 100, 400] {
        s.run(
            "text.insert",
            &json!({"text":" More editable note content. ".repeat(repeat)}),
        )
        .unwrap();
        check(&mut s, &format!("{label}:type:{repeat}"));
    }
    for n in 0..4 {
        if s.can_undo() {
            s.run("edit.undo", &json!({})).unwrap();
            check(&mut s, &format!("{label}:undo:{n}"));
        }
    }
    for n in 0..4 {
        if s.can_redo() {
            s.run("edit.redo", &json!({})).unwrap();
            check(&mut s, &format!("{label}:redo:{n}"));
        }
    }
    for distance in [14., 28.35, 35.45, 60.] {
        let mut opt = wordcraft_gnome::page_options::Options::current(&s, 0);
        opt.footer = distance;
        wordcraft_gnome::page_options::apply(&mut s, &opt).unwrap();
        check(&mut s, &format!("{label}:footer:{distance}"));
    }
}
fn main() {
    if let Some(folder) = std::env::var_os("LEAF_NOTE_REPRO_DIR") {
        let folder = std::path::PathBuf::from(folder);
        std::fs::create_dir_all(&folder).unwrap();
        let mut generated = table_fixture(28.35);
        let layout = generated.layout();
        std::fs::write(
            folder.join("table-note.png"),
            wordcraft_render::render_page(
                &generated.doc,
                &layout.pages[0],
                1.5,
                &Default::default(),
            )
            .to_png(),
        )
        .unwrap();
        wordcraft_gnome::saving::save(&mut generated, &folder.join("table-note.docx")).unwrap();
        wordcraft_gnome::pdf_export::export(&mut generated, &folder.join("table-note.pdf"))
            .unwrap();
        println!("SYNTHETIC_REPRO_EXPORTED");
    }

    for single in [false, true] {
        for lines in [1, 3] {
            for d in [14., 28.35, 35.45, 60.] {
                exercise(
                    fixture(d, single, lines),
                    &format!("synthetic:{d}:single={single}:footer-lines={lines}"),
                );
            }
        }
    }
    for distance in [28.35, 35.45, 60.] {
        exercise(
            table_fixture(distance),
            &format!("synthetic-table:{distance}"),
        );
    }
    if let Some(file) = std::env::args().nth(1) {
        exercise(
            wordcraft_gnome::open_document(std::path::Path::new(&file)).unwrap(),
            "local-input",
        );
    }
}
