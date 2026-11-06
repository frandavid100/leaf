# Alpha.31 candidate: issue pass, validation and remaining work

David requested resolving every currently feasible issue and an **unpublished
candidate to test**. This does not authorize automatic promotion to Releases.
WordCraft remains at 0.14.0, commit
`5cd53f184aab11def13b7fced18727159b9bc96a`; renderer, dependencies and existing
format patches are unchanged.

## Changes being validated

| Issue | Result |
| --- | --- |
| #133 | Retain and reconcile proofing findings during rescans. Remap only unchanged text outside the changed span, revalidate against current paragraph proofing and update suggestions for changed paragraphs. Reuse unaffected correction, comment and change cards; preserve scroll/focus. Every retained action verifies document identity and current targets. Async success and failure reject stale generations/revisions. |
| #134 | A native vertical SizeGroup synchronizes the complete sidebar and editor header regions, including the Review filter, Navigation dropdown and document tabs. No screenshot-specific pixel offset. |
| #135 | Pages/Structure uses one full-width native DropDown matching Review. Existing representation controllers keep per-document state and filters; hidden private toggle state is not presented as duplicate controls. Sidebar/tab remapping restores browsing only for an unchanged document revision and editing target. |
| #136 | Partial: actual AdwSplitButton component/presentation shared with history, independent options/reapply targets, specific Text/Paragraph titles and corrected Back ancestry, Size icon/label within one field, searchable Language choices and selected-state indication. The full cross-menu request is not complete and remains open. |

The focused GTK diagnostic verifies incremental pending results, canonical
offsets after unequal-length Unicode replacement, unaffected card identity,
Undo, header allocation, native dropdown mouse/keyboard input and Text Back
paths. Previous Alpha.30 checks continue to cover per-document representation,
scroll/filter/expansion, navigation and property reapply behavior. Native
SplitButton activation uses its public MenuButton create-popup hook for the
independent repeat action; it does not manufacture a fake menu or perform Redo.

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

#136 still requires the direct searchable Font page, embedded native color
choosers throughout applicable menus, and completion of the Style/Elements
cross-menu audit and short-window/color interaction cases. GTK's public
ColorChooserWidget remains available but deprecated since 4.10; the replacement
ColorDialog is modal rather than embeddable. The pinned document Highlight type
also represents Word's fixed palette, not arbitrary RGB. These are investigation
constraints, not a reason to claim the requested chooser has been delivered or
to change the engine without authorization.

#121/#124 retain their caption dependency. #126/#127 require canonical persistent
object/caption ownership; adjacency is not a safe replacement.
#1/#2 retain measured engine layout limitations; #107 retains rollback for the
manual-page-break anchor case. #9/#52/#99/#100 still need the original affected
context/provider or document to isolate their unresolved symptoms. #84 requires
specific native/Flatpak platform integration and end-to-end dictionary access.
No partial issue is closed by this candidate.

## Verification record

Local Rust, gettext, documentation and real-widget results and the final CI and
candidate identity are recorded after completion. Diagnostic captures remain
internal validation and do not replace the public screenshot gallery.
