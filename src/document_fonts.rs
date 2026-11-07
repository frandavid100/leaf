//! Font declarations and explicit substitutions on WordCraft's authoritative model.
use std::{collections::BTreeMap, sync::Arc};
use wordcraft_doc::{Document, Pos, StoryRef, props::CharProps};
use wordcraft_engine::{CmdError, CommandSpec, Registry, Session};
#[derive(Clone, Debug)]
pub struct Font {
    pub name: String,
    pub declarations: usize,
    pub uses: Vec<(Pos, Pos)>,
}
fn names(c: &CharProps) -> impl Iterator<Item = &str> {
    [
        c.font.as_deref(),
        c.font_cs.as_deref(),
        c.font_ea.as_deref(),
        c.sym_font.as_deref(),
    ]
    .into_iter()
    .flatten()
}
/// One visitor covers the same authored declarations for inventory and substitution.
fn visit(doc: &mut Document, mut f: impl FnMut(&mut CharProps)) {
    let styles = Arc::make_mut(&mut doc.styles);
    f(&mut styles.default_chr);
    for style in &mut styles.styles {
        f(&mut style.chr);
        if let Some(t) = &mut style.table {
            for chr in [
                &mut t.header_chr,
                &mut t.band_chr,
                &mut t.first_col_chr,
                &mut t.total_chr,
                &mut t.last_col_chr,
                &mut t.col_band_chr,
            ] {
                f(chr)
            }
            for region in t.regions.values_mut() {
                f(&mut region.chr)
            }
        }
    }
    let numbering = Arc::make_mut(&mut doc.numbering);
    for abstract_num in &mut numbering.abstracts {
        for level in &mut abstract_num.levels {
            f(&mut level.chr)
        }
    }
    for num in &mut numbering.nums {
        for (_, level) in &mut num.level_overrides {
            f(&mut level.chr)
        }
    }
    let stories = std::iter::once(StoryRef::Body)
        .chain(doc.parts.keys().map(|id| StoryRef::Part(*id)))
        .collect::<Vec<_>>();
    for story in stories {
        for path in doc.para_paths(story) {
            if let Ok(p) = doc.para_mut(story, &path) {
                f(&mut p.mark);
                for run in &mut p.runs {
                    f(&mut run.props)
                }
                p.touch();
            }
        }
    }
}
pub fn inventory(s: &Session) -> Vec<Font> {
    let mut fonts = BTreeMap::<String, Font>::new();
    // A disposable copy-on-write snapshot is read once; it is never an editable model.
    visit(&mut s.doc.clone(), |c| {
        for name in names(c) {
            fonts
                .entry(name.to_owned())
                .or_insert_with(|| Font {
                    name: name.to_owned(),
                    declarations: 0,
                    uses: Vec::new(),
                })
                .declarations += 1;
        }
    });
    for story in s.doc.shown_stories() {
        for path in s.doc.para_paths(story) {
            let Some(p) = s.doc.para(story, &path) else {
                continue;
            };
            for (range, chr) in p.run_ranges() {
                let resolved = s.doc.styles.resolve_char(p.props.style.as_deref(), chr);
                let mut names = vec![resolved.font, resolved.font_cs];
                names.extend(resolved.font_ea);
                names.extend(chr.sym_font.clone());
                names.sort();
                names.dedup();
                for name in names {
                    let font = fonts.entry(name.clone()).or_insert_with(|| Font {
                        name,
                        declarations: 0,
                        uses: Vec::new(),
                    });
                    font.uses.push((
                        Pos {
                            story,
                            path: path.clone(),
                            off: range.start,
                        },
                        Pos {
                            story,
                            path: path.clone(),
                            off: range.end,
                        },
                    ));
                }
            }
        }
    }
    fonts.into_values().collect()
}
pub fn substitute(s: &mut Session, from: &str, to: &str) -> Result<(), String> {
    let to = to.trim();
    if to.is_empty() || to.len() > 128 {
        return Err(wordcraft_gnome::t!("Choose a replacement font.").into());
    }
    if s.registry.get("leaf.replaceFont").is_none() {
        let mut specs = s.registry.all().to_vec();
        specs.push(CommandSpec::new(
            "leaf.replaceFont",
            "Replace Document Font",
            "Document",
            replace,
        ));
        s.registry = Arc::new(Registry::new(specs));
    }
    s.run(
        "leaf.replaceFont",
        &serde_json::json!({"from":from,"to":to}),
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}
fn replace(s: &mut Session, v: &serde_json::Value) -> wordcraft_engine::CmdResult {
    let (Some(from), Some(to)) = (v["from"].as_str(), v["to"].as_str()) else {
        return Err(CmdError::Params("Missing font".into()));
    };
    if to.trim().is_empty() || to.len() > 128 {
        return Err(CmdError::Params("Invalid font".into()));
    }
    if s.doc.settings.track_changes {
        return Err(CmdError::Failed(
            wordcraft_gnome::t!("Turn off change tracking before replacing document fonts.").into(),
        ));
    }
    visit(&mut s.doc, |c| {
        for field in [&mut c.font, &mut c.font_cs, &mut c.font_ea, &mut c.sym_font] {
            if field
                .as_deref()
                .is_some_and(|font| font.eq_ignore_ascii_case(from))
            {
                *field = Some(to.to_owned())
            }
        }
    });
    // Pending caret formatting is transient; preserve it, except for the requested name.
    if let Some(c) = s.pending.as_mut() {
        for field in [&mut c.font, &mut c.font_cs, &mut c.font_ea, &mut c.sym_font] {
            if field
                .as_deref()
                .is_some_and(|font| font.eq_ignore_ascii_case(from))
            {
                *field = Some(to.to_owned())
            }
        }
    }
    Ok(serde_json::json!({}))
}
