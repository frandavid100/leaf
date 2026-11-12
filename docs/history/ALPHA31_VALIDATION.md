# Alpha.31 candidate: issue pass, validation and remaining work

David requested resolving every currently feasible issue and an **unpublished
candidate to test**. This does not authorize automatic promotion to Releases.
WordCraft remains at 0.14.0, commit
`5cd53f184aab11def13b7fced18727159b9bc96a`; renderer, dependencies and existing
format patches are unchanged.

## Final candidate scope

| Issue | Result |
| --- | --- |
| #133 | Retain and reconcile proofing findings during rescans. Remap only unchanged text outside the changed span, revalidate against current paragraph proofing and update suggestions for changed paragraphs. Reuse unaffected correction, comment and change cards; preserve scroll/focus. Every retained action verifies document identity and current targets. Async success and failure reject stale generations/revisions. |
| #134 | Withdrawn from the final candidate. Complete-header alignment remains open until it can pass installed input regressions. |
| #135 | Withdrawn from the final candidate. Published Pages/Structure tabs and their controllers are retained; the requested dropdown remains open. |
| #136 | Not included: the experimental formatting changes were withdrawn after the installed font-paste regression failed while published Alpha30 passed on the same GNOME 49 runtime. Text/Paragraph controls return to the published implementation; the issue remains open in full. |

The focused GTK diagnostic verifies incremental pending results, canonical
offsets after unequal-length Unicode replacement, unaffected card identity,
Undo and rescanning edits made while Review is hidden. Header/dropdown assertions belong only to withdrawn iterations. Previous Alpha.30 checks continue to cover per-document representation,
scroll/filter/expansion, navigation and property reapply behavior.

## #132 investigation

The supplied original was recovered locally and its SHA-256 matched
`85d7e722d989657ce5941ef5410e84cd4acca1891a88cac75ccdc0dbf91216b7`.
The document was not rewritten to hide the reported problem.

On Ubuntu 24.04, the pinned public layout, PNG/PDF export and Leaf's actual
window did not reproduce overlapping lines, both before and after installing
Liberation and Noto Color Emoji. With those fonts, the target paragraph's
baselines were 10.317871 and 24.864136 points within its placed block, a separation
of 14.546265 points. The emoji remained visible. This does not establish that
the maintainer's GNOME OS/runtime/font configuration is fixed. #132 stays open;
there is no confirmed engine defect to report from this measurement.

`tools/make_line_reproducer.py` creates two minimal, synthetic DOCX controls
using the reported paragraph, Arial 11pt, inherited automatic 276/240 line
spacing and the direct Before/After properties. The original private document
is not committed. The regression measures distinct baselines at four widths,
with/without emoji, renders three scales and checks text after DOCX reopening.
`examples/line_geometry.rs` can inspect any local DOCX through public APIs.

## Remaining requirements

#136 remains open in full. Its former partial implementation is preserved in Git
history for investigation, but is not shipped by the final candidate. Native
format controls, titles/Back paths, direct searchable Font/Language lists, embedded
color choosers and the Style/Elements audit require a later verified pass. The
public Alpha30 formatting implementation is retained.

