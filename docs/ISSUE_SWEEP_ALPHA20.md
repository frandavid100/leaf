# Fifth issue sweep — in progress

David authorized another pass over all current issues, new and old. Base:
`21667e8b649538d3a61d230b327a658e9906fd66` on main; published alpha.19 remains
unchanged while this work is validated. Existing pending work, releases/tags,
archive branches and the separate Weblate PR are preserved.

## Current candidates

- #58/#59: exact compass-symbolic/comment-symbolic assets from the existing
  pinned, CC0 GNOME Icon Development Kit; native symbolic GTK presentation.
- #60: Pages tooltip uses the same current/total layout state as thumbnails,
  with a localized complete message and a safe empty-document fallback.
- #18: GNOME Text Editor's established grouped-checkbutton theme selector and
  attributed stylesheet replace the former Cairo circles. Libpanel is absent
  from the supported native dependency graph; no new dependency is introduced.
- #49: black/white contrasting insertion caret, WordCraft's 100ms idle opacity
  fade, native blink timing/timeout, static reduced-motion/blink-disabled state,
  and inactive caret animation cancellation. Geometry remains engine-owned.
- #50: contextual Redo/Repeat body, with the last supported formatting result
  captured per document. Toggle results are canonicalized; selection changes do
  not supply old target IDs. IO, typing and structural edits are not replayed.
- #51: real engine undo labels and redo snapshots, cumulative engine traversal,
  shared toolbar/document-menu models, shortcut columns and document/revision
  guards. Redo inspection uses a disposable public edit snapshot, never a
  guessed frontend history or mutations of the live session.
- #48: ruler API/layout investigation ongoing.

No issue is closed by this work until native and installed verification passes.

## Capability audit before authoring UI

`examples/issue20_capabilities.rs` exercises public commands and real files:

| Feature | DOCX reopen | ODT reopen with the pinned exporter |
| --- | --- | --- |
| #56 fraction/power/root equation | Structured editable Equation retained. | No Equation object survives. |
| #55 footnote | Independent Footnote part and reference retained. | Footnote part/reference structure lost. |
| #54 multipage automatic TOC | TOC field retained and update command works. | TOC field absent; visible entries alone are static. |
| #53 floating image | Square wrapping/position retained. | Image survives as inline; wrapping/position lost. |

These are runtime results from synthetic public-API cases, not a claim about
all imported files. Their DOCX/PDF and on-screen editing requirements still need
feature-specific verification. Do not expose lossy ODT save as a successful
editable round trip or modify the engine/dependency pins to complete these issues.

Generated DOCX controls contain no embedded thumbnail part, and Leaf registers
no system thumbnailer. That does not attribute the maintainer's GNOME Files
report (#52); the actual affected files/cache/provider remain uninspected.

Old engine/layout reports #1/#2/#7 retain their confirmed or tracked upstream
limits. #9 still needs its original contextual-spacing case; ordinary spacing
success is insufficient. #37's actual AdwInlineViewSwitcher requires a newer
native API baseline; substituting the old widget does not meet its specification.

## Validation so far

Baseline: 128 Rust tests. Four new engine-history/repeat tests and the new caret
fade/static-state test pass. GUI compilation succeeds. Real GTK interaction,
visual inspection, translations and installed packaging are in progress.
