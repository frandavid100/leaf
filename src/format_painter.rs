//! Capture effective source appearance while WordCraft applies it and owns Undo.
use serde_json::json;
use wordcraft_engine::Session;

fn source(s: &Session) -> Result<(wordcraft_doc::CharProps, wordcraft_doc::ParaProps), String> {
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
    Ok((chr.formatting().without_symbol(), para))
}
pub fn capture(s: &mut Session, sticky: bool) -> Result<(), String> {
    let (chr, para) = source(s)?;
    // The public command activates the tool; its public transient payload carries
    // the effective source properties, never a second editable document.
    s.painter = None;
    s.run("edit.formatPainter", &json!({"sticky":sticky}))
        .map_err(|e| e.to_string())?;
    s.painter = Some((chr, para, sticky));
    Ok(())
}

/// Ordinary Copy carries appearance separately from clipboard text/structure.
#[derive(Clone)]
pub struct FormatTransfer {
    chr: wordcraft_doc::CharProps,
    para: serde_json::Value,
}
impl FormatTransfer {
    pub fn capture(s: &Session) -> Result<Self, String> {
        let (chr, _) = source(s)?;
        let pos = s.sel.ordered().0;
        let p = s.doc.para_at(&pos).ok_or("No source paragraph")?;
        let resolved =
            serde_json::to_value(s.doc.styles.resolve_para(&p.props)).map_err(|e| e.to_string())?;
        // Never transfer source-document IDs, sections, numbering or object ownership.
        let keys = [
            "align",
            "bidi",
            "indentLeft",
            "indentRight",
            "indentFirst",
            "lineSpacing",
            "spaceBefore",
            "spaceAfter",
            "keepNext",
            "keepLines",
            "pageBreakBefore",
            "widowControl",
            "borders",
            "shading",
            "tabs",
        ];
        let props: serde_json::Map<_, _> = keys
            .into_iter()
            .filter_map(|k| {
                resolved
                    .get(k)
                    .filter(|v| !v.is_null())
                    .map(|v| (k.into(), v.clone()))
            })
            .collect();
        Ok(Self {
            chr,
            para: json!({"props":props}),
        })
    }
    pub fn apply(&self, s: &mut Session) -> Result<(), String> {
        if s.registry.get("leaf.pasteAppearance").is_none() {
            let mut specs = s.registry.all().to_vec();
            specs.push(wordcraft_engine::CommandSpec::new(
                "leaf.pasteAppearance",
                "Paste Formatting",
                "Clipboard",
                replace_character_appearance,
            ));
            s.registry = std::sync::Arc::new(wordcraft_engine::Registry::new(specs));
        }
        crate::operations::atomic(s, crate::t!("Paste Formatting"), |s| {
            s.run("leaf.pasteAppearance", &json!({"props":self.chr}))
                .map_err(|e| e.to_string())?;
            s.run("para.set", &self.para).map_err(|e| e.to_string())?;
            Ok(())
        })
    }
}

fn replace_character_appearance(
    s: &mut Session,
    args: &serde_json::Value,
) -> wordcraft_engine::CmdResult {
    let props: wordcraft_doc::CharProps = serde_json::from_value(args["props"].clone())
        .map_err(|e| wordcraft_engine::CmdError::Params(e.to_string()))?;
    wordcraft_engine::cmd::format::apply(s, &|c| {
        let link = c.link.clone();
        *c = c.cleared().overlaid(&props);
        c.link = link;
    })
}
