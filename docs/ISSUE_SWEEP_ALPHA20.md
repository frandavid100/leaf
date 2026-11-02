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
- #48: 175ms native overlay, hover strip/pin/menu keyboard entry, public layout
  alignment and indent/tab commands. Gestures retain selection and commit once;
  stale document/revision/selection leases are rejected. Drag cancellation and
  keyboard Undo/Redo are supported; no permanent page space is reserved.

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

Baseline: 128 Rust tests; current full suite: **140 passed**. Four new engine
history/repeat tests, one caret fade/static-state test, three ruler regressions
and four structural ODT save-preservation tests pass. Four Python tests,
project links and 611 complete English/Spanish gettext messages also pass.

The real pointer/keyboard driver `tools/sweep20-pointer-test.sh` verifies
Repeat at another selection, ordinary Undo/Redo, cumulative history traversal,
non-mutating opening/cancellation, ruler hover/pin, drag, keyboard adjustment,
Undo, selection retention, zoom/scroll and current/total tooltip synchronization.
The native English run passes. Spanish/HiDPI, optimized native CI and installed
Flatpak checks are still pending; no issue is closed and no alpha.20 release is
claimed at this checkpoint.

Native interaction testing found and fixed two reentrancy/focus problems: focus
notifications must not overwrite font/size input, and changing the Pages tooltip
must occur after releasing the document borrow because GTK can immediately query
the canvas tooltip. Native ruler keyboard Undo is explicitly routed while ruler
focus is held, without taking Ctrl+Z from native text fields.

The source-identical engine boundary remains intact (`Cargo.toml`, `Cargo.lock`,
`vendor`, `patches`). Full DOCX/ODT corpus and new Orca stages were not added.
No claim of human Wayland/GNOME OS/IME/device testing is made.
