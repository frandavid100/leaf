//! Bookmarks and comment threads backed exclusively by public WordCraft APIs.
//! These values are read-only snapshots for the sidebar, never a second model.
use serde_json::json;
use wordcraft_doc::{Pos, StoryRef};
use wordcraft_engine::Session;

#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
    pub id: u32,
    pub parent: Option<u32>,
    pub author: String,
    pub date: String,
    pub text: String,
    pub resolved: bool,
    pub anchor: Option<Pos>,
}

pub fn bookmarks(session: &Session) -> Vec<String> {
    session
        .doc
        .bookmarks()
        .into_iter()
        .filter_map(|(name, _)| (!name.starts_with('_')).then_some(name))
        .collect()
}

pub fn can_bookmark(session: &Session) -> bool {
    session.sel.anchor.story == StoryRef::Body
        && session.sel.focus.story == StoryRef::Body
        && session.doc.settings.protection.is_none()
}

pub fn can_comment(session: &Session) -> bool {
    !matches!(
        session.doc.settings.protection.as_deref(),
        Some("readOnly" | "forms")
    )
}

pub fn add_bookmark(session: &mut Session, name: &str) -> Result<(), String> {
    let name = name.trim();
    if !can_bookmark(session) {
        return Err(wordcraft_gnome::t!(
            "Bookmarks can only be added to the editable document body."
        )
        .into());
    }
    if name.starts_with('_') || !wordcraft_engine::cmd::insert::bookmark_name_ok(name) {
        return Err(wordcraft_gnome::t!("Use a name beginning with a letter, with letters, numbers or underscores, and at most 40 characters.").into());
    }
    // Upstream replaces duplicate names. The UI must not silently move an existing mark.
    let exists = std::iter::once(StoryRef::Body)
        .chain(session.doc.parts.keys().copied().map(StoryRef::Part))
        .any(|story| {
            session.doc.para_paths(story).iter().any(|path| {
                session.doc.para(story, path).is_some_and(|p| {
                    p.objects.iter().any(|o| {
                matches!(o, wordcraft_doc::InlineObject::BookmarkStart { name: n } if n == name)
            })
                })
            })
        });
    if exists {
        return Err(wordcraft_gnome::t!("A bookmark with this name already exists.").into());
    }
    command(session, "insert.bookmark", json!({"name": name}))
}

pub fn delete_bookmark(session: &mut Session, name: &str) -> Result<(), String> {
    command(session, "insert.deleteBookmark", json!({"name": name}))
}

pub fn goto_bookmark(session: &mut Session, name: &str) -> Result<(), String> {
    command(session, "edit.goto", json!({"bookmark": name}))
}

pub fn comments(session: &Session) -> Vec<Comment> {
    wordcraft_engine::cmd::review::comment_list(session)
        .into_iter()
        .filter_map(|(id, anchor)| {
            let c = session.doc.comments.get(&id)?;
            Some(Comment {
                id,
                anchor,
                parent: c.parent,
                author: c.author.clone(),
                date: c.date.clone(),
                resolved: c.resolved,
                text: session.doc.plain_text(StoryRef::Part(c.part)),
            })
        })
        .collect()
}

/// Replies inherit their parent's anchor. Broken/cyclic imported threads are safe.
pub fn comment_anchor(session: &Session, id: u32) -> Option<Pos> {
    let list = comments(session);
    anchor_in(&list, id)
}

pub fn anchor_in(list: &[Comment], id: u32) -> Option<Pos> {
    let mut at = id;
    let mut seen = std::collections::HashSet::new();
    while seen.insert(at) {
        let c = list.iter().find(|c| c.id == at)?;
        if let Some(anchor) = &c.anchor {
            return Some(anchor.clone());
        }
        at = c.parent?;
    }
    None
}

