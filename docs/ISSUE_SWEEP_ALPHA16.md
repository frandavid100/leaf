# Open-issue review for alpha.16

Requested by David on 2026-11-01: review all open issues, close those resolved and
publish another alpha. Starting source: `5582a5fff91ebf52ab5461fee2aabb11a20f8bcb`,
branch `wordcraft-saving`. All 24 open issues reviewed. Publication/closure are
pending full native and installed validation at this source stage.

## Fixes prepared

| Issue | Result | Focused verification |
| --- | --- | --- |
| #3 | Fixed spread slots, including orphan page/mixed sizes. | Geometry/hit testing at multiple zooms. |
| #4 | Outside click exits header/footer; no mode bar. | Native header test and real pointer click; Escape retained. |
| #8 | Page numbers at captured selection; no position/alignment. | Body/footer, undo, round trips/protection. |
| #10 | Immediate valid scalars, complete typing, grouped undo. | Actual paragraph/size controls, invalid/no-op input, pin suite. |
| #11 | Named presets disable dimensions; Custom enables. | Five presets, orientation, cancel. |
| #12 | Style/Paragraph/Text follow selection and mixed state. | Model aggregation, actual pinned Style/Paragraph, pin/undo suite. |
| #13 | Editable Font/Size row, previews, effective-size menus. | Font activation, quarter points, preview attributes, typing; inherited styles/parts/complex scripts. |
| #17 | Anchor click reveals overlapping root threads. | Model overlap/reply plus GTK click/drag/document replacement. |
| #18 | Persistent grouped circular theme choices. | Three toggles, StyleManager, restart, live document appearance. |
| #19 | Zoom row inside document menu. | Reset/increment/fit, existing zoom/HiDPI regressions. |
| #20 | Screen page frame has 6px rounded clipping. | Same engine/layout; native/installed screenshots. |
| #21 | Save uses the opened source with any recovery preference. | Save/reopen on native path and real installed document portal. |
| #22 | Persistent user name from system account. | Preferences, both tabs, prior authorship, restart/Flatpak. |
| #23 | Selection link editor with mixed/cancel/stale handling. | Actual context/dialog; text/format/undo/protection; DOCX/ODT. |
| #24 | Stateful per-document Editing/Reviewing. | Tracked edit, revisions retained, selection/other-tab isolation. |

## Kept open

| Issue | Review result and remaining work |
| --- | --- |
| #1, #2 | Known unequal-column wrapping and separator defects remain in the pinned upstream (#2382/#5178). No engine change or duplicate report. |
| #5, #6 | Mouse resize/selection/movement/insertion handles need their own implementation and merged/multipage/zoom tests. Existing keyboard/menu table features remain. |
| #7 | Absent even slot can fall back to default; the public edit command then edits that shared story. This source evidence remains a candidate explanation for the supplied file. Independent allocation and round-trip/undo verification remain. |
| #9 | Ordinary two-paragraph spacing changes measured layout by 24pt and survives DOCX reopen. This does not reproduce the reported file/contextual suppression. Keep open until its cause is isolated; #10 removes the extra Apply step. |
| #14–#16 | Coordinated new-document preferences, inheritable document defaults and explicit fragment overrides are not implemented here. Persistence and migration need joint validation. |

## Explicit submission exceptions

Font/size, character spacing/scale/position, paragraph indents/spacing, exact line
spacing, RGB colors and table measurements apply directly. Table operations have
directly labelled buttons instead of generic Apply rows. Explicit submission
remains for coordinated page/margin/header/metadata properties, column geometry,
aspect-constrained image resizing, batch table border width/color, structural
insert/split/convert operations, phonetic guides, style creation/rename and links.
These collect an atomic action; cancel must preserve the document.

## Validation and limits

Focused model checks: `tests/issue_sweep.rs`. Real native controls:
`src/gnome/diagnostics/issues.rs`. Full CI also checks pointer input, both locales,
adaptive menus, selection/undo, headers, annotations, proofing, lifecycle, HiDPI,
portals and the existing Orca/AT-SPI stage. Publication requires successful native
and installed Flatpak checks; release builds stay optimized with unwinding.

Dependency boundary checked against `f11fbe558c5d7903b4044cfd99399dec948ac8c8`:
no engine/render, Cargo revisions, vendor or patch change. Public screenshots
remain David's. No claim of human GNOME OS, braille, physical-printer or
all-document testing. ODT comments/category still require DOCX to avoid loss.
