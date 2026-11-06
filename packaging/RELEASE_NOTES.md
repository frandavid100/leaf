Leaf 0.1.0-alpha.31 — second candidate for maintainer testing.

- Review keeps valid findings visible while rechecking, preserving the current filter, browsing position and unaffected comment/change cards (#133).
- Sidebar and editor header regions align through native sizing, including their selector/tab rows (#134).
- Navigation uses a native Pages/Structure dropdown while retaining each document's browsing, filters, expansion and editing target (#135).
- Text and Paragraph property/repeat controls use the same native flat split component as Undo/Redo. Submenus have their own titles and correct Back paths; Size contains its icon and label in the field (#136).
- Font opens its searchable list immediately. Language provides searchable choices and a reachable final action. Colour pages embed the real native palette and custom editor. Style and Elements share the navigation, spacing and scrolling refinements (#136).
- Document rulers reject popover-surface events, preventing a click in a formatting control from stealing focus or redirecting typing to the document.
- Opening paragraph spacing preserves proportional spacing and blank unset fields; GTK initialization no longer applies an exact point value to the document.

This is an unpublished candidate. WordCraft, the renderer, dependency revisions and existing patches are unchanged. The original-document overlap (#132), caption ownership and other incomplete issues remain open. Source, validation, checksums and limits are documented in docs/ALPHA31_UI_VALIDATION.md.
