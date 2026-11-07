Leaf 0.1.0-alpha.35 — personal templates and editing workflows

- Choose the following paragraph style from the existing Style details. The document remains the source of each style's appearance.
- Create untitled copies from the system Templates folder or a folder chosen through the native chooser. Manage the original files in Files.
- Paste plain text with Ctrl+Shift+V or the context menu. Copy formatting once, or keep the painter active for multiple destinations; Escape cancels.
- Move complete heading sections, with their children, tables, images and references, from Structure by dragging or using the arrow buttons. One Undo restores the move.
- Search and replace whole words, restrict operations to a captured selection, preserve capitalization, and search notes, headers and footers. Advanced options support regular expressions and tab, soft-break and paragraph-break codes, with inline examples and errors.
- Inspect document font declarations and uses, see actual fallback fonts, navigate uses, and explicitly substitute a font throughout the document with Undo. Opening documents preserves their authored font names.
- Paste external images/screenshots and drop image files at a document position; images preserve their proportions and initially fit the text width.

Safety limits: moving a heading inside a table, moving across a page-section break, and moving sections while tracking changes are refused without altering the document. Document-wide font replacement also requires change tracking to be off. A selected search range is invalidated by unrelated document edits. Regex replacements do not insert at empty matches.

WordCraft remains pinned at 0.16.0. No new conflict-protection or symbol-picker workflow is included. Native and installed GNOME 49/51 automated checks are required before publication; they do not claim human GNOME OS validation.
