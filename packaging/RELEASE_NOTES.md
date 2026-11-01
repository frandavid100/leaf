Leaf 0.1.0-alpha.17 fixes interaction and presentation reports from the alpha.16 review.

- Correct canvas input-method ownership so Font and Size accept real keyboard input. Font choices show each family once in its own typeface.
- Shift+click keeps the original anchor; double-click excludes trailing spaces; double-click dragging selects complete words in both directions, with viewport-edge scrolling.
- New ordinary links are blue/underlined. Ctrl+click opens the destination using the native URI launcher and Flatpak portal; the canvas tooltip explains the gesture.
- The theme selector uses larger circles and the current accent. Zoom is a flat row at the top of the document menu and keeps that menu open during adjustments.
- Author preferences default to the account's real/full name, preserving explicit custom names and existing authorship.
- Heading jumps align near the top. Page rows show as many pages as fit, with stable incomplete-row alignment; thumbnail gaps are slightly reduced.
- Pinned Columns controls follow section/selection changes. Formatting headers have less padding; Document Properties uses the supported native libadwaita view switcher.
- Spelling entries have proofing-only menus sharing the document's suggestions/actions. Sidebar headings/manual refresh and the routine Pending Changes toast are removed; visible spelling results refresh automatically.

The spelling/default inheritance redesign (#14–#16 and the remaining part of #30), upstream columns, advanced mouse table handles, odd/even independence and the document-specific spacing report remain open. Engine, dependencies, vendor and patches are unchanged. ODT comments/category still require DOCX for preservation.

Install Leaf-0.1.0-alpha.17-x86_64.flatpak over alpha.16. Experimental, AI-assisted development. See docs/ISSUE_SWEEP_ALPHA17.md for verified scope and limitations.
