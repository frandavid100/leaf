# Fifth issue sweep — alpha.20

Published [alpha.20](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.20), 2 November 2026.
Source/tag: `e5d65dde0c788ef563b5263dbcf3e4e823df92ba`.
**Six verified closures:** #49–#51 and #58–#60. **Two implementations await final
visual review:** #18 and #48. Final issue count: 47 closed / 12 open.

## Delivered

- #49: black insertion caret on light pages and contrasting white on dark pages.
  WordCraft's inspected 100 ms linear idle fade is combined with native GTK blink
  period/timeout and disabled/reduced-motion settings. Activity restarts it and
  inactive/hidden carets stop repainting; canonical geometry/history are untouched.
- #50: contextual Redo/Repeat body. Real Redo takes priority. Supported text/basic
  paragraph formatting repeats its canonical chosen result at the current target,
  per document, through public commands. IO, typing and unsupported structure are
  excluded. Undo/no-op/protection/redo-branch rules remain engine-owned.
- #51: separate native linked history chevrons, shared document-menu models,
  actual engine undo labels and disposable-snapshot redo inspection. Deeper rows
  traverse cumulatively; only the first has the native accelerator. Document/revision
  guards reject stale targets. Leaf-exposed engine descriptions are localized.
- #58/#59: exact compass-symbolic/comment-symbolic assets from the existing pinned
  CC0 GNOME Icon Development Kit, with native sizing/color and identity preserved.
- #60: localized current/total Pages-tab tooltip using actual layout and the same
  active-page state as thumbnails, with a safe empty fallback and no panel switch.

## Implemented; final visual review remains open

#18 now uses actual grouped GtkCheckButtons and the attributed GNOME Text Editor
stylesheet: 44 logical-pixel previews, native accent/check and diagonal System.
Libpanel is absent from the supported native/locked build and needs a dependency
change outside this frontend pass. [Attribution](../../data/THEME_SELECTOR_SOURCES.md).
Functional/persistence checks and eight native/installed light/dark/high-contrast
1×/2× capture/keyboard-preference cases pass. Native variants and normal installed
rendering were inspected. Complete final installed-image comparison remains pending.

#48 implements a 175 ms non-measuring viewport overlay, hover activation, pin and
native document-menu keyboard entry. Public layout supplies page/column/zoom
coordinates; public indent/tab commands commit one undoable edit. Leases protect
identity/revision/selection; dragging, cancellation and keyboard controls retain
the target. Native/installed English/Spanish/HiDPI pointer checks pass. Local dark
inspection also covered hide, tab removal outside the ruler, Undo, Escape and
keyboard pin/unpin. Final installed visual assessment remains pending.

The execution environment disconnected after publication, before the final
[installed capture artifact](https://github.com/frandavid100/leaf/actions/runs/41471767320/artifacts/13156181480)
could be opened. Capture success is not substituted for a completed visual review;
#18 and #48 remain open for that requirement and any resulting adjustments.
Public asset metadata/server SHA256 digests are verified. Independent public
binary download/readback could not be completed after the disconnection.

## Source and verification

- Full optimized [native CI](https://github.com/frandavid100/leaf/actions/runs/41466739518)
  passed at `f6fc319c05139896964fc3c402d6035e2264b068`: 141 Rust tests, four Python tests,
  format/project links, 678 complete English/Spanish messages, GUI and prior input,
  lifecycle, portal, localization, printing and geometry regressions.
- [Final installed/publication CI](https://github.com/frandavid100/leaf/actions/runs/41471767320)
  passed at `e5d65dde0c788ef563b5263dbcf3e4e823df92ba`. Packaging-only changes add capture validation;
  a guard rejects any difference from the tested native executable across src,
  Cargo files, tests, vendor, PO/data, examples, tools and patches before reuse.
- Installed GNOME 49: upgrade from alpha.19, reinstall/data retention, new and prior
  real pointer/keyboard controls, portals/printing and existing AT-SPI/Orca test.
  All eight theme cases assert actual keyboard preference changes. They isolate
  temporary config/state/data and use libadwaita's own high-contrast debug setting.
- Eight public assets; tag/source verified by GitHub API. Bundle: 11059320 bytes;
  SHA256 `0e178093f421f08be493086198943c042c4e719d431506a8381a41de89e90e5b`.
- All 18 previous release IDs, targets and assets remain unchanged; archive and
  wordcraft-saving refs are retained. Separate Weblate PR #57 is untouched.

No engine, dependency, vendor or compatibility patch changed. No new Orca stages,
public screenshot replacement, Flathub work or mandatory full corpus was added.
Automated success does not establish human Wayland/GNOME OS/IME/device coverage.
[Machine-readable evidence](ISSUE_SWEEP_ALPHA20_VALIDATION.json).

## Saving safeguards and remaining authoring work

`examples/issue20_capabilities.rs` is a reduced synthetic public-command audit:

| Feature | DOCX reopen | Pinned ODT export/reopen |
| --- | --- | --- |
| #56 fraction/script/radical equation | Editable structure retained | No Equation object |
| #55 footnote | Part/reference retained | Structure lost |
| #54 automatic TOC, 13-page control | Field retained and update works | Field lost; static entries |
| #53 floating image | Square wrap/position retained | Becomes inline |

Alpha.20 refuses these lossy ODT saves and offers DOCX. Four tests confirm refusal
before writing existing files, unchanged session/selection/history/path and valid
DOCX round trips. Recovery also selects the preserving format. These safeguards
are not the full native authoring interfaces requested in #53–#56; those stay open.
Existing upstream [#2669](https://github.com/storytold/wordcraft/issues/2669) already
covers ODT model-depth gaps; no duplicate report or engine fork was introduced.

## Other older reports

- #1: upstream [#2382](https://github.com/storytold/wordcraft/issues/2382) remains open;
  retained widths do not establish correct automatic paragraph continuation.
- #2: upstream [#5178](https://github.com/storytold/wordcraft/issues/5178) is closed,
  but Leaf's pin is unchanged and its two-versus-three selected-text case is unverified.
- #7: [missing-even-slot reproduction](ISSUE7_TRIAGE.md) still applies. Direct
  createHeader/createFooter uses own-section allocation and different inheritance;
  it is not a verified general replacement for page-based editing.
- #9: ordinary spacing works; the original DOCX/contextual/automatic case still
  needs its reproducer. Plain paragraph success is insufficient to close it.
- #37: actual AdwInlineViewSwitcher needs a newer native API/binding baseline.
- #52: Leaf has no system thumbnailer and synthetic DOCX controls have no embedded
  thumbnail. The affected files/provider/cache remain needed for attribution;
  no user cache was cleared.

## Failures caught during validation

First candidate `f52a36d5a8889ba9cefe57c5bbeb0117c0409604`,
[CI 41463228240](https://github.com/frandavid100/leaf/actions/runs/41463228240), caught
passive link-tooltip hit testing changing the active header page. The dynamic
Pages tooltip caused GTK to request that query immediately. Passive probe and
committed selection are now separate; the unchanged header UI/pointer checks and
new page/selection/revision/history regression pass. Font/size focus notifications
also avoid overwriting input during focus transitions.

The packaging audit at `20a8961227ee5990b814bf63f8ecea582e79377a`,
[CI 41469643437](https://github.com/frandavid100/leaf/actions/runs/41469643437), passed
functional installed checks but failed a fixed-count theme keyboard driver with
recovery UI present. The follow-up isolates all data and observes the actual
changed preference instead of assuming a tab-stop count. All eight cases pass.
Neither failed candidate published alpha.20.
