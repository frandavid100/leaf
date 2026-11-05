//! Native equation authoring through WordCraft's public UnicodeMath representation.
use serde_json::json;
use wordcraft_doc::{
    InlineObject, Pos,
    math::{parse_linear, to_linear},
};
use wordcraft_engine::Session;
pub fn target(s: &Session) -> Option<(Pos, String, bool, bool)> {
    let (a, b) = s.sel.ordered();
    let candidates = [
        s.sel.focus.clone(),
        Pos {
            off: s
                .sel
                .focus
                .off
                .saturating_sub(wordcraft_doc::para::OBJ.len_utf8()),
            ..s.sel.focus.clone()
        },
        a.clone(),
    ];
    for pos in candidates {
        if !s.sel.is_collapsed()
            && (a.path != b.path || b.off != a.off + wordcraft_doc::para::OBJ.len_utf8())
        {
            return None;
        }
        if let Some(InlineObject::Equation {
            linear,
            math,
            display,
        }) = s.doc.para_at(&pos).and_then(|p| p.object_at(pos.off))
        {
            let text = if math.nodes.is_empty() {
                linear.clone()
            } else {
                to_linear(&math.nodes)
            };
            let editable = math.nodes.is_empty() || parse_linear(&text) == math.nodes;
            return Some((pos, text, *display, editable));
        }
    }
    None
}
pub fn validate(text: &str) -> Result<(), String> {
    if text.trim().is_empty() || text.len() > 4096 || text.contains('\\') {
        return Err(wordcraft_gnome::t!(
            "Use UnicodeMath input, up to 4096 characters. LaTeX commands are not supported here."
        )
        .into());
    }
    let mut stack = Vec::new();
    for c in text.chars() {
        match c {
            '(' | '[' | '{' => stack.push(c),
            ')' | ']' | '}' => {
                let expected = match c {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                if stack.pop() != Some(expected) {
                    return Err(wordcraft_gnome::t!(
                        "Complete the expression and match its brackets."
                    )
                    .into());
                }
            }
            _ => {}
        }
    }
    if !stack.is_empty() || text.trim_end().ends_with(['^', '_', '/']) || text.contains("()") {
        return Err(wordcraft_gnome::t!("Complete the expression and match its brackets.").into());
    }
    Ok(())
}
pub fn apply(s: &mut Session, text: &str, display: bool) -> Result<(), String> {
    validate(text)?;
    let existing = target(s);
    if existing
        .as_ref()
        .is_some_and(|(_, old, mode, _)| old == text && *mode == display)
    {
        return Ok(());
    }
    if existing
        .as_ref()
        .is_some_and(|(_, _, _, editable)| !*editable)
    {
        return Err(wordcraft_gnome::t!("This imported equation cannot be edited faithfully with the basic editor. Its original structure is preserved.").into());
    }
    let snapshot = s.edit_snapshot();
    let mark = s.undo_mark();
    let result = (|| {
        if let Some((at, old, mode, _)) = existing {
            crate::selection::set_caret(s, &at, false)?;
            if old != text {
                s.run("equation.set", &json!({"linear":text}))
                    .map_err(|e| e.to_string())?;
            }
            if mode != display {
                s.join_next_undo();
                s.run("equation.display", &json!({"value":display}))
                    .map_err(|e| e.to_string())?;
            }
        } else {
            s.run(
                "insert.equation",
                &json!({"linear":text,"display":display,"edit":false}),
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    })();
    if result.is_err() {
        s.restore(snapshot)
    } else {
        s.merge_undo_since(mark)
    }
    result
}
