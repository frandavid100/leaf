# Alpha41 candidate: scope and evidence

Status: automated candidate validation passed; #152 has been reopened after
David's RC trial. Deleting a whole preceding paragraph still transfers its format;
deleting text first and then the empty paragraph works. See
[the updated issue and screenshots](https://github.com/frandavid100/leaf/issues/152).
The latest public release remains alpha.40.

The authorized WordCraft 0.17 base is
`d9f5e5353d4acf38f6b551714307ac9bcb356fe1`, pinned to upstream
`5574b32600aa7d782c64f5a6c6bb24696152220d`. Language data retains upstream's
unchanged pin. The formats snapshot was recopied and the residual ODT preservation
delta reapplied; see [compatibility patch](../patches/README.md). Other engine,
layout, document and renderer crates are unmodified upstream dependencies.


## Verified download and identity

[Download the candidate ZIP](https://github.com/frandavid100/leaf/actions/runs/42659019759/artifacts/13694293397).
The installable `Leaf-0.1.0-alpha.41-x86_64.flatpak` is at its root. The same
archive contains release files, checksums and private automated QA evidence.
This immutable artifact remains available to reproduce the finding. Corrections
require a new validated RC for David to test before publication.

| Item | Verified value |
| --- | --- |
| Candidate source | `d6adbe5def2266640f84c1219cf1f95ad5d8a2d7` |
| Native executable source | `2d9b1fda80c3f7c5ed2ce7ea545daecda95f69fb` |
| Completed packaging/runtime run | [42659019759](https://github.com/frandavid100/leaf/actions/runs/42659019759), success |
| Candidate artifact | `13694293397` |
| Candidate ZIP SHA-256 | `575435c060838df27b7934f77f48172bfa3914f0d64a4e01e8bef80c7c300605` |
| Flatpak SHA-256 | `88f62c69befca2ccdb48a5e81c7f5db17082dfc4d69cf346f7166efb8f12eee9` |

## Changes

| Issue | Behavior and focused verification |
| --- | --- |
| #107 | The existing public picture adapter accepts the manual-page-break target. Root regression verifies page, one undo step, Undo/Redo and DOCX reopening. |
| #152 | The previously empty paragraph case passed Delete/Backspace, Undo/Redo and DOCX/ODT tests. Reopened after the RC trial: deleting an entire non-empty heading paragraph selected by triple click still changes the following list paragraph's format; the two-step text-only deletion control works. |
| #217 | Normalize the predominant contrasting neutral pair in Chrome wrappers before public HTML paste. Real Chrome/Firefox light/dark clipboard captures are preserved in test fixtures. Local red, highlights, bold, italic and links remain. The test checks one paste undo step, redo, DOCX/ODT and both screen-renderer themes. |
| #226 | The style launcher keeps a fixed text allowance and ellipsizes long names; adjacent style actions have separate targets. Native acceptance moves between Normal, Heading1 and a long custom name and measures the neighboring controls. |
| #227 | Shared panels derive their pointing rectangle and root page from the direct launcher; nested overflow navigation retains Back. Visible controls are omitted from overflow; List is one control. |
| #228 | Table, Image and Contents keep their insert action outside the scrollable options. Borrowed element bodies retain their own scroll/footer structure. |
| #229 | Pure toggles retain off/on/mixed state. Pickers use one click target. Underline and strikethrough keep separate options targets. Reapply-color commands live inside their picker. |
| #230 | Three native preference lists persist order and visibility and immediately refresh open windows. Dragging and keyboard actions reorder within a group. Narrow layouts do not overwrite saved choices; all-off groups remain reachable through overflow. |
| #231 | Independent recovery inventories exclude live locked snapshots. Normal save/discard remove their copies; interruption retains unsaved work. A superseded asynchronous scan is ignored before it can show stale recovery/discard rows. |
| #232 | Start-only previews use a 30×42 logical-unit envelope and 2 px corners; template names can wrap on two lines. Start uses the allocated sidebar width and a wrapping folder action. Navigation previews are unchanged. |

Replacing a picture now retains its width and follows the replacement's aspect
ratio, as explicitly specified by WordCraft 0.17. The former stretched-height
expectation is updated to that public command behavior, not worked around in Leaf.

## Validation and limits

Source `2d9b1fda80c3f7c5ed2ce7ea545daecda95f69fb` has passed 226 Rust tests,
15 Python checks, gettext generation/check (906 complete English/Spanish messages),
project links and the default optimized GUI build in
[run 42652504898](https://github.com/frandavid100/leaf/actions/runs/42652504898).
The local toolbar acceptance also passed real pointer dragging, keyboard ordering,
immediate switches, a cold restart, direct/overflow roots, fixed insertion footers,
constant style widths and the mapped two-line Start row/divider assertions in
English and Spanish at 200% scale. General remains the default Preferences page.
All five native groups (editing, text, documents, lifecycle and appearance) and
their aggregate gate passed for that source. The source run then failed in a
packaging artifact download, not in native validation. Packaging-only follow-ups
explicitly require those seven successful native jobs, source equivalence and a
byte-for-byte comparison against the stripped validated executable. The installed
runner uses separate profiles for legacy overflow probes and default-toolbar
acceptance so it tests both supported configurations independently.
The complete GNOME 49 and relocated GNOME 51 jobs passed. The final candidate
checksums and executable comparison passed before closing the ten issues. The normal distribution build preserves the release
profile and panic unwinding. A local diagnostic executable may override host optimization and cross-crate
LTO; only the default release-profile CI bytes are eligible for distribution.

The relocated GNOME 51 job passed in
[packaging run 42659019759](https://github.com/frandavid100/leaf/actions/runs/42659019759)
using candidate source `d6adbe5def2266640f84c1219cf1f95ad5d8a2d7` and the native
source above. Both English/100% and Spanish/200% passed the real toolbar drag,
cold preference reload and Start assertions. The complete GNOME 49 job, print portal handoff, AT-SPI/Orca automation, final
candidate checksums and exact executable comparison also passed. These are
automated checks, not a claim of human desktop, physical printer or braille testing. Clipboard-based entry checks now wait for
new ownership replies instead of assuming a fixed 100 ms response time.

Browser capture provenance is recorded in the fixture metadata and
`tools/capture_clipboard217.py`. The actual public alpha.38 release was copied with
Chrome 157.0.8103.38 and Firefox 142.0.1. Office365 evidence is the reporter's
attached light/dark screenshots, not a claim of an independent signed-in Office
session. Clipboard HTML does not reveal which computed neutral colors were
intentionally authored, so the normalization policy is deliberately limited;
this is not a general proof of source-color intent.

GNOME references: [native split controls](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.SplitButton.html),
[native drag gestures](https://docs.gtk.org/gtk4/class.GestureDrag.html),
and [popover pointing rectangles](https://docs.gtk.org/gtk4/method.Popover.set_pointing_to.html).
Public store screenshots remain David's responsibility. No Flathub, Weblate,
new Orca scope, or occasional Windows release is started by this candidate.
