//! A native arbitrary-colour picker backed by canonical WordCraft properties.
//! OOXML highlight colours keep their native enum; other RGB backgrounds use
//! character shading, which WordCraft already renders and exports losslessly.
use serde_json::{Value, json};
use wordcraft_doc::props::{CharProps, Highlight, Rgb};
use wordcraft_engine::{CmdError, CmdResult, CommandSpec, Registry, Session};

fn choice(value: &str) -> Result<(Highlight, Option<Rgb>), CmdError> {
    if let Some(h) = Highlight::ALL.iter().copied().find(|h| h.ooxml().eq_ignore_ascii_case(value)) {
        return Ok((h, None));
    }
    let rgb = Rgb::parse(value).ok_or_else(|| CmdError::Params("Invalid RGB background".into()))?;
    Ok(Highlight::ALL.iter().copied().find(|h| h.rgb() == Some(rgb))
        .map_or((Highlight::None, Some(rgb)), |h| (h, None)))
}
fn command(s: &mut Session, v: &Value) -> CmdResult {
    let (highlight, shading) = choice(v["color"].as_str().unwrap_or("none"))?;
    if shading.is_none() {
        // Dispatch the existing public command handler inside this one Session
        // transaction, retaining the engine's native keyboard palette choice.
        let native = s.registry.get("format.highlight").unwrap().run;
        native(s, &json!({"color":highlight.ooxml()}))?;
    }
    let set = |p: &mut CharProps| {
        p.highlight = Some(highlight);
        p.shading = shading;
    };
    // Like native highlighting, a caret formats subsequent typing rather than
    // the surrounding word. Let the public formatter handle selections/cells
    // and selected text boxes, and empty-paragraph mark formatting.
    if s.sel.is_collapsed() && s.column_segments().is_none()
        && wordcraft_engine::cmd::objects::object_selection(s).is_none()
        && s.doc.para_at(&s.sel.focus).is_some_and(|p| !p.is_empty())
    {
        let mut props = s.typing_props();
        set(&mut props);
        s.pending = Some(props);
        return Ok(json!({"typing":true}));
    }
    wordcraft_engine::cmd::format::apply(s, &set)
}
pub fn register(s: &mut Session) {
    if s.registry.get("leaf.highlight").is_some() { return; }
    let mut specs = s.registry.all().to_vec();
    specs.push(CommandSpec::new("leaf.highlight", "Text Highlight Color", "Text", command)
        .when(|s| wordcraft_engine::cmd::protect::enforced(&s.doc)
            .then_some("Document protection prevents changing text backgrounds")));
    s.registry = std::sync::Arc::new(Registry::new(specs));
}
pub fn matches(s: &Session, value: &str) -> bool {
    let Ok((h, shading)) = choice(value) else { return false };
    let expected = h.rgb().or(shading);
    let props = crate::text_properties::selected_resolved(s);
    !props.is_empty() && props.iter().all(|p| p.highlight.or(p.shading) == expected)
}
