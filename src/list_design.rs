//! Explicit list choices use public WordCraft commands; counters remain engine-owned.
use serde_json::{Value, json};
use std::sync::Arc;
use wordcraft_doc::{
    numbering::{ListKind, levels_for},
    props::NumRef,
};
use wordcraft_engine::{CommandSpec, Registry, Session};
pub fn matches(s: &Session, kind: &str) -> bool {
    let Some(kind) = ListKind::parse(kind) else {
        return false;
    };
    let want = levels_for(kind);
    let positions = crate::text_properties::paragraph_positions(s);
    !positions.is_empty()
        && positions.iter().all(|pos| {
            s.doc
                .para_at(pos)
                .and_then(|p| s.doc.styles.resolve_para(&p.props).numbering)
                .filter(|n| n.num != 0)
                .is_some_and(|n| {
                    want.iter().enumerate().all(|(i, w)| {
                        s.doc.numbering.level(n.num, i as u8).is_some_and(|l| {
                            l.format == w.format
                                && l.text == w.text
                                && (l.indent - w.indent).abs() < 0.01
                                && (l.hanging - w.hanging).abs() < 0.01
                        })
                    })
                })
        })
}
pub fn can_apply(s: &Session, kind: &str) -> bool {
    if matches(s, kind) {
        return true;
    }
    let Some(wanted) = ListKind::parse(kind) else {
        return false;
    };
    if wanted.is_bullet() {
        return true;
    }
    // The pinned command toggles/restarts when replacing a different numbered
    // design. Expose this gap instead of silently losing an existing sequence.
    !crate::text_properties::paragraph_positions(s)
        .iter()
        .any(|pos| {
            s.doc
                .para_at(pos)
                .and_then(|p| s.doc.styles.resolve_para(&p.props).numbering)
                .and_then(|n| s.doc.numbering.level(n.num, n.level))
                .is_some_and(|l| l.format != wordcraft_doc::section::NumFormat::Bullet)
        })
}
pub fn apply(s: &mut Session, command: &str, args: &Value) -> Result<(), String> {
    if args["off"] == true {
        return s
            .run(
                "para.set",
                &json!({"props":{"numbering":{"num":0,"level":0}}}),
            )
            .map(|_| ())
            .map_err(|e| e.to_string());
    }
    let kind = args["kind"]
        .as_str()
        .unwrap_or(if command == "para.bullets" {
            "bullet"
        } else if command == "para.multilevel" {
            "legal"
        } else {
            "numbered"
        });
    if matches(s, kind) {
        return Ok(());
    }
    if !can_apply(s, kind) {
        return Err(crate::t!("WordCraft cannot yet change this numbering design without restarting it. Customize the current list definition instead.").into());
    }
    s.run(command, args).map(|_| ()).map_err(|e| e.to_string())
}
pub fn previous_compatible(s: &Session) -> Option<NumRef> {
    let at = s.sel.ordered().0;
    let current = s
        .doc
        .para_at(&at)
        .and_then(|p| s.doc.styles.resolve_para(&p.props).numbering)
        .filter(|n| n.num != 0)?;
    for path in s
        .doc
        .para_paths(at.story)
        .into_iter()
        .rev()
        .filter(|p| p < &at.path && p.parent() == at.path.parent())
    {
        let Some(previous) = s
            .doc
            .para(at.story, &path)
            .and_then(|p| s.doc.styles.resolve_para(&p.props).numbering)
            .filter(|n| n.num != 0)
        else {
            continue;
        };
        if (0..9).all(|level| {
            let (a, b) = (
                s.doc.numbering.level(current.num, level),
                s.doc.numbering.level(previous.num, level),
            );
            match (a, b) {
                (Some(a), Some(b)) => a.format == b.format && a.text == b.text,
                (None, None) => true,
                _ => false,
            }
        }) {
            return Some(NumRef {
                num: previous.num,
                level: current.level,
            });
        }
    }
    None
}
pub fn continue_previous(s: &mut Session) -> Result<(), String> {
    let previous =
        previous_compatible(s).ok_or(crate::t!("No previous compatible list was found."))?;
    s.run(
        "para.set",
        &json!({"props":{"numbering":{"num":previous.num,"level":previous.level}}}),
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}
pub fn start_at(s: &mut Session, value: u32) -> Result<(), String> {
    if value > 32767 {
        return Err("Invalid list start".into());
    }
    if s.registry.get("leaf.listStart").is_none() {
        let mut specs = s.registry.all().to_vec();
        specs.push(CommandSpec::new(
            "leaf.listStart",
            "Set List Start",
            "Paragraph",
            set_start,
        ));
        s.registry = Arc::new(Registry::new(specs));
    }
    crate::operations::atomic(s, crate::t!("Set Starting Number"), |s| {
        s.run("para.restartNumbering", &json!({}))
            .map_err(|e| e.to_string())?;
        s.run("leaf.listStart", &json!({"value":value}))
            .map(|_| ())
            .map_err(|e| e.to_string())
    })
}
fn set_start(s: &mut Session, args: &Value) -> wordcraft_engine::CmdResult {
    let value = args["value"]
        .as_u64()
        .filter(|v| *v <= 32767)
        .ok_or_else(|| wordcraft_engine::CmdError::Params("Invalid list start".into()))?
        as u32;
    let p = s.sel.focus.clone();
    let n = s
        .doc
        .para_at(&p)
        .and_then(|p| p.props.numbering)
        .ok_or_else(|| wordcraft_engine::CmdError::Failed("Not in a list".into()))?;
    let Some(num) = Arc::make_mut(&mut s.doc.numbering)
        .nums
        .iter_mut()
        .find(|x| x.id == n.num)
    else {
        return Err(wordcraft_engine::CmdError::Failed("No list".into()));
    };
    num.start_overrides.retain(|(level, _)| *level != n.level);
    num.start_overrides.push((n.level, value));
    Ok(json!({}))
}
