//! Headers and footers of the master pages in `styles.xml` (ODF 1.3 §16.10), read with the body
//! reader (#863).
use std::collections::HashSet;

use wordcraft_doc::{Document, PartKind};

use super::{Body, Ev, FBlock, Pages, SectionProps, Styles, events, get, model};

/// Most XML events kept for all headers and footers together; a slot past it is left out.
const MAX_EVENTS: usize = 1 << 20;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    Header,
    HeaderLeft,
    HeaderFirst,
    Footer,
    FooterLeft,
    FooterFirst,
}

impl Slot {
    fn of(q: &str) -> Option<Self> {
        Some(match q {
            "style:header" => Self::Header,
            "style:header-left" => Self::HeaderLeft,
            "style:header-first" | "loext:header-first" => Self::HeaderFirst,
            "style:footer" => Self::Footer,
            "style:footer-left" => Self::FooterLeft,
            "style:footer-first" | "loext:footer-first" => Self::FooterFirst,
            _ => return None,
        })
    }

    fn footer(self) -> bool {
        matches!(self, Self::Footer | Self::FooterLeft | Self::FooterFirst)
    }

    fn set(self, sec: &mut SectionProps, id: u32) {
        let set = if self.footer() { &mut sec.footers } else { &mut sec.headers };
        match self {
            Self::Header | Self::Footer => set.default = Some(id),
            Self::HeaderLeft | Self::FooterLeft => set.even = Some(id),
            Self::HeaderFirst | Self::FooterFirst => {
                set.first = Some(id);

            }
        }
    }
}

/// The headers and footers read, in part-id order from 1, and whether one is for left pages.
#[derive(Default)]
pub(super) struct Parts {
    parts: Vec<(PartKind, Vec<FBlock>)>,
    even_odd: bool,
    discard: HashSet<u32>,
    /// The watermark a header holds (the first found).
    pub(super) watermark: Option<wordcraft_doc::Watermark>,
}

impl Parts {
    /// The blocks of every header and footer read.
    pub(super) fn blocks_mut(&mut self) -> impl Iterator<Item = &mut Vec<FBlock>> {
        self.parts.iter_mut().map(|(_, b)| b)
    }
}

/// Read each visible header and footer of the master pages in `pages`, and name it on its master
/// page's section as part id 1, 2, … in order, so the sections the body starts carry them.
/// [`add`] puts those parts into the document.
/// `objects`: the package's entry names and manifest media types, for embedded objects.
/// `spaces_left` is the document's [`MAX_EXTRA_SPACES`] budget, which the body read after them
/// goes on with.
pub(super) fn read(
    xml: &[u8],
    styles: &Styles,
    pages: &mut Pages,
    fetch: &mut dyn FnMut(&str) -> Option<Vec<u8>>,
    objects: (&[String], &std::collections::HashMap<String, String>),
    spaces_left: &mut usize,
) -> Parts {
    let mut found: Vec<(String, Slot, bool, Vec<Ev>)> = Vec::new();
    let mut links = Vec::new();
    let mut leaf_namespace = false;
    let mut master: Option<String> = None;
    // The slot being read, its nesting depth and whether it is kept.
    let mut slot: Option<(usize, bool)> = None;
    let mut seen = HashSet::new();
    let mut room = MAX_EVENTS;
    events(xml, |ev| {
        if let Some((depth, keep)) = &mut slot {
            match &ev {
                Ev::End(_) if *depth == 0 => {
                    slot = None;
                    return;
                }
                Ev::Start(..) => *depth += 1,
                Ev::End(_) => *depth -= 1,
                Ev::Text(_) => {}
            }
            if *keep && room == 0 {
                *keep = false;
                found.pop();
            }
            if *keep && let Some((.., evs)) = found.last_mut() {
                room -= 1;
                evs.push(ev);
            }
            return;
        }
        match ev {
            Ev::Start(q, a) if q == "office:document-styles" => {
                leaf_namespace = get(&a, "xmlns:leaf") == Some("https://github.com/frandavid100/leaf/ns/odt/1.0");
            }
            Ev::Start(q, a) if q == "style:master-page" => {
                master = get(&a, "style:name").filter(|n| pages.masters.contains_key(*n)).map(str::to_string);
                if leaf_namespace && let Some(name) = &master {
                    let mask = |key| get(&a, key).and_then(|v| v.parse::<u8>().ok()).filter(|v| *v <= 7);
                    if let (Some(h), Some(f)) = (mask("leaf:header-links"), mask("leaf:footer-links")) {
                        links.push((name.clone(), h, f, get(&a, "leaf:first-page") == Some("true"), get(&a, "leaf:odd-even") == Some("true")));
                    }
                }
            }
            Ev::End(q) if q == "style:master-page" => master = None,
            Ev::Start(q, a) => {
                if let (Some(m), Some(kind)) = (&master, Slot::of(&q)) {
                    let enabled = get(&a, "style:display") != Some("false");
                    let keep = (enabled || leaf_namespace) && seen.insert((m.clone(), q.clone()));
                    if keep {
                        found.push((m.clone(), kind, enabled, Vec::new()));
                    }
                    slot = Some((0, keep));
                }
            }
            _ => {}
        }
    });
    let none = Pages::default();
    let mut out = Parts::default();
    for (name, kind, enabled, evs) in found {
        let mut reader = Body::new(styles, &none, fetch);
        reader.in_text = true;
        (reader.entries, reader.manifest) = (objects.0, Some(objects.1));
        reader.spaces_left = *spaces_left;
        for ev in evs {
            reader.feed(ev);
        }
        *spaces_left = reader.spaces_left;
        out.watermark = out.watermark.take().or(reader.watermark.take());
        out.parts.push((if kind.footer() { PartKind::Footer } else { PartKind::Header }, reader.take_blocks()));
        if let Some(sec) = pages.masters.get_mut(&name) {
            kind.set(sec, out.parts.len() as u32);
            sec.title_page |= enabled && matches!(kind, Slot::HeaderFirst | Slot::FooterFirst);
        }
        out.even_odd |= enabled && matches!(kind, Slot::HeaderLeft | Slot::FooterLeft);
    }
    // Each master page stands alone, but a section without its own header or footer continues
    // the previous one's: give those an empty one, and left pages the right pages' one.
    for footer in [false, true] {
        let pick = |s: &SectionProps| if footer { s.footers } else { s.headers };
        if !pages.masters.values().any(|s| pick(s).default.is_some() || pick(s).even.is_some()) {
            continue;
        }
        let mut empty = None;
        for sec in pages.masters.values_mut() {
            let set = if footer { &mut sec.footers } else { &mut sec.headers };
            if set.default.is_none() {
                set.default = Some(*empty.get_or_insert_with(|| {
                    out.parts.push((if footer { PartKind::Footer } else { PartKind::Header }, Vec::new()));
                    out.parts.len() as u32
                }));
            }
            if out.even_odd && set.even.is_none() {
                set.even = set.default;
            }
        }
    }
    restore_links(pages, &mut out, links);
    if let Some(first) = pages.first_name.as_ref().and_then(|n| pages.masters.get(n)) {
        pages.first = Some(first.clone());
    }
    out
}


