# Open-issue review for alpha.16

Requested by David on 2026-11-01: review all open issues, close those resolved and
publish another alpha. Starting source: `5582a5fff91ebf52ab5461fee2aabb11a20f8bcb`,
branch `wordcraft-saving`. All 24 original open issues were reviewed, followed by
eight reports added during the work (#25–#32). **14 closed; 18 remain open**.

[Alpha.16 published](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.16)
from `a7e3f349f8a4a2fdb3033acaecbc36d20d5f3cd2` after [successful native and installed validation](https://github.com/frandavid100/leaf/actions/runs/41357492415).
Public bundle: **10,894,408 bytes**, SHA256 `e1f08d048733bcdc6725066c81785df8bd65775db67e574242947fb157993120`.
All eight public assets match their advertised sizes/digests; SHA256SUMS and
source-commit.txt verified from the actual downloads.

Native artifact ZIP SHA256:
`842a59576d34f5b19235ee7c8dd393edf473cf2dac132390206661ab170e64be`.
Package artifact ZIP SHA256:
`ec61112ab4d0dc9b754051f360ce3a77660dd507465cc1180695415cbb6754a1`.
Both downloaded/extracted and checked; native menu/text/spread and installed dark
page screenshots inspected as internal QA, not public product screenshots.

## Closed after validation

| Issue | Result | Focused verification |
| --- | --- | --- |
| #3 | Fixed spread slots, including orphan page/mixed sizes. | Geometry/hit testing at multiple zooms. |
| #4 | Outside click exits header/footer; no mode bar. | Native header test and real pointer click; Escape retained. |
| #8 | Page numbers at captured selection; no position/alignment. | Body/footer, undo, round trips/protection. |
| #10 | Immediate valid scalars, complete typing, grouped undo. | Actual paragraph/size controls, invalid/no-op input, pin suite. |
| #11 | Named presets disable dimensions; Custom enables. | Five presets, orientation, cancel. |
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
| #7 | [Reduced API reproduction](ISSUE7_TRIAGE.md) confirms absent even header/footer slots reuse default story IDs; edits are shared and DOCX preserves that allocation. Original-file/direct-click/PDF/undo and a complete isolation fix remain. |
| #9 | Ordinary two-paragraph spacing changes measured layout by 24pt and survives DOCX reopen. This does not reproduce the reported file/contextual suppression. Keep open until its cause is isolated; #10 removes the extra Apply step. |
| #12 | Partially improved: current Style, core Paragraph/Text properties and mixed states pass native synchronization checks. The embedded Columns editor depends on selection/section state beyond the paragraph-property snapshot; keep open until that and broader coverage are complete. |
| #14–#16 | Coordinated new-document preferences, inheritable document defaults and explicit fragment overrides are not implemented here. Persistence and migration need joint validation. |

## Reports added during this review

All eight were read and received review notes; none is established as fixed by
alpha.16. They remain open for a subsequent targeted change.

| Issue | Remaining work |
| --- | --- |
| #25 | Explicit heading navigation near the viewport top, independent of ordinary caret scrolling. |
| #26 | Entry-specific spelling/grammar menus with applicable actions and stale-range protection. |
| #27 | Reproduce Shift+click failure and investigate click/drag anchor coordination; existing static extension code alone is not proof of a fix. |
| #28 | Double-click drag in whole-word units, reversal and autoscrolling. |
| #29 | Native header-group spacing, actual geometry/hover/focus and adaptive/HiDPI checks. |
| #30 | Remove sidebar headings/refresh and complete automatic results under effective review settings. |
| #31 | Modest reduction of thumbnail row/padding spacing. |
| #32 | Verify selected word ranges exclude trailing whitespace and coordinate boundaries with #28. |

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
portals and the existing Orca/AT-SPI stage. The final native and installed Flatpak
workflow passed: 113 Rust tests, four
Python tests and 590 source gettext messages, plus the full GTK regression suite.
Installed checks include new controls, user/theme restart, upgrade from alpha.15,
reinstall, real document-portal file save/reopen, print and Orca/AT-SPI. Release
builds stay optimized with unwinding.

Dependency boundary checked against `f11fbe558c5d7903b4044cfd99399dec948ac8c8`:
no engine/render, Cargo revisions, vendor or patch change. Public screenshots
remain David's. No claim of human GNOME OS, braille, physical-printer or
all-document testing. ODT comments/category still require DOCX to avoid loss.
