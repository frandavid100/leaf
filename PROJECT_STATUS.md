# Current state of Leaf

<!-- candidate-alpha:start -->
## 13/11/2026 — alpha.46 candidate awaiting David's testing

Code review for speed and clarity: side panels refresh after a typing pause,
fields and the table of contents are recomputed in the background, autosave
stops repeating work, diagnostics are gathered in one module and contributor
documentation is in English. No issues are closed. Publication waits for
David's approval of this candidate. [Scope, measurements and limits](docs/ALPHA46_REVIEW.md).
<!-- candidate-alpha:end -->

## 12/11/2026 — risk-free cleanup

First PR of the size, speed and clarity review. The 44 alpha and sweep reports
move from `docs/` to `docs/history/` with updated links; the workflows of
closed investigations (alpha31-locale-check, clipboard217) and three old,
unreferenced CI screenshots are removed and remain in the git history;
`ARCHITECTURE.md` no longer starts with a version note; outdated comments
(egui, saving) are corrected. No changes to behavior, dependencies, engine or
publishing. Validation: formatting, Python tests, catalogs, links and
`cargo test --locked` (247 tests).

## 11/11/2026 — store listing submitted to FlatPark

David asks to resume the FlatPark listing. AppStream uses the four approved
screenshots with their original transparency, corners and shadow. A static
metainfo and icon update is prepared in the fork, with the missing bilingual
release notes for alpha.44/.42/.41/.40. At David's request, the EN/ES description
is renewed following the README: power and simplicity first, alpha and AI last.
The executable does not change and no alpha is published. The earlier automation
was not integrated: it remains reserved for FlatPark. David approves the result
and [PR #1139](https://github.com/flatpark/flatpark/pull/1139) is opened, with no
conflicts. Its CI needs authorization from a maintainer. Publication and checking
in the client are still pending; #185 stays open. [Review and draft](docs/FLATPARK_GALLERY_2026_11.md).

## 11/11/2026 — README and approved screenshots

The public presentation uses the current blue icon and four real screenshots
provided and approved by David: front page, layout, review and the underline
menu. The text summarizes the current features, declares the vibe coding and
distinguishes FlatPark, with updates, from the manual bundles and experimental
Windows. [Provenance](docs/screenshots/README.md). No new alpha is published
and the store gallery is not considered updated; #185 keeps that work.





Usual procedure: a validated RC for testing; after David's approval, promotion
of the same files without recompiling. [Process](docs/CANDIDATES.md).

## Alpha.41: final review and authorized direct publication

On 9/11/2026 David authorizes publishing this delivery automatically when done,
without waiting for another personal test. #233–#237 are incorporated: large
thumbnails in Start, fixed footer, single style, native dropdowns and fixed
spacing/icons. Validation of the new source precedes publication; the previous
RC remains as historical evidence, not as validation of these changes.
[Scope, tests and limits](docs/history/ALPHA41_FINAL.md).

#152 stays open: the reproduction without GTK confirms that WordCraft 0.17
keeps the formatting when deleting text and then the empty paragraph, but
transfers it when deleting the whole paragraph. #1, #52, #99, #171 and #185 also stay open.
The deferrals of public screenshots, Weblate and occasional Windows are kept.

<!-- published-alpha:start -->
## Latest verified release: 0.1.0-alpha.45

[Release](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.45). Closed issues: #259, #262, #278, #280.
Still open: #1, #99, #152, #185, #231, #252, #261, #265, #267, #268, #277, #279, #281, #282.
[Source, validation and limits](docs/LATEST_RELEASE.md).
<!-- published-alpha:end -->

## Published and verified: alpha.38 — start view and recovery

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.38).
#207 and #209 closed. Includes the fix for #208/PR #210: the Navigation/Review
tabs stay fixed and shared by all views.
New windows and tabs offer titled rows for templates and recoveries only when
they contain items, followed by the editable sheet.
Recovery replaces its old menu/dialog; when typing, the rows slide smoothly
upward without deleting pending copies or blocking input.

215 Rust tests, nine Python tests and 891 complete messages in English/Spanish.
Optimized native suite, GNOME 49 installation and GNOME 51 prefix passed;
final comparison of the executable, tag and thirteen public files verified.
WordCraft 0.16.0, permissions, patches and the occasional Windows version are kept.
[Sources, execution, digests and limits](docs/history/ALPHA38_START_VIEW.md).

Reports that stay open: #1, #52, #99, #107, #152, #171, #185.

## Urgent fix after alpha.37 — #208

