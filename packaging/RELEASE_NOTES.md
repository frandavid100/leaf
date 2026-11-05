Leaf 0.1.0-alpha.29 improves document tools and picture interaction, and adds a basic equation editor.

- References and the table of contents update automatically after edits and before saving, PDF export and printing. Undo/redo retains the source edit. Unrelated date/time fields keep their cached values while typing.
- Click a footnote number to move between its reference and note, or use Alt+Return. Footnote numbering is configured in document/section Properties, preserving unchanged inherited settings.
- Edit the existing table of contents from its context menu. Add a numbered caption directly to the selected image or current table, ready to type its description.
- Insert groups content, references, headers/footers, automatic fields and breaks. Link handles both URLs and structured links to headings, bookmarks, figures and tables.
- Picture selection is translucent. All eight resize handles work; corners preserve proportions, edges adjust one dimension. Drag previews show the image, support scrolling and valid page targets, cancel with Escape, and undo in one step.
- Pages reveals the active thumbnail and labels physical positions with the document total, without overriding manual sidebar browsing on redraw.
- Insert and edit basic UnicodeMath equations with Alt+=, including fractions, scripts, roots and Greek letters. Complex imported equations that cannot round-trip through this editor remain protected.

Known limits: the reported original footnote placement/missing-number cases still need their source document to reproduce. Moving a floating image onto a paragraph beginning with a manual page break can fail in the pinned engine; Leaf now rejects that placement and preserves the original image instead of silently putting it on the wrong page. Unequal columns and selected-text continuous sections retain their documented engine limitations. Equations, notes, references, automatic fields and floating images require DOCX to preserve their structure.

Prepared through Leaf’s candidate process: David tests the package before approving publication. Its manifest identifies the source and workflow; SHA256SUMS identifies the exact packages. Approval promotes these files without rebuilding. The regular bundle uses GNOME 49; the FlatPark archive is checked under GNOME 51. Leaf is experimental and its development has used substantial AI assistance.
