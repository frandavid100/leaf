//! Explicit caption ownership in the canonical document. Pictures use named
//! bookmark ranges; tables use a standard, unlocked OOXML content-control tag.
//! Caption text remains an ordinary editable paragraph with native SEQ fields.
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use wordcraft_doc::{
    CharProps, Document, Path, Pos, StoryRef, control::ContentControl, para::InlineObject,
};
use wordcraft_engine::{CmdError, CmdResult, CommandSpec, Registry, Selection, Session};

const OBJECT: &str = "_LeafObject";
const CAPTION: &str = "_LeafCaption";
const TABLE: &str = "leaf:caption:";
const HIDDEN: &str = "Leaf.Caption.Hidden.";
const OBJ: usize = wordcraft_doc::para::OBJ.len_utf8();

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Image(Pos),
    Table { story: StoryRef, path: Path },
}
#[derive(Clone, Debug)]
pub struct Caption {
    pub id: String,
    pub owner: Target,
    pub pos: Pos,
    pub visible: bool,
}

fn identifier<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
    name.strip_prefix(prefix)
        .filter(|id| !id.is_empty() && id.len() <= 20 && id.bytes().all(|c| c.is_ascii_digit()))
}
fn tables(doc: &Document) -> Vec<Path> {
    let mut found = HashSet::new();
    for mut path in doc.para_paths(StoryRef::Body) {
        while let Some((table, _, _)) = path.cell() {
            found.insert(table.clone());
            path = table;
        }
    }
    let mut out: Vec<_> = found.into_iter().collect();
    out.sort();
    out
}

