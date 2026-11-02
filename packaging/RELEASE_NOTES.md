Leaf 0.1.0-alpha.21 updates the document engine and addresses the latest interface reports.

- WordCraft 0.14.0, pinned to its exact stable release commit. The formats snapshot was recopied and the four isolated ODT compatibility patches reapplied.
- Undo/Redo use the same native AdwSplitButton as Paragraph, preserving independent history menus and Repeat behavior.
- Font and Size selectors share the full text-popover row; their choice popups follow the actual control width, including native popup chrome.
- Thin white horizontal and vertical rulers follow the active sheet, zoom and scroll. They appear independently on edge hover; Pin and the document-menu ruler entry are removed. Ctrl+Alt+R focuses the horizontal ruler for keyboard use; Escape returns to the document.

The vertical ruler displays page coordinates; it does not expose unsupported vertical editing. Floating-image, TOC, footnote and equation authoring remain open. Existing ODT saving safeguards remain enabled.

Install Leaf-0.1.0-alpha.21-x86_64.flatpak over alpha.20. Experimental, AI-assisted development. Native and installed checks gate publication. See docs/ISSUE_SWEEP_ALPHA21.md for validation and remaining limits.
