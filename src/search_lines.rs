//! Bridge pasted line endings to canonical positions without an editable text model.
//! The pinned matcher cannot span literal paragraph boundaries. Reuse its public
//! compiled literal pattern against disposable visible runs for this one case.
use regex_automata::meta::Regex;
use wordcraft_doc::{Document, Path, Pos, StoryRef};
struct Span {
    at: usize,
    len: usize,
    from: Pos,
    to: Pos,
    paragraph_mark: bool,
}
#[derive(Default)]
struct Chunk {
    text: String,
    spans: Vec<Span>,
}
impl Chunk {
    fn position(&self, offset: usize, end: bool) -> Option<Pos> {
        let span = self.spans.iter().find(|s| {
            if end {
                s.at < offset && offset <= s.at + s.len
            } else {
                s.at <= offset && offset < s.at + s.len
            }
        })?;
        if span.paragraph_mark {
            return Some(if end {
                span.to.clone()
            } else {
                span.from.clone()
            });
        }
        let mut pos = span.from.clone();
        pos.off += offset - span.at;
        Some(pos)
    }
    fn finish(&mut self, re: &Regex, out: &mut Vec<(Pos, Pos)>) {
        for m in re.find_iter(&self.text) {
            if m.start() < m.end() {
                if let Some(pair) = self
                    .position(m.start(), false)
                    .zip(self.position(m.end(), true))
                {
                    out.push(pair);
                }
            }
        }
        self.text.clear();
        self.spans.clear();
    }
}
pub(crate) fn find(doc: &Document, story: StoryRef, re: &Regex) -> Vec<(Pos, Pos)> {
    let mut out = Vec::new();
    let mut chunk = Chunk::default();
    let mut previous: Option<(Path, usize, bool)> = None;
    for path in doc.para_paths(story) {
        let Some(p) = doc.para(story, &path) else {
            chunk.finish(re, &mut out);
            previous = None;
            continue;
        };
        if let Some((old, length, may_join)) = &previous {
            let starts_live = p
                .run_ranges()
                .next()
                .is_none_or(|(r, c)| r.start == 0 && c.del.is_none());
            if *may_join
                && starts_live
                && old.parent() == path.parent()
                && old.last() + 1 == path.last()
            {
                let at = chunk.text.len();
                chunk.text.push('\n');
                chunk.spans.push(Span {
                    at,
                    len: 1,
                    from: Pos {
                        story,
                        path: old.clone(),
                        off: *length,
                    },
                    to: Pos {
                        story,
                        path: path.clone(),
                        off: 0,
                    },
                    paragraph_mark: true,
                });
            } else {
                chunk.finish(re, &mut out);
            }
        }
        let mut last = 0;
        for (range, props) in p.run_ranges() {
            if props.del.is_some() {
                chunk.finish(re, &mut out);
                continue;
            }
            if range.start != last {
                chunk.finish(re, &mut out);
            }
            let at = chunk.text.len();
            chunk.text.push_str(&p.text[range.clone()]);
            chunk.spans.push(Span {
                at,
                len: range.len(),
                from: Pos {
                    story,
                    path: path.clone(),
                    off: range.start,
                },
                to: Pos {
                    story,
                    path: path.clone(),
                    off: range.end,
                },
                paragraph_mark: false,
            });
            last = range.end;
        }
        previous = Some((
            path,
            p.text.len(),
            last == p.text.len() && p.section.is_none(),
        ));
    }
    chunk.finish(re, &mut out);
    out
}
