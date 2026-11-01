# Alpha.17 issue review

Requested by David on 2026-11-02: close all issues that can be resolved and publish
another alpha. Starting branch `wordcraft-saving`, source
`27bbfcb5930964db2c2e179a679430c9ce6d5c0c`. The 28 currently open reports were read.
This candidate is awaiting full native/installed validation and publication;
no issue is closed before that verification.

## Candidates for closure

| Issues | Implemented change and targeted checks |
| --- | --- |
| #12 | Columns detail includes document/revision/selection identity in refresh decisions, in addition to effective paragraph state. Native readonly section-only refresh regression. |
| #18 | 44 logical-pixel circles, distributed row, native accent lookup and grouped accessible toggles. Native screenshots/theme regression. |
| #19 | Flat symbolic zoom row in the first menu section; fit/reset retain the open menu. Native visibility/state regression. |
| #22 | Default name from the UID-matched GECOS full-name field, not login. Empty lookup stays editable; explicit stored values are preserved. Deterministic full-name/missing-data test plus native/Flatpak identity tests. Alpha.16 did not persist an untouched automatic default, so no saved value is guessed to be automatic. |
| #25 | Deferred, identity/selection-checked heading top alignment after sidebar layout, clamped to the valid range. Ordinary typing scroll is unchanged. |
| #26 | Per-entry menus reuse the exact proofing-only action builder; suggestions/dictionary actions capture the issue and reject stale documents. Row body remains navigation. |
| #27 | Shift drag/click keeps the existing anchor; tiny pointer jitter does not replace the range. Real X11 Shift+click regression. |
| #28 | Engine-derived complete-word ranges in either drag direction, plus bounded weak/cancellable viewport-edge scrolling. Headless direction/boundary and actual pointer tests. |
| #29 | Six logical-pixel spacing in the shared adaptive Style/Paragraph/Text group. Existing wide/narrow/HiDPI regression and screenshots. |
| #31 | Thumbnail padding/margins reduce total row spacing by 13 logical pixels without changing image size; preallocation fallback matches. |
| #32 | Trim the engine's trailing whitespace from the selection range itself. Unicode, spaces, tabs and paragraph boundaries; actual double-click copy assertion. |
| #33 | Arbitrary fitting page count, conservative mixed-size slot widths, stable last-row alignment and shared hit geometry. One through six columns and mixed pages tested. |
| #34 | Remove the duplicate family caption; one accessible, selectable preview per family at the current size. |
| #35 | Attach the canvas IM context only while the canvas owns focus. Real keyboard typing, paste, Backspace, pinned/unpinned Font and typed Size checks. |
| #36 | Remove stacked top/header spacing while retaining native button targets and natural large-font sizing. Shared home/back header stays consistent. |
| #37 | Replace the sole dialog GtkStackSwitcher with AdwViewStack/ViewSwitcher supported by libadwaita 1.4; no dependency/minimum bump. Native pending-value/read-only page-switch test. |
| #38 | Remove routine unsaved toast; keep modified indicator, errors and discard confirmation. Existing editing/save/close regression. |
| #39 | Blue/underlined default new links, explicit-color preservation and Ctrl+click destination lookup. GtkUriLauncher for real host/sandbox dispatch; fake HTTPS handler records actual delivery without network access. |

## Kept open

- #1/#2: pinned upstream column wrapping/separator defects; no engine update.
- #5/#6: advanced mouse table resize/select/move/insert handles need their own
  merged-cell and multipage gesture implementation.
- #7: the reduced absent-even-slot case is understood, but independent variants,
  inheritance, undo and persistence are not solved in this batch.
- #9: ordinary spacing works in geometry/DOCX checks; the reported file-specific
  cause is still not isolated.
- #14/#15/#16: coordinated inheritable language/review defaults and the revised
  direct Language selector remain unimplemented; do not flatten overrides.
- #30: headings/manual refresh removed and visible results refresh automatically,
  but the full enabled-review contract still depends on #14–#16. Keep this report
  open rather than claiming the default/settings redesign is complete.

## Validation boundary

Model/adapter tests, GTK compilation, catalogs, Python and documentation checks
are local; full native input is tested in Xvfb CI because local Unix sockets are
unavailable. New pointer tests use real mouse/keyboard actions rather than only
programmatic entry updates. URI dispatch uses a temporary fake browser in isolated
CI sessions; it records the real launcher/portal URL without contacting a website.
The installed Flatpak repeats the interaction test and URI dispatch.

Native/installed screenshots are internal QA only. Keep the existing Orca stage
as regression, without new accessibility scope. No claim of human GNOME OS,
physical-printer, braille or every-document testing. Dependency boundary unchanged.
