Leaf 0.1.0-alpha.31 — candidate for maintainer testing.

- Review keeps valid findings visible while rechecking, preserving the current filter, browsing position and unaffected comment/change cards (#133).
- Sidebar and editor header regions align through native sizing, including their selector/tab rows (#134).
- Navigation uses a native Pages/Structure dropdown while retaining each document's browsing, filters, expansion and editing target (#135).
- Text and Paragraph property/repeat controls use the same native flat split component as Undo/Redo. Submenus have their own titles and correct Back paths; Size contains its icon and label in the field (#136).
- Font opens its searchable list immediately. Language provides searchable choices and a reachable final action. Colour pages embed the real native palette and custom editor. Style and Elements share the navigation, spacing and scrolling refinements (#136).
- Document rulers reject popover-surface events, preventing a click in a formatting control from stealing focus or redirecting typing to the document.
- Opening paragraph spacing preserves proportional spacing and blank unset fields; GTK initialization no longer applies an exact point value to the document.
- Image and Table properties offer Show Caption above the insertion action. Captions retain automatic numbering and direct editing; hiding and showing them preserves authored text with native Undo/Redo (#121, #127).
- Explicit image/table ownership survives DOCX reopening, image movement, identical adjacent images and deletion of a table's first row. Nested-table captions belong to the actual nested table; unassociated imported captions remain editable without guessing ownership (#127).
- Structure navigation shows symbolic content-type icons, caption labels and related caption entries. Typed destinations distinguish tables from images in their first cells, including nested tables (#124).
- Hovering a linked object or caption, or placing the caret in the caption, shows subtle theme-aware editor backing. It does not alter the document, pagination, history or exported output (#126).

This is an unpublished candidate. WordCraft, the renderer, dependency revisions and existing patches are unchanged. Numbered caption creation is available for objects in the document body and requires DOCX. The original-document overlap (#132) remains open because its triggering editing state has not been reproduced. Current caption validation and limits are recorded in docs/CAPTION_OWNERSHIP.md; the preceding interface candidate is documented in docs/ALPHA31_UI_VALIDATION.md.