The bottom Navigation/Review bar belongs again to the fixed container of the
sidebar. The top selectors keep their scrolling; the duplicated mode bars of
Structure and Review are removed. David's clarification corrects the earlier
scope of #200. Merged into main through
[PR #210](https://github.com/frandavid100/leaf/pull/210), with the full native
suite passed. Also distributed and checked as a Flatpak with alpha.38. [Scope and tests](docs/SIDEBAR208.md).

## Published and verified: alpha.37 — scrolling, focus and dialogs

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.37).
#199–#204 closed: gallery and page with shared scrolling, sidebar controls
inside their content, integrated adaptive dialogs, position restoration that
new input can cancel, focus continuity and name suggestions
when saving. Re-layouts of a page with a gallery also recover its image
without moving the browsing position.

214 Rust tests, nine Python tests and 893 complete messages in English/Spanish.
Optimized native suite and GNOME 49 and GNOME 51 installations passed;
the same validated bytes were published, with tag and thirteen public files verified.
WordCraft stays at 0.16.0. The limits of #171, the deferrals and the
store and Windows work in their own conversations are kept.
[Source, tests and limits](docs/history/ALPHA37_INTERFACE.md).

Eight reports remain: the previous seven and the new proposal #207 for template
and recovery rows, filed after this alpha was published.

## Published: first experimental version for Windows

[Windows 0.1.0-alpha.36-windows.1](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.36-windows.1), published on 8/11/2026: per-user installer and x86_64 ZIP with GTK/libadwaita,
dictionaries and translations included. 210 Rust tests on Windows and 211 on Linux;
nine Python tests and 892 complete messages. ZIP moved to a path with spaces/ñ,
en/es interface, DOCX/ODT/PDF, preferences, windows, installation and removal
checked automatically. Complete notices added without changing the executable,
and final packages checked again before publishing; four public assets
with SHA-256 digests and the source tag verified. Occasional Windows deliveries;
no other Linux alpha is started and no issues are closed for this port.
[Sources, execution and limits](docs/WINDOWS.md).


## Published and verified: alpha.36 — interface review

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.36). #177/#181/#182/#186–#191 closed,
including the new #191: literal search and compact options, fonts inside
Properties, split Copy/Paste rows, recovery of the tab space,
stability and menus of Structure, and an exact marker for dragging images.

#171 remains partial and open. The gallery uses real thumbnails over each new
document; the Flatpak uses the XDG templates folder in read-only mode by default.
Choosing another folder is disabled in the sandbox because the portal
grants write access and refuses to withdraw that permission. Host access is not widened.

211 Rust tests, nine Python tests and 892 complete en/es messages; optimized
native suite, GNOME 49 and GNOME 51 prefix correct. Same validated bytes,
without recompiling; tag and thirteen public files verified.
[Source, tests and limits](docs/history/ALPHA36_INTERFACE.md).

Seven remain: #1/#52/#99/#107/#152/#171/#185. David's deferrals and the engine
integration blockers are kept. The new public screenshots
remain David's responsibility; this delivery already fixes #186/#191.

## Store listing and release notes — #185

Prepared: the approved English/Spanish description, the light/dark colors and
a shared source of release notes for AppStream and GitHub. The previous
screenshots are kept with immutable references. It is not a new executable version.
FlatPark PR #1074 awaits authorization/acceptance from its maintainers. The new
screenshots can now be prepared after fixing #186/#191; checking the published
catalog in Bazaar is still pending.
[Process, editorial source and limits](docs/STORE_METADATA.md).

