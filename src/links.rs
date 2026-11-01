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
fn command(s: &mut Session, v: &Value) -> CmdResult {
    let url = v["url"]
        .as_str()
        .filter(|u| !u.trim().is_empty() && u.len() <= 4096)
        .ok_or_else(|| CmdError::Params("Enter a link destination".into()))?
        .trim()
        .to_string();
    let creating = v["creating"].as_bool().unwrap_or(false);
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
    })
}
pub fn apply(s: &mut Session, url: &str) -> Result<bool, String> {
    if s.sel.is_collapsed() {
        return Err(wordcraft_gnome::t!("Select the text to link.").into());
    }
    if state(s) == LinkState::Uniform(url.trim().into()) {
        return Ok(false);
    }
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
    let creating = state(s) == LinkState::None;
    let before = s.doc.clone();
    s.run("leaf.link", &json!({"url":url,"creating":creating}))
        .map_err(|e| e.to_string())?;
    Ok(s.doc != before)
}
