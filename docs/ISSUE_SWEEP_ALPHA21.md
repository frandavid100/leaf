# Sixth issue sweep and WordCraft 0.14.0

Candidate alpha.21, 2 November 2026. Publication and installed visual validation
are pending; the last published release remains alpha.20 until verified.

## Engine update

The latest stable upstream release is v0.14.0, commit
`5cd53f184aab11def13b7fced18727159b9bc96a`. Engine/doc/render and the standalone
formats manifest use this exact commit. The formats crate was recopied before
reapplying the four compatibility patches, in their recorded order, without
conflicts. They remain needed for ODT preservation. Rust minimum 1.95 and the
craft-libs PDF backend are unchanged. Cargo.lock changes only WordCraft packages
from 0.12.0 to 0.14.0; the formats crate now uses upstream's metafile dependency.

141 Rust tests pass, including headers, fields, standard ODF geometry, languages,
saving/recovery, history, protection, tables and DOCX/ODT preservation. Four Python
tests, current documentation links and 678 English/Spanish messages also pass.

## Current issue changes

- #51: native AdwSplitButton, matching Paragraph's flat style and native arrow;
  centered 18px history glyphs, independent body/dropdown sensitivity and normal
  spacing. Existing history models, cumulative navigation and stale-target guards
  remain intact.
- #61: Size expands to the same row width as Font. Both choice popups derive their
  width from the actual selector allocation and measured native popup chrome.
  A mapped-only, explicitly removed callback follows adaptive resizing. Preview
  text cannot expand the popup; long names retain their complete tooltip.
- #48: separate 24px white horizontal/vertical overlays, measured from the real
  sheet, clipped at the viewport and following zoom/scroll. Horizontal has an 8px
  titlebar gap. Separate edge hover fades each axis in 175ms, honoring reduced
  motion. No Pin or document-menu entry. Horizontal indents/tabs retain public
  engine commands and one-undo gestures; Ctrl+Alt+R provides transient keyboard
  focus and Escape returns to the document. Vertical displays page coordinates;
  no unsupported vertical mutation is exposed.
- #18: closed after completing the pending visual review of the actual alpha.20
  installed capture artifact 13156181480: all eight theme/scale variants, selected,
  hover and keyboard focus/choice states. Its actual GNOME Text Editor grouped
  checkbuttons and attributed stylesheet are retained. Existing real keyboard /
  persistence checks passed; this is not a new human GNOME OS/device test.

## Older issues

#1, #2, #7, #9, #37 and #52 retain their specific scope. Upstream closure alone
is insufficient to close Leaf's actual selected-text column cases. Existing
header inheritance, ODT geometry and paragraph-spacing tests remain required.
AdwInlineViewSwitcher still needs a deliberate native baseline upgrade; the
reported Files previews still need affected-file/provider evidence.

Floating images, TOC, footnotes and equations (#53–#56) still lack complete native
authoring. The existing DOCX preservation tests and ODT loss guards remain in
place. Do not infer that an engine update implements those interfaces.

Weblate PR #57, Flathub, new Orca stages and public screenshot replacement are
outside this sweep. Existing tags/releases and archive/saving branches are kept.

## Validation progress

The initial candidate `f302822c177e1c2c881a2d3e6cd418b752dd8bf3`
builds in the standard optimized release profile in
[PR CI 41483944577](https://github.com/frandavid100/leaf/actions/runs/41483944577).
The independent local toolchain hit a PDF-writer ThinLTO link failure; no engine
patch or project-profile workaround was accepted on that basis. The shipped
profile remains opt-level 3, ThinLTO, codegen-units 1 with unwinding.

The first graphical PR run caught a large-font sizing failure (#61): a preview's
minimum width could force the popup wider than its selector, and changing the
scroller's bounded minimum before its previous maximum triggered a GTK critical.
The fix bounds the inner preview and updates the width range in a valid order.
The checks retain exact selector-width comparisons rather than weakening the
expectation after the failure.
