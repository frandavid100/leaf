//! Stable application preference identities; no document or engine state.
#[derive(Clone, Copy)]
pub struct Tool {
    pub id: &'static str,
    pub icon: &'static str,
    pub label: &'static str,
    pub default_visible: bool,
}
pub fn tools(group: &str) -> Vec<Tool> {
    let definitions: &[(&str, &str, &str, bool)] = match group {
        "paragraph" => &[
            (
                "alignment",
                "format-justify-left-symbolic",
                crate::t!("Alignment"),
                true,
            ),
            ("list", "view-list-symbolic", crate::t!("List"), true),
            ("columns", "view-dual-symbolic", crate::t!("Columns"), false),
            (
                "indents",
                "format-indent-more-symbolic",
                crate::t!("Indents"),
                false,
            ),
            (
                "spacing",
                "format-justify-fill-symbolic",
                crate::t!("Line Spacing"),
                false,
            ),
            ("borders", "view-grid-symbolic", crate::t!("Borders"), false),
            (
                "shading",
                "color-select-symbolic",
                crate::t!("Shading"),
                false,
            ),
        ],
        "text" => &[
            ("bold", "format-text-bold-symbolic", crate::t!("Bold"), true),
            (
                "italic",
                "format-text-italic-symbolic",
                crate::t!("Italic"),
                true,
            ),
            (
                "underline",
                "format-text-underline-symbolic",
                crate::t!("Underline"),
                true,
            ),
            (
                "strike",
                "format-text-strikethrough-symbolic",
                crate::t!("Strikethrough"),
                false,
            ),
            (
                "superscript",
                "format-text-rich-symbolic",
                crate::t!("Superscript"),
                false,
            ),
            ("font", "font-select-symbolic", crate::t!("Font"), false),
            (
                "ink",
                "font-select-symbolic",
                crate::t!("Text Color"),
                false,
            ),
            (
                "highlight",
                "color-select-symbolic",
                crate::t!("Highlight"),
                false,
            ),
            (
                "language",
                "preferences-desktop-locale-symbolic",
                crate::t!("Language"),
                false,
            ),
        ],
        "elements" => &[
            ("image", "insert-image-symbolic", crate::t!("Image"), true),
            ("table", "view-grid-symbolic", crate::t!("Table"), true),
            (
                "toc",
                "view-list-symbolic",
                crate::t!("Table of Contents"),
                false,
            ),
            (
                "equation",
                "accessories-calculator-symbolic",
                crate::t!("Equation…"),
                false,
            ),
            (
                "bookmark",
                "bookmark-new-symbolic",
                crate::t!("Bookmark…"),
                false,
            ),
            (
                "comment",
                "user-available-symbolic",
                crate::t!("Comment…"),
                false,
            ),
            ("link", "insert-link-symbolic", crate::t!("Link…"), false),
            (
                "footnote",
                "format-text-rich-symbolic",
                crate::t!("Footnote"),
                false,
            ),
        ],
        _ => &[],
    };
    definitions
        .iter()
        .map(|&(id, icon, label, default_visible)| Tool {
            id,
            icon,
            label,
            default_visible,
        })
        .collect()
}
/// Ignore stale/duplicate/unknown IDs while retaining known order and switches.
pub fn normalize(group: &str, saved: &serde_json::Value) -> Vec<(String, bool)> {
    let tools = tools(group);
    let mut result = Vec::new();
    if let Some(saved) = saved.as_array() {
        for item in saved {
            if let (Some(id), Some(on)) = (item["id"].as_str(), item["visible"].as_bool()) {
                if tools.iter().any(|t| t.id == id) && !result.iter().any(|(k, _)| k == id) {
                    result.push((id.to_owned(), on));
                }
            }
        }
    }
    for tool in tools {
        if !result.iter().any(|(id, _)| id == tool.id) {
            result.push((tool.id.into(), tool.default_visible));
        }
    }
    result
}
