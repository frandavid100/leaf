//! Native Find/Replace options; WordCraft remains the matcher and editing engine.
use crate::selection::{self, Rect};
use regex_automata::{
    Input, PatternID,
    meta::{self, Regex},
    util::syntax,
};
use serde_json::json;
use wordcraft_doc::{Document, PartKind, Pos, StoryRef};
use wordcraft_engine::{Selection, Session, cmd::find_pattern};

#[derive(Clone, Debug)]
pub struct Hit {
    pub start: Pos,
    pub end: Pos,
    pub before: String,
    pub matched: String,
    pub after: String,
    pub page: Option<usize>,
    pub rects: Vec<Rect>,
    pub revision: u64,
    pub document: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    Document,
    Body,
    Notes,
    HeadersFooters,
    Selection,
}
#[derive(Clone, Debug, Default)]
pub struct Options {
    pub match_case: bool,
    pub whole_word: bool,
    pub special: bool,
    pub regex: bool,
    pub preserve_case: bool,
    pub scope: Scope,
}
/// A disposable range index, never another document. Foreign edits invalidate it.
#[derive(Clone, Debug)]
pub struct Range {
    pub document: u64,
    pub revision: u64,
    pub story: StoryRef,
    pub from: usize,
    pub to: usize,
}
struct StoryText {
    paragraphs: Vec<(wordcraft_doc::Path, usize, usize)>,
    text: String,
}
impl StoryText {
    fn new(doc: &Document, story: StoryRef) -> Self {
        let mut paragraphs = Vec::new();
        let mut text = String::new();
        let mut separator = '\r';
        for path in doc.para_paths(story) {
            let Some(p) = doc.para(story, &path) else {
                continue;
            };
            if !paragraphs.is_empty() {
                text.push(separator);
            }
            let start = text.len();
            text.push_str(&p.text);
            paragraphs.push((path, start, text.len()));
            separator = if p.section.is_some() { '\u{1d}' } else { '\r' };
        }
        Self { paragraphs, text }
    }
    fn offset(&self, pos: &Pos) -> Option<usize> {
        let (_, start, end) = self.paragraphs.iter().find(|(p, _, _)| p == &pos.path)?;
        let offset = start.checked_add(pos.off)?;
        (offset <= *end && self.text.is_char_boundary(offset)).then_some(offset)
    }
    fn pos(&self, story: StoryRef, offset: usize) -> Option<Pos> {
        self.paragraphs
            .iter()
            .find(|(_, start, end)| *start <= offset && offset <= *end)
            .map(|(path, start, _)| Pos {
                story,
                path: path.clone(),
                off: offset - start,
            })
    }
}
impl Range {
    pub fn capture(s: &Session) -> Option<Self> {
        let (a, b) = s.sel.ordered();
        if a == b || a.story != b.story {
            return None;
        }
        let text = StoryText::new(&s.doc, a.story);
        Some(Self {
            document: s.document_id(),
            revision: s.rev(),
            story: a.story,
            from: text.offset(&a)?,
            to: text.offset(&b)?,
        })
    }
    fn bounds(&self, s: &Session) -> Result<(Pos, Pos), String> {
        if s.document_id() != self.document || s.rev() != self.revision {
            return Err(wordcraft_gnome::t!(
                "The selection changed. Select the search range again."
            )
            .into());
        }
        let text = StoryText::new(&s.doc, self.story);
        text.pos(self.story, self.from)
            .zip(text.pos(self.story, self.to))
            .ok_or_else(|| wordcraft_gnome::t!("Select the text to search first.").into())
    }
}
fn pattern_options(o: &Options) -> find_pattern::Opts {
    find_pattern::Opts {
        match_case: o.match_case,
        whole_word: o.whole_word,
        regex: o.regex,
        literal: !o.special && !o.regex,
        ..Default::default()
    }
}
fn compiled(text: &str, o: &Options) -> Result<Regex, String> {
    let c = find_pattern::compile(text, &pattern_options(o))?;
    Regex::builder()
        .syntax(syntax::Config::new().case_insensitive(c.case_insensitive))
        .configure(meta::Config::new().nfa_size_limit(Some(1 << 20)))
        .build(&c.pattern)
        .map_err(|e| e.to_string())
}
fn args(text: &str, o: &Options) -> serde_json::Value {
    json!({"text":text,"matchCase":o.match_case,"wholeWord":o.whole_word,"regex":o.regex,
        "special":o.special||o.regex,"wildcards":false,"matchPrefix":false,"matchSuffix":false,
        "ignorePunct":false,"ignoreSpace":false,"highlight":false})
}
pub fn query(s: &mut Session, text: &str, match_case: bool) -> Result<Vec<Hit>, String> {
    query_with(
        s,
        text,
        &Options {
            match_case,
            scope: Scope::Body,
            ..Default::default()
        },
        None,
    )
}
pub fn query_with(
    s: &mut Session,
    text: &str,
    o: &Options,
    range: Option<&Range>,
) -> Result<Vec<Hit>, String> {
    if text.is_empty() {
        s.find.results.clear();
        return Ok(Vec::new());
    }
    compiled(text, o)?;
    let bounds = if o.scope == Scope::Selection {
        Some(
            range
                .ok_or_else(|| wordcraft_gnome::t!("Select the text to search first.").to_owned())?
                .bounds(s)?,
        )
    } else {
        None
    };
    let stories = if let Some((a, _)) = &bounds {
        vec![a.story]
    } else {
        s.doc.shown_stories().into_iter().filter(|story|match o.scope {
            Scope::Document=>true, Scope::Body=>*story==StoryRef::Body,
            Scope::Notes=>matches!(story,StoryRef::Part(id) if s.doc.parts.get(id).is_some_and(|p|matches!(p.kind,PartKind::Footnote|PartKind::Endnote))),
            Scope::HeadersFooters=>matches!(story,StoryRef::Part(id) if s.doc.parts.get(id).is_some_and(|p|matches!(p.kind,PartKind::Header|PartKind::Footer))),
            Scope::Selection=>false,
        }).collect()
    };
    let saved = (
        s.sel.clone(),
        s.pending.clone(),
        s.goal_x,
        s.page_hint,
        s.hf_return.clone(),
    );
    let result = (|| {
        let mut positions = Vec::new();
        for story in stories {
            let Some(path) = s.doc.para_paths(story).into_iter().next() else {
                continue;
            };
            s.sel = Selection::caret(Pos {
                story,
                path,
                off: 0,
            });
            s.run("edit.find", &args(text, o))
                .map_err(|e| e.to_string())?;
            positions.extend(
                s.find
                    .results
                    .iter()
                    .filter(|(a, b)| {
                        bounds
                            .as_ref()
                            .is_none_or(|(from, to)| a >= from && b <= to)
                    })
                    .cloned(),
            );
        }
        Ok::<_, String>(positions)
    })();
    (s.sel, s.pending, s.goal_x, s.page_hint, s.hf_return) = saved;
    let positions = result?;
    s.find.results = positions.clone();
    let layout = s.layout();
    let mut out = Vec::new();
    let mut sources = std::collections::BTreeMap::new();
    for (start, end) in positions {
        let source = sources
            .entry(start.story)
            .or_insert_with(|| StoryText::new(&s.doc, start.story));
        let (Some(a), Some(b)) = (source.offset(&start), source.offset(&end)) else {
            continue;
        };
        let before = source.text[..a]
            .chars()
            .rev()
            .take(28)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let after = source.text[b..].chars().take(55).collect();
        let matched = source.text[a..b].to_owned();
        let rects: Vec<Rect> = layout
            .selection_rects(&s.doc, &start, &end, 0)
            .into_iter()
            .map(|(page, r)| Rect {
                page,
                x: r.x as f64,
                y: r.y as f64,
                width: r.w as f64,
                height: r.h as f64,
            })
            .collect();
        out.push(Hit {
            start,
            end,
            before,
            matched,
            after,
            page: rects.first().map(|r| r.page),
            rects,
            revision: s.rev(),
            document: s.document_id(),
        });
    }
    Ok(out)
}
pub fn select(s: &mut Session, hit: &Hit) -> Result<(), String> {
    if s.document_id() != hit.document || s.rev() != hit.revision {
        return Err(wordcraft_gnome::t!(
            "The document has changed; search results have been updated"
        )
        .into());
    }
    if let Some(page) = hit.page {
        s.page_hint = page;
    }
    selection::select_range(s, &hit.start, &hit.end)
}
fn encoded(text: &str) -> String {
    text.replace('^', "^^")
        .replace('\t', "^t")
        .replace('\n', "^l")
        .replace('\r', "^p")
        .replace('\u{1d}', "^b")
}
fn capitalization(source: &str, with: String) -> String {
    let letters: Vec<char> = source.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return with;
    }
    if letters.iter().all(|c| c.is_uppercase()) {
        return with.to_uppercase();
    }
    if letters.iter().all(|c| c.is_lowercase()) {
        return with.to_lowercase();
    }
    if letters[0].is_uppercase() && letters.iter().skip(1).all(|c| c.is_lowercase()) {
        let mut first = true;
        return with
            .chars()
            .flat_map(|c| {
                if first && c.is_alphabetic() {
                    first = false;
                    c.to_uppercase().collect::<Vec<_>>()
                } else {
                    c.to_lowercase().collect()
                }
            })
            .collect();
    }
    with
}
/// Disposable paragraph context for capture expansion. The engine omits tracked
/// deletions and prevents matches crossing their gaps; keep those same boundaries.
struct RegexText {
    text: String,
    runs: Vec<(std::ops::Range<usize>, usize)>,
    cuts: Vec<usize>,
}
impl RegexText {
    fn new(p: &wordcraft_doc::Paragraph) -> Self {
        let mut text = String::new();
        let mut runs = Vec::new();
        let mut cuts = vec![0];
        let mut expected = 0;
        for (range, chr) in p.run_ranges() {
            if chr.del.is_some() {
                continue;
            }
            if range.start != expected {
                cuts.push(text.len());
            }
            let start = text.len();
            text.push_str(&p.text[range.clone()]);
            expected = range.end;
            runs.push((range, start));
        }
        cuts.push(text.len());
        Self { text, runs, cuts }
    }
    fn offset(&self, raw: usize) -> Option<usize> {
        self.runs
            .iter()
            .find(|(range, _)| range.start <= raw && raw <= range.end)
            .map(|(range, start)| start + raw - range.start)
    }
    fn captures(
        &self,
        re: &Regex,
        hit: &Hit,
    ) -> Result<regex_automata::util::captures::Captures, String> {
        let invalid = || wordcraft_gnome::t!("The search result no longer matches.").to_owned();
        let (a, b) = self
            .offset(hit.start.off)
            .zip(self.offset(hit.end.off))
            .ok_or_else(invalid)?;
        let end = self
            .cuts
            .iter()
            .copied()
            .find(|cut| *cut >= b && *cut > a)
            .ok_or_else(invalid)?;
        let mut captures = re.create_captures();
        re.search_captures(&Input::new(&self.text).range(a..end), &mut captures);
        if !captures
            .get_match()
            .is_some_and(|m| m.start() == a && m.end() == b)
        {
            return Err(invalid());
        }
        Ok(captures)
    }
}
fn replacement(
    re: &Regex,
    hit: &Hit,
    with: &str,
    o: &Options,
    clip: &str,
    context: Option<&RegexText>,
) -> Result<String, String> {
    let pieces = find_pattern::resolve_names(
        find_pattern::parse_replace(with, &pattern_options(o)),
        &|name| re.group_info().to_index(PatternID::ZERO, name),
    );
    let captures = context
        .map(|context| context.captures(re, hit))
        .transpose()?;
    let out = find_pattern::expand(
        &pieces,
        &hit.matched,
        &|i| {
            captures
                .as_ref()
                .and_then(|c| c.get_group(i))
                .and_then(|span| context?.text.get(span.range()))
                .map(str::to_owned)
        },
        clip,
    );
    Ok(if o.preserve_case {
        capitalization(&hit.matched, out)
    } else {
        out
    })
}
fn apply_hit(s: &mut Session, hit: &Hit, with: &str) -> Result<(), String> {
    // Run the engine's exact selected-match replacement; an explicit literal
    // encoding preserves paragraph versus soft breaks and never reinterprets $1.
    selection::select_range(s, &hit.start, &hit.end)?;
    s.run("edit.replace",&json!({"text":encoded(&hit.matched),"with":encoded(with),"matchCase":true,
        "wholeWord":false,"special":true,"regex":false,"wildcards":false,"ignorePunct":false,"ignoreSpace":false,"matchPrefix":false,"matchSuffix":false})).map_err(|e|e.to_string())?;
    Ok(())
}
pub fn replace_with(
    s: &mut Session,
    hits: &[Hit],
    text: &str,
    with: &str,
    o: &Options,
    mut range: Option<&mut Range>,
) -> Result<usize, String> {
    if hits.is_empty() {
        return Ok(0);
    }
    let re = compiled(text, o)?;
    let mut prepared = Vec::new();
    let mut contexts = std::collections::BTreeMap::new();
    for hit in hits {
        if hit.document != s.document_id() || hit.revision != s.rev() {
            return Err(wordcraft_gnome::t!(
                "The document has changed; search results have been updated"
            )
            .into());
        }
        if hit.start == hit.end {
            return Err(wordcraft_gnome::t!(
                "Zero-length matches cannot be replaced. Use a pattern that selects text."
            )
            .into());
        }
        if let Some(r) = range.as_deref() {
            let (a, b) = r.bounds(s)?;
            if hit.start < a || hit.end > b {
                return Err(wordcraft_gnome::t!(
                    "The result is outside the selected search range."
                )
                .into());
            }
        }
        let context = if o.regex {
            let paragraph = s.doc.para_at(&hit.start).ok_or_else(|| {
                wordcraft_gnome::t!("The search result no longer matches.").to_owned()
            })?;
            Some(
                &*contexts
                    .entry((hit.start.story, hit.start.path.clone()))
                    .or_insert_with(|| RegexText::new(paragraph)),
            )
        } else {
            None
        };
        prepared.push((
            hit.clone(),
            replacement(&re, hit, with, o, &s.clipboard_text, context)?,
        ));
    }
    let old_length = range
        .as_deref()
        .map(|r| StoryText::new(&s.doc, r.story).text.len());
    prepared.sort_by(|(a, _), (b, _)| a.start.cmp(&b.start));
    crate::operations::atomic(s, wordcraft_gnome::t!("Replace"), |s| {
        for (hit, with) in prepared.iter().rev() {
            apply_hit(s, hit, with)?;
        }
        Ok(())
    })?;
    if let (Some(r), Some(old)) = (range.as_mut(), old_length) {
        let new = StoryText::new(&s.doc, r.story).text.len();
        r.to = r.to.saturating_add_signed(new as isize - old as isize);
        r.revision = s.rev();
    }
    Ok(prepared.len())
}
pub fn replace(s: &mut Session, hit: &Hit, with: &str) -> Result<(), String> {
    replace_with(
        s,
        std::slice::from_ref(hit),
        &hit.matched,
        with,
        &Options::default(),
        None,
    )
    .map(|_| ())
}
pub fn replace_all(
    s: &mut Session,
    text: &str,
    with: &str,
    match_case: bool,
) -> Result<usize, String> {
    let o = Options {
        match_case,
        scope: Scope::Body,
        ..Default::default()
    };
    let hits = query_with(s, text, &o, None)?;
    replace_with(s, &hits, text, with, &o, None)
}
