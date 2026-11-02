# Alpha.17 issue review

Requested by David on 2026-11-02: close all issues that can be resolved and publish
another alpha. Starting branch `wordcraft-saving`, source
`27bbfcb5930964db2c2e179a679430c9ce6d5c0c`. The 28 currently open reports were read.
Published and verified on 2026-11-02 as [v0.1.0-alpha.17](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.17), source `d1bbc41fdcd79de3b88c974e3f3549f37e58a00a`. The 18 candidates below were closed only after native/installed validation and public download/checksum verification. [Machine-readable evidence](ISSUE_SWEEP_ALPHA17_VALIDATION.json).

## Resolved and closed

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

## Publication recovery

The first publication attempts stopped in the native regression suite; alpha.16
remained the latest published release. Run 41367988401 reached the table keyboard
test, where Tab navigation worked but typing `GNOME` did not insert text. The
focus-scoped IM context had only weak references while detached, so it could be
destroyed before the canvas gained focus. The tab now owns that context and
clears its client widget during teardown. The real-keyboard interaction test
also returns from the Font/Size entries to the canvas and inserts `GNOME`; this
runs both natively and in the installed Flatpak. Full validation and publication
completed successfully in [run 41373872997](https://github.com/frandavid100/leaf/actions/runs/41373872997); the 18 candidate issues are now closed.

Native/installed screenshots are internal QA only. Keep the existing Orca stage
as regression, without new accessibility scope. No claim of human GNOME OS,
physical-printer, braille or every-document testing. Dependency boundary unchanged.

## Final verification and remaining work

117 Rust and four Python tests, catalogs, GUI compilation, native controls,
real pointer/keyboard input, GTK lifecycle, screenshots and 100/200 percent HiDPI
passed. Installed Flatpak upgrade from alpha.16, input, native URI delivery,
file/print portals, reinstall/data retention and AT-SPI/Orca regression passed.
All eight public release assets match their GitHub digests; SHA256SUMS and the
source file were independently checked after downloading the published assets.

Earlier split-button Xvfb attempts failed despite matching coordinates and an
enabled widget. The final probe retains raw event diagnostics and asserts that
the detail page remains open after release. Focus/root-coordinate changes alone
did not establish the cause of those earlier failures. Document gestures now
deny foreign-surface sequences and avoid refreshing on an unrelated drag end;
the paragraph home page records its displayed state to avoid redundant rebuilds.
Do not turn this single automated desktop into a claim about every environment.

The ten preserved reports (#1, #2, #5, #6, #7, #9, #14, #15, #16, #30) stay open.
The later #40–#43 also remain open: the three-row Text redesign, launcher icons,
system selection colors and current-heading highlighting are not in alpha.17.
There are 14 open reports after this release. main, WordCraft, dependencies,
vendor and patches are unchanged by this publication recovery.
