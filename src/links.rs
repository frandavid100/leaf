//! Hyperlink editing through an engine command; no duplicate text or link model.
use serde_json::{Value, json};
use wordcraft_engine::{CmdError, CmdResult, CommandSpec, Registry, Session};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LinkState {
    None,
    Uniform(String),
    Mixed,
}
pub fn state(s: &Session) -> LinkState {
    let props = crate::text_properties::selected_resolved(s);
    let Some(first) = props.first().map(|p| p.link.clone()) else {
        return LinkState::None;
    };
    if props.iter().any(|p| p.link != first) {
        return LinkState::Mixed;
    }
    first.map(LinkState::Uniform).unwrap_or(LinkState::None)
}
pub fn at(s: &Session, pos: &wordcraft_doc::Pos) -> Option<String> {
    let p = s.doc.para_at(pos)?;
    let resolved = |off| {
        s.doc
            .styles
            .resolve_char(p.props.style.as_deref(), p.props_of_char(off))
            .link
    };
    resolved(pos.off).or_else(|| {
        p.text
            .get(..pos.off)?
            .char_indices()
            .next_back()
            .and_then(|(off, _)| resolved(off))
    })
}

/// A contiguous link within one document paragraph; character formatting may
/// split it into runs. Visual line wrapping does not split the link.
pub fn range_at(
    s: &Session,
    pos: &wordcraft_doc::Pos,
) -> Option<(wordcraft_doc::Pos, wordcraft_doc::Pos, String)> {
    let p = s.doc.para_at(pos)?;
    let url = at(s, pos)?;
    let mut groups: Vec<(usize, usize, String)> = Vec::new();
    for (range, props) in p.run_ranges() {
        let link = s
            .doc
            .styles
            .resolve_char(p.props.style.as_deref(), props)
            .link;
        if let Some(link) = link {
            if let Some(last) = groups
                .last_mut()
                .filter(|g| g.1 == range.start && g.2 == link)
            {
                last.1 = range.end;
            } else {
                groups.push((range.start, range.end, link));
            }
        }
    }
    let (start, end, _) = groups
        .iter()
        .find(|(a, b, u)| *a <= pos.off && pos.off < *b && *u == url)
        .or_else(|| {
            groups
                .iter()
                .find(|(a, b, u)| *a < pos.off && pos.off == *b && *u == url)
        })?;
    let mut a = pos.clone();
    a.off = *start;
    let mut b = pos.clone();
    b.off = *end;
    Some((a, b, url))
}
fn command(s: &mut Session, v: &Value) -> CmdResult {
    let url = v["url"]
        .as_str()
        .filter(|u| !u.trim().is_empty() && u.len() <= 4096)
        .ok_or_else(|| CmdError::Params("Enter a link destination".into()))?
        .trim()
        .to_string();
    let target = v
        .get("target")
        .map(|v| serde_json::from_value::<wordcraft_engine::Selection>(v.clone()))
        .transpose()
        .map_err(|_| CmdError::Params("Invalid link target".into()))?;
    let collapsed = target.as_ref().unwrap_or(&s.sel).is_collapsed();
    let text = if collapsed {
        Some(
            v["text"]
                .as_str()
                .filter(|t| !t.trim().is_empty())
                .ok_or_else(|| CmdError::Params("Enter the link text".into()))?,
        )
    } else {
        None
    };
    if let Some(target) = target {
        s.run(
            "select.range",
            &json!({"anchor": target.anchor, "focus": target.focus}),
        )?;
    }
    let creating = collapsed || state(s) == LinkState::None;
    if let Some(text) = text {
        let start = s.sel.focus.clone();
        // Compose public command handlers inside this one engine history step.
        let insert = s
            .registry
            .get("text.insert")
            .ok_or_else(|| CmdError::Failed("Text insertion unavailable".into()))?
            .run;
        insert(s, &json!({"text":text,"raw":true}))?;
        s.run("select.range", &json!({"anchor":start,"focus":s.sel.focus}))?;
    }
    wordcraft_engine::cmd::format::apply(s, &|c| {
        c.link = Some(url.clone());
        if creating {
            if c.style.is_none() {
                c.style = Some("Hyperlink".into());
            }
            if c.color.is_none()
                || matches!(c.color, Some(wordcraft_doc::props::TextColor::Auto))
                || matches!(c.color,Some(wordcraft_doc::props::TextColor::Rgb(rgb)) if rgb.hex()=="000000")
            {
                c.color = Some(wordcraft_doc::props::TextColor::Rgb(
                    wordcraft_doc::props::Rgb::parse("0066CC").unwrap(),
                ));
            }
            if c.underline.is_none() || c.underline == Some(wordcraft_doc::props::Underline::None) {
                c.underline = Some(wordcraft_doc::props::Underline::Single);
            }
        }
    })?;
    if collapsed {
        s.run("caret.set", &json!({"pos":s.sel.focus}))?;
    }
    Ok(json!({}))
}
pub fn apply(s: &mut Session, url: &str) -> Result<bool, String> {
    if s.sel.is_collapsed() {
        return Err(wordcraft_gnome::t!("Select the text to link.").into());
    }
    if state(s) == LinkState::Uniform(url.trim().into()) {
        return Ok(false);
    }
    apply_to(s, &s.sel.clone(), url, "")
}

pub fn apply_to(
    s: &mut Session,
    target: &wordcraft_engine::Selection,
    url: &str,
    text: &str,
) -> Result<bool, String> {
    if s.registry.get("leaf.link").is_none() {
        let mut specs = s.registry.all().to_vec();
        specs.push(CommandSpec::new(
            "leaf.link",
            "Link",
            "Insert › Links",
            command,
        ));
        s.registry = std::sync::Arc::new(Registry::new(specs));
    }
    let before = s.doc.clone();
    s.run("leaf.link", &json!({"url":url,"target":target,"text":text}))
        .map_err(|e| e.to_string())?;
    Ok(s.doc != before)
}
