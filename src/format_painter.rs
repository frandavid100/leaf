//! Capture effective source appearance while WordCraft applies it and owns Undo.
use serde_json::json;
use wordcraft_engine::Session;

pub fn capture(s: &mut Session, sticky: bool) -> Result<(), String> {
    let pos = s.sel.ordered().0;
    let p = s
        .doc
        .para_at(&pos)
        .ok_or_else(|| "No source paragraph".to_owned())?;
    let raw = p
        .props_of_char(pos.off)
        .clone()
        .without_symbol()
        .formatting();
    let (_, mut chr) = s
        .doc
        .styles
        .para_style_props(p.props.style.as_deref().unwrap_or("Normal"));
    if let Some(id) = raw.style.as_deref() {
        for style in s.doc.styles.chain(id) {
            chr.overlay(&style.chr.formatting());
        }
    }
    chr.overlay(&raw);
    let effective = s.doc.styles.resolve_char(p.props.style.as_deref(), &raw);
    // An unset source attribute must not inherit a different destination style.
    chr.style = None;
    chr.font_ea = Some(
        effective
            .font_ea
            .clone()
            .unwrap_or_else(|| effective.font.clone()),
    );
    chr.font = Some(effective.font);
    chr.font_cs = Some(effective.font_cs);
    chr.size = Some(effective.size);
    chr.size_cs = Some(effective.size_cs);
    chr.bold = Some(effective.bold);
    chr.italic = Some(effective.italic);
    chr.bold_cs = Some(effective.bold_cs);
    chr.italic_cs = Some(effective.italic_cs);
    chr.underline = Some(effective.underline);
    chr.strike = Some(effective.strike);
    chr.double_strike = Some(effective.double_strike);
    chr.color = Some(effective.color);
    chr.hidden = Some(effective.hidden);
    chr.outline = Some(effective.outline);
    chr.shadow = Some(effective.shadow);
    chr.emboss = Some(effective.emboss);
    chr.engrave = Some(effective.engrave);
    chr.vert_align = Some(effective.vert_align);
    chr.kern = Some(effective.kern);
    chr.emphasis = Some(effective.emphasis);
    if chr.highlight.is_none() {
        chr.highlight = Some(wordcraft_doc::props::Highlight::None);
    }
    chr.caps = Some(effective.caps);
    chr.small_caps = Some(effective.small_caps);
    chr.spacing = Some(effective.spacing);
    chr.scale = Some(effective.scale);
    chr.position = Some(effective.position);
    let para = p.props.clone();
    // The public command activates the tool; its public transient payload carries
    // the effective source properties, never a second editable document.
    s.painter = None;
    s.run("edit.formatPainter", &json!({"sticky":sticky}))
        .map_err(|e| e.to_string())?;
    s.painter = Some((chr.formatting().without_symbol(), para, sticky));
    Ok(())
}
