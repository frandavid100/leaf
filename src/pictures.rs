//! Inline picture operations through the public engine commands.
use serde_json::json;
use std::path::Path;
use wordcraft_doc::{Pos, para::InlineObject};
use wordcraft_engine::{Session, cmd::objects};

pub fn selected(s: &Session) -> Option<(Pos, f32, f32)> {
    let (pos, obj) = if s.sel.is_collapsed() {
        objects::selected(s)?
    } else {
        let (p, o) = objects::object_selection(s)?;
        (p, o.clone())
    };
    match obj {
        InlineObject::Image { w, h, .. } => Some((pos, w, h)),
        _ => None,
    }
}
pub fn select(s: &mut Session, pos: &Pos) -> Result<(), String> {
    if !matches!(
        s.doc.para_at(pos).and_then(|p| p.object_at(pos.off)),
        Some(InlineObject::Image { .. })
    ) {
        return Err(wordcraft_gnome::t!("Select an Image").into());
    }
    crate::selection::select_range(
        s,
        pos,
        &Pos {
            off: pos.off + wordcraft_doc::para::OBJ.len_utf8(),
            ..pos.clone()
        },
    )
}
pub fn insert(s: &mut Session, path: &Path) -> Result<(), String> {
    let p = path
        .to_str()
        .ok_or(wordcraft_gnome::t!("The path is not UTF-8"))?;
    insert_args(s, &json!({"path":p}))
}
/// Clipboard data follows the same validated engine path as an image file.
pub fn insert_data(s: &mut Session, data: &[u8]) -> Result<(), String> {
    insert_args(
        s,
        &json!({"data":wordcraft_engine::cmd::insert::base64_encode(data)}),
    )
}
fn insert_args(s: &mut Session, args: &serde_json::Value) -> Result<(), String> {
    crate::operations::atomic(s, wordcraft_gnome::t!("Insert Image"), |s| {
        s.run("insert.picture", args).map_err(|e| e.to_string())?;
        let Some((pos, width, _)) = selected(s) else {
            return Err(
                wordcraft_gnome::t!("Images cannot be inserted inside a drawing canvas").into(),
            );
        };
        select(s, &pos)?;
        let block = pos.path.0.first().copied().unwrap_or(0) as usize;
        let section = s.doc.section_index_of(block);
        let max = s.doc.sections()[section].1.text_width();
        if width > max {
            s.run("picture.size", &json!({"width":max}))
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    })
}

pub fn replace(s: &mut Session, path: &Path) -> Result<(), String> {
    selected(s).ok_or(wordcraft_gnome::t!("Select an Image"))?;
    s.run(
        "picture.change",
        &json!({"path":path.to_str().ok_or(wordcraft_gnome::t!("The path is not UTF-8"))?}),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn resize(
    s: &mut Session,
    width: f32,
    height: f32,
    lock: bool,
    width_changed: bool,
) -> Result<(), String> {
    selected(s).ok_or(wordcraft_gnome::t!("Select an Image"))?;
    if !width.is_finite()
        || !height.is_finite()
        || !(1.0..=4000.0).contains(&width)
        || !(1.0..=4000.0).contains(&height)
    {
        return Err(wordcraft_gnome::t!("Size must be between 1 and 4000 points").into());
    }
    // The engine preserves proportions when only one dimension is supplied.
    let args = if !lock {
        json!({"width":width,"height":height,"lockAspect":false})
    } else if width_changed {
        json!({"width":width})
    } else {
        json!({"height":height})
    };
    s.run("picture.size", &args).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn original(s: &mut Session) -> Result<(), String> {
    selected(s).ok_or(wordcraft_gnome::t!("Select an Image"))?;
    s.run("picture.size", &json!({"original":true}))
        .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn delete(s: &mut Session) -> Result<(), String> {
    let (p, _, _) = selected(s).ok_or(wordcraft_gnome::t!("Select an Image"))?;
    select(s, &p)?;
    s.run("text.delete", &json!({}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn layout(s: &Session) -> Option<wordcraft_doc::para::Float> {
    let (p, _, _) = selected(s)?;
    match s.doc.para_at(&p)?.object_at(p.off)? {
        InlineObject::Image { float, .. } => Some(float.clone()),
        _ => None,
    }
}

/// Register a document command so placement and spacing use engine history/protection.
pub fn placement(s: &mut Session, mode: Option<&str>, distance: f32) -> Result<(), String> {
    use wordcraft_engine::{CommandSpec, Registry};
    if !distance.is_finite() || !(0.0..=144.0).contains(&distance) {
        return Err(wordcraft_gnome::t!("Image spacing must be between 0 and 144 points.").into());
    }
    if s.registry.get("leaf.pictureLayout").is_none() {
        let mut specs = s.registry.all().to_vec();
        specs.push(CommandSpec::new(
            "leaf.pictureLayout",
            "Image Placement",
            "Image",
            placement_command,
        ));
        s.registry = std::sync::Arc::new(Registry::new(specs));
    }
    s.run(
        "leaf.pictureLayout",
        &json!({"mode":mode,"distance":distance}),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn placement_command(s: &mut Session, v: &serde_json::Value) -> wordcraft_engine::CmdResult {
    use wordcraft_doc::para::{Anchor, FloatAlign, Wrap};
    use wordcraft_engine::CmdError;
    let (pos, _, _) = selected(s).ok_or_else(|| CmdError::Disabled("Select an image".into()))?;
    let mode = v["mode"].as_str();
    let distance = v["distance"].as_f64().unwrap_or(0.) as f32;
    if !distance.is_finite() || !(0.0..=144.0).contains(&distance) {
        return Err(CmdError::Params("Invalid image spacing".into()));
    }
    if mode.is_some_and(|m| !["inline", "left", "right", "topAndBottom"].contains(&m)) {
        return Err(CmdError::Params("Invalid image placement".into()));
    }
    let p = s.doc.para_mut(pos.story, &pos.path)?;
    let Some(InlineObject::Image { float, .. }) = p.object_at_mut(pos.off) else {
        return Err(CmdError::Disabled("Select an image".into()));
    };
    if let Some(mode) = mode {
        float.wrap = match mode {
            "inline" => Wrap::Inline,
            "topAndBottom" => Wrap::TopAndBottom,
            _ => Wrap::Square,
        };
        if mode != "inline" {
            float.h_rel = Anchor::Column;
            float.v_rel = Anchor::Paragraph;
            float.h_align = Some(if mode == "right" {
                FloatAlign::End
            } else {
                FloatAlign::Start
            });
            float.v_align = None;
            float.x = 0.;
            float.y = 0.;
        }
    }
    float.dist = distance;
    float.dist_top = distance;
    float.dist_bottom = distance;
    p.touch();
    Ok(json!({}))
}

/// Commit a page-aware drop atomically. A rejected engine anchor must not strand
/// the object on another page or clip it at the original sheet edge.
pub fn drop_at(
    s: &mut Session,
    page: usize,
    to: Option<&Pos>,
    x: f32,
    y: f32,
) -> Result<(), String> {
    let (_, _, _) = selected(s).ok_or(wordcraft_gnome::t!("Select an Image"))?;
    let inline = layout(s).is_some_and(|f| f.wrap == wordcraft_doc::para::Wrap::Inline);
    let snapshot = s.edit_snapshot();
    let hint = s.page_hint;
    let args = if inline {
        json!({"to":to.ok_or(wordcraft_gnome::t!("Choose a destination inside the document."))?})
    } else {
        json!({"page":page,"x":x,"y":y})
    };
    let result=crate::captions::image_bounds(s,&args).and_then(|_|{
        if !inline{
            let (pos,_,_)=selected(s).ok_or(wordcraft_gnome::t!("Select an Image"))?;
            if s.layout().object(&pos,page).is_none_or(|hit|hit.page!=page){
                return Err(wordcraft_gnome::t!("WordCraft cannot anchor this image on the destination page. Its original position has been kept.").into())
            }
        }
        Ok(())
    });
    if result.is_err() {
        s.restore(snapshot);
        s.page_hint = hint;
    }
    result
}
