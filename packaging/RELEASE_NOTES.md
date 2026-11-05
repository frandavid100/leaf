Leaf 0.1.0-alpha.28 adds document references and floating picture controls.

- Insert and edit footnotes on the document canvas, navigate between notes and their references, and configure numbering. Ctrl+Alt+F inserts a footnote.
- Insert an automatic table of contents from document headings, choose its levels and page numbers, and update it from References.
- Add numbered figure/table captions and cross-references to headings, captions or bookmarks. References → Update References refreshes fields and contents. Ctrl+click an internal reference to follow it.
- Place images beside text or above/below it, set spacing and move-with-text behavior, and move or resize pictures with the pointer. Each drag has a preview and a single undo step.
- Fit portrait and landscape sections independently. Explicit percentage zoom uses the same scale throughout the document.
- Keep page thumbnails at a common width and highlight the actual active page in multi-page rows.

Use DOCX for notes, automatic contents, cross-references, captions and floating images. The current ODT exporter loses those structures, so Leaf blocks saves that would discard them.

The regular GitHub Flatpak uses GNOME 49 and updates alpha.27. The FlatPark archive is checked with GNOME 51. Leaf is experimental and its development has used substantial AI assistance.
