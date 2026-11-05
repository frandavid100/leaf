//! Document references use WordCraft's canonical stories, fields and history.
use serde_json::json;
use wordcraft_doc::{
    PartKind, Pos, StoryRef,
    para::{InlineObject, NoteKind},
};
use wordcraft_engine::Session;

pub fn insert_footnote(s: &mut Session) -> Result<(), String> {
    if s.sel.focus.story != StoryRef::Body {
        return Err(wordcraft_gnome::t!(
            "Place the caret in the document body to insert a footnote."
        )
        .into());
    }
    s.run("references.footnote", &json!({}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn footnote_refs(s: &Session) -> Vec<(Pos, u32)> {
    s.doc
        .para_paths(StoryRef::Body)
        .into_iter()
        .flat_map(|path| {
            let p = s.doc.para(StoryRef::Body, &path).unwrap();
            p.object_offsets()
                .into_iter()
                .filter_map(move |off| match p.object_at(off) {
                    Some(InlineObject::NoteRef {
                        kind: NoteKind::Footnote,
                        id,
                        ..
                    }) => Some((
                        Pos {
                            story: StoryRef::Body,
                            path: path.clone(),
                            off,
                        },
                        *id,
                    )),
                    _ => None,
                })
        })
        .collect()
}

pub fn open_footnote(s: &mut Session, next: bool) -> Result<(), String> {
    let refs = footnote_refs(s);
    let current = match s.sel.focus.story {
        StoryRef::Part(id) => refs
            .iter()
            .find(|(_, note)| *note == id)
            .map(|(p, _)| p.clone()),
        StoryRef::Body => Some(s.sel.focus.clone()),
    };
    let target = if next {
        refs.iter()
            .find(|(p, _)| current.as_ref().is_some_and(|c| p > c))
            .or(refs.first())
    } else {
        refs.iter().find(|(p, _)| {
            p.path == s.sel.focus.path
                && p.story == s.sel.focus.story
                && (p.off == s.sel.focus.off
                    || p.off + wordcraft_doc::para::OBJ.len_utf8() == s.sel.focus.off)
        })
    };
    let (_, id) = target.ok_or(wordcraft_gnome::t!(
        "Select a footnote reference, or use Next Footnote."
    ))?;
    let pos = s.doc.end_of(StoryRef::Part(*id));
    crate::selection::set_caret(s, &pos, false)
}

pub fn return_to_reference(s: &mut Session) -> Result<(), String> {
    let StoryRef::Part(id) = s.sel.focus.story else {
        return Err(wordcraft_gnome::t!("Place the caret in a footnote first.").into());
    };
    if s.doc
        .parts
        .get(&id)
        .is_none_or(|p| p.kind != PartKind::Footnote)
    {
        return Err(wordcraft_gnome::t!("Place the caret in a footnote first.").into());
    }
    let (mut pos, _) = footnote_refs(s)
        .into_iter()
        .find(|(_, note)| *note == id)
        .ok_or(wordcraft_gnome::t!(
            "The footnote reference could not be found."
        ))?;
    pos.off += wordcraft_doc::para::OBJ.len_utf8();
    crate::selection::set_caret(s, &pos, false)
}

pub fn update(s: &mut Session) -> Result<(), String> {
    let snap = s.edit_snapshot();
    let mark = s.undo_mark();
    s.checkpoint(wordcraft_gnome::t!("Update References"));
    let result = (|| {
        s.join_next_undo();
        s.run("references.updateFields", &json!({"all":true}))
            .map_err(|e| e.to_string())?;
        if s.doc.para_paths(StoryRef::Body).iter().any(|path| {
            s.doc.para(StoryRef::Body, path).is_some_and(|p| p.object_offsets().iter().any(|off|
                matches!(p.object_at(*off), Some(InlineObject::Field { instr, .. }) if instr.trim_start().starts_with("TOC "))))
        }) {
            s.join_next_undo();
            s.run("references.updateToc", &json!({})).map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    if result.is_err() {
        s.restore(snap);
    } else {
        s.merge_undo_since(mark);
    }
    result
}
