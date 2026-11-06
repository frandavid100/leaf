# Alpha.31 candidate: issue pass, validation and remaining work

David requested resolving every currently feasible issue and an **unpublished
candidate to test**. This does not authorize automatic promotion to Releases.
WordCraft remains at 0.14.0, commit
`5cd53f184aab11def13b7fced18727159b9bc96a`; renderer, dependencies and existing
format patches are unchanged.

## Changes being validated (final candidate scope)

| Issue | Result |
| --- | --- |
| #133 | Retain and reconcile proofing findings during rescans. Remap only unchanged text outside the changed span, revalidate against current paragraph proofing and update suggestions for changed paragraphs. Reuse unaffected correction, comment and change cards; preserve scroll/focus. Every retained action verifies document identity and current targets. Async success and failure reject stale generations/revisions. |
| #134 | A native vertical SizeGroup synchronizes the complete sidebar and editor header regions, including the Review filter, Navigation dropdown and document tabs. No screenshot-specific pixel offset. |
| #135 | Pages/Structure uses one full-width native DropDown matching Review. Existing representation controllers keep per-document state and filters; hidden private toggle state is not presented as duplicate controls. Sidebar/tab remapping restores browsing only for an unchanged document revision and editing target. |
| #136 | Not included: the experimental formatting changes were withdrawn after the installed font-paste regression failed while published Alpha30 passed on the same GNOME 49 runtime. Text/Paragraph controls return to the published implementation; the issue remains open in full. |

The focused GTK diagnostic verifies incremental pending results, canonical
offsets after unequal-length Unicode replacement, unaffected card identity,
Undo, header allocation and native dropdown mouse/keyboard input. Previous Alpha.30 checks continue to cover per-document representation,
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

## Verification record

The first candidate run `42212447711` failed the installed font paste regression;
the same interaction passed locally and in native run `42211043826`. The pointer
driver now waits for actual native Entry paste delivery before Backspace instead
of assuming a fixed 250ms clipboard latency. Assertions on document text,
selection and the resulting font remain intact.

Native run `42211043826` subsequently failed an outdated proofing diagnostic that
searched for Review's filter inside the content stack. The filter now belongs to
the complete sidebar header (#134); the diagnostic checks that actual header and
requires the filter to be mapped. This is a test-location update, not a product
workaround. Neither failed run constitutes an approved candidate.

The later proofing demo also still activated the private ListBox after its cards
had moved to unified Review. It now activates the actual visible Review target.
The font interaction keeps its current Font page while pinning, instead of
artificially removing/reparenting the focused Entry by reopening the same page.
Clipboard preflight is logged and each input stage retains its own screenshot;
neither the font nor document/selection assertions are weakened.

The final CI and candidate identity are recorded after completion. Diagnostic
captures remain internal validation and do not replace the public screenshot gallery.

## Withdrawal of experimental formatting changes

Installed run `42218301357` failed Alpha31 font paste, while its published
Alpha30 control and original driver passed on the same GNOME 49 runtime
(`ALPHA30_FONT_CONTROL_EXIT=0`). Clipboard preflight could read `monospacex`, but
the native Entry did not receive the pasted value. This is a confirmed regression
in the experimental source, not evidence that the clipboard test may be ignored.
The final scope restores the four formatting modules and their Alpha30 controls
regression to published source `868ac18`; #133–#135 remain. No partial #136
acceptance or formatting improvement is claimed. Full native and installed checks
remain required for the final package.
