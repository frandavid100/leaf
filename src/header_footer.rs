//! Native host adapter for WordCraft's editable page stories and automatic fields.
//! Never use insert.pageNumber's top/bottom presets: they replace existing content.
use serde_json::json;
use wordcraft_doc::{PartKind, Pos, StoryRef};
use wordcraft_engine::Session;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Header,
    Footer,
}

pub fn active(s: &Session) -> Option<Region> {
    let StoryRef::Part(id) = s.sel.focus.story else {
        return None;
    };
    match s.doc.parts.get(&id)?.kind {
        PartKind::Header => Some(Region::Header),
        PartKind::Footer => Some(Region::Footer),
        _ => None,
    }
}

/// Existing content is navigation, even though upstream touches its layout revision.
pub fn enter(s: &mut Session, page: usize, region: Region) -> Result<bool, String> {
    let before = s.doc.clone();
    let dirty = s.dirty;
    s.run(
        if region == Region::Header {
            "insert.editHeader"
        } else {
            "insert.editFooter"
        },
        &json!({"page":page}),
    )
    .map_err(|e| e.to_string())?;
    let changed = s.doc != before;
    if !changed {
        s.dirty = dirty;
    }
    Ok(changed)
}

pub fn close(s: &mut Session) -> Result<bool, String> {
    if active(s).is_none() {
        return Ok(false);
    }
    let before = s.doc.clone();
    let dirty = s.dirty;
    s.run("insert.closeHeader", &json!({}))
        .map_err(|e| e.to_string())?;
    let changed = s.doc != before;
    if !changed {
        s.dirty = dirty;
    }
    Ok(changed)
}

/// Header/footer hit testing uses the story actually displayed on this page,
/// including inherited, first-page and even-page variants.
pub fn hit(s: &mut Session, page: usize, x: f32, y: f32) -> Option<Pos> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    let layout = s.layout();
    let story = layout
        .header_footer_at(page, y)
        .map(|(story, _)| story)
        .or_else(|| layout.story_at(page, x, y))
        .unwrap_or(StoryRef::Body);
    let pos = layout.hit(page, x, y, story)?;
    s.page_hint = page;
    Some(pos)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberPosition {
    Current,
    Header,
    Footer,
}

/// One undo step, with rollback on any failure. Existing header/footer text is
/// retained; a new paragraph is appended for top/bottom insertion when needed.
pub fn insert_number(
    s: &mut Session,
    page: usize,
    position: NumberPosition,
    align: &str,
    total: bool,
) -> Result<bool, String> {
    if !["left", "center", "right"].contains(&align) {
        return Err("Invalid alignment".into());
    }
    let before = s.doc.clone();
    let snap = s.edit_snapshot();
    let pending = s.pending.clone();
    let back = s.hf_return.clone();
    let hint = s.page_hint;
    let mark = s.undo_mark();
    s.checkpoint(wordcraft_gnome::t!("Page Number"));
    let result = (|| {
        if position != NumberPosition::Current {
            s.join_next_undo();
            enter(
                s,
                page,
                if position == NumberPosition::Header {
                    Region::Header
                } else {
                    Region::Footer
                },
            )?;
            // A collapsed caret at the story end avoids replacing selected text.
            if s.sel.focus.off > 0 {
                s.join_next_undo();
                s.run("text.newParagraph", &json!({}))
                    .map_err(|e| e.to_string())?;
            }
            s.join_next_undo();
            s.run(
                match align {
                    "left" => "para.alignLeft",
                    "right" => "para.alignRight",
                    _ => "para.alignCenter",
                },
                &json!({}),
            )
            .map_err(|e| e.to_string())?;
        }
        s.join_next_undo();
        s.run("insert.pageNumber", &json!({"position":"current"}))
            .map_err(|e| e.to_string())?;
        if total {
            s.join_next_undo();
            s.run("text.insert", &json!({"text":" / "}))
                .map_err(|e| e.to_string())?;
            s.join_next_undo();
            s.run("insert.field", &json!({"instr":"NUMPAGES"}))
                .map_err(|e| e.to_string())?;
        }
        Ok::<_, String>(())
    })();
    match result {
        Ok(()) => {
            s.merge_undo_since(mark);
            Ok(s.doc != before)
        }
        Err(e) => {
            s.restore(snap);
            s.pending = pending;
            s.hf_return = back;
            s.page_hint = hint;
            Err(e)
        }
    }
}
