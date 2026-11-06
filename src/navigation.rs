//! Read the resolved heading levels; never infer headings from font size or names.
use wordcraft_doc::{Document, Pos, StoryRef};

/// Use the actual preceding line tops, including paragraph spacing. Near a
/// page boundary, use the available context on that page rather than jumping
/// into another page/column and losing the heading's section.
pub fn heading_view_target(
    session: &mut wordcraft_engine::Session,
    target: &Pos,
) -> Option<(usize, f32)> {
    let layout = session.layout();
    let caret = layout.caret_on(target, session.page_hint)?;
    let mut preceding = std::collections::VecDeque::with_capacity(3);
    for item in &layout.pages.get(caret.page)?.items {
        if let wordcraft_engine::layout::Placed::Lines {
            story,
            path,
            para,
            l0,
            l1,
            y,
            ..
        } = item
        {
            if *story == target.story && path < &target.path {
                for line in &para.lines[*l0..*l1] {
                    let top = *y + line.top - para.lines[*l0].top;
                    if top < caret.top {
                        if preceding.len() == 3 {
                            preceding.pop_front();
                        }
                        preceding.push_back(top);
                    }
                }
            }
        }
    }
    Some((
        caret.page,
        preceding
            .front()
            .copied()
            .unwrap_or(caret.top - caret.height * 2.5),
    ))
}
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
/// Body text belongs to the nearest preceding heading, regardless of page.
pub fn containing_heading(headings: &[Heading], pos: &Pos) -> Option<usize> {
    (pos.story == StoryRef::Body)
        .then(|| headings.iter().rposition(|h| h.pos <= *pos))
        .flatten()
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
    fn active_heading_uses_body_boundaries_and_nearest_nested_section() {
        let h = vec![
            Heading {
                pos: Pos::body(1, 0),
                level: 0,
                text: "Top".into(),
            },
            Heading {
                pos: Pos::body(3, 0),
                level: 1,
                text: "Nested".into(),
            },
            Heading {
                pos: Pos::body(6, 0),
                level: 0,
                text: "Next".into(),
            },
        ];
        for (p, expected) in [
            (0, None),
            (1, Some(0)),
            (2, Some(0)),
            (3, Some(1)),
            (5, Some(1)),
            (6, Some(2)),
            (9, Some(2)),
        ] {
            assert_eq!(containing_heading(&h, &Pos::body(p, 0)), expected);
        }
        let mut p = Pos::body(4, 0);
        p.story = StoryRef::Part(1);
        assert_eq!(containing_heading(&h, &p), None);
    }
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

/// Each noninitial section starts after the preceding section's final body block.
pub fn section_starts(doc: &Document) -> Vec<(usize, Pos)> {
    let sections = doc.sections();
    if sections.len() < 2 {
        return Vec::new();
    }
    let paths = doc.para_paths(StoryRef::Body);
    sections
        .windows(2)
        .enumerate()
        .filter_map(|(index, pair)| {
            let path = paths
                .iter()
                .find(|path| {
                    path.0.first().is_some_and(|block| {
                        (*block as usize) > pair[0].0 && (*block as usize) <= pair[1].0
                    })
                })?
                .clone();
            Some((
                index + 2,
                Pos {
                    story: StoryRef::Body,
                    path,
                    off: 0,
                },
            ))
        })
        .collect()
}

/// Read-only element index. Positions are canonical anchors, including nested
/// table cells and floating images; previews never become an editable model.
#[derive(Clone, Debug)]
pub struct Element {
    pub kind: u32,
    pub pos: Pos,
    pub label: String,
    pub object: bool,
    pub table: Option<wordcraft_doc::Path>,
    pub owner: Option<Pos>,
}
pub fn elements(doc: &Document) -> Vec<Element> {
    use wordcraft_doc::para::{InlineObject as O, NoteKind};
    let mut out = Vec::new();
    let mut tables = std::collections::HashSet::new();
    let mut ordinals = [0usize; 13];
    for path in doc.para_paths(StoryRef::Body) {
        let Some(p) = doc.para(StoryRef::Body, &path) else {
            continue;
        };
        let mut table_path = path.clone();
        while let Some((table, _, _)) = table_path.cell() {
            if tables.insert(table.clone()) {
                ordinals[4] += 1;
                out.push(Element {
                    kind: 4,
                    pos: Pos {
                        story: StoryRef::Body,
                        path: path.clone(),
                        off: 0,
                    },
                    label: crate::i18n::interpolate(
                        crate::t!("Table {number}"),
                        &[("number", ordinals[4].to_string())],
                    ),
                    object: false,
                    table: Some(table.clone()),
                    owner: None,
                });
            }
            table_path = table;
        }
        for off in p.object_offsets() {
            let Some(o) = p.object_at(off) else { continue };
            let (kind, title, preview) = match o {
                O::Image { alt, .. } => (3, crate::t!("Image"), alt.clone()),
                O::NoteRef { kind, id, .. } => (
                    if *kind == NoteKind::Footnote { 7 } else { 8 },
                    if *kind == NoteKind::Footnote {
                        crate::t!("Footnote")
                    } else {
                        crate::t!("Endnote")
                    },
                    doc.para_paths(StoryRef::Part(*id))
                        .first()
                        .and_then(|p| doc.para(StoryRef::Part(*id), p))
                        .map(|p| p.plain_text())
                        .unwrap_or_default(),
                ),
                O::Equation { linear, .. } => (9, crate::t!("Equation"), linear.clone()),
                O::Field { instr, .. } | O::FieldStart { instr, .. } => {
                    let field = wordcraft_engine::cmd::references::FieldCode::parse(instr);
                    if field.name == "TOC" {
                        (5, crate::t!("Table of Contents"), String::new())
                    } else if field.name == "SEQ" && p.props.style.as_deref() == Some("Caption") {
                        (6, crate::t!("Caption"), p.plain_text())
                    } else {
                        (11, crate::t!("Automatic Fields"), instr.clone())
                    }
                }
                _ => continue,
            };
            ordinals[kind as usize] += 1;
            let preview = preview
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(70)
                .collect::<String>();
            let label = if preview.is_empty() {
                format!("{title} {}", ordinals[kind as usize])
            } else {
                format!("{title}: {preview}")
            };
            out.push(Element {
                kind,
                pos: Pos {
                    story: StoryRef::Body,
                    path: path.clone(),
                    off,
                },
                label,
                object: !matches!(o, O::FieldStart { .. }),
                table: None,
                owner: None,
            });
        }
        let mut previous = None;
        for (range, props) in p.run_ranges() {
            let link = doc
                .styles
                .resolve_char(p.props.style.as_deref(), props)
                .link;
            if let Some(url) = link.as_ref().filter(|u| previous.as_ref() != Some(*u)) {
                let text = p
                    .text
                    .get(range.clone())
                    .unwrap_or("")
                    .replace(wordcraft_doc::para::OBJ, "");
                let preview = if text.trim().is_empty() {
                    url.as_str()
                } else {
                    text.trim()
                };
                out.push(Element {
                    kind: 10,
                    pos: Pos {
                        story: StoryRef::Body,
                        path: path.clone(),
                        off: range.start,
                    },
                    label: format!(
                        "{}: {}",
                        crate::t!("Link"),
                        preview.chars().take(70).collect::<String>()
                    ),
                    object: false,
                    table: None,
                    owner: None,
                });
            }
            previous = link;
        }
    }
    for caption in crate::captions::all(doc) {
        let owner = out
            .iter()
            .find(|e| match &caption.owner {
                crate::captions::Target::Image(p) => e.kind == 3 && e.pos == *p,
                crate::captions::Target::Table { path, .. } => {
                    e.kind == 4 && e.table.as_ref() == Some(path)
                }
            })
            .map(|e| e.pos.clone());
        let label = crate::captions::text(doc, &caption);
        for e in &mut out {
            if e.kind == 6 && e.pos.story == caption.pos.story && e.pos.path == caption.pos.path {
                e.owner = owner.clone();
            }
            if caption.visible
                && (match &caption.owner {
                    crate::captions::Target::Image(p) => e.kind == 3 && e.pos == *p,
                    crate::captions::Target::Table { path, .. } => {
                        e.kind == 4 && e.table.as_ref() == Some(path)
                    }
                })
                && !label.is_empty()
            {
                e.label = label.chars().take(90).collect();
            }
        }
    }
    out.retain(|e| {
        let visible = doc
            .para_at(&e.pos)
            .is_none_or(|p| p.mark.hidden != Some(true));
        visible
    });
    out.sort_by(|a, b| {
        a.pos
            .cmp(&b.pos)
            // A table and the first inline object in its first cell can share
            // a position. List containers before their children at that anchor.
            .then_with(|| b.table.is_some().cmp(&a.table.is_some()))
            .then_with(|| {
                a.table
                    .as_ref()
                    .map(|p| p.0.len())
                    .cmp(&b.table.as_ref().map(|p| p.0.len()))
            })
            .then(a.kind.cmp(&b.kind))
            .then(a.table.cmp(&b.table))
    });
    out
}

/// Active navigation targets use their actual canonical extent, not merely the
/// nearest paragraph. A table is a fallback container for its nested objects.
pub fn element_contains(s: &wordcraft_engine::Session, e: &Element, focus: &Pos) -> bool {
    if let Some(cells) = s.cell_range() {
        return e.kind == 4 && e.table.as_ref() == Some(&cells.table) && e.pos.story == cells.story;
    }
    match e.kind {
        4 => e
            .table
            .as_ref()
            .is_some_and(|table| focus.story == e.pos.story && focus.path.0.starts_with(&table.0)),
        5 => crate::references::toc_span_at(s, focus).is_some_and(|(a, _)| a.path == e.pos.path),
        6 => focus.story == e.pos.story && focus.path == e.pos.path,
        7 | 8 => match s.doc.para_at(&e.pos).and_then(|p| p.object_at(e.pos.off)) {
            Some(wordcraft_doc::para::InlineObject::NoteRef { id, .. })
                if focus.story == StoryRef::Part(*id) =>
            {
                true
            }
            _ => {
                focus.story == e.pos.story
                    && focus.path == e.pos.path
                    && (e.pos.off..=e.pos.off + wordcraft_doc::para::OBJ.len_utf8())
                        .contains(&focus.off)
            }
        },
        10 => {
            focus.story == e.pos.story
                && focus.path == e.pos.path
                && crate::links::range_at(s, focus).is_some_and(|(a, _, _)| a == e.pos)
        }
        _ => {
            focus.story == e.pos.story
                && focus.path == e.pos.path
                && (e.pos.off..=e.pos.off + wordcraft_doc::para::OBJ.len_utf8())
                    .contains(&focus.off)
        }
    }
}

/// Activate the typed canonical destination, even when several entries share
/// the first paragraph of a table. Navigation remains a pure selection change.
pub fn activate_element(s: &mut wordcraft_engine::Session, e: &Element) -> Result<(), String> {
    if let Some(table) = &e.table {
        let path = s
            .doc
            .para_paths(e.pos.story)
            .into_iter()
            .find(|p| p.cell().is_some_and(|(t, _, _)| &t == table))
            .ok_or(crate::t!("There is no table at this position"))?;
        crate::selection::set_caret(
            s,
            &Pos {
                story: e.pos.story,
                path,
                off: 0,
            },
            false,
        )?;
        s.run("table.selectTable", &serde_json::json!({}))
            .map_err(|e| e.to_string())?;
        return Ok(());
    }
    if e.object {
        if let Some(wordcraft_doc::para::InlineObject::NoteRef { id, .. }) =
            s.doc.para_at(&e.pos).and_then(|p| p.object_at(e.pos.off))
        {
            let pos = s.doc.start_of(StoryRef::Part(*id));
            return crate::selection::set_caret(s, &pos, false);
        }
        return crate::selection::select_range(
            s,
            &e.pos,
            &Pos {
                off: e.pos.off + wordcraft_doc::para::OBJ.len_utf8(),
                ..e.pos.clone()
            },
        );
    }
    crate::selection::set_caret(s, &e.pos, false)
}
