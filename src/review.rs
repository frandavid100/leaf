//! Read-only revision snapshots and targeted public WordCraft review commands.
use serde_json::{Value, json};
use wordcraft_doc::Pos;
use wordcraft_engine::Session;
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub start: Pos,
    pub end: Pos,
    pub kind: String,
    pub author: String,
    pub text: String,
    pub description: String,
    raw: Value,
}
pub fn changes(s: &mut Session) -> Result<Vec<Change>, String> {
    let value = s
        .run("review.changes", &json!({}))
        .map_err(|e| e.to_string())?;
    value
        .as_array()
        .ok_or("Invalid revision list")?
        .iter()
        .map(|v| {
            Ok(Change {
                start: serde_json::from_value(v["start"].clone()).map_err(|e| e.to_string())?,
                end: serde_json::from_value(v["end"].clone()).map_err(|e| e.to_string())?,
                kind: v["kind"].as_str().unwrap_or("").into(),
                author: v["author"].as_str().unwrap_or("").into(),
                text: v["text"].as_str().unwrap_or("").into(),
                description: v["description"].as_str().unwrap_or("").into(),
                raw: v.clone(),
            })
        })
        .collect()
}
pub fn select(s: &mut Session, change: &Change) -> Result<(), String> {
    if !changes(s)?.iter().any(|c| c == change) {
        return Err(crate::t!("This change is out of date. Refresh Review.").into());
    }
    crate::selection::select_range(s, &change.start, &change.end)
}
pub fn resolve(s: &mut Session, change: &Change, accept: bool) -> Result<(), String> {
    if !matches!(
        change.kind.as_str(),
        "insert" | "delete" | "format" | "moveFrom" | "moveTo"
    ) {
        return Err(crate::t!("This type of change cannot be resolved here.").into());
    }
    let original = s.edit_snapshot();
    let result = select(s, change).and_then(|_| {
        s.run(
            if accept {
                "review.accept"
            } else {
                "review.reject"
            },
            &json!({}),
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
    });
    if result.is_err() {
        s.restore(original);
    }
    result
}
