Leaf 0.1.0-alpha.31

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

- Returning from Review restores the real Navigation selector before switching the filter stack, including repeated Pages/Structure changes (#135).
- Proofing starts after installing each opened document and continues asynchronously while Review or the whole sidebar is hidden (#155).
- Paragraph lists Alignment, List, Columns, Indents, Line Spacing, Borders and Shading separately. Border and shading repeats are independent; full native colour palettes retain grayscale and custom colours (#149).
- Font Size uses a flat, full-width native row; Text and Paragraph end with matching compact text-only options actions (#150).
- Table operations are grouped into Select, Insert, Delete, Merge Cells and Split, with distinct Split Cells configuration and Table Properties. Table creation remains at the bottom and available without a table target (#151).

Publication of the validated packages is authorized by the maintainer. WordCraft, the renderer, dependency revisions and existing patches are unchanged. Numbered caption creation is available for objects in the document body and requires DOCX. The original-document overlap report (#132) was closed after the maintainer confirmed the unchanged original appears fixed; its original trigger and fixing change remain unidentified. Current caption validation and limits are recorded in docs/CAPTION_OWNERSHIP.md; the preceding interface candidate is documented in docs/ALPHA31_UI_VALIDATION.md.

The remaining reports #1, #2, #9, #52, #84, #99, #100, #107, #152 and #156 are not claimed resolved. See docs/ALPHA31_REMAINING.md for reproduction results and boundaries.
