//! Headers and footers of the master pages in `styles.xml` (ODF 1.3 §16.10), read with the body
//! reader (#863).
use std::collections::HashSet;

use wordcraft_doc::{Document, Part, PartKind};

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
}

/// Read each visible header and footer of the master pages in `pages`, and name it on its master
/// page's section as part id 1, 2, … in order, so the sections the body starts carry them.
/// [`add`] puts those parts into the document.
pub(super) fn read(xml: &[u8], styles: &Styles, pages: &mut Pages, fetch: &mut dyn FnMut(&str) -> Option<Vec<u8>>) -> Parts {
    let mut found: Vec<(String, Slot, bool, Vec<Ev>)> = Vec::new();
    let mut master: Option<String> = None;
    let mut links=Vec::new();
    let mut leaf_namespace=false;
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
            Ev::Start(q,a) if q=="office:document-styles"=>{leaf_namespace=get(&a,"xmlns:leaf")==Some("https://github.com/frandavid100/leaf/ns/odt/1.0");},
            Ev::Start(q, a) if q == "style:master-page" => {
                master = get(&a, "style:name").filter(|n| pages.masters.contains_key(*n)).map(str::to_string);
                if leaf_namespace{if let Some(name)=&master{if let (Some(h),Some(f))=(get(&a,"leaf:header-links").and_then(|s|s.parse::<u8>().ok()).filter(|n|*n<=7),get(&a,"leaf:footer-links").and_then(|s|s.parse::<u8>().ok()).filter(|n|*n<=7)){links.push((name.clone(),h,f,get(&a,"leaf:first-page")==Some("true"),get(&a,"leaf:odd-even")==Some("true")));}}}
            }
            Ev::End(q) if q == "style:master-page" => master = None,
            Ev::Start(q, a) => {
                if let (Some(m), Some(kind)) = (&master, Slot::of(&q)) {
                    let enabled=get(&a,"style:display")!=Some("false");
                    let keep = (enabled||matches!(kind,Slot::HeaderFirst|Slot::HeaderLeft|Slot::FooterFirst|Slot::FooterLeft)) && seen.insert((m.clone(), q.clone()));
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
        for ev in evs {
            reader.feed(ev);
        }
        out.parts.push((if kind.footer() { PartKind::Footer } else { PartKind::Header }, reader.take_blocks()));
        if let Some(sec) = pages.masters.get_mut(&name) {
            kind.set(sec, out.parts.len() as u32);
            sec.title_page|=enabled&&matches!(kind,Slot::HeaderFirst|Slot::FooterFirst);
        }
        out.even_odd |= enabled&&matches!(kind, Slot::HeaderLeft | Slot::FooterLeft);
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
    let mut previous_headers=wordcraft_doc::section::HeaderSet::default();
    let mut previous_footers=wordcraft_doc::section::HeaderSet::default();
    for(name,h,f,first,odd)in links{
        if let Some(sec)=pages.masters.get_mut(&name){
            let current_headers=sec.headers;let current_footers=sec.footers;
            for(set,mask,previous)in[(&mut sec.headers,h,previous_headers),(&mut sec.footers,f,previous_footers)]{
                for(slot,bit,old)in[(&mut set.default,1,previous.default),(&mut set.even,2,previous.even),(&mut set.first,4,previous.first)]{
                    if mask&bit==0{continue;}
                    let same=match (*slot,old){
                        (Some(a),Some(b))=>out.parts.get(a as usize-1).zip(out.parts.get(b as usize-1)).is_some_and(|(a,b)|a==b),
                        (Some(a),None)=>out.parts.get(a as usize-1).is_some_and(|(_,bs)|bs.iter().all(|b|matches!(b,FBlock::Para(p)if p.is_blank()))),
                        (None,_)=>true,
                    };
                    if same{if let Some(id)=slot.take(){out.discard.insert(id);}}
                }
            }
            previous_headers=current_headers;previous_footers=current_footers;
            sec.title_page=first;out.even_odd|=odd;
        }
    }
    let referenced:HashSet<u32>=pages.masters.values().flat_map(|s|[s.headers.default,s.headers.first,s.headers.even,s.footers.default,s.footers.first,s.footers.even].into_iter().flatten()).collect();
    out.discard.retain(|id|!referenced.contains(id));
    if let Some(first) = pages.first_name.as_ref().and_then(|n| pages.masters.get(n)) {
        pages.first = Some(first.clone());
    }
    out
}

/// Put the parts [`read`] found into `doc` under the ids its sections name. `doc` comes from
/// [`model::to_doc`], which adds no parts of its own.
pub(super) fn add(doc: &mut Document, parts: Parts) {
    for (i, (kind, blocks)) in parts.parts.into_iter().enumerate() {
        if parts.discard.contains(&(i as u32+1)){continue;}
        let blocks = model::part_blocks(doc, &blocks);
        doc.parts.insert(i as u32 + 1, Part { kind, blocks, body: Default::default() });
    }
    doc.settings.even_odd_headers |= parts.even_odd;
}
