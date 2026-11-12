# Sixth issue sweep — alpha.21 and WordCraft 0.14.0

Published and verified on 2 November 2026. Development remains on `main`.

- [Release v0.1.0-alpha.21](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.21)
- Exact source/tag: `3849d2973670ba05093f178108bab182778579f1`.
- Source tree: `b845e6c9aff03f73c34a829ac965e140491b049b`.
- [Full native and installed CI](https://github.com/frandavid100/leaf/actions/runs/41496070439).
- [Machine-readable evidence](ISSUE_SWEEP_ALPHA21_VALIDATION.json).

## Result

Four reports closed: #18, #48, #51 and #61. Fifty reports are closed and ten
remain open at the end of this sweep; PRs are excluded from those counts.
The separate translation PR #57 is preserved.

| Report | Final behavior and evidence |
| --- | --- |
| #18, theme selector | Completed the outstanding actual alpha.20 installed visual review: eight light/dark/high-contrast/high-contrast-dark × 1×/2× variants, including selected, hover, focus and keyboard choice. Retained GNOME Text Editor's attributed grouped GtkCheckButton/CSS pattern and its previous real keyboard/persistence checks. |
| #48, rulers | Two separate white 24px overlays aligned to the visible sheet and clipped at viewport edges; horizontal has an 8px header gap. Independent hover, 175ms fade honoring disabled animations, no pin/menu entry. Horizontal public indent/tab commands keep selection targets, one Undo, Escape cancellation and drags released outside the strip. Ctrl+Alt+R gives transient keyboard focus. Vertical displays page coordinates without unsupported edits. Hidden revealers no longer intercept clicks. |
| #51, history controls | Actual AdwSplitButton and Paragraph's flat/image-button style, centered 18px glyphs, native arrows and normal spacing. Independent body/menu sensitivity preserves Repeat with an empty Redo stack; actual engine history, cumulative traversal, shortcuts and stale-target protection are retained. |
| #61, Font/Size | Size expands to Font's row width. Both popup widths follow actual selector allocation and measured native chrome. Preview layout cannot force excessive width; callback only runs while mapped and is removed on unmap. Names retain full tooltips; previews, scrolling, selection, typing and immediate application remain. Large-font/narrow/2× geometry verifies 260/260 logical pixels. |

Installed capture review covers normal/hover/focused/open/disabled/Repeat history
states, Font and Size independently at normal/large sizes, and page rulers in
multi-page views, dark/wide, light/narrow and dark/narrow at 2× with offscreen
clipping. Internal CI evidence does not replace David's public screenshots.

## Engine update

The latest stable official upstream release checked for this sweep is
[WordCraft v0.14.0](https://github.com/storytold/wordcraft/releases/tag/v0.14.0),
commit `5cd53f184aab11def13b7fced18727159b9bc96a` (published 1 November 2026).
Engine/doc/render and the standalone formats manifest use that exact commit.
Formats was recopied from it before reapplying all four recorded ODT patches,
in order, without conflicts. Comparison with a freshly patched upstream copy
confirms all vendored source files match; licenses/provenance remain included.

The isolated engine update is commit
`4707d77882afb6432d2ba521ad551985324d11cb`; frontend changes are separate.
Cargo.lock changes the WordCraft family from 0.12.0 to 0.14.0. Formats now uses
upstream's metafile dependency. Rust minimum 1.95 and the existing craft-libs PDF
backend revision are unchanged. No engine/render fork was introduced for controls.

The synthetic `examples/issue20_capabilities.rs` audit was rerun against 0.14.0:
DOCX retains an updatable 13-page TOC, structured fraction/script/root math, one
footnote story and floating-image wrapping. ODT still loses the TOC field, note
story and equation and flattens floating wrapping. Existing loss guards remain;
this does not provide native advanced authoring or universal format fidelity.

## Validation

141 Rust tests, four Python tests, 678 complete English/Spanish messages, Leaf
formatting and current project links pass. GitHub builds the unchanged standard
optimized profile: opt-level 3, ThinLTO, codegen-units 1, stripping and unwinding.
The final source's full native and installed stages pass, including:

- Existing real pointer/keyboard font, history, ruler, selection, table and link
  gestures; English/Spanish at 1×/2×; current-value/focus behavior.
- Page/header/footer/field/options/proofing and annotation checks; independent
  tab/window ownership and memory regressions; rendering and HiDPI geometry.
- Flatpak upgrade from alpha.20, reinstall/persistent data, launcher, portal open
  and save, print-portal handoff and the existing real AT-SPI/Orca regression.
- Additional installed history/ruler captures in dark/wide, light/narrow and
  dark/narrow 2× views. These use the first seven existing history stages for
  presentation evidence; the complete wide-window gesture checks still run.

Eight public assets were independently downloaded after publication, compared
with SHA256SUMS and GitHub's digests. The bundled source marker matches the tag
and CI source; runtime and installed permissions are recorded in the JSON report.
The native-artifact reuse guard is advanced only after complete validation to
this exact source/run; its source-identity comparison remains intact.

## Regressions caught during this sweep

The first graphical run reproduced a large-font preview forcing the popup wider
than its selector, plus an invalid old-minimum/new-maximum GTK constraint update.
The fix bounds preview layout and updates constraints in valid order; exact
width assertions were retained.

An unconditional annotation boundary still compared against the pre-upgrade
engine snapshot. It now retains that comparison against the isolated authorized
engine-update commit. This corrects the CI boundary without relaxing frontend
engine isolation. All annotation and lifetime checks then pass.

An actual GTK pick probe reproduced hidden GtkRevealers intercepting clicks in
their allocated rectangle outside the activation zone. Both axes disable picking
while hidden; the real picked widget must belong to document/viewport. The final
source supersedes the previous full green integration before publication.

The independent local toolchain encountered a PDF-writer ThinLTO link failure.
No engine patch or shipping-profile workaround was accepted. GitHub's standard
optimized build and installed package pass; the nonshipping local diagnostic
binary is not the released artifact.

## Open work and limits

| Reports | What remains |
| --- | --- |
| #1, #2 | Unequal-width automatic paragraph continuation and selected-text separator geometry still require the actual two/three/custom-column cases. Stored properties and upstream issue closure do not demonstrate visible correctness in Leaf. |
| #7 | Creating absent even header/footer variants still needs a complete public-API solution with page/section inheritance, direct clicks, independent content, Undo/Redo and PDF verification. Existing inheritance tests passing is insufficient. |
| #9 | The ordinary two-paragraph spacing regression works; the original DOCX/contextual/automatic-spacing case remains unisolated. |
| #37 | AdwInlineViewSwitcher needs a deliberate newer native baseline and complete dialog conversion. The existing supported GTK/libadwaita baseline is retained. |
| #52 | GNOME Files blank previews need the affected originals, actual thumbnail provider and cache evidence to establish responsibility. No cache was cleared. |
| #53–#56 | Floating images, TOC, footnotes and structured equations lack complete native insertion/editing interfaces. DOCX preservation and ODT guards do not close them. |

No human GNOME OS/Wayland/IME/fractional-scale/multimonitor, physical printer,
braille or arbitrary-document validation is claimed. Existing automated Orca
coverage is maintained without starting a new accessibility phase. Weblate
activation, Flathub and public screenshot replacement remain outside this sweep.
The 19 existing release/tag objects, ApexDesk archive and wordcraft-saving are
preserved. Neither branch history is discarded.