## Published and verified: alpha.35 — accepted proposals

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.35). The thirteen issues #170–#182 are
implemented and closed: next style, personal templates, paste without formatting,
moving sections, format painter, advanced search, document
fonts and external images. [PR #183](https://github.com/frandavid100/leaf/pull/183)
and [lifecycle fix #184](https://github.com/frandavid100/leaf/pull/184).

205 Rust tests (fifteen new regressions), four Python tests and 879 en/es messages;
real controls and gestures at 1/2 scale, GNOME 49 installation and FlatPark
GNOME 51 prefix passed. The same validated bytes were published, without recompiling;
tag, identity and thirteen public files checked.
[Source, execution, scope and limits](docs/history/ALPHA35_WORKFLOWS.md).

A modified next style requires DOCX when the ODT exporter would lose
the choice. The templates folder is read without modifying the originals.
#84 was already checked and closed by David in the testing conversation.

Updated: 8 November 2026. This page describes the present;
the [history](docs/history/README.md) keeps earlier deliveries and decisions.

## Where to work

- Repository: https://github.com/frandavid100/leaf
- Active branch, build and PR target: **`main`**. ApexDesk is kept in
  `archive/apexdesk-main-2026-11-02`; `wordcraft-saving` is still available.
- [Integration of histories and validation](docs/MAIN_MIGRATION.md).
- [How to resume the project](docs/CONTINUING.md), [architecture and file map](docs/ARCHITECTURE.md),
  [current decisions](docs/DECISIONS.md), [work plan](ROADMAP.md).

## Reports predating the new proposals

#1/#99/#107/#152 await the integration of the fixes proposed in
WordCraft. #52 is deferred by David's decision. #84 was already tested and closed
in the validation conversation. The new #170–#182 were initially closed with alpha.35; #171 is kept
open after reviewing the design and permissions. The other reopened ones were verified
with alpha.36.
[Historical audit of the blockers](docs/history/REMAINING_AFTER_ALPHA34.md).

## Historical: alpha.33 published and verified — End/Home and zoom restoration

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.33),
published automatically on 7/11/2026. #156 closed: the editor receives End/Home
before GTK's scroll shortcuts, and the pending zoom respects a later
navigation, even when End does not change the position because it is already at the end of the line.

#9 has measured native controls and a visible explanation of the spacing; its
specific edited copy still needs confirmation. #99 has a synthetic reproduction of a note in
a table below the footer; it needs an engine fix. Both stay open.

189 Rust tests, four Python tests, 811 complete messages and
[full native suite](https://github.com/frandavid100/leaf/actions/runs/42381417100)
correct. [GNOME 49/51 installations](https://github.com/frandavid100/leaf/actions/runs/42382604979),
final comparison of the executable and
[publication without recompiling](https://github.com/frandavid100/leaf/actions/runs/42384540865)
correct. Tag and twelve public files checked; eleven internal checksums verified.
Published source `529e086d7e6aec85c1ed4409af8c7e55aaa6c8b7`.
Eight remain: #1/#2/#9/#52/#84/#99/#107/#152.
[Evidence and limits](docs/history/ALPHA33_VALIDATION.md).

## Historical: alpha.32 published and verified — WordCraft 0.16 and optimization

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.32),
published automatically on 7/11/2026 in accordance with David's authorization.
Engine updated from WordCraft 0.14.0 to the latest stable 0.16.0, with formats
re-copied and residual compatibility documented. ODT keeps notes, comments,
floating images, new basic equations and column widths; the safeguards for
options without demonstrated fidelity are kept. Language data
pinned and licenses included. Native HTML/RTF paste with cancellation of stale
targets; clipboard and derived indexes separated to make review easier.

The measured tests reduce repeated traversals of the index (80→0) and ruler
refreshes (160→80) during 80 cursor movements; medians for that phase ~5–10%
lower. This is not extrapolated to the whole application, nor is lower overall memory claimed.
[Integration, results and limits](docs/WORDCRAFT_016_REVIEW.md).

189 Rust tests, four Python tests, 810 complete English/Spanish messages and
[full native suite](https://github.com/frandavid100/leaf/actions/runs/42371069496)
passed. [GNOME 49/51 installations](https://github.com/frandavid100/leaf/actions/runs/42372618858)
and [promotion without recompiling](https://github.com/frandavid100/leaf/actions/runs/42374275590)
correct. Eleven internal checksums and the twelve public files verified.
Published source `2524dbc789cfbfa0f3b6e153fde16f1df5887b8a`.
The initial update did not close additional reports. The later review
closed #100: note numbers and navigation verified in this same release.
After that initial review nine remained: #1/#2/#9/#52/#84/#99/#107/#152/#156.
[Pending checks and limits](docs/history/ISSUE_TRIAGE_AFTER_ALPHA32.md).

## Historical: alpha.31 published and verified

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.31),
published automatically on 7/11/2026 with David's authorization. In this last
pass #135, #149, #150, #151 and #155 are closed: Navigation selector, review
with the panel hidden and Paragraph, Text and Table settings. It accumulates the earlier
alpha.31 candidates. WordCraft and its dependencies keep the pinned revision.

[Full native suite](https://github.com/frandavid100/leaf/actions/runs/42360443648),
[GNOME 49/51 installations](https://github.com/frandavid100/leaf/actions/runs/42361339472)
and [promotion without recompiling](https://github.com/frandavid100/leaf/actions/runs/42362957503)
correct. 187 Rust tests, four Python tests and 811 complete English/Spanish messages;
the eleven internal checksums and the twelve public files match by SHA-256.
Published source `e3bdb1f318cd078d7e24e0a7fef8c84e3a192756`.

When alpha.31 was published, ten reports remained open: #1, #2, #9, #52, #84, #99, #100, #107, #152 and #156.
Columns #1/#2 are worked on separately in WordCraft; engine/host limits and cases
without a reproduction are not presented as resolved.
[Implementation, identity and limits](docs/history/ALPHA31_REMAINING.md).

## Historical: third alpha.31 candidate — Captions and navigation

#121, #124, #126 and #127 completed: shared Elements properties,
explicit association of images/tables and captions, reversible Show Caption,
navigation with typed targets and temporary decoration of the relationship.

[The verified candidate](https://github.com/frandavid100/leaf/actions/runs/42304211527/artifacts/13547105434)
passes the full native suite (187 Rust tests, four Python tests and GTK
controls), GNOME 49 installation and a relocated FlatPark archive on GNOME 51. Includes
clicks/keyboard, DOCX, history and checks in English/Spanish, light/dark and
1×/2× scale. The original font/paste/canvas regression is kept unchanged.
The eleven checksums of the download have been verified. [Source, checksums and limits](docs/CAPTION_OWNERSHIP.md).

Package source `da5b3f96c672eceb41218855e73ffba0897c126d`; native executable from
`486a8b2c2050f09a6ffd31ce47cacab48e5051f8`. This candidate required approval before being published; the new authorization
for automatic publication of the last pass replaces that requirement.

#132 closed on 7/11/2026 after David's check with an identical copy
of the original document: he confirms it seems fixed in his environment. Neither the original
trigger nor the specific responsible fix has been identified. Creation of numbered captions is limited to the body and DOCX;
imported captions without an explicit association are not assigned by proximity.
After that candidate #135 was reopened and #149–#152 appeared; they were not
part of those four closures. The last pass described above sets their current state.

## Historical: second alpha.31 candidate — Native interface

[Candidate for testing](https://github.com/frandavid100/leaf/actions/runs/42265503961/artifacts/13527444066), without automatic publication.
#134, #135 and #136 closed: aligned headers, native Pages/Structure selector
and shared refinements of Text, Paragraph, Style and Elements. #133 is kept.
The ruler rejects events from floating surfaces, preventing the font
selector from losing focus and diverting the keyboard to the document.

178 Rust tests, four Python tests and 824 complete English/Spanish messages.
[Native CI](https://github.com/frandavid100/leaf/actions/runs/42263872786),
[GNOME 49 / FlatPark GNOME 51 installations](https://github.com/frandavid100/leaf/actions/runs/42265503961)
and [menus actually translated in both environments](https://github.com/frandavid100/leaf/actions/runs/42269593968)
passed. Download, eleven checksums and installed screenshots checked; the original
font/paste regression keeps its sequence and assertions.

Package source `79292491050943559df21d9082657fe38ee46254`; executable from
`179ef4bec1609c355c3dae325c3358fd6bb948a8`, with equivalence and bytes checked.
[Scope, identity, checksums and limits](docs/history/ALPHA31_UI_VALIDATION.md).
At that delivery 13 issues remained: #1, #2, #9, #52, #84, #99, #100, #107, #121, #124, #126, #127 and #132.
At that delivery alpha.30 was still the latest public version and the
candidate had to be tested and approved. The later authorization of automatic
publication replaces that requirement for the last alpha.31 pass.

## First alpha.31 candidate — Historical, replaced by the second

The [first candidate](https://github.com/frandavid100/leaf/actions/runs/42230422027/artifacts/13510771784)
only closed #133 on top of the Alpha.30 interface. It passed 175 Rust tests, four
Python tests and the GNOME 49/51 installations. In that delivery the
experiments for #134–#136 were withdrawn after installed input failed; 16 issues remained.
Its [identity and evidence](docs/history/ALPHA31_VALIDATION.md) are kept as history.
The second candidate incorporates that fix and resolves the three interface
reports after isolating the ruler's event capture.

## Previous published version: alpha.30

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.30).
Published automatically with David's authorization, after passing the
[native CI](https://github.com/frandavid100/leaf/actions/runs/42184270093),
[GNOME 49 and FlatPark/GNOME 51 installations](https://github.com/frandavid100/leaf/actions/runs/42187179877)
and [promotion without recompiling](https://github.com/frandavid100/leaf/actions/runs/42189125319).
Published source `85d0134e413f1cb4587e2cf2de9d9928a1e85a1c`; executable from
`3f898df0f42f280d932eaa2bb3790dd6b45d1190`, with equivalence and bytes checked.
The 12 public files match the candidate by SHA-256.

Seven issues closed in this pass: #122, #123, #125, #128, #129, #130 and #131.
Includes a stable image viewport, cursors, thumbnails by caret, a navigable
index, review cards, Text/Paragraph and Pages/Structure Navigation.
Accumulates the whole alpha.29 candidate. 173 Rust tests, four Python tests and 855 complete
English/Spanish messages. [Evidence, checksums and limits](docs/history/ALPHA30_VALIDATION.md).

When alpha.30 was finished 12 remained open: #121/#124 partial, #126/#127 pending canonical caption
association, and the eight old ones #1, #2, #9, #52, #84, #99, #100 and #107. The
engine is not updated and pending requirements are not hidden through partial closures.

## Alpha.29 candidate: incorporated into alpha.30

[PR #110](https://github.com/frandavid100/leaf/pull/110) integrates the pass
#99–#109 and #111–#121, plus the basic equation editor (#56).
[Full native validation](https://github.com/frandavid100/leaf/actions/runs/42142299796) and
[packaging and GNOME 49/FlatPark 51 installations](https://github.com/frandavid100/leaf/actions/runs/42143216380) passed.
Candidate source `361a25c286a1a06f6c007b9abdf23d7d1d36b8e3`; executable source
`25ae0ba569bc374e84ad78d0a314fe7b9c611b5b`, with equivalence checked.
171 Rust tests, four Python tests and 837 complete English/Spanish messages.

19 issues resolved: #56, #101–#106, #108–#109 and #111–#120. Review brings together
corrections, suggested changes and comments; Text/Paragraph use rows and
Elements shares the object tools. Includes rulers that snap to
0.5 cm, automatic references, notes, and improved images and thumbnails.
Cases #99, #100, #107 and #121 keep pending work or limitations;
#1, #2, #9, #52 and #84 stay open. [Evidence and limits](docs/history/ALPHA29_VALIDATION.md).

The alpha.29 candidate was not published separately: its changes are distributed in
alpha.30, authorized and verified. The manifest and SHA256SUMS identify the
files. [Candidate process](docs/CANDIDATES.md).

## Previous delivery: alpha.28

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.28).
[Native CI, GNOME 49 Flatpak and GNOME 51 FlatPark archive](https://github.com/frandavid100/leaf/actions/runs/42057994674)
complete. Source `d2ba113d9ed2bc0dbb4794732c6035a195cba755`.

Includes per-section zoom and navigation/thumbnails (#89, #94, #95), floating
images with wrapping, spacing and mouse gestures (#53), footnotes (#55),
automatic index (#54), captions and cross-references. [PR #98](https://github.com/frandavid100/leaf/pull/98)
integrates native controls through WordCraft's public APIs. Layout invalidation
when changing an image's wrapping was also fixed.

151 Rust tests, four Python tests and 766 complete English/Spanish messages.
Verified: history, interface, languages, HiDPI, update from alpha.27,
file and print portals, reinstallation, AT-SPI text and Orca output.
These are automatic checks; they do not replace human tests on devices.

Editing/saving/reopening and PDF were checked with external samples from
Word and Writer, including a note imported from Word. [Evidence and limits](docs/history/ALPHA28_VALIDATION.md).
These advanced structures require DOCX: the ODT safeguards still prevent
losses. Engine, renderer, dependencies and patches were not modified.

The [public gallery](docs/screenshots/README.md) uses four real screenshots of the
installed Flatpak, expressly authorized by David. Leaf's AppStream listing
is updated; FlatPark keeps another copy that its maintainers must
update to incorporate the new gallery and description.

## Previous delivery: alpha.27

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.27).
[Full native CI](https://github.com/frandavid100/leaf/actions/runs/41995941521)
and [packaging and GNOME 49/51 installations](https://github.com/frandavid100/leaf/actions/runs/42000544951)
passed. Release source `1259aa95fbde9650fe17d5a9476cb4d1e73685cc`;
the executable reuses the validated build from `74865717f99d63bab8000f1470db7b42718ba761`,
with source equivalence checked by CI.

Includes the tarball requested by FlatPark, with `bin/` and `share/` at the root,
its checksum and `flatpark-smoke.log`. The test installs the archive through
extra-data in `/app/extra/leaf` with GNOME 51 and verifies English/Spanish translations
and spell checking. The usual bundle uses GNOME 49 and verifies updating
from alpha.26, interaction, saving through the portal, printing and Orca/AT-SPI.
The first tarball test failed because Flatpak omitted the reserved directory
`extra` when exporting the test package. The corrected checker uses a local HTTP
download with checksum and `apply_extra` during installation.

FlatPark has accepted the inclusion; its publication in the catalog is pending
packaging by its maintainers. Notify them of changes to desktop/icons/
metainfo and future runtime requirements. The tests above are automatic.

## Previous delivery: alpha.25

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.25).
[Native and installed CI](https://github.com/frandavid100/leaf/actions/runs/41649488931)
passed. Includes collapsible comments, style previews and changes
to thumbnails and rulers. Integration with the system language settings
(#84) and the specific checks for scrolling and dragging
margins (#83, #87) are still pending.

[Alpha.24](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.24)
added section properties, translucent rulers and dictionary import.

## Compatibility validation for the MVP

[Audit with LibreOffice Writer](https://github.com/frandavid100/leaf/actions/runs/41630136413).
The DOCX sheet created in Writer keeps Leaf's edit, the table cells
and the 24-point spacing when reopened. The exported PDF keeps the edited
text and the table content. The CI report records differences
in the links between sections and in excluding text from spell
checking after saving from Writer. The original spacing case (#9)
is still awaiting reproduction.

## Previous delivery: alpha.23

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.23), source `9cda589709aff7d0c21446c9b9b105b0d0750acc`. [Native and installed CI](https://github.com/frandavid100/leaf/actions/runs/41606977263) passed and download checksums verified. #64, #65, #7 and #37 closed.

## Previous delivery: alpha.22 — formatting, comments and printing

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.22).
Source: `1cda6b2df2eeabd46dbc3331c5adb64c452b679f`. GNOME 49 and update
from alpha.21. [Native and installed CI](https://github.com/frandavid100/leaf/actions/runs/41597951947)
passed. The eight public files, the checksums and the package source verified.

Includes WordCraft 0.14.0, its specific API for printing, the optimization of the
Styles panel and the client cleanup. Seven closures: #66–#68, #70, #71,
#73 and #74. #69 stays open for the dictionary installation link.

## Review of formatting, comments and insertion

[PR #72](https://github.com/frandavid100/leaf/pull/72) compacts Font,
Size, Text, Paragraph and Styles (#66–#71), adds an editor for comments and
replies in the sidebar (#73) and shares the Insert submenu between
Document and the context menu (#74). The [review report](docs/FORMAT_POPOVER_REVIEW.md)
records the comparison of prototypes and the pending reproductions of
columns (#1, #2), breaks (#65) and dictionaries (#69).

## Additional review: integration and code clarity

David authorizes reviewing the use made of WordCraft 0.14 and cleaning up the client's structure,
formatting and redundant work. The PDF API is integrated for printing,
controllers and dialogs are separated and traversing hidden styles is avoided.
[PR #63](https://github.com/frandavid100/leaf/pull/63) is merged after passing the
[native CI](https://github.com/frandavid100/leaf/actions/runs/41508412922). [Research and scope](docs/WORDCRAFT_014_REVIEW.md).

## Previous delivery: alpha.21 — WordCraft 0.14.0

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.21), published and verified on 2 November 2026.
Source/tag: `3849d2973670ba05093f178108bab182778579f1`. GNOME 49; update from alpha.20.
Independent download of the eight public assets, checksums and source checked.
Bundle: 11028040 bytes; SHA256 `6b332768aeb50d26c8db44f769a5e014fa71fb546fffc413591bcf2ad2290f40`.

Engine updated to WordCraft **0.14.0**, commit
`5cd53f184aab11def13b7fced18727159b9bc96a`, with a new copy of formats and the
four ODT patches reapplied. Usual optimized profile kept.

**Four closures:** #18, #48, #51 and #61. Theme selector reviewed in the eight
earlier installed variants; thin white rulers per sheet and only transient;
Undo/Redo with the same AdwSplitButton as Paragraph; Font/Size with
bounded widths and Size on a full row. Hidden rulers let clicks through.

141 Rust tests, four Python tests, 678 gettext messages, formatting and links correct.
[Full native and installed CI](https://github.com/frandavid100/leaf/actions/runs/41496070439) passed, including interaction,
HiDPI, update/reinstallation, portals, printing and existing Orca regression tests.
Final installed screenshots reviewed in light/dark and narrow windows at 1×/2×.
[Result and limits](docs/history/ISSUE_SWEEP_ALPHA21.md) ·
[evidence](docs/history/ISSUE_SWEEP_ALPHA21_VALIDATION.json).

Status: **50 closed and 10 open**: #1, #2, #7, #9, #37 and #52–#56.
The synthetic audit of the updated engine keeps automatic index, notes,
equations and floating images in DOCX; ODT still loses those structures and
the safeguards are kept. Their complete interfaces are not implemented.
No human validation of Wayland, IME, devices or arbitrary documents is claimed.
Weblate PR #57 stays separate; the archive, reference branch and earlier
tags are kept.

## Previous delivery: alpha.20 (historical state)

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.20),
published on 2 November 2026. Source/tag: `e5d65dde0c788ef563b5263dbcf3e4e823df92ba`.
GNOME 49; update from alpha.19. Eight public assets and digests recorded
by GitHub. Bundle: 11,059,320 bytes; SHA256
`0e178093f421f08be493086198943c042c4e719d431506a8381a41de89e90e5b`.

**Six reports closed:** #49–#51 and #58–#60. Fading cursor, Repeat,
Undo/Redo histories, Navigation/Comments icons and pages tooltip.
The ruler (#48) and the GNOME selector (#18) are implemented and pass the automatic
tests; they stay open to complete the installed visual comparison.
Leaf also blocks saving to ODT notes, equations, automatic index and floating
images that the exporter would lose; their complete interfaces (#53–#56) are still pending.

141 Rust tests, four Python tests, 678 gettext messages and optimized build correct.
[Native CI](https://github.com/frandavid100/leaf/actions/runs/41466739518) and
[installed Flatpak](https://github.com/frandavid100/leaf/actions/runs/41471767320)
complete, with source equivalence checked before reusing the executable.
Includes gestures, update/reinstallation, portals, printing and the existing Orca
regression; the eight theme screenshots also verify the actual change by keyboard.

When alpha.20 was closed, the execution environment disconnected. Still pending were
the complete visual review of its installed screenshots and the independent
download of its assets; publication, source/tag and digests are confirmed
by GitHub. No human validation of Wayland, IME or devices is claimed.
[Scope and pending items](docs/history/ISSUE_SWEEP_ALPHA20.md) ·
[evidence](docs/history/ISSUE_SWEEP_ALPHA20_VALIDATION.json).

Historical state of that delivery: **47 closed and 12 open**. Releases, tags, the
ApexDesk archive, wordcraft-saving and the separate Weblate PR are preserved.

## Previous delivery: alpha.19

[Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.19), published and verified on 2 November
2026. Source/tag: `1f8694dd8cf51c8f981baf8aa23093ea4e09b625`. GNOME 49 runtime;
update from alpha.18, eight public assets and checksums verified.
Bundle: 10,984,832 bytes; SHA256
`71656fe9c658fff1f24804f2ee7c2ed9d9086de72919b66053594efa9ffdf39c`.

**Nine reports closed:** #25, #26, #28, #34, #35 and #44–#47. Readable font
lists and stable typing, links with Ctrl+K/context/visible URL, dragging
by paragraphs, heading context and review/bookmark settings.
128 Rust tests, four Python tests, 588 gettext messages and optimized
build passed. The [native CI](https://github.com/frandavid100/leaf/actions/runs/41439845294) and the [installed validation](https://github.com/frandavid100/leaf/actions/runs/41442198035) have also passed, including update,
portals, printing and automated Orca. [Scope](docs/history/ISSUE_SWEEP_ALPHA19.md) ·
[evidence](docs/history/ISSUE_SWEEP_ALPHA19_VALIDATION.json).

#35 was actually reproduced on GNOME 49 (`18,75` ended up as `75`) before fixing
the numeric filter and the focus of the delegates during repagination. Both
fields were verified with a real keyboard; this is not equivalent to a human Wayland/IME test.
State when finished: 41 closed and 18 open. The new features,
the engine limits and the later reports keep their own scope.

## Previous delivery: alpha.18

[Alpha.18, Flatpak x86_64](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.18),
published and verified on 2 November 2026. Source and tag:
`a8207cbdb5b11f050373287a44aecd835aefd50c`.
ID `io.github.frandavid100.leaf`, GNOME 49 runtime; update from alpha.17.
Bundle: 10,956,624 bytes; SHA256
`2c3aeff8a530f7ba644825a099922f53c30a93b743487e79146fde9974c783e5`.
Public download of the eight assets, checksums and source checked.
There is no automatic feed and no publication on Flathub.

**Ten reports closed:** #5, #6, #14–#16, #30 and #40–#43. Includes table mouse
gestures, inherited language/review, compact Text, exact GNOME icons,
native selection and active heading according to the caret. [Scope](docs/history/ISSUE_SWEEP_ALPHA18.md)
and [evidence](docs/history/ISSUE_SWEEP_ALPHA18_VALIDATION.json).

123 Rust tests, four Python tests, 587 gettext messages and the full native graphical
round passed. Installed Flatpak: update/reinstallation, portals,
text input, printing, links and automated Orca/AT-SPI regression.
[Full CI](https://github.com/frandavid100/leaf/actions/runs/41402369921).
This is not equivalent to human tests of all documents or devices.

DOCX keeps the inheritance and the review exceptions. ODT flattens language
inheritance; Leaf requires DOCX for disabled default review or explicit
exceptions with review enabled that ODT would lose. WordCraft was not altered.

**Historical state after alpha.18: 32 closed and 20 open, out of 52 reports.** Remaining were
#1, #2, #7 and #9 from the initial pass. During the work #44–#52 appeared and
#18, #25, #26, #28, #34, #35 and #37 were reopened. Their new requirements remained
pending from that delivery; alpha.18 did not consider them resolved. The fourth
review and its updated result are described above.

The [earlier alpha.17](docs/history/ISSUE_SWEEP_ALPHA17.md) and its results are kept
as history. The Paragraph arrow test was simplified and now sends movement
and click consistently, without adding capture controllers to the tested case.

## Completed cumulative delivery

**Navigation** integrates bookmarks under their heading, with section folding;
those before the first heading appear at the beginning. The third tab is left
only for **Comments**: each thread shares a bubble, with separators between
messages and guides for replies. All actions and history are kept.
[Details](docs/BOOKMARKS_COMMENTS.md).

**Spelling and Grammar** keeps the side list, without initial instructions
or a corrections block. Clicking a row only selects and shows the text.
The entry in the document menu is removed; F7 and the side selector open it.
Corrections remain in the text's context menu; the new pass changes
review to per-document inheritance, with preferences only for new documents. [Details](docs/PROOFING_SIDEBAR.md).

**Properties → Information** allows editing title, subject, author, keywords
and category. Apply confirms metadata, page and headers in a single undo
step; Cancel discards the form. The document author does not change the
identity of new comments. [Usage](docs/USER_GUIDE_ES.md).

DOCX keeps the five fields and the threads. The pinned ODT exporter **omits
comments and category, and converts bookmark ranges into positions**.
Leaf prevents losing comments/category when saving ODT, proposes DOCX and keeps
recovery if direct autosave is incompatible. The other four fields
are kept in ODT. Engine, dependencies, vendor and patches remain intact.
The upstream reproduction is documented; no issue has been opened from here.

The earlier organization and continuity review is finished and validated:
[review](docs/MAINTAINABILITY_REVIEW.md), [evidence](docs/MAINTAINABILITY_VALIDATION.json).

## What is implemented

| Area | Status and details |
| --- | --- |
| Documents | DOCX/ODT, editing, formatting, styles, tables, inline images, undo/redo and saving. [Guide](docs/USER_GUIDE_ES.md) |
| Page | Columns, properties, headers/footers, page fields and per-section options. [Headers](docs/HEADERS_FOOTERS.md), [options and language](docs/PAGE_OPTIONS_PROOFING.md) |
| Review and annotations | Side list of errors, check-as-you-type in Preferences, bookmarks and comment threads. [Review](docs/PROOFING_SIDEBAR.md), [annotations](docs/BOOKMARKS_COMMENTS.md) |
| Interface | Overlaid ruler, histories and format repetition; GTK4/libadwaita, tabs/windows, search/replace, thumbnails/headings, native context menu and adaptive controls. |
| Output and recovery | PDF, printing through the portal and autosave/recovery preferences. [Printing](docs/KEYBOARD_PRINTING.md) |
| Languages and appearance | 678 gettext messages, English/Spanish and dark pages. [Translations](docs/TRANSLATING.md) |
| Accessibility | Plain text, cursor and selection through GtkAccessibleText; real automated AT-SPI/Orca test. [Scope](docs/ACCESSIBLE_DOCUMENT_TEXT.md) |
| Performance | Bounded thumbnails, invalidation of affected layers and closed search without redundant work. [Measurements](docs/HOST_PERFORMANCE_AUDIT.md) |

## Limits that remain open

- Fidelity of complex documents; [ODT](docs/ODT_PRESERVATION_2026-10-31.md)
  and [limits observed with Writer](docs/INTEROPERABILITY_AUDIT.md).
- Columns: David reports failures with unequal widths and separators when applying
  columns to selected text; he also observes them in WordCraft. [Human report
  and test scope](docs/TEST_RESULTS.md). Awaiting a minimal reproduction;
  keeping DOCX properties does not prove a correct visual layout.
- Floating images, table handles, formulas and other advanced features
  not yet implemented. They are not considered tasks authorized by this list.
- Complete accessible semantics for tables/images/fields and advanced AT
  operations; extensions and human validation deferred by the user.
- Human confirmation of Wayland/fractional scaling/multi-monitor,
  a physical printer and long sessions, when users are available.

## Who owns what is pending

David has provided the main README screenshot, incorporated with its
[provenance](docs/screenshots/README.md). The public presentation and the installation
guide have been simplified; the release notes are still linked in
[docs](docs/RELEASE_NOTES.md). Weblate registration is still pending on David's side.
Do not replace public screenshots with CI ones or repeat the access attempt
in the remote browser.
Translations through PRs are already possible. Preparing for Flathub will require
reviewing its current requirements and human participation; it is not started now.

Check the current issues on GitHub; do not infer their status from this summary.
The [roadmap](ROADMAP.md) separates proposals and deferred work: it does not add
features automatically or block development waiting for testers.
