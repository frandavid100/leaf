//! Native rich-paste policy, before the public WordCraft HTML paste command.
//!
//! Chrome serializes computed web theme colors; Firefox usually leaves those
//! inherited. There is no authored/computed provenance in that HTML. Only the
//! predominant, contrasting neutral pair in Chrome-style wrappers is adapted;
//! local color exceptions remain. This is an explicit paste policy, not a
//! reconstruction of the source author's intention or another document model.
use quick_xml::{Reader, events::Event};
use std::{borrow::Cow, collections::BTreeMap, ops::Range};
use wordcraft_doc::props::Rgb;
use wordcraft_formats::model::parse_color;

struct Style {
    range: Range<usize>,
    declarations: Vec<(String, String)>,
    color: Option<Rgb>,
    background: Option<Rgb>,
    chrome: bool,
}
struct Element {
    parent: Option<usize>,
    style: Option<Style>,
    tag: Vec<u8>,
}

// A declaration separator inside a function, quoted family or URL is not CSS
// punctuation. Preserve unknown properties rather than interpreting their data.
fn declarations(css: &str) -> Vec<(String, String)> {
    let (mut quote, mut depth, mut escaped, mut start) = (None, 0usize, false, 0);
    let mut out = Vec::new();
    for (i, c) in css.char_indices().chain(std::iter::once((css.len(), ';'))) {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if quote == Some(c) {
            quote = None;
        } else if quote.is_none() {
            match c {
                '\'' | '"' => quote = Some(c),
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ';' if depth == 0 => {
                    if let Some((k, v)) = css[start..i].split_once(':') {
                        out.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }
    }
    out
}
fn value(v: &str) -> &str {
    let v = v.trim();
    if v.to_ascii_lowercase().ends_with("!important") {
        v[..v.len() - 10].trim_end()
    } else {
        v
    }
}
fn opaque_color(v: &str) -> Option<Rgb> {
    let v = value(v).to_ascii_lowercase();
    // Do not turn an alpha color into an opaque color. In particular transparent
    // backgrounds must not become black. Unknown CSS remains verbatim.
    if v.starts_with("rgba") || v.starts_with("hsla") || v.contains('/') {
        return None;
    }
    parse_color(&v)
}
fn neutral(Rgb(r, g, b): Rgb) -> bool {
    r.max(g).max(b) - r.min(g).min(b) <= 32
}
fn contrasting(a: Rgb, b: Rgb) -> bool {
    let light = |Rgb(r, g, b): Rgb| (u32::from(r) + u32::from(g) + u32::from(b)) / 3;
    let (a, b) = (light(a), light(b));
    (a >= 200 && b <= 80) || (b >= 200 && a <= 80)
}
fn elements(html: &str) -> Option<Vec<Element>> {
    let mut reader = Reader::from_str(html);
    reader.config_mut().check_end_names = false;
    let (mut elements, mut stack) = (Vec::<Element>::new(), Vec::<usize>::new());
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) | Event::Empty(e) => {
                if elements.len() >= 100_000 {
                    return None;
                }
                let tag = e.name().as_ref().to_ascii_lowercase();
                let mut style = None;
                for attr in e.attributes() {
                    let attr = attr.ok()?;
                    if !attr.key.as_ref().eq_ignore_ascii_case(b"style") {
                        continue;
                    }
                    let offset =
                        (attr.value.as_ptr() as usize).checked_sub(html.as_ptr() as usize)?;
                    let end = offset.checked_add(attr.value.len())?;
                    if html.as_bytes().get(offset..end)? != attr.value.as_ref() {
                        return None;
                    }
                    let declarations = declarations(&attr.unescape_value().ok()?);
                    let get = |key: &str| {
                        declarations
                            .iter()
                            .rev()
                            .find(|(k, _)| k == key)
                            .and_then(|(_, v)| opaque_color(v))
                    };
                    let color = get("color");
                    let background = declarations
                        .iter()
                        .rev()
                        .find(|(k, _)| k == "background" || k == "background-color")
                        .and_then(|(_, v)| opaque_color(v));
                    let chrome = declarations
                        .iter()
                        .any(|(k, _)| k == "-webkit-text-stroke-width")
                        && declarations.iter().any(|(k, _)| k == "white-space");
                    style = Some(Style {
                        range: offset..end,
                        declarations,
                        color,
                        background,
                        chrome,
                    });
                }
                let index = elements.len();
                let void = matches!(
                    tag.as_slice(),
                    b"area"
                        | b"base"
                        | b"br"
                        | b"col"
                        | b"embed"
                        | b"hr"
                        | b"img"
                        | b"input"
                        | b"link"
                        | b"meta"
                        | b"param"
                        | b"source"
                        | b"track"
                        | b"wbr"
                );
                // Empty tags end in '/', which BytesStart retains before the
                // closing '>'. The event's borrowed bytes are otherwise intact.
                let empty =
                    html.as_bytes().get(reader.buffer_position() as usize - 2) == Some(&b'/');
                elements.push(Element {
                    parent: stack.last().copied(),
                    style,
                    tag,
                });
                if !void && !empty {
                    stack.push(index);
                }
            }
            Event::End(e) => {
                if let Some(i) = stack
                    .iter()
                    .rposition(|i| elements[*i].tag.eq_ignore_ascii_case(e.name().as_ref()))
                {
                    stack.truncate(i);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Some(elements)
}

/// Normalize a bounded external HTML fragment without touching plain text,
/// internal engine fragments, RTF, the document or any theme rendering options.
pub fn for_paste(html: &str) -> Cow<'_, str> {
    if html.len() > 8 * 1024 * 1024 {
        return Cow::Borrowed(html);
    }
    let Some(elements) = elements(html) else {
        return Cow::Borrowed(html);
    };
    let mut votes = BTreeMap::<([u8; 3], [u8; 3]), usize>::new();
    for style in elements.iter().filter_map(|e| e.style.as_ref()) {
        if let (true, Some(a), Some(b)) = (style.chrome, style.color, style.background)
            && neutral(a)
            && neutral(b)
            && contrasting(a, b)
        {
            *votes.entry(([a.0, a.1, a.2], [b.0, b.1, b.2])).or_default() += 1;
        }
    }
    let total: usize = votes.values().sum();
    let base = votes
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .filter(|(_, n)| n * 2 > total)
        .map(|((a, b), _)| (Rgb(a[0], a[1], a[2]), Rgb(b[0], b[1], b[2])));
    let (mut protected, mut changes) = (Vec::new(), Vec::new());
    for e in &elements {
        let inherited = e.parent.is_some_and(|p| protected[p]);
        let preserve = inherited
            || e.style.as_ref().is_some_and(|s| {
                base.is_some_and(|(fg, bg)| {
                    s.color.is_some_and(|c| c != fg)
                        || s.background.is_some_and(|c| c != bg)
                        || (s.background.is_none()
                            && s.declarations
                                .iter()
                                .any(|(k, _)| k == "background" || k == "background-color"))
                })
            });
        protected.push(preserve);
        let Some(s) = &e.style else { continue };
        let mut changed = false;
        let mut css = String::new();
        for (k, v) in &s.declarations {
            let remove = !preserve
                && base.is_some_and(|(fg, bg)| {
                    (k == "color" && opaque_color(v) == Some(fg))
                        || ((k == "background" || k == "background-color")
                            && opaque_color(v) == Some(bg))
                });
            if remove {
                changed = true;
                continue;
            }
            // CSS spelling only: the engine currently splits spaced rgb()
            // backgrounds into tokens. Keep intentional RGB fills through its
            // public importer, with no private formats/renderer patch (#217).
            let canonical = if (k == "background" || k == "background-color")
                && value(v).to_ascii_lowercase().starts_with("rgb(")
            {
                opaque_color(v).map(|c| {
                    format!(
                        "#{}{}",
                        c.hex(),
                        if value(v) != v { " !important" } else { "" }
                    )
                })
            } else {
                None
            };
            if canonical.is_some() {
                changed = true;
            }
            css.push_str(k);
            css.push(':');
            css.push_str(canonical.as_deref().unwrap_or(v));
            css.push(';');
        }
        if changed {
            changes.push((
                s.range.clone(),
                quick_xml::escape::escape(&css).into_owned(),
            ));
        }
    }
    if changes.is_empty() {
        return Cow::Borrowed(html);
    }
    let mut out = html.to_string();
    for (range, css) in changes.into_iter().rev() {
        out.replace_range(range, &css);
    }
    Cow::Owned(out)
}
