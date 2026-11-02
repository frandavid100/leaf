Leaf 0.1.0-alpha.20 adds native history pickers, repeated formatting and a viewport ruler, with interface and saving safeguards.

- Undo and Redo retain separate main buttons and history chevrons. The document menu uses the same real engine steps. Choosing a deeper row traverses all steps through that row; only the top row shows the normal shortcut.
- When nothing can be redone, the Redo body repeats the last supported text/paragraph formatting result at the current target. Selection changes retain it; typing, file operations and structural commands are not replayed. Normal Redo takes priority.
- Hover at the viewport's top edge to reveal the overlaid ruler, or open Ruler from the document menu. Pin keeps it visible. Drag indents/tab stops, click to add a left tab, or use its keyboard controls. Visibility does not change page layout; edits use engine commands and normal Undo.
- The theme selector uses attributed GNOME Text Editor checkbuttons and styling. Navigation and Comments use the requested compass-symbolic and comment-symbolic icons. Pages has a localized current/total tooltip.
- The insertion caret is black on light pages and contrasting white on dark pages, with smooth 100ms opacity transitions and native blink/reduced-motion settings.
- Leaf refuses ODT saving when the pinned exporter would lose editable equations, footnotes, an automatic table of contents or floating-image wrapping. Save as DOCX instead. This protection does not add the full authoring interfaces requested in #53–#56.

WordCraft, dependencies and compatibility patches are unchanged. Unequal columns, selected-text separators, absent independent even header/footer variants, the original spacing report, modern AdwInlineViewSwitcher and GNOME Files thumbnails remain open. Weblate activation and human/device testing remain outside this release.

Install Leaf-0.1.0-alpha.20-x86_64.flatpak over alpha.19. Experimental, AI-assisted development. Native and installed checks gate publication; see docs/ISSUE_SWEEP_ALPHA20.md for evidence and limits.
