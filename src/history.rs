//! Inspect and traverse WordCraft history, and remember a replayable formatting result.
use serde_json::{Value, json};
use wordcraft_engine::Session;

/// Redo has a public top-label API, but no enumeration API. Inspect a disposable
/// session restored from the engine's own edit snapshot; never touch the live one.
pub fn labels(s: &Session, redo: bool) -> Vec<String> {
    if !redo {
        return s.undo_labels();
    }
    if !s.can_redo() {
        return Vec::new();
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
                "para.indents" => vec!["left", "right", "firstLine"],
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
        "Alignment" => wordcraft_gnome::t!("Alignment"),
        // Engine descriptions used by Leaf commands remain literal gettext keys.
        "All Caps" => wordcraft_gnome::t!("All Caps"),
        "Apply Style" => wordcraft_gnome::t!("Apply Style"),
        "AutoFit" => wordcraft_gnome::t!("AutoFit"),
        "AutoFit Column by Double-Clicking" => {
            wordcraft_gnome::t!("AutoFit Column by Double-Clicking")
        }
        "Backspace" => wordcraft_gnome::t!("Backspace"),
        "Bookmark" => wordcraft_gnome::t!("Bookmark"),
        "Change List Level" => wordcraft_gnome::t!("Change List Level"),
        "Character Border" => wordcraft_gnome::t!("Character Border"),
        "Clear All Formatting" => wordcraft_gnome::t!("Clear All Formatting"),
        "Close Header and Footer" => wordcraft_gnome::t!("Close Header and Footer"),
        "Create a Style" => wordcraft_gnome::t!("Create a Style"),
        "Decrease Font Size" => wordcraft_gnome::t!("Decrease Font Size"),
        "Delete Columns" => wordcraft_gnome::t!("Delete Columns"),
        "Delete Rows" => wordcraft_gnome::t!("Delete Rows"),
        "Different Odd & Even Pages" => wordcraft_gnome::t!("Different Odd & Even Pages"),
        "Distribute Columns" => wordcraft_gnome::t!("Distribute Columns"),
        "Distribute Rows" => wordcraft_gnome::t!("Distribute Rows"),
        "Double Strikethrough" => wordcraft_gnome::t!("Double Strikethrough"),
        "Edit Footer" => wordcraft_gnome::t!("Edit Footer"),
        "Edit Header" => wordcraft_gnome::t!("Edit Header"),
        "Emboss" => wordcraft_gnome::t!("Emboss"),
        "Enclose Characters" => wordcraft_gnome::t!("Enclose Characters"),
        "Engrave" => wordcraft_gnome::t!("Engrave"),
        "Field" => wordcraft_gnome::t!("Field"),
        "Font Color" => wordcraft_gnome::t!("Font Color"),
        "Formatting at Selection" => wordcraft_gnome::t!("Formatting at Selection"),
        "Hidden" => wordcraft_gnome::t!("Hidden"),
        "Increase Font Size" => wordcraft_gnome::t!("Increase Font Size"),
        "Indents" => wordcraft_gnome::t!("Indents"),
        "Insert Above" => wordcraft_gnome::t!("Insert Above"),
        "Insert Below" => wordcraft_gnome::t!("Insert Below"),
        "Insert Left" => wordcraft_gnome::t!("Insert Left"),
        "Insert Right" => wordcraft_gnome::t!("Insert Right"),
        "Insert Row or Column by Clicking" => {
            wordcraft_gnome::t!("Insert Row or Column by Clicking")
        }
        "Left-to-Right Text Direction" => wordcraft_gnome::t!("Left-to-Right Text Direction"),
        "Line Break" => wordcraft_gnome::t!("Line Break"),
        "Line and Paragraph Spacing" => wordcraft_gnome::t!("Line and Paragraph Spacing"),
        "Modify Style" => wordcraft_gnome::t!("Modify Style"),
        "Move Table by Dragging" => wordcraft_gnome::t!("Move Table by Dragging"),
        "New Comment" => wordcraft_gnome::t!("New Comment"),
        "Normal Style" => wordcraft_gnome::t!("Normal Style"),
        "Page Setup" => wordcraft_gnome::t!("Page Setup"),
        "Pictures" => wordcraft_gnome::t!("Pictures"),
        "Properties" => wordcraft_gnome::t!("Properties"),
        "Repeat Header Rows" => wordcraft_gnome::t!("Repeat Header Rows"),
        "Resize Table by Dragging" => wordcraft_gnome::t!("Resize Table by Dragging"),
        "Resolve" => wordcraft_gnome::t!("Resolve"),
        "Restart at 1" => wordcraft_gnome::t!("Restart at 1"),
        "Reviewing Pane" => wordcraft_gnome::t!("Reviewing Pane"),
        "Right-to-Left Text Direction" => wordcraft_gnome::t!("Right-to-Left Text Direction"),
        "Select Table Cells by Clicking" => wordcraft_gnome::t!("Select Table Cells by Clicking"),
        "Set Character Formatting" => wordcraft_gnome::t!("Set Character Formatting"),
        "Shift+Tab" => wordcraft_gnome::t!("Shift+Tab"),
        "Sort" => wordcraft_gnome::t!("Sort"),
        "Spelling Suggestions" => wordcraft_gnome::t!("Spelling Suggestions"),
        "Tab" => wordcraft_gnome::t!("Tab"),
        "Tab Character" => wordcraft_gnome::t!("Tab Character"),
        "Table Column Width" => wordcraft_gnome::t!("Table Column Width"),
        "Table Row Height" => wordcraft_gnome::t!("Table Row Height"),
        "Tabs" => wordcraft_gnome::t!("Tabs"),
        "Text Highlight Color" => wordcraft_gnome::t!("Text Highlight Color"),
        "Track Changes" => wordcraft_gnome::t!("Track Changes"),
        "Type Text" => wordcraft_gnome::t!("Type Text"),
        "Underline Color" => wordcraft_gnome::t!("Underline Color"),
        "Update Style to Match Selection" => wordcraft_gnome::t!("Update Style to Match Selection"),
        "Widow/Orphan Control" => wordcraft_gnome::t!("Widow/Orphan Control"),
        _ => return crate::i18n::translate(label),
    };
    localized.into()
}