pub fn thread_root(list: &[Comment], id: u32) -> u32 {
    let mut at = id;
    let mut seen = std::collections::HashSet::new();
    while seen.insert(at) {
        let Some(parent) = list
            .iter()
            .find(|c| c.id == at)
            .and_then(|c| c.parent)
            .filter(|p| list.iter().any(|c| c.id == *p))
        else {
            return at;
        };
        at = parent;
    }
    seen.into_iter().min().unwrap_or(id)
}

/// All matching threads in reading order, including overlapping comment ranges.
pub fn threads_at(session: &Session, pos: &Pos) -> Vec<u32> {
    let list = comments(session);
    let mut matches = Vec::new();
    for c in &list {
        let Some(anchor) = &c.anchor else { continue };
        if anchor.story != pos.story || anchor > pos {
            continue;
        }
        let end = session
            .doc
            .para_paths(anchor.story)
            .into_iter()
            .skip_while(|p| p != &anchor.path)
            .find_map(|path| {
                let p = session.doc.para(anchor.story, &path)?;
                let off = p.objects_with_offsets().find_map(|(off, o)| {
                    matches!(o,wordcraft_doc::InlineObject::CommentEnd{id}if *id==c.id)
                        .then_some(off)
                })?;
                Some(Pos {
                    story: anchor.story,
                    path,
                    off,
                })
            });
        if end.as_ref().is_some_and(|end| pos < end) {
            let root = thread_root(&list, c.id);
            if !matches.contains(&root) {
                matches.push(root);
            }
        }
    }
    matches
}

pub fn goto_comment(session: &mut Session, id: u32) -> Result<(), String> {
    let list = comments(session);
    let mut anchor = anchor_in(&list, id).ok_or(wordcraft_gnome::t!(
        "This comment has no visible anchor in the document."
    ))?;
    let root = list
        .iter()
        .find(|c| c.anchor.as_ref() == Some(&anchor))
        .map(|c| c.id)
        .unwrap_or(id);
    anchor.off += wordcraft_doc::para::OBJ.len_utf8();
    let end = session
        .doc
        .para_paths(anchor.story)
        .into_iter()
        .skip_while(|p| *p != anchor.path)
        .find_map(|path| {
            let para = session.doc.para(anchor.story, &path)?;
            let off = para.objects_with_offsets().find_map(|(off, object)| {
                (matches!(object, wordcraft_doc::InlineObject::CommentEnd { id } if *id == root)
                    && (path != anchor.path || off >= anchor.off))
                    .then_some(off)
            })?;
            Some(Pos {
                story: anchor.story,
                path,
                off,
            })
        })
        .unwrap_or_else(|| anchor.clone());
    crate::selection::select_range(session, &anchor, &end)
}

fn nonempty(text: &str) -> Result<(), String> {
    if text.trim().is_empty() {
        Err(wordcraft_gnome::t!("Enter some comment text.").into())
    } else {
        Ok(())
    }
}

pub fn add_comment(session: &mut Session, text: &str) -> Result<(), String> {
    nonempty(text)?;
    command(session, "review.newComment", json!({"text": text}))
}

pub fn edit_comment(session: &mut Session, id: u32, text: &str) -> Result<(), String> {
    nonempty(text)?;
    command(
        session,
        "review.editComment",
        json!({"id": id, "text": text}),
    )
}

pub fn reply(session: &mut Session, id: u32, text: &str) -> Result<(), String> {
    nonempty(text)?;
    command(session, "review.reply", json!({"id": id, "text": text}))
}

pub fn resolve(session: &mut Session, id: u32, value: bool) -> Result<(), String> {
    command(
        session,
        "review.resolveComment",
        json!({"id": id, "value": value}),
    )
}

pub fn delete_comment(session: &mut Session, id: u32) -> Result<(), String> {
    command(session, "review.deleteComment", json!({"id": id}))
}

fn command(session: &mut Session, id: &str, params: serde_json::Value) -> Result<(), String> {
    session
        .run(id, &params)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
