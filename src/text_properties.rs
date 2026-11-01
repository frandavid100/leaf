//! Read-only effective text properties for native selection controls.
use wordcraft_doc::{Pos, resolve::ResolvedChar};
use wordcraft_engine::Session;

pub fn formatting_matches(s: &Session, command: &str, args: &serde_json::Value) -> bool {
    let props = selected_resolved(s);
    if props.is_empty() {
        return false;
    }
    match command {
        "format.font" => args["name"]
            .as_str()
            .is_some_and(|font| props.iter().all(|p| p.font == font)),
        "format.size" => args["size"]
            .as_f64()
            .is_some_and(|size| props.iter().all(|p| (p.size as f64 - size).abs() < 0.0001)),
        "format.color" => args["color"].as_str().is_some_and(|color| {
            props.iter().all(|p| match p.color {
                wordcraft_doc::props::TextColor::Auto => color == "auto",
                wordcraft_doc::props::TextColor::Rgb(c) => c.hex().eq_ignore_ascii_case(color),
            })
        }),
        "format.set" => args["props"].as_object().is_some_and(|fields| {
            props.iter().all(|p| {
                serde_json::to_value(p).ok().is_some_and(|v| {
                    fields
                        .iter()
                        .all(|(k, wanted)| match (wanted.as_f64(), v[k].as_f64()) {
                            (Some(a), Some(b)) => (a - b).abs() < 0.0001,
                            _ => v.get(k) == Some(wanted),
                        })
                })
            })
        }),
        "para.shading" => args["color"].as_str().is_some_and(|color| {
            paragraph_positions(s).iter().all(|pos| {
                s.doc.para_at(pos).is_some_and(|p| {
                    s.doc
                        .styles
                        .resolve_para(&p.props)
                        .shading
                        .is_some_and(|c| c.hex().eq_ignore_ascii_case(color))
                })
            })
        }),
        _ => false,
    }
}

pub fn selected_resolved(s: &Session) -> Vec<ResolvedChar> {
    if s.sel.is_collapsed() {
        let p = s.doc.para_at(&s.sel.focus);
        return vec![
            s.doc
                .styles
                .resolve_char(p.and_then(|p| p.props.style.as_deref()), &s.typing_props()),
        ];
    }
    let ranges = s.column_segments().map(|r| r.to_vec()).unwrap_or_else(|| {
        let (a, b) = s.sel.ordered();
        vec![(a.clone(), b.clone())]
    });
    let cells = s
        .cell_range()
        .map(|_| wordcraft_engine::cmd::table::selected_cells(s));
    let mut out = Vec::new();
    for (a, b) in ranges {
        for path in s.doc.paths_between(&a, &b) {
            if let Some(cells) = &cells {
                if path
                    .cell()
                    .is_some_and(|(_, r, c)| !cells.contains(&(r, c)))
                {
                    continue;
                }
            }
            let Some(p) = s.doc.para(a.story, &path) else {
                continue;
            };
            let from = if path == a.path { a.off } else { 0 };
            let to = if path == b.path { b.off } else { p.len() };
            for (r, c) in p.run_ranges() {
                if r.end > from && r.start < to {
                    let resolved = s.doc.styles.resolve_char(p.props.style.as_deref(), c);
                    let text = &p.text[r.start.max(from)..r.end.min(to)];
                    if text.chars().any(|ch| resolved.uses_complex(ch)) {
                        out.push(resolved.complex());
                    }
                    if text.chars().any(|ch| !resolved.uses_complex(ch)) {
                        out.push(resolved);
                    }
                }
            }
        }
    }
    if out.is_empty() {
        let p = s.doc.para_at(&s.sel.focus);
        out.push(
            s.doc
                .styles
                .resolve_char(p.and_then(|p| p.props.style.as_deref()), &s.typing_props()),
        );
    }
    out
}

pub fn used_sizes(s: &Session) -> Vec<f64> {
    let mut sizes = vec![10.0, 11.0, 12.0, 14.0, 16.0, 18.0, 24.0, 36.0];
    for story in std::iter::once(wordcraft_doc::StoryRef::Body).chain(
        s.doc
            .parts
            .keys()
            .copied()
            .map(wordcraft_doc::StoryRef::Part),
    ) {
        for path in s.doc.para_paths(story) {
            let Some(p) = s.doc.para(story, &path) else {
                continue;
            };
            if p.is_empty() {
                sizes.push(
                    s.doc
                        .styles
                        .resolve_char(p.props.style.as_deref(), &p.mark)
                        .size as f64,
                );
            }
            for (range, c) in p.run_ranges() {
                let r = s.doc.styles.resolve_char(p.props.style.as_deref(), c);
                let text = &p.text[range];
                if text.chars().any(|ch| r.uses_complex(ch)) {
                    sizes.push(r.size_cs as f64);
                }
                if text.chars().any(|ch| !r.uses_complex(ch)) {
                    sizes.push(r.size as f64);
                }
            }
        }
    }
    sizes.retain(|v| v.is_finite() && *v > 0.0);
    sizes.sort_by(f64::total_cmp);
    sizes.dedup_by(|a, b| (*a - *b).abs() < 0.0001);
    sizes
}

pub fn paragraph_positions(s: &Session) -> Vec<Pos> {
    let (a, b) = s.sel.ordered();
    let cells = s
        .cell_range()
        .map(|_| wordcraft_engine::cmd::table::selected_cells(s));
    s.doc
        .paths_between(&a, &b)
        .into_iter()
        .filter(|p| p != &b.path || a.path == b.path || b.off > 0)
        .filter(|p| {
            cells
                .as_ref()
                .is_none_or(|cells| p.cell().is_none_or(|(_, r, c)| cells.contains(&(r, c))))
        })
        .map(|path| Pos {
            story: a.story,
            path,
            off: 0,
        })
        .collect()
}