#121/#124 retain their caption dependency. #126/#127 require canonical persistent
object/caption ownership; adjacency is not a safe replacement.
#1/#2 retain measured engine layout limitations; #107 retains rollback for the
manual-page-break anchor case. A fresh public-API reproduction (without Leaf's
rollback) still places the requested page-1 image on page 0 at BodyPath([1]),
offset 1. Existing WordCraft reports [#4262](https://github.com/storytold/wordcraft/issues/4262)
and [#4243](https://github.com/storytold/wordcraft/issues/4243) remain open. The
upstream comment attempt returned HTTP 403, “Resource not accessible by
integration”; the reproduction and access limit were instead recorded on
[Leaf #107](https://github.com/frandavid100/leaf/issues/107#issuecomment-6683816460).
No upstream publication is claimed. #9/#52/#99/#100 still need the original affected
context/provider or document to isolate their unresolved symptoms. #84 requires
specific native/Flatpak platform integration and end-to-end dictionary access.
No partial issue is closed by this candidate.

## Verification history and final source

Experimental candidates failed the mandatory installed font-paste regression
while the published Alpha30 control passed on the same GNOME 49 runtime.
Formatting rollback alone and both header-parenting alternatives still failed.
Runs `42218301357`, `42223264771` and `42226216072` are rejected builds,
not validated candidates. #134–#136 were therefore withdrawn together.

The final source restores `tab.rs`, `state.rs`, `navigation.rs`, `view.rs`
and the four formatting modules to the published Alpha30 implementation.
Review's filter occupies its published content row. Both
`diagnostics/interactions.rs` and `tools/interaction-pointer-test.sh` match
Alpha30 byte for byte; their font, paste, document and selection assertions
remain mandatory. Intermediate clipboard acknowledgements, preflight reads,
key tracing and timing changes are not shipped.

The proofing demo was corrected to activate the actual unified Review target,
rather than its unused former private ListBox. The filter diagnostic checks
its final content-stack location. The candidate PR marker skips only the
optional performance comparison; functional native and installed tests remain
mandatory. Diagnostic captures do not replace the public screenshot gallery.

## Final scope: Review on the published interface

After both generic header wrappers and separate native header rows failed the
same installed input regression, #134 and #135 are withdrawn as well. The final
candidate restores published tab construction, Ui fields, Navigation, status and
Navigation regression, and returns Review's filter to its published content row.
The intended functional change is #133 only; #132 investigation and synthetic
geometry controls are retained. #134–#136 remain open without partial closures.

The 40-finding native XTest check corrected paragraphs 1, 2 and 25. Saving and
reading the DOCX confirmed those exact targets and 37 remaining misspellings.
Three native toolbar Undo operations restored all 40 original misspellings;
Redo restored exactly paragraphs 1, 2 and 25. The filter and visible list position
were retained and the list stayed populated. This is an automated native-input check, not human validation of every desktop
or document.

## Accepted unpublished candidate — 6 November 2026

| Identity | Value |
| --- | --- |
| Version | `0.1.0-alpha.31` |
| Candidate source | `78c9650f8abd8f7adf69ebf57930fda8980d3597` |
| Native source | `2130568e08d618fbce30ffbac9f4aa498d66a7c4` |
| Native validation | [42229072026](https://github.com/frandavid100/leaf/actions/runs/42229072026), success |
| Package and installed validation | [42230422027](https://github.com/frandavid100/leaf/actions/runs/42230422027), success |
| Download | [Complete candidate ZIP](https://github.com/frandavid100/leaf/actions/runs/42230422027/artifacts/13510771784) |
| ZIP SHA-256 | `5e51c69827c19daac34ff05464926e13bd478adbee16c9c3bf69db478c236952` |
| Flatpak size | 11,222,672 bytes |
| Flatpak SHA-256 | `5e727bf688f902b185fe8799e2cfcff45929064b3fb2664c0021079ce068ab41` |
| FlatPark SHA-256 | `3101d946903db6ac5158d6555bcceb7e897b0a22bf9bdc67e7e75f98571a7b24` |

Passed 175 Rust tests, four Python tests, formatting/docs checks and all 855
English/Spanish source messages. The complete native graphical battery and
installed GNOME 49 and relocated FlatPark/GNOME 51 checks succeeded. This
includes the unchanged published font/paste regression, incremental Review,
history, document portals, printing, existing AT-SPI/Orca, lifecycle and geometry
checks. The validated native binary and packaged binary compare byte for byte.

The final ZIP was independently downloaded, its GitHub artifact digest matched,
and every file in `SHA256SUMS` was recomputed successfully. `candidate.json`
matches both source commits and run IDs. Installed Review frames from GNOME 49
at 1× and FlatPark/GNOME 51 at 2× were visually inspected; the populated mixed
Review list, document text and controls render correctly. These diagnostic
fixtures are not public gallery replacements.

**#133 is closed.** Sixteen reports remain open: #1, #2, #9, #52, #84, #99,
#100, #107, #121, #124, #126, #127, #132 and #134–#136. The candidate does not
resolve the withdrawn interface requests or the original-document/font issue.
This is automated validation plus inspection of captured frames; human testing
of the maintainer's affected document and desktop remains useful.

No release or tag was created. Alpha.30 remains public; promotion of these exact
candidate files requires David's approval.