/// Ambiguous, copied or deleted anchors never acquire a nearby object by guess.
pub fn all(doc: &Document) -> Vec<Caption> {
    let mut owners: HashMap<String, Vec<Target>> = HashMap::new();
    let mut captions: HashMap<String, Vec<Pos>> = HashMap::new();
    for path in doc.para_paths(StoryRef::Body) {
        let Some(p) = doc.para(StoryRef::Body, &path) else {
            continue;
        };
        let objects: Vec<_> = p.objects_with_offsets().collect();
        for (off, object) in &objects {
            let InlineObject::BookmarkStart { name } = object else {
                continue;
            };
            if let Some(id) = identifier(name, CAPTION) {
                let complete = objects.iter().any(|(end, o)| {
                    end > off && matches!(o, InlineObject::BookmarkEnd { name: n } if n == name)
                });
                if complete {
                    captions.entry(id.into()).or_default().push(Pos {
                        story: StoryRef::Body,
                        path: path.clone(),
                        off: *off,
                    });
                }
            }
            if let Some(id) = identifier(name, OBJECT) {
                let end = objects.iter().find_map(|(end, o)| {
                    (end > off && matches!(o, InlineObject::BookmarkEnd { name: n } if n == name))
                        .then_some(*end)
                });
                if let Some(end) = end {
                    let images: Vec<_> = objects
                        .iter()
                        .filter_map(|(at, o)| {
                            (*at > *off && *at < end && matches!(o, InlineObject::Image { .. }))
                                .then_some(*at)
                        })
                        .collect();
                    if let [at] = images.as_slice() {
                        owners
                            .entry(id.into())
                            .or_default()
                            .push(Target::Image(Pos {
                                story: StoryRef::Body,
                                path: path.clone(),
                                off: *at,
                            }));
                    }
                }
            }
        }
    }
    for path in tables(doc) {
        if let Some(t) = doc.table(StoryRef::Body, &path) {
            for control in &t.controls.open {
                if let Some(id) = identifier(&control.tag, TABLE) {
                    owners.entry(id.into()).or_default().push(Target::Table {
                        story: StoryRef::Body,
                        path: path.clone(),
                    });
                }
            }
        }
    }
    let mut out = Vec::new();
    for (id, positions) in captions {
        if positions.len() != 1 {
            continue;
        }
        let Some(owner) = owners
            .remove(&id)
            .filter(|x| x.len() == 1)
            .and_then(|x| x.into_iter().next())
        else {
            continue;
        };
        let pos = positions.into_iter().next().unwrap();
        let visible = doc
            .para_at(&pos)
            .is_some_and(|p| p.mark.hidden != Some(true));
        out.push(Caption {
            id,
            owner,
            pos,
            visible,
        });
    }
    out.sort_by(|a, b| a.pos.cmp(&b.pos));
    out
}
pub fn target(s: &Session) -> Option<Target> {
    if let Some((pos, _, _)) = crate::pictures::selected(s) {
        return (pos.story == StoryRef::Body).then_some(Target::Image(pos));
    }
    table_target(s)
}
pub fn table_target(s: &Session) -> Option<Target> {
    if s.sel.focus.story != StoryRef::Body {
        return None;
    }
    if let Some(cells) = s.cell_range() {
        return Some(Target::Table {
            story: cells.story,
            path: cells.table.clone(),
        });
    }
    s.sel
        .focus
        .path
        .cell()
        .map(|(path, _, _)| Target::Table {
            story: s.sel.focus.story,
            path,
        })
        .or_else(|| {
            all(&s.doc).into_iter().find_map(|c| {
                (c.pos.story == s.sel.focus.story
                    && c.pos.path == s.sel.focus.path
                    && matches!(c.owner, Target::Table { .. }))
                .then_some(c.owner)
            })
        })
}
pub fn for_target(doc: &Document, owner: &Target) -> Option<Caption> {
    all(doc).into_iter().find(|c| &c.owner == owner)
}
pub fn at(doc: &Document, pos: &Pos) -> Option<Caption> {
    all(doc).into_iter().find(|c| {
        (c.pos.story == pos.story && c.pos.path == pos.path)
            || match &c.owner {
                Target::Image(p) => {
                    p.story == pos.story
                        && p.path == pos.path
                        && (p.off..=p.off + OBJ).contains(&pos.off)
                }
                Target::Table { story, path } => {
                    *story == pos.story && pos.path.0.starts_with(&path.0)
                }
            }
    })
}
pub fn text(doc: &Document, caption: &Caption) -> String {
    doc.para_at(&caption.pos)
        .map(|p| {
            p.plain_text()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
}
pub fn requires_docx(doc: &Document) -> bool {
    doc.bookmarks()
        .iter()
        .any(|(name, _)| identifier(name, CAPTION).is_some() || identifier(name, OBJECT).is_some())
        || doc.custom_props.iter().any(|p| p.name.starts_with(HIDDEN))
}
fn next_id(doc: &Document) -> Result<String, CmdError> {
    let mut largest = 0u64;
    for (name, _) in doc.bookmarks() {
        for prefix in [CAPTION, OBJECT] {
            if let Some(n) = identifier(&name, prefix).and_then(|id| id.parse::<u64>().ok()) {
                largest = largest.max(n);
            }
        }
    }
    for path in tables(doc) {
        if let Some(t) = doc.table(StoryRef::Body, &path) {
            for control in &t.controls.open {
                if let Some(n) =
                    identifier(&control.tag, TABLE).and_then(|id| id.parse::<u64>().ok())
                {
                    largest = largest.max(n);
                }
            }
        }
    }
    largest
        .checked_add(1)
        .map(|n| n.to_string())
        .ok_or_else(|| CmdError::Failed("Caption identifiers are exhausted".into()))
}
fn bookmark(s: &mut Session, pos: &Pos, end: usize, name: &str) -> Result<(), CmdError> {
    s.doc.insert_object(
        &Pos {
            off: end,
            ..pos.clone()
        },
        InlineObject::BookmarkEnd { name: name.into() },
        &CharProps::default(),
    )?;
    s.doc.insert_object(
        pos,
        InlineObject::BookmarkStart { name: name.into() },
        &CharProps::default(),
    )?;
    Ok(())
}
fn create(s: &mut Session, owner: Target) -> Result<Caption, CmdError> {
    let id = next_id(&s.doc)?;
    let (label, desired) = match &owner {
        Target::Image(p) => ("Figure", p.path.with_last(p.path.last() + 1)),
        Target::Table { path, .. } => ("Table", path.with_last(path.last() + 1)),
    };
    let native = s
        .registry
        .get("references.caption")
        .ok_or_else(|| CmdError::Failed("Caption command unavailable".into()))?
        .run;
    native(s, &json!({"label":label,"position":"below"}))?;
    // The public command places a nested table's caption after its top-level
    // container. Move that canonical paragraph into the actual table's container.
    if s.sel.focus.path != desired {
        let old = s.sel.focus.path.clone();
        let block = s.doc.remove_block(s.sel.focus.story, &old)?;
        s.doc.insert_block(s.sel.focus.story, &desired, block)?;
        s.sel = Selection::caret(Pos {
            path: desired.clone(),
            ..s.sel.focus.clone()
        });
    }
    let insert = s
        .registry
        .get("text.insert")
        .ok_or_else(|| CmdError::Failed("Text command unavailable".into()))?
        .run;
    let end = s.doc.para(StoryRef::Body, &desired).map_or(0, |p| p.len());
    s.sel = Selection::caret(Pos {
        story: StoryRef::Body,
        path: desired.clone(),
        off: end,
    });
    insert(s, &json!({"text":": "}))?;
    let end = s.doc.para(StoryRef::Body, &desired).map_or(0, |p| p.len());
    bookmark(
        s,
        &Pos {
            story: StoryRef::Body,
            path: desired.clone(),
            off: 0,
        },
        end,
        &format!("{CAPTION}{id}"),
    )?;
    let owner = match owner {
        Target::Image(pos) => {
            bookmark(s, &pos, pos.off + OBJ, &format!("{OBJECT}{id}"))?;
            Target::Image(Pos {
                off: pos.off + OBJ,
                ..pos
            })
        }
        Target::Table { story, path } => {
            let t = s.doc.table_mut(story, &path)?;
            // A whole-table content control survives row/cell edits and nesting.
            t.controls.open.push(ContentControl {
                tag: format!("{TABLE}{id}"),
                block: true,
                ..Default::default()
            });
            t.controls.close = t.controls.close.saturating_add(1);
            Target::Table { story, path }
        }
    };
    let pos = Pos {
        story: StoryRef::Body,
        path: desired,
        off: 0,
    };
    s.sel = Selection::caret(Pos {
        off: end + OBJ,
        ..pos.clone()
    });
    Ok(Caption {
        id,
        owner,
        pos,
        visible: true,
    })
}
fn visibility(s: &mut Session, caption: &Caption, show: bool) -> Result<(), CmdError> {
    let key = format!("{HIDDEN}{}", caption.id);
    if !show {
        let p = s
            .doc
            .para_at(&caption.pos)
            .ok_or_else(|| CmdError::Failed("Caption disappeared".into()))?;
        let ranges: Vec<_> = p
            .run_ranges()
            .map(|(r, c)| (r.start, r.end, c.hidden))
            .collect();
        let previous = json!({"mark":p.mark.hidden,"ranges":ranges});
        s.doc.set_custom_prop(&key, &previous.to_string());
        let p = s.doc.para_mut(caption.pos.story, &caption.pos.path)?;
        p.format(0, p.len(), &|c| c.hidden = Some(true))?;
        p.mark.hidden = Some(true);
    } else {
        let previous: Value = s
            .doc
            .custom_prop(&key)
            .and_then(|v| serde_json::from_str(v).ok())
            .unwrap_or(Value::Null);
        let p = s.doc.para_mut(caption.pos.story, &caption.pos.path)?;
        p.format(0, p.len(), &|c| c.hidden = None)?;
        if let Some(ranges) = previous["ranges"].as_array() {
            for r in ranges {
                let a = r[0].as_u64().unwrap_or(0) as usize;
                let b = (r[1].as_u64().unwrap_or(0) as usize).min(p.len());
                if a <= b && p.text.is_char_boundary(a) && p.text.is_char_boundary(b) {
                    p.format(a, b, &|c| c.hidden = r[2].as_bool())?;
                }
            }
        }
        p.mark.hidden = previous["mark"].as_bool();
        s.doc.remove_custom_prop(&key);
    }
    Ok(())
}
fn command(s: &mut Session, v: &Value) -> CmdResult {
    let owner = target(s).ok_or_else(|| CmdError::Disabled("Select an image or table".into()))?;
    let show = v["show"]
        .as_bool()
        .ok_or_else(|| CmdError::Params("show must be a boolean".into()))?;
    let caption = match for_target(&s.doc, &owner) {
        Some(c) => {
            if c.visible != show {
                visibility(s, &c, show)?;
            }
            c
        }
        None if show => create(s, owner)?,
        None => return Ok(json!({"shown":false})),
    };
    if show {
        let p = s
            .doc
            .para_at(&caption.pos)
            .ok_or_else(|| CmdError::Failed("Caption disappeared".into()))?;
        let end=p.object_offsets().into_iter().find(|off|matches!(p.object_at(*off),Some(InlineObject::BookmarkEnd { name }) if name==&format!("{CAPTION}{}",caption.id))).unwrap_or(p.len());
        s.sel = Selection::caret(Pos {
            off: end,
            ..caption.pos
        });
    }
    Ok(json!({"shown":show}))
}
fn register(s: &mut Session) {
    if s.registry.get("leaf.caption").is_some() {
        return;
    }
    let mut specs = s.registry.all().to_vec();
    specs.push(
        CommandSpec::new("leaf.caption", "Show Caption", "Elements", command).when(|s| {
            wordcraft_engine::cmd::protect::enforced(&s.doc)
                .then_some("Document protection prevents changing captions")
        }),
    );
    specs.push(
        CommandSpec::new(
            "leaf.captionImageBounds",
            "Move or Resize",
            "Elements",
            image_bounds_command,
        )
        .when(|s| {
            s.registry
                .get("arrange.bounds")
                .and_then(|c| (c.enabled)(s))
        }),
    );
    s.registry = std::sync::Arc::new(Registry::new(specs));
}

fn image_bounds_command(s: &mut Session, v: &Value) -> CmdResult {
    let old =
        crate::pictures::selected(s).and_then(|(p, _, _)| for_target(&s.doc, &Target::Image(p)));
    let native = s
        .registry
        .get("arrange.bounds")
        .ok_or_else(|| CmdError::Failed("Image movement unavailable".into()))?
        .run;
    let result = native(s, v)?;
    let Some(caption) = old else {
        return Ok(result);
    };
    let Some((mut at, _, _)) = crate::pictures::selected(s) else {
        return Err(CmdError::Failed("Moved image disappeared".into()));
    };
    if for_target(&s.doc, &Target::Image(at.clone())).is_some() {
        return Ok(result);
    };
    let name = format!("{OBJECT}{}", caption.id);
    let mut markers = Vec::new();
    for path in s.doc.para_paths(StoryRef::Body) {
        if let Some(p) = s.doc.para(StoryRef::Body, &path) {
            for (off, o) in p.objects_with_offsets() {
                if matches!(o,InlineObject::BookmarkStart { name:n } | InlineObject::BookmarkEnd { name:n } if n==&name)
                {
                    markers.push(Pos {
                        story: StoryRef::Body,
                        path: path.clone(),
                        off,
                    });
                }
            }
        }
    }
    markers.sort();
    for p in markers.into_iter().rev() {
        s.doc.delete_range(
            &p,
            &Pos {
                off: p.off + OBJ,
                ..p.clone()
            },
        )?;
        if p.story == at.story && p.path == at.path && p.off < at.off {
            at.off = at.off.saturating_sub(OBJ);
        }
    }
    bookmark(s, &at, at.off + OBJ, &name)?;
    at.off += OBJ;
    s.sel = Selection {
        anchor: at.clone(),
        focus: Pos {
            off: at.off + OBJ,
            ..at
        },
    };
    Ok(result)
}

/// Preserve the explicit owner range in the same native history transaction.
pub fn image_bounds(s: &mut Session, v: &Value) -> Result<(), String> {
    register(s);
    s.run("leaf.captionImageBounds", v)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct CueRect {
    pub page: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl CueRect {
    pub fn contains(&self, page: usize, x: f32, y: f32) -> bool {
        self.page == page
            && x >= self.x
            && x <= self.x + self.w
            && y >= self.y
            && y <= self.y + self.h
    }
    fn union(&mut self, other: &Self) {
        let right = (self.x + self.w).max(other.x + other.w);
        let bottom = (self.y + self.h).max(other.y + other.h);
        self.x = self.x.min(other.x);
        self.y = self.y.min(other.y);
        self.w = right - self.x;
        self.h = bottom - self.y;
    }
}
#[derive(Clone, Debug)]
pub struct Cue {
    pub caption: Caption,
    pub object: Vec<CueRect>,
    pub text: Vec<CueRect>,
    pub bridge: Vec<CueRect>,
}
impl Cue {
    pub fn rects(&self) -> impl Iterator<Item = &CueRect> {
        self.object.iter().chain(&self.text).chain(&self.bridge)
    }
}

fn clear_bridge(
    layout: &wordcraft_engine::layout::DocLayout,
    r: &CueRect,
    caption: &Caption,
) -> bool {
    use wordcraft_engine::layout::Placed;
    let overlaps = |x: f32, y: f32, w: f32, h: f32| {
        r.x < x + w && r.x + r.w > x && r.y < y + h && r.y + r.h > y
    };
    layout.pages.get(r.page).is_some_and(|page|page.items.iter().all(|item|match item {
        Placed::Lines {story,path,para,l0,l1,x,y,..}=> {
            if *story==caption.pos.story && *path==caption.pos.path {return true;}
            if matches!(&caption.owner,Target::Image(p) if p.story==*story && p.path==*path) {return true;}
            let origin=para.lines.get(*l0).map_or(0.,|l|l.top);
            para.lines[*l0..*l1].iter().all(|line|!overlaps(*x+line.left,*y+line.top-origin,line.right-line.left,line.height))
        },
        Placed::Object {rect,story,path,off,..}=> {
            if matches!(&caption.owner,Target::Image(p) if p.story==*story && p.path==*path && p.off==*off) {return true;}
            !overlaps(rect.x,rect.y,rect.w,rect.h)
        },
        _=>true,
    }))
}

/// Disposable editor geometry only. Neither reading it nor hovering changes the
/// document, pagination, source history, or exported output.
pub fn cues(s: &mut Session) -> Vec<Cue> {
    use wordcraft_engine::layout::Placed;
    let captions = all(&s.doc);
    let layout = s.layout();
    let mut out = Vec::new();
    for caption in captions.into_iter().filter(|c| c.visible) {
        let mut object = Vec::new();
        match &caption.owner {
            Target::Image(p) => {
                if let Some(hit) = layout.object(p, s.page_hint) {
                    object.push(CueRect {
                        page: hit.page,
                        x: hit.rect.x,
                        y: hit.rect.y,
                        w: hit.rect.w,
                        h: hit.rect.h,
                    });
                }
            }
            Target::Table { story, path } => {
                for (page, sheet) in layout.pages.iter().enumerate() {
                    let mut bound: Option<CueRect> = None;
                    for item in &sheet.items {
                        if let Placed::Cell {
                            rect,
                            table,
                            story: st,
                            ..
                        } = item
                        {
                            if st == story && table == path {
                                let r = CueRect {
                                    page,
                                    x: rect.x,
                                    y: rect.y,
                                    w: rect.w,
                                    h: rect.h,
                                };
                                if let Some(b) = &mut bound {
                                    b.union(&r);
                                } else {
                                    bound = Some(r);
                                }
                            }
                        }
                    }
                    object.extend(bound);
                }
            }
        }
        let Some(p) = s.doc.para_at(&caption.pos) else {
            continue;
        };
        let mut start = caption.pos.clone();
        start.off = 0;
        let end = Pos {
            off: p.len(),
            ..start.clone()
        };
        let mut text: Vec<CueRect> = Vec::new();
        for (page, r) in layout.selection_rects(&s.doc, &start, &end, s.page_hint) {
            if r.w <= 0. || r.h <= 0. {
                continue;
            }
            let rect = CueRect {
                page,
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
            };
            // Keep separate columns separate instead of drawing a box through
            // unrelated content between them.
            if let Some(bound) = text
                .iter_mut()
                .find(|b| b.page == page && (b.x - (rect.x)).abs() < 24.)
            {
                bound.union(&rect);
            } else {
                text.push(rect);
            }
        }
        let owner_path = match &caption.owner {
            Target::Image(p) => &p.path,
            Target::Table { path, .. } => path,
        };
        let adjacent = owner_path.with_last(owner_path.last() + 1) == caption.pos.path;
        let clean = match &caption.owner {
            Target::Image(p) => s
                .doc
                .para_at(p)
                .is_some_and(|p| p.plain_text().trim().is_empty()),
            Target::Table { .. } => true,
        };
        let mut bridge = Vec::new();
        if adjacent && clean {
            for a in &object {
                for b in &text {
                    let x = a.x.max(b.x);
                    let right = (a.x + a.w).min(b.x + b.w);
                    let y = a.y + a.h;
                    let gap = b.y - y;
                    if a.page == b.page && right > x && (0. ..=24.).contains(&gap) {
                        let r = CueRect {
                            page: a.page,
                            x,
                            y,
                            w: right - x,
                            h: gap,
                        };
                        if clear_bridge(&layout, &r, &caption) {
                            bridge.push(r);
                        }
                    }
                }
            }
        }
        out.push(Cue {
            caption,
            object,
            text,
            bridge,
        });
    }
    out
}
pub fn set_visible(s: &mut Session, show: bool) -> Result<(), String> {
    let owner = target(s).ok_or_else(|| {
        wordcraft_gnome::t!("Select an image or place the caret in a table to insert a caption.")
            .to_string()
    })?;
    if let Some(c) = for_target(&s.doc, &owner) {
        if c.visible == show {
            if show {
                let end = s
                    .doc
                    .para_at(&c.pos)
                    .map_or(0, |p| p.len().saturating_sub(OBJ));
                crate::selection::set_caret(s, &Pos { off: end, ..c.pos }, false)?;
            }
            return Ok(());
        }
    } else if !show {
        return Ok(());
    }
    register(s);
    s.run("leaf.caption", &json!({"show":show}))
        .map_err(|e| e.to_string())?;
    Ok(())
}
