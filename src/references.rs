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
    let before = s.doc.clone();
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
            // The regenerated contents can move reference targets onto new pages.
            s.join_next_undo();
            s.run("references.updateFields", &json!({"all":true})).map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    if result.is_err() || s.doc == before {
        s.restore(snap);
    } else {
        s.merge_undo_since(mark);
    }
    result
}

/// A caption belongs only to an explicitly selected picture or the targeted table.
pub fn caption_kind(s: &Session) -> Option<&'static str> {
    if crate::pictures::selected(s).is_some() {
        Some("Figure")
    } else if crate::tables::in_table(s) {
        Some("Table")
    } else {
        None
    }
}
pub fn insert_caption(s: &mut Session) -> Result<(), String> {
    crate::captions::set_visible(s, true)
}

pub fn insert_field(s: &mut Session, instr: &str) -> Result<(), String> {
    s.run("insert.field", &json!({"instr":instr}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn derived_field(instr: &str) -> bool {
    let field = wordcraft_engine::cmd::references::FieldCode::parse(instr);
    // AUTHOR/TITLE with a literal argument are setters in the public engine.
    // Automatic display refresh must not overwrite the user's metadata.
    if matches!(field.name.as_str(), "AUTHOR" | "TITLE") && !field.args.is_empty() {
        return false;
    }
    matches!(
        field.name.as_str(),
        "REF"
            | "PAGEREF"
            | "SEQ"
            | "STYLEREF"
            | "TOC"
            | "PAGE"
            | "NUMPAGES"
            | "AUTHOR"
            | "TITLE"
            | "FILENAME"
    )
}

/// Refresh only document-derived values. Other fields retain their values and locks.
/// Generated values are recomputed after source Undo/Redo without adding a history step.
pub fn refresh_derived(s: &mut Session) -> Result<bool, String> {
    let mut probe = Session::new(s.doc.clone());
    probe.sel = s.sel.clone();
    probe.page_hint = s.page_hint;
    probe.path = s.path.clone();
    probe.pending = s.pending.clone();
    if !refresh_derived_inner(&mut probe)? {
        return Ok(false);
    }
    // Derived values are recomputable document data, not a separate user edit.
    // Keep the engine's source undo/redo stacks intact, including redo after Undo.
    let dirty = s.dirty;
    s.doc = probe.doc;
    s.sel = probe.sel;
    s.touch();
    s.dirty = dirty;
    Ok(true)
}

fn refresh_derived_inner(s: &mut Session) -> Result<bool, String> {
    let needed = std::iter::once(StoryRef::Body)
        .chain(s.doc.parts.keys().copied().map(StoryRef::Part))
        .any(|story| s.doc.para_paths(story).into_iter().any(|path| {
            s.doc.para(story, &path).is_some_and(|p| p.objects.iter().any(|o| {
                matches!(o, InlineObject::Field { instr, .. } | InlineObject::FieldStart { instr, .. } if derived_field(instr))
            }))
        }));
    if !needed {
        return Ok(false);
    }
    let snapshot = s.edit_snapshot();
    let before = s.doc.clone();
    let selection = s.sel.clone();
    let mark = s.undo_mark().saturating_sub(1);
    let pending = s.pending.clone();
    let hint = s.page_hint;
    // Lock non-derived fields while invoking the canonical public update command.
    let stories: Vec<_> = std::iter::once(StoryRef::Body)
        .chain(s.doc.parts.keys().copied().map(StoryRef::Part))
        .collect();
    let mut locks = Vec::new();
    for story in &stories {
        for path in s.doc.para_paths(*story) {
            let p = s.doc.para_mut(*story, &path).map_err(|e| e.to_string())?;
            for o in &mut p.objects {
                if let InlineObject::Field { instr, locked, .. }
                | InlineObject::FieldStart { instr, locked, .. } = o
                {
                    if !derived_field(instr) {
                        locks.push(*locked);
                        *locked = true;
                    }
                }
            }
        }
    }
    let result = update(s);
    if result.is_err() {
        s.restore(snapshot);
        s.pending = pending;
        s.page_hint = hint;
        return result.map(|_| false);
    }
    let mut locks = locks.into_iter();
    for story in &stories {
        for path in s.doc.para_paths(*story) {
            let p = s.doc.para_mut(*story, &path).map_err(|e| e.to_string())?;
            for o in &mut p.objects {
                if let InlineObject::Field { instr, locked, .. }
                | InlineObject::FieldStart { instr, locked, .. } = o
                {
                    if !derived_field(instr) {
                        *locked = locks.next().unwrap_or(*locked);
                    }
                }
            }
        }
    }
    if s.doc == before {
        s.restore(snapshot);
        s.pending = pending;
        s.page_hint = hint;
        return Ok(false);
    }
    // TOC regeneration can insert/remove top-level entry paragraphs ahead of the caret.
    let delta = s.doc.body.len() as isize - before.body.len() as isize;
    let toc_end=before.body.iter().enumerate().filter(|(_,b)|b.as_para().is_some_and(|p|
        p.props.style.as_deref().is_some_and(|st|st.starts_with("TOC"))
        || p.objects.iter().any(|o|matches!(o,InlineObject::Field{instr,..}|InlineObject::FieldStart{instr,..} if instr.trim_start().starts_with("TOC ")))))
        .map(|(i,_)|i).max();
    let shift = |mut p: Pos| {
        if p.story == StoryRef::Body
            && let Some(i) = p.path.0.first_mut()
        {
            if toc_end.is_some_and(|end| *i as usize > end) {
                *i = (*i as isize + delta).max(0) as u32;
            }
        }
        s.doc.clamp(&p)
    };
    s.sel = wordcraft_engine::Selection {
        anchor: shift(selection.anchor),
        focus: shift(selection.focus),
    };
    s.pending = pending;
    s.page_hint = hint;
    s.merge_undo_since(mark);
    Ok(true)
}

pub fn toc_at(s: &Session, pos: &Pos) -> bool {
    let mut probe = Session::new(s.doc.clone());
    probe.sel = wordcraft_engine::Selection::caret(pos.clone());
    wordcraft_engine::cmd::references::caret_in_toc(&probe)
}

/// The pinned engine's insertion command replaces the document's existing TOC,
/// independently of the caret. Never turn an explicit Create operation into Edit.
pub fn insert_toc(s: &mut Session, args: &serde_json::Value) -> Result<(), String> {
    if toc_field(s).is_some() {
        return Err(crate::t!("The current engine supports one table of contents per document. Edit the existing table of contents instead.").into());
    }
    s.run("references.toc", args)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn toc_field(s: &Session) -> Option<(Pos, String)> {
    for path in s.doc.para_paths(StoryRef::Body) {
        let p = s.doc.para(StoryRef::Body, &path)?;
        for off in p.object_offsets() {
            if let Some(
                InlineObject::Field {
                    instr,
                    locked: false,
                    ..
                }
                | InlineObject::FieldStart {
                    instr,
                    locked: false,
                    ..
                },
            ) = p.object_at(off)
            {
                let f = wordcraft_engine::cmd::references::FieldCode::parse(instr);
                if f.name == "TOC" && !f.has('c') {
                    return Some((
                        Pos {
                            story: StoryRef::Body,
                            path,
                            off,
                        },
                        instr.clone(),
                    ));
                }
            }
        }
    }
    None
}

pub fn edit_toc(s: &mut Session, code: &str, title: Option<&str>) -> Result<(), String> {
    let (at, _) = toc_field(s).ok_or(wordcraft_gnome::t!(
        "The table of contents could not be found."
    ))?;
    let snapshot = s.edit_snapshot();
    let mark = s.undo_mark();
    let result = (|| {
        crate::selection::set_caret(s, &at, false)?;
        s.run("fields.setCode", &json!({"code":code}))
            .map_err(|e| e.to_string())?;
        if let Some(title) = title {
            if let Some((at, _)) = toc_field(s) {
                if s.doc
                    .para_at(&at)
                    .is_some_and(|p| p.props.style.as_deref() == Some("TOCHeading"))
                {
                    let start = Pos {
                        off: 0,
                        ..at.clone()
                    };
                    crate::selection::select_range(s, &start, &at)?;
                    s.join_next_undo();
                    s.run("text.insert", &json!({"text":title}))
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    })();
    if result.is_err() {
        s.restore(snapshot)
    } else {
        s.merge_undo_since(mark)
    }
    result
}

/// Canonical TOC span: generated entries belong to their TOC field; imported
/// range fields retain their own explicit boundaries. Styles alone are not a TOC.
pub fn toc_span_at(s: &Session, pos: &Pos) -> Option<(Pos, Pos)> {
    if pos.story != StoryRef::Body {
        return None;
    }
    for range in s.doc.field_ranges(StoryRef::Body) {
        let field = wordcraft_engine::cmd::references::FieldCode::parse(&range.instr);
        if field.name == "TOC" && !field.has('c') && range.start <= *pos && *pos <= range.end {
            return Some((range.start, range.end));
        }
    }
    for (start, block) in s.doc.body.iter().enumerate() {
        let Some(p) = block.as_para() else { continue };
        if !p.objects.iter().any(|o| matches!(o, InlineObject::Field{instr,..} if {
            let f=wordcraft_engine::cmd::references::FieldCode::parse(instr); f.name == "TOC" && !f.has('c')
        })) { continue; }
        let mut end = start;
        while s
            .doc
            .body
            .get(end + 1)
            .and_then(|b| b.as_para())
            .is_some_and(|p| {
                p.props
                    .style
                    .as_deref()
                    .is_some_and(|st| st.starts_with("TOC") && st != "TOCHeading")
            })
        {
            end += 1;
        }
        let a = Pos::body(start, 0);
        let z = Pos::body(end, s.doc.body[end].as_para()?.len());
        if a <= *pos && *pos <= z {
            return Some((a, z));
        }
    }
    None
}

/// Follow the stored hyperlink/bookmark, never the cached page number or title.
pub fn toc_destination(s: &Session, pos: &Pos) -> Option<String> {
    toc_span_at(s, pos)?;
    let (_, _, url) = crate::links::range_at(s, pos)?;
    let name = url.strip_prefix('#')?;
    s.doc
        .bookmarks()
        .iter()
        .any(|(n, _)| n == name)
        .then_some(url)
}
