//! Read the resolved heading levels; never infer headings from font size or names.
use wordcraft_doc::{Document, Pos, StoryRef};
#[derive(Clone, Debug)]
pub struct Heading {
    pub pos: Pos,
    pub level: u8,
    pub text: String,
}

/// Body bookmarks belong to the last preceding heading, including marks in that
/// heading's own paragraph. A missing owner keeps front matter navigable.
#[derive(Clone, Debug)]
pub struct Bookmark {
    pub name: String,
    pub pos: Pos,
    pub heading: Option<usize>,
}

pub fn bookmarks(doc: &Document, headings: &[Heading]) -> Vec<Bookmark> {
    doc.bookmarks()
        .into_iter()
        .filter(|(name, _)| !name.starts_with('_'))
        .map(|(name, pos)| Bookmark {
            heading: headings.iter().rposition(|h| h.pos <= pos),
            name,
            pos,
        })
        .collect()
}
pub fn headings(doc: &Document) -> Vec<Heading> {
    let mut counters = wordcraft_doc::numbering::Counters::default();
    doc.para_paths(StoryRef::Body)
        .into_iter()
        .filter_map(|path| {
            let pos = Pos {
                story: StoryRef::Body,
                path,
                off: 0,
            };
            let p = doc.para_at(&pos)?;
            let resolved = doc.styles.resolve_para(&p.props);
            let number = p
                .props
                .numbering
                .or(resolved.numbering)
                .filter(|n| n.num != 0)
                .and_then(|n| counters.next_label(&doc.numbering, n.num, n.level))
                .filter(|(label, def)| {
                    !label.is_empty()
                        && !matches!(
                            def.format,
                            wordcraft_doc::section::NumFormat::Bullet
                                | wordcraft_doc::section::NumFormat::None
                        )
                });
            let level = resolved.outline_level.filter(|l| *l < 9)?;
            let text = p.plain_text().trim().to_owned();
            if text.is_empty() || resolved.style.starts_with("TOC") {
                return None;
            }
            let text = match number {
                Some((label, def))
                    if def.suffix == wordcraft_doc::numbering::LevelSuffix::Nothing =>
                {
                    format!("{label}{text}")
                }
                Some((label, _)) => format!("{label} {text}"),
                None => text,
            };
            Some(Heading { pos, level, text })
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wordcraft_engine::Session;
    #[test]
    fn bookmarks_follow_the_nearest_heading_and_refresh_after_outline_edits() {
        let mut s = Session::new(Document::from_text("Preface\nTop\nBody\nSub\nMore\nNext"));
        for (para, command) in [
            (1, "para.heading1"),
            (3, "para.heading2"),
            (5, "para.heading1"),
        ] {
            s.run("caret.set", &json!({"pos": Pos::body(para, 0)}))
                .unwrap();
            s.run(command, &json!({})).unwrap();
        }
        for (para, name) in [
            (0, "Front"),
            (1, "Title"),
            (2, "Main"),
            (4, "Nested"),
            (5, "Last"),
        ] {
            s.run("caret.set", &json!({"pos": Pos::body(para, 1)}))
                .unwrap();
            crate::annotations::add_bookmark(&mut s, name).unwrap();
        }
        s.run("insert.bookmark", &json!({"name":"_internal"}))
            .unwrap();
        let rev = s.rev();
        let marks = bookmarks(&s.doc, &headings(&s.doc));
        assert_eq!(
            marks.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
            ["Front", "Title", "Main", "Nested", "Last"]
        );
        assert_eq!(
            marks.iter().map(|m| m.heading).collect::<Vec<_>>(),
            [None, Some(0), Some(0), Some(1), Some(2)]
        );
        assert_eq!(
            s.rev(),
            rev,
            "navigation snapshots do not edit the document"
        );
        s.run("caret.set", &json!({"pos": Pos::body(3, 0)}))
            .unwrap();
        s.run("outline.level", &json!({"level":"body"})).unwrap();
        assert_eq!(bookmarks(&s.doc, &headings(&s.doc))[3].heading, Some(0));
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(bookmarks(&s.doc, &headings(&s.doc))[3].heading, Some(1));
    }
    #[test]
    fn inherited_levels_explicit_body_and_undo_are_respected() {
        let mut s = Session::new(Document::from_text("Sección\nSubsección\nTexto"));
        s.run("para.heading1", &json!({})).unwrap();
        s.run("caret.set", &json!({"pos":Pos::body(1,0)})).unwrap();
        s.run("para.heading2", &json!({})).unwrap();
        let h = headings(&s.doc);
        assert_eq!(h.iter().map(|h| h.level).collect::<Vec<_>>(), vec![0, 1]);
        assert_eq!(h[1].text, "Subsección");
        s.run("outline.level", &json!({"level":"body"})).unwrap();
        assert_eq!(headings(&s.doc).len(), 1);
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(headings(&s.doc).len(), 2);
    }
}
