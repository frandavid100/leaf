//! Section-aware header/footer configuration through the engine's public command registry.
use serde_json::{Value, json};
use wordcraft_doc::{
    Paragraph, PartKind, Pos, StoryRef,
    section::{HeaderSet, NumFormat},
};
use wordcraft_engine::{CmdError, CmdResult, CommandSpec, Registry, Session};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Links {
    Keep,
    Previous,
    Independent,
}
#[derive(Clone, Debug)]
pub struct Options {
    pub section: usize,
    pub first: bool,
    pub odd_even: bool,
    pub header: f32,
    pub footer: f32,
    pub format: NumFormat,
    pub start: Option<u32>,
    pub headers: Links,
    pub footers: Links,
}
pub fn current_section(s: &mut Session) -> usize {
    if s.in_header_footer() {
        let hint = s.page_hint;
        if let Some(p) = s.layout().pages.get(hint) {
            return p.section;
        }
    }
    wordcraft_engine::cmd::page::section_index_of_focus(s)
}
impl Options {
    pub fn current(s: &Session, section: usize) -> Self {
        let p = s
            .doc
            .sections()
            .get(section)
            .map(|(_, p)| (**p).clone())
            .unwrap_or_default();
        Self {
            section,
            first: p.title_page,
            odd_even: s.doc.settings.even_odd_headers,
            header: p.header,
            footer: p.footer,
            format: p.page_num_format,
            start: p.page_num_start,
            headers: Links::Keep,
            footers: Links::Keep,
        }
    }
}
fn effective(s: &Session, section: usize, header: bool) -> HeaderSet {
    s.doc
        .sections()
        .iter()
        .take(section + 1)
        .fold(HeaderSet::default(), |old, (_, p)| {
            let own = if header { p.headers } else { p.footers };
            HeaderSet {
                default: own.default.or(old.default),
                first: own.first.or(old.first),
                even: own.even.or(old.even),
            }
        })
}
fn independent(s: &mut Session, section: usize, header: bool, own: HeaderSet) -> HeaderSet {
    let shown = effective(s, section, header);
    let previous = if section > 0 {
        effective(s, section - 1, header)
    } else {
        HeaderSet::default()
    };
    let mut copy = |own: Option<u32>, inherited: Option<u32>, previous: Option<u32>| {
        if let Some(id) = own {
            if Some(id) != previous {
                return Some(id);
            }
        }
        let blocks = inherited
            .and_then(|id| s.doc.parts.get(&id))
            .map(|p| p.blocks.clone())
            .unwrap_or_else(|| {
                vec![std::sync::Arc::new(wordcraft_doc::Block::Para(
                    Paragraph::new().styled(if header { "Header" } else { "Footer" }),
                ))]
            });
        Some(s.doc.add_part(
            if header {
                PartKind::Header
            } else {
                PartKind::Footer
            },
            blocks,
        ))
    };
    HeaderSet {
        default: copy(own.default, shown.default, previous.default),
        first: copy(own.first, shown.first, previous.first),
        even: copy(own.even, shown.even, previous.even),
    }
}
fn command(s: &mut Session, v: &Value) -> CmdResult {
    let section = v["section"]
        .as_u64()
        .ok_or_else(|| CmdError::Params("section".into()))? as usize;
    let sections = s.doc.sections();
    let (_, old) = sections
        .get(section)
        .ok_or_else(|| CmdError::Params("section".into()))?;
    let mut p = (**old).clone();
    let end = sections[section].0;
    let from = section
        .checked_sub(1)
        .map(|i| sections[i].0 + 1)
        .unwrap_or(0);
    let pos = s
        .doc
        .para_paths(StoryRef::Body)
        .into_iter()
        .find(|p| {
            p.0.first()
                .is_some_and(|i| (*i as usize) >= from && (*i as usize) <= end)
        })
        .map(|path| Pos {
            story: StoryRef::Body,
            path,
            off: 0,
        })
        .ok_or_else(|| CmdError::Failed("no paragraph in section".into()))?;
    let back = if s.in_header_footer() {
        s.hf_return
            .clone()
            .unwrap_or_else(|| wordcraft_engine::Selection::caret(pos.clone()))
    } else {
        s.sel.clone()
    };
    p.title_page = v["first"].as_bool().unwrap_or(p.title_page);
    p.header = v["header"].as_f64().unwrap_or(p.header as f64) as f32;
    p.footer = v["footer"].as_f64().unwrap_or(p.footer as f64) as f32;
    p.page_num_format = NumFormat::from_ooxml(v["format"].as_str().unwrap_or("decimal"));
    p.page_num_start = v["start"].as_u64().map(|n| n as u32);
    p.page_num_set |=
        p.page_num_start != old.page_num_start || p.page_num_format != old.page_num_format;
    for header in [true, false] {
        let set = if header { p.headers } else { p.footers };
        let new = match v[if header { "headers" } else { "footers" }].as_str() {
            Some("previous") if section > 0 => HeaderSet::default(),
            Some("independent") => independent(s, section, header, set),
            _ => set,
        };
        if header {
            p.headers = new;
        } else {
            p.footers = new;
        }
    }
    s.run("caret.set", &json!({"pos":pos}))?;
    s.join_next_undo();
    s.run("layout.pageSetup", &json!({"section":p}))?;
    if let Some(on) = v["oddEven"].as_bool() {
        if on != s.doc.settings.even_odd_headers {
            s.join_next_undo();
            s.run("layout.differentOddEven", &json!({"value":on}))?;
        }
    }
    s.run(
        "select.range",
        &json!({"anchor":back.anchor,"focus":back.focus}),
    )?;
    s.hf_return = None;
    Ok(json!({"changed":true}))
}
pub fn apply(s: &mut Session, o: &Options) -> Result<bool, String> {
    if o.section >= s.doc.sections().len() {
        return Err(wordcraft_gnome::t!("Choose a valid section.").into());
    }
    if [o.header, o.footer]
        .iter()
        .any(|n| !n.is_finite() || !(0.0..=1584.0).contains(n))
        || o.start.is_some_and(|n| n > 100000)
    {
        return Err(wordcraft_gnome::t!(
            "Check the header/footer distances and starting page number."
        )
        .into());
    }
    let old = Options::current(s, o.section);
    if old.first == o.first
        && old.odd_even == o.odd_even
        && (old.header - o.header).abs() < 0.02
        && (old.footer - o.footer).abs() < 0.02
        && old.format == o.format
        && old.start == o.start
        && o.headers == Links::Keep
        && o.footers == Links::Keep
    {
        return Ok(false);
    }
    if s.registry.get("leaf.headerOptions").is_none() {
        let mut specs = s.registry.all().to_vec();
        specs.push(
            CommandSpec::new(
                "leaf.headerOptions",
                "Header and Footer Options",
                "Document Properties",
                command,
            )
            .when(|s| {
                if wordcraft_engine::cmd::protect::enforced(&s.doc) {
                    Some(wordcraft_gnome::t!(
                        "Document protection prevents changing header/footer settings"
                    ))
                } else {
                    None
                }
            }),
        );
        s.registry = std::sync::Arc::new(Registry::new(specs));
    }
    let link = |l| match l {
        Links::Keep => "keep",
        Links::Previous => "previous",
        Links::Independent => "independent",
    };
    let before = s.doc.clone();
    let snap = s.edit_snapshot();
    let pending = s.pending.clone();
    let mark = s.undo_mark();
    s.checkpoint(wordcraft_gnome::t!("Header and Footer Options"));
    s.join_next_undo();
    let result=s.run("leaf.headerOptions",&json!({"section":o.section,"first":o.first,"oddEven":o.odd_even,"header":o.header,"footer":o.footer,"format":o.format.ooxml(),"start":o.start,"headers":link(o.headers),"footers":link(o.footers)}));
    match result {
        Ok(_) => {
            if s.doc == before {
                s.restore(snap);
                s.pending = pending;
                return Ok(false);
            }
            s.pending = pending;
            s.merge_undo_since(mark);
            Ok(true)
        }
        Err(e) => {
            s.restore(snap);
            s.pending = pending;
            Err(e.to_string())
        }
    }
}
