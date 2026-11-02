//! Inspect and traverse WordCraft history, and remember a replayable formatting result.
use serde_json::{Value, json};
use wordcraft_engine::Session;

/// Redo has a public top-label API, but no enumeration API. Inspect a disposable
/// session restored from the engine's own edit snapshot; never touch the live one.
pub fn labels(s: &Session, redo: bool) -> Vec<String> {
    if !redo {
        return s.undo_labels();
    }
    let mut copy = Session::new(wordcraft_doc::Document::new());
    copy.restore(s.edit_snapshot());
    let mut result = Vec::new();
    while let Some(label) = copy.redo_label() {
        result.push(label.to_string());
        if copy.run("edit.redo", &json!({})).is_err() {
            break;
        }
    }
    result
}
pub fn travel(s: &mut Session, redo: bool, count: usize) -> Result<usize, String> {
    let mut done = 0;
    while done < count && if redo { s.can_redo() } else { s.can_undo() } {
        if let Err(e) = s.run(if redo { "edit.redo" } else { "edit.undo" }, &json!({})) {
            return if done == 0 {
                Err(e.to_string())
            } else {
                Ok(done)
            };
        }
        done += 1;
    }
    Ok(done)
}

#[derive(Clone, Debug)]
pub struct RepeatAction {
    document: u64,
    command: String,
    params: Value,
}
impl RepeatAction {
    /// Deliberately supports scalar character/paragraph formatting only. No
    /// selection IDs, clipboard content, IO or structural commands are replayed.
    pub fn capture(s: &Session) -> Option<Self> {
        let (id, args) = s.last_command.as_ref()?;
        let toggle = match id.as_str() {
            "format.bold" => Some("bold"),
            "format.italic" => Some("italic"),
            "format.underline" => Some("underline"),
            "format.strike" => Some("strike"),
            "format.superscript" => Some("superscript"),
            "format.subscript" => Some("subscript"),
            _ => None,
        };
        let (command, params) = if let Some(key) = toggle {
            let props = crate::text_properties::selected_resolved(s);
            let first = serde_json::to_value(props.first()?).ok()?;
            let value = first.get(key)?.clone();
            if !props.iter().all(|p| {
                serde_json::to_value(p)
                    .ok()
                    .is_some_and(|v| v[key] == value)
            }) {
                return None;
            }
            ("format.set".into(), json!({"props":{key:value}}))
        } else {
            let allowed = match id.as_str() {
                "format.set" => vec!["props"],
                "format.font" => vec!["name"],
                "format.size" => vec!["size"],
                "format.color" | "format.highlight" => vec!["color"],
                "para.spacing" => vec!["before", "after"],
                "para.lineSpacing" => vec!["rule", "value"],
                "para.alignLeft" | "para.alignCenter" | "para.alignRight" | "para.justify"
                | "para.indent" | "para.outdent" => vec![],
                "para.style" => vec!["style"],
                _ => return None,
            };
            let source = args.as_object()?;
            if source.keys().any(|k| !allowed.contains(&k.as_str())) {
                return None;
            }
            if id == "format.set" {
                let props = args["props"].as_object()?;
                if props.is_empty()
                    || props.keys().any(|k| {
                        !matches!(
                            k.as_str(),
                            "bold"
                                | "italic"
                                | "underline"
                                | "strike"
                                | "size"
                                | "font"
                                | "color"
                                | "highlight"
                                | "superscript"
                                | "subscript"
                                | "spacing"
                                | "scale"
                                | "position"
                                | "caps"
                                | "small_caps"
                        )
                    })
                {
                    return None;
                }
            }
            (id.clone(), args.clone())
        };
        Some(Self {
            document: s.document_id(),
            command,
            params,
        })
    }
    pub fn applies_to(&self, s: &Session) -> bool {
        self.document == s.document_id()
    }
    pub fn apply(&self, s: &mut Session) -> Result<bool, String> {
        if !self.applies_to(s) {
            return Err(
                wordcraft_gnome::t!("The document has changed; open this dialog again.").into(),
            );
        }
        if crate::text_properties::formatting_matches(s, &self.command, &self.params) {
            return Ok(false);
        }
        let rev = s.rev();
        s.run(&self.command, &self.params)
            .map_err(|e| e.to_string())?;
        Ok(s.rev() != rev)
    }
}

pub fn description(label: &str) -> String {
    // Labels remain the engine's actual grouped steps; this is presentation only.
    let localized = match label {
        "Typing" => wordcraft_gnome::t!("Typing"),
        "Set Formatting" | "Character Formatting" | "Format" => {
            wordcraft_gnome::t!("Text Formatting")
        }
        "Paste Text" => wordcraft_gnome::t!("Paste"),
        "Paragraph Spacing" => wordcraft_gnome::t!("Paragraph Spacing"),
        "Line Spacing" => wordcraft_gnome::t!("Line Spacing"),
        "New Paragraph" => wordcraft_gnome::t!("New Paragraph"),
        "Delete Backward" => wordcraft_gnome::t!("Delete"),
        _ => return crate::i18n::translate(label),
    };
    localized.into()
}
