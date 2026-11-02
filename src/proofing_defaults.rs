//! Inheritable proofing properties in WordCraft's own styles and character runs.
use serde_json::{Value, json};
use wordcraft_doc::{Document, props::CharProps};
use wordcraft_engine::{CmdError, CmdResult, CommandSpec, Registry, Session};

fn defaults_command(s: &mut Session, v: &Value) -> CmdResult {
    let lang = v["lang"]
        .as_str()
        .map(|tag| {
            wordcraft_engine::proof::lang::normalize_tag(tag)
                .ok_or_else(|| CmdError::Params("Invalid language tag".into()))
        })
        .transpose()?;
    let review = v["review"].as_bool();
    let old = &s.doc.styles.default_chr;
    if lang.as_ref().is_none_or(|l| old.lang.as_ref() == Some(l))
        && review.is_none_or(|on| old.no_proof.unwrap_or(false) == !on)
    {
        return Ok(json!({"changed":false}));
    }
    if let Some(lang) = lang {
        if s.doc.styles.default_chr.lang.as_ref() != Some(&lang) {
            s.doc.styles.default_chr.lang_bidi = Some(lang.clone());
            s.doc.styles.default_chr.lang_ea = Some(lang.clone());
            s.doc.styles.default_chr.lang = Some(lang);
        }
    }
    if let Some(on) = review {
        s.doc.styles.default_chr.no_proof = Some(!on);
    }
    Ok(json!({"changed":true}))
}
fn text_command(s: &mut Session, v: &Value) -> CmdResult {
    let choice = v["choice"].as_str().unwrap_or("default");
    let lang = if !matches!(choice, "default" | "off") {
        Some(
            wordcraft_engine::proof::lang::normalize_tag(choice)
                .ok_or_else(|| CmdError::Params("Invalid language tag".into()))?,
        )
    } else {
        None
    };
    wordcraft_engine::cmd::format::apply(s, &|c| match choice {
        "default" => {
            c.lang = None;
            c.lang_bidi = None;
            c.lang_ea = None;
            c.no_proof = None;
        }
        "off" => c.no_proof = Some(true),
        _ => {
            c.lang = lang.clone();
            c.lang_bidi = lang.clone();
            c.lang_ea = lang.clone();
            c.no_proof = Some(false);
        }
    })
}
pub fn register(s: &mut Session) {
    if s.registry.get("leaf.proofDefaults").is_some() {
        return;
    }
    let mut specs = s.registry.all().to_vec();
    specs.push(
        CommandSpec::new(
            "leaf.proofDefaults",
            "Document Language and Review",
            "Document Properties",
            defaults_command,
        )
        .when(|s| {
            if wordcraft_engine::cmd::protect::enforced(&s.doc) {
                Some("Document protection prevents changing language/review defaults")
            } else {
                None
            }
        }),
    );
    specs.push(CommandSpec::new(
        "leaf.proofText",
        "Language",
        "Text",
        text_command,
    ));
    s.registry = std::sync::Arc::new(Registry::new(specs));
}
pub fn apply(s: &mut Session, lang: &str, review: bool) -> Result<bool, String> {
    register(s);
    let before = s.doc.clone();
    s.run("leaf.proofDefaults", &json!({"lang":lang,"review":review}))
        .map_err(|e| e.to_string())?;
    Ok(s.doc != before)
}
pub fn text(s: &mut Session, choice: &str) -> Result<bool, String> {
    register(s);
    let before = s.doc.clone();
    s.run("leaf.proofText", &json!({"choice":choice}))
        .map_err(|e| e.to_string())?;
    Ok(s.doc != before)
}
pub fn language(s: &Session) -> String {
    s.doc
        .styles
        .default_chr
        .lang
        .clone()
        .unwrap_or_else(|| "en-US".into())
}
pub fn review(s: &Session) -> bool {
    !s.doc.styles.default_chr.no_proof.unwrap_or(false)
}
pub fn new_document() -> Session {
    let mut doc = Document::new();
    doc.styles.default_chr.lang = Some(preferred_language());
    doc.styles.default_chr.no_proof = Some(!crate::preferences::check_spelling());
    Session::new(doc)
}
pub fn preferred_language() -> String {
    crate::preferences::default_language().unwrap_or_else(|| {
        if std::env::var("LANGUAGE")
            .or_else(|_| std::env::var("LANG"))
            .unwrap_or_default()
            .starts_with("es")
        {
            "es-ES".into()
        } else {
            "en-US".into()
        }
    })
}
pub fn direct(s: &Session) -> Vec<CharProps> {
    if s.sel.is_collapsed() {
        return vec![s.typing_props()];
    }
    let ranges = s
        .column_segments()
        .map(|r| r.to_vec())
        .unwrap_or_else(|| vec![s.sel.ordered()]);
    let cells = s
        .cell_range()
        .map(|_| wordcraft_engine::cmd::table::selected_cells(s));
    let mut out = Vec::new();
    for (a, b) in ranges {
        for path in s.doc.paths_between(&a, &b) {
            if cells.as_ref().is_some_and(|cells| {
                path.cell()
                    .is_some_and(|(_, r, c)| !cells.contains(&(r, c)))
            }) {
                continue;
            }
            let Some(p) = s.doc.para(a.story, &path) else {
                continue;
            };
            let start = if path == a.path { a.off } else { 0 };
            let end = if path == b.path { b.off } else { p.len() };
            out.extend(
                p.run_ranges()
                    .filter(|(r, _)| r.end > start && r.start < end)
                    .map(|(_, c)| c.clone()),
            );
        }
    }
    out
}
pub fn choice(s: &Session) -> String {
    let direct = direct(s);
    if !direct.is_empty()
        && direct.iter().all(|p| {
            p.lang.is_none() && p.lang_bidi.is_none() && p.lang_ea.is_none() && p.no_proof.is_none()
        })
    {
        return "default".into();
    }
    let resolved = crate::text_properties::selected_resolved(s);
    if !resolved.is_empty() && resolved.iter().all(|p| p.no_proof) {
        return "off".into();
    }
    if let Some(first) = resolved.first() {
        if resolved
            .iter()
            .all(|p| p.lang == first.lang && p.no_proof == first.no_proof)
        {
            return first.lang.clone().unwrap_or_else(|| "default".into());
        }
    }
    "mixed".into()
}
/// The pinned ODT exporter resolves runs and drops false/absent proofing overrides.
/// Such a save would destroy future inheritance; DOCX preserves the native props.
pub fn odt_loses_inheritance(s: &Session) -> bool {
    if s.doc.styles.default_chr.no_proof == Some(true) {
        return true;
    }
    s.doc.shown_stories().into_iter().any(|story| {
        s.doc.para_paths(story).into_iter().any(|path| {
            s.doc.para(story, &path).is_some_and(|p| {
                p.run_ranges()
                    .any(|(r, c)| !r.is_empty() && c.no_proof == Some(false))
            })
        })
    })
}
