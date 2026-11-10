//! Paragraph border property edits run inside WordCraft's command/history boundary.
use serde_json::{Value, json};
use std::sync::Arc;
use wordcraft_doc::props::{Border, BorderStyle, Rgb};
use wordcraft_engine::{CommandSpec, Registry, Session};
pub fn change(
    s: &mut Session,
    placement: Option<&str>,
    property: Option<&(&str, Value)>,
    recipe: &Value,
) -> Result<(), String> {
    if s.registry.get("leaf.paragraphBorders").is_none() {
        let mut specs = s.registry.all().to_vec();
        specs.push(CommandSpec::new(
            "leaf.paragraphBorders",
            "Paragraph Borders",
            "Paragraph",
            edit,
        ));
        s.registry = Arc::new(Registry::new(specs));
    }
    let property = property.map(|(key, value)| json!({"key":key,"value":value}));
    s.run(
        "leaf.paragraphBorders",
        &json!({"placement":placement,"property":property,"recipe":recipe}),
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}
fn edit(s: &mut Session, args: &Value) -> wordcraft_engine::CmdResult {
    let placement = args["placement"].as_str();
    let property = args.get("property").filter(|v| v.is_object());
    let recipe = &args["recipe"];
    if placement.is_some_and(|k| {
        ![
            "top", "bottom", "left", "right", "outside", "all", "inside", "none",
        ]
        .contains(&k)
    }) {
        return Err(wordcraft_engine::CmdError::Params(
            "Unknown border placement".into(),
        ));
    }
    if let Some(p) = property {
        match p["key"].as_str() {
            Some("width")
                if p["value"]
                    .as_f64()
                    .is_some_and(|v| v.is_finite() && (0.25..=6.).contains(&v)) => {}
            Some("color") if p["value"].as_str().and_then(Rgb::parse).is_some() => {}
            Some("style") if p["value"].is_string() => {}
            _ => {
                return Err(wordcraft_engine::CmdError::Params(
                    "Invalid border property".into(),
                ));
            }
        }
    }
    let styles = s.doc.styles.clone();
    wordcraft_engine::cmd::para::fmt(s, &|p| {
        let mut borders = styles.resolve_para(p).borders.unwrap_or_default();
        if let Some(property) = property {
            if !borders.any_visible() {
                return;
            }
            for edge in [
                &mut borders.top,
                &mut borders.bottom,
                &mut borders.left,
                &mut borders.right,
                &mut borders.between,
            ]
            .into_iter()
            .flatten()
            {
                match property["key"].as_str() {
                    Some("width") => edge.width = property["value"].as_f64().unwrap() as f32,
                    Some("style") => {
                        edge.style = BorderStyle::from_ooxml(property["value"].as_str().unwrap())
                    }
                    Some("color") => edge.color = property["value"].as_str().and_then(Rgb::parse),
                    _ => {}
                }
            }
        }
        if let Some(kind) = placement {
            let line = Border {
                style: BorderStyle::from_ooxml(recipe["style"].as_str().unwrap_or("single")),
                width: recipe["width"].as_f64().unwrap_or(0.5).clamp(0.25, 6.) as f32,
                color: recipe["color"].as_str().and_then(Rgb::parse),
                space: 1.,
            };
            if kind == "none" {
                borders = Default::default();
            } else {
                for edge in ["top", "bottom", "left", "right"] {
                    if edge == kind || matches!(kind, "all" | "outside") {
                        match edge {
                            "top" => borders.top = Some(line),
                            "bottom" => borders.bottom = Some(line),
                            "left" => borders.left = Some(line),
                            _ => borders.right = Some(line),
                        }
                    }
                }
                if kind == "all" || kind == "inside" {
                    borders.between = Some(line);
                }
            }
        }
        p.borders = Some(borders);
    })
}
