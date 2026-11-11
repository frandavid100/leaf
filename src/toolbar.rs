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
            (
                "bullets",
                "view-list-bullet-symbolic",
                crate::t!("Bulleted List"),
                false,
            ),
            (
                "numbering",
                "view-list-ordered-symbolic",
                crate::t!("Numbered List"),
                false,
            ),
            (
                "multilevel",
                "view-list-symbolic",
                crate::t!("Multilevel List"),
                false,
            ),
            (
                "list-remove",
                "edit-clear-symbolic",
                crate::t!("No List"),
                false,
            ),
            (
                "list-level",
                "format-indent-more-symbolic",
                crate::t!("List Level"),
                false,
            ),
            (
                "list-continue",
                "view-list-ordered-symbolic",
                crate::t!("Continue Numbering"),
                false,
            ),
            (
                "list-restart",
                "view-refresh-symbolic",
                crate::t!("Restart Numbering at 1"),
                false,
            ),
            (
                "list-start",
                "view-list-ordered-symbolic",
                crate::t!("Set Starting Number"),
                false,
            ),
            (
                "list-custom",
                "document-edit-symbolic",
                crate::t!("Customize List…"),
                false,
            ),
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
                crate::t!("Spacing"),
                false,
            ),
            ("borders", "view-grid-symbolic", crate::t!("Borders"), false),
            (
                "shading",
                "color-select-symbolic",
                crate::t!("Shading"),
                false,
            ),
            (
                "direction",
                "format-text-direction-symbolic",
                crate::t!("Text Direction"),
                false,
            ),
            (
                "marks",
                "format-text-plaintext-symbolic",
                crate::t!("Show Formatting Marks (¶)"),
                false,
            ),
            (
                "keepNext",
                "view-continuous-symbolic",
                crate::t!("Keep with Next"),
                false,
            ),
            (
                "keepLines",
                "view-paged-symbolic",
                crate::t!("Keep Lines Together"),
                false,
            ),
            (
                "pageBreakBefore",
                "view-dual-symbolic",
                crate::t!("Page Break Before"),
                false,
            ),
            (
                "widowControl",
                "view-wrapped-symbolic",
                crate::t!("Widow and Orphan Control"),
                false,
            ),
            (
                "sort",
                "view-sort-ascending-symbolic",
                crate::t!("Sort"),
                false,
            ),
            (
                "tabs",
                "format-indent-more-symbolic",
                crate::t!("Tab Stops"),
                false,
            ),
            (
                "reset",
                "edit-clear-all-symbolic",
                crate::t!("Reset Paragraph Formatting"),
                false,
            ),
            (
                "indent",
                "format-indent-more-symbolic",
                crate::t!("Increase Indent"),
                false,
            ),
            (
                "outdent",
                "format-indent-less-symbolic",
                crate::t!("Decrease Indent"),
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
                crate::t!("Text Position"),
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
            (
                "size",
                "font-select-symbolic",
                crate::t!("Font Size"),
                false,
            ),
            ("case", "font-symbolic", crate::t!("Change Case"), false),
            (
                "clear",
                "edit-clear-all-symbolic",
                crate::t!("Clear Text Formatting"),
                false,
            ),
            (
                "character-spacing",
                "format-justify-fill-symbolic",
                crate::t!("Character Spacing"),
                false,
            ),
            (
                "smallCaps",
                "font-smallcaps-symbolic",
                crate::t!("Small Caps"),
                false,
            ),
            (
                "caps",
                "headings-allcap-symbolic",
                crate::t!("All Caps"),
                false,
            ),
            (
                "hidden",
                "view-conceal-symbolic",
                crate::t!("Hidden Text"),
                false,
            ),
            (
                "outline",
                "font-outline-symbolic",
                crate::t!("Outline"),
                false,
            ),
            ("shadow", "font-shadow-symbolic", crate::t!("Shadow"), false),
            (
                "emboss",
                "font-emboss-symbolic",
                crate::t!("Embossed"),
                false,
            ),
            (
                "engrave",
                "font-engrave-symbolic",
                crate::t!("Engraved"),
                false,
            ),
            (
                "text-border",
                "view-grid-symbolic",
                crate::t!("Text Border"),
                false,
            ),
            (
                "enclose",
                "font-enclose-symbolic",
                crate::t!("Enclosed Characters"),
                false,
            ),
            (
                "phonetic",
                "font-phonetic-symbolic",
                crate::t!("Phonetic Guide"),
                false,
            ),
        ],
        "elements" => &[
            ("image", "insert-image-symbolic", crate::t!("Image"), true),
            (
                "insert-new-toc",
                "list-add-symbolic",
                crate::t!("Insert Table of Contents…"),
                false,
            ),
            ("table", "view-grid-symbolic", crate::t!("Table"), true),
            (
                "insert-new-table",
                "list-add-symbolic",
                crate::t!("Insert Table…"),
                false,
            ),
            (
                "insert-new-image",
                "insert-image-symbolic",
                crate::t!("Insert Image…"),
                false,
            ),
            (
                "table-select-cell",
                "view-grid-symbolic",
                crate::t!("Select Cell"),
                false,
            ),
            (
                "table-select-row",
                "view-list-symbolic",
                crate::t!("Select Row"),
                false,
            ),
            (
                "table-select-column",
                "view-dual-symbolic",
                crate::t!("Select Column"),
                false,
            ),
            (
                "table-select-all",
                "view-grid-symbolic",
                crate::t!("Select Table"),
                false,
            ),
            (
                "table-insert-row-above",
                "go-up-symbolic",
                crate::t!("Insert Row Above"),
                false,
            ),
            (
                "table-insert-row-below",
                "go-down-symbolic",
                crate::t!("Insert Row Below"),
                false,
            ),
            (
                "table-insert-column-left",
                "go-previous-symbolic",
                crate::t!("Insert Column to the Left"),
                false,
            ),
            (
                "table-insert-column-right",
                "go-next-symbolic",
                crate::t!("Insert Column to the Right"),
                false,
            ),
            (
                "table-delete-row",
                "list-remove-symbolic",
                crate::t!("Delete Row"),
                false,
            ),
            (
                "table-delete-column",
                "list-remove-symbolic",
                crate::t!("Delete Column"),
                false,
            ),
            (
                "table-delete-all",
                "edit-delete-symbolic",
                crate::t!("Delete Table"),
                false,
            ),
            (
                "table-split-cells",
                "view-grid-symbolic",
                crate::t!("Split Cells"),
                false,
            ),
            (
                "table-split-table",
                "view-dual-symbolic",
                crate::t!("Split Table"),
                false,
            ),
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
            (
                "header",
                "format-text-rich-symbolic",
                crate::t!("Header"),
                false,
            ),
            (
                "footer",
                "format-text-rich-symbolic",
                crate::t!("Footer"),
                false,
            ),
            (
                "fields",
                "format-text-rich-symbolic",
                crate::t!("Automatic Fields"),
                false,
            ),
            ("breaks", "view-paged-symbolic", crate::t!("Breaks"), false),
            (
                "break-textWrapping",
                "view-paged-symbolic",
                crate::t!("Line Break"),
                false,
            ),
            (
                "break-page",
                "view-paged-symbolic",
                crate::t!("Page Break"),
                false,
            ),
            (
                "break-column",
                "view-paged-symbolic",
                crate::t!("Column Break"),
                false,
            ),
            (
                "break-nextPage",
                "view-paged-symbolic",
                crate::t!("Section Break (Next Page)"),
                false,
            ),
            (
                "break-continuous",
                "view-paged-symbolic",
                crate::t!("Section Break (Continuous)"),
                false,
            ),
            (
                "break-evenPage",
                "view-paged-symbolic",
                crate::t!("Section Break (Even Page)"),
                false,
            ),
            (
                "break-oddPage",
                "view-paged-symbolic",
                crate::t!("Section Break (Odd Page)"),
                false,
            ),
            (
                "field-page",
                "format-text-rich-symbolic",
                crate::t!("Page Number"),
                false,
            ),
            (
                "field-pages",
                "format-text-rich-symbolic",
                crate::t!("Total Pages"),
                false,
            ),
            (
                "field-author",
                "format-text-rich-symbolic",
                crate::t!("Author"),
                false,
            ),
            (
                "field-title",
                "format-text-rich-symbolic",
                crate::t!("Document Title"),
                false,
            ),
            (
                "field-filename",
                "format-text-rich-symbolic",
                crate::t!("File Name"),
                false,
            ),
            (
                "field-date",
                "format-text-rich-symbolic",
                crate::t!("Date"),
                false,
            ),
            (
                "field-time",
                "format-text-rich-symbolic",
                crate::t!("Time"),
                false,
            ),
            (
                "table-select",
                "edit-select-symbolic",
                crate::t!("Select Table Cells"),
                false,
            ),
            (
                "table-add",
                "list-add-symbolic",
                crate::t!("Insert Rows or Columns"),
                false,
            ),
            (
                "table-delete",
                "edit-delete-symbolic",
                crate::t!("Delete Rows or Columns"),
                false,
            ),
            (
                "table-merge",
                "view-grid-symbolic",
                crate::t!("Merge Cells"),
                false,
            ),
            (
                "table-split",
                "view-grid-symbolic",
                crate::t!("Split Cells or Table"),
                false,
            ),
            (
                "table-properties",
                "document-properties-symbolic",
                crate::t!("Table Properties"),
                false,
            ),
            (
                "table-caption",
                "format-text-rich-symbolic",
                crate::t!("Table Caption"),
                false,
            ),
            (
                "table-alignment",
                "format-justify-center-symbolic",
                crate::t!("Table Alignment"),
                false,
            ),
            (
                "table-autofit",
                "view-fullscreen-symbolic",
                crate::t!("AutoFit"),
                false,
            ),
            (
                "table-distribute-columns",
                "view-grid-symbolic",
                crate::t!("Distribute Columns Evenly"),
                false,
            ),
            (
                "table-distribute-rows",
                "view-grid-symbolic",
                crate::t!("Distribute Rows Evenly"),
                false,
            ),
            (
                "table-column-width",
                "view-dual-symbolic",
                crate::t!("Column Width"),
                false,
            ),
            (
                "table-row-height",
                "view-paged-symbolic",
                crate::t!("Row Height"),
                false,
            ),
            (
                "table-auto-height",
                "view-restore-symbolic",
                crate::t!("Automatic Row Height"),
                false,
            ),
            (
                "table-header-row",
                "view-list-symbolic",
                crate::t!("Repeat as Header"),
                false,
            ),
            (
                "table-split-rows",
                "view-paged-symbolic",
                crate::t!("Allow Row to Split Across Pages"),
                false,
            ),
            (
                "cell-alignment",
                "format-justify-center-symbolic",
                crate::t!("Cell Alignment"),
                false,
            ),
            (
                "cell-direction",
                "format-text-direction-symbolic",
                crate::t!("Cell Text Direction"),
                false,
            ),
            (
                "cell-shading",
                "color-select-symbolic",
                crate::t!("Cell Background"),
                false,
            ),
            (
                "table-borders",
                "view-grid-symbolic",
                crate::t!("Table Borders"),
                false,
            ),
            (
                "image-size",
                "image-x-generic-symbolic",
                crate::t!("Image Size"),
                false,
            ),
            (
                "image-spacing",
                "format-indent-more-symbolic",
                crate::t!("Space Around Image"),
                false,
            ),
            (
                "image-wrap",
                "view-wrapped-symbolic",
                crate::t!("Text Wrapping"),
                false,
            ),
            (
                "image-move",
                "object-move-symbolic",
                crate::t!("Move with Text"),
                false,
            ),
            (
                "image-original",
                "view-restore-symbolic",
                crate::t!("Restore Original Size"),
                false,
            ),
            (
                "image-replace",
                "insert-image-symbolic",
                crate::t!("Replace Image…"),
                false,
            ),
            (
                "image-delete",
                "edit-delete-symbolic",
                crate::t!("Delete Image"),
                false,
            ),
            (
                "image-caption",
                "format-text-rich-symbolic",
                crate::t!("Image Caption"),
                false,
            ),
            (
                "image-crop",
                "image-crop-symbolic",
                crate::t!("Crop Image"),
                false,
            ),
            (
                "image-alt",
                "image-alt-symbolic",
                crate::t!("Alternative Text"),
                false,
            ),
            (
                "image-rotate-left",
                "object-rotate-left-symbolic",
                crate::t!("Rotate Left"),
                false,
            ),
            (
                "image-rotate-right",
                "object-rotate-right-symbolic",
                crate::t!("Rotate Right"),
                false,
            ),
            (
                "image-flip-horizontal",
                "object-flip-horizontal-symbolic",
                crate::t!("Flip Horizontally"),
                false,
            ),
            (
                "image-flip-vertical",
                "object-flip-vertical-symbolic",
                crate::t!("Flip Vertically"),
                false,
            ),
            (
                "image-decorative",
                "image-decorative-symbolic",
                crate::t!("Mark as Decorative"),
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
/// Placement is presentation state and never changes document content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    Pinned,
    Adaptive,
    More,
}
impl Placement {
    pub fn key(self) -> &'static str {
        match self {
            Self::Pinned => "pinned",
            Self::Adaptive => "adaptive",
            Self::More => "more",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pinned" => Some(Self::Pinned),
            "adaptive" => Some(Self::Adaptive),
            "more" => Some(Self::More),
            _ => None,
        }
    }
}
/// Migrate old switches while ignoring stale or duplicate tool identities.
pub fn placements(group: &str, saved: &serde_json::Value) -> Vec<(String, Placement)> {
    let tools = tools(group);
    let mut result = Vec::new();
    if let Some(saved) = saved.as_array() {
        for item in saved {
            let mode = item["placement"]
                .as_str()
                .and_then(Placement::parse)
                .or_else(|| {
                    item["visible"].as_bool().map(|on| {
                        if on {
                            Placement::Adaptive
                        } else {
                            Placement::More
                        }
                    })
                });
            if let (Some(id), Some(mode)) = (item["id"].as_str(), mode) {
                if tools.iter().any(|t| t.id == id) && !result.iter().any(|(k, _)| k == id) {
                    result.push((id.to_owned(), mode));
                }
            }
        }
    }
    for tool in tools {
        if !result.iter().any(|(id, _)| id == tool.id) {
            result.push((
                tool.id.into(),
                if tool.default_visible {
                    Placement::Adaptive
                } else {
                    Placement::More
                },
            ));
        }
    }
    result
}
/// Compatibility for saved pre-three-placement settings and regression fixtures.
pub fn normalize(group: &str, saved: &serde_json::Value) -> Vec<(String, bool)> {
    placements(group, saved)
        .into_iter()
        .map(|(id, mode)| (id, mode != Placement::More))
        .collect()
}