/// Restore only links whose exported copy still agrees with its predecessor.
/// An independently edited master page must keep its own content.
fn restore_links(pages: &mut Pages, out: &mut Parts, links: Vec<(String, u8, u8, bool, bool)>) {
    let mut previous_headers = wordcraft_doc::section::HeaderSet::default();
    let mut previous_footers = wordcraft_doc::section::HeaderSet::default();
    for (name, headers, footers, first, odd) in links {
        let Some(sec) = pages.masters.get_mut(&name) else {
            continue;
        };
        let current_headers = sec.headers;
        let current_footers = sec.footers;
        for (set, mask, previous) in [
            (&mut sec.headers, headers, previous_headers),
            (&mut sec.footers, footers, previous_footers),
        ] {
            for (slot, bit, old) in [
                (&mut set.default, 1, previous.default),
                (&mut set.even, 2, previous.even),
                (&mut set.first, 4, previous.first),
            ] {
                if mask & bit == 0 {
                    continue;
                }
                let same = match (*slot, old) {
                    (Some(a), Some(b)) => out
                        .parts
                        .get(a as usize - 1)
                        .zip(out.parts.get(b as usize - 1))
                        .is_some_and(|(a, b)| a == b),
                    (Some(a), None) => out.parts.get(a as usize - 1).is_some_and(|(_, blocks)| {
                        blocks
                            .iter()
                            .all(|b| matches!(b, FBlock::Para(p) if p.is_empty()))
                    }),
                    (None, _) => true,
                };
                if same && let Some(id) = slot.take() {
                    out.discard.insert(id);
                }
            }
        }
        previous_headers = current_headers;
        previous_footers = current_footers;
        sec.title_page = first;
        out.even_odd |= odd;
    }
    let referenced: HashSet<_> = pages
        .masters
        .values()
        .flat_map(|s| {
            [
                s.headers.default,
                s.headers.first,
                s.headers.even,
                s.footers.default,
                s.footers.first,
                s.footers.even,
            ]
            .into_iter()
            .flatten()
        })
        .collect();
    out.discard.retain(|id| !referenced.contains(id));
}

/// Put the parts [`read`] found into `doc`. The sections name them 1, 2…, in the order read;
/// `doc`'s notes, comments and text boxes already hold ids, so each part takes a free one and
/// the sections are pointed at it.
pub(super) fn add(doc: &mut Document, parts: Parts) {
    if parts.watermark.is_some() {
        doc.settings.watermark.clone_from(&parts.watermark);
    }
    let mut ids: Vec<Option<u32>> = Vec::with_capacity(parts.parts.len());
    for (index, (kind, blocks)) in parts.parts.into_iter().enumerate() {
        if parts.discard.contains(&(index as u32 + 1)) {
            ids.push(None);
            continue;
        }
        let blocks = model::part_blocks(doc, &blocks);
        ids.push(doc.try_add_part(kind, blocks).ok());
    }
    let new_id = |old: Option<u32>| old.and_then(|o| o.checked_sub(1)).and_then(|i| ids.get(i as usize).copied().flatten());
    let remap = |sec: &mut SectionProps| {
        for set in [&mut sec.headers, &mut sec.footers] {
            (set.default, set.first, set.even) = (new_id(set.default), new_id(set.first), new_id(set.even));
        }
    };
    remap(&mut doc.last_section);
    for b in &mut doc.body {
        if let wordcraft_doc::Block::Para(p) = std::sync::Arc::make_mut(b)
            && let Some(sec) = p.section.as_deref_mut()
        {
            remap(sec);
        }
    }
    doc.settings.even_odd_headers |= parts.even_odd;
}
