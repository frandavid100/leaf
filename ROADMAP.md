# Leaf roadmap

<!-- candidate-alpha:start -->
## 13/11/2026 — alpha.46 candidate awaiting David's testing

Code review for speed and clarity: side panels refresh after a typing pause,
fields and the table of contents are recomputed in the background, autosave
stops repeating work, diagnostics are gathered in one module and contributor
documentation is in English. No issues are closed. Publication waits for
David's approval of this candidate. [Scope, measurements and limits](docs/ALPHA46_REVIEW.md).
<!-- candidate-alpha:end -->

## 11/11/2026 — store listing submitted to FlatPark

David asks to resume the FlatPark listing. AppStream uses the four approved
screenshots with their original transparency, corners and shadow. A static
metainfo and icon update is prepared in the fork, with the missing bilingual
release notes for alpha.44/.42/.41/.40. At David's request, the EN/ES description
is rewritten following the README: power and simplicity first, alpha and AI last.
The executable does not change and no alpha is published. The earlier automation
was not merged: it remains reserved for FlatPark. David approves the result and
[PR #1139](https://github.com/flatpark/flatpark/pull/1139) is opened, with no conflicts. Its CI
needs authorization from a maintainer. Publication and verification in the client
are still pending; #185 remains open. [Review and draft](docs/FLATPARK_GALLERY_2026_11.md).

## 11/11/2026 — README and approved screenshots

The public presentation uses the current blue icon and four real screenshots
provided and approved by David: start view, layout, proofing and underline
menu. The text summarizes the current features, discloses the vibe coding and
distinguishes FlatPark, with updates, from the manual bundles and experimental
Windows. [Provenance](docs/screenshots/README.md). No new alpha is published and
the store gallery is not considered updated; #185 keeps that work.





Usual procedure: a validated RC for testing; after David's approval,
promotion of the same files without rebuilding. [Process](docs/CANDIDATES.md).

## Alpha.41: final review and authorized direct publication

On 9/11/2026 David authorizes publishing this release automatically when done,
without waiting for another personal test. #233–#237 are included: large thumbnails
on Home, fixed footer, single style, native drop-downs and fixed spacing/icons.
Validation of the new source precedes publication; the previous RC remains
as historical evidence, not as validation of these changes.
[Scope, tests and limits](docs/history/ALPHA41_FINAL.md).

#152 remains open: the GTK-free reproduction confirms that WordCraft 0.17
keeps the formatting when deleting the text and then the empty paragraph, but
transfers it when deleting the whole paragraph. #1, #52, #99, #171 and #185 also remain open.
The deferrals of public screenshots, Weblate and occasional Windows builds are kept.

<!-- published-alpha:start -->
## Latest verified release: 0.1.0-alpha.45

[Release](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.45). Closed issues: #259, #262, #278, #280.
Still open: #1, #99, #152, #185, #231, #252, #261, #265, #267, #268, #277, #279, #281, #282.
[Source, validation and limits](docs/LATEST_RELEASE.md).
<!-- published-alpha:end -->

## Published and verified: alpha.38 — start view and recovery

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.38).
Closed #207 and #209. Includes the fix for #208/PR #210: the
Navigation/Proofing tabs stay fixed and shared by all views.
New windows and tabs offer titled rows for templates and
recoveries only when they contain items, followed by the editable page.
Recovery replaces its old menu/dialog; when typing, the rows slide away
smoothly upward without deleting pending copies or blocking input.

215 Rust tests, nine Python tests and 891 complete messages in English/Spanish.
Optimized native suite, GNOME 49 installation and GNOME 51 prefix passed;
final comparison of the executable, tag and thirteen public files verified.
WordCraft 0.16.0, permissions, patches and the occasional Windows build are kept.
[Sources, execution, digests and limits](docs/history/ALPHA38_START_VIEW.md).

Reports that remain open: #1, #52, #99, #107, #152, #171, #185.

## Completed with alpha.38 — fixed bottom bar

The regression [#208](https://github.com/frandavid100/leaf/issues/208) corrects the
interpretation of #200: only the top controls scroll with the
results; Navigation/Proofing stays fixed and shared by all views.
Fix merged into main through
[PR #210](https://github.com/frandavid100/leaf/pull/210), with the full native
suite passed. The fix ships with alpha.38, together with the already completed
row redesigns #207 and animated removal #209.
[Scope and tests](docs/SIDEBAR208.md).

## Published and verified: alpha.37 — scrolling, focus and dialogs

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.37).
Closed #199–#204: gallery and page with shared scrolling, sidebar controls
inside their content, integrated adaptive dialogs, position restoration
that new input can cancel, focus continuity and name suggestions
when saving. Re-layouts of a page with a gallery also recover their image
without moving the browsing position.

214 Rust tests, nine Python tests and 893 complete messages in English/Spanish.
Optimized native suite and GNOME 49 and GNOME 51 installations passed;
the same validated bytes published, with tag and thirteen public files verified.
WordCraft stays at 0.16.0. The limits of #171, the deferrals and the
store and Windows work in their conversations are kept.
[Source, tests and limits](docs/history/ALPHA37_INTERFACE.md).

Eight reports remain: the previous seven and the new proposal #207 for template
and recovery rows, filed after this alpha was published.

## Published: first experimental Windows version

[Windows 0.1.0-alpha.36-windows.1](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.36-windows.1), published on 8/11/2026: per-user installer and x86_64 ZIP with GTK/libadwaita,
dictionaries and translations included. 210 Rust tests on Windows and 211 on Linux;
nine Python tests and 892 complete messages. ZIP moved to a path with spaces/ñ,
en/es interface, DOCX/ODT/PDF, preferences, windows, installation and removal
checked automatically. Complete notices added without changing the executable,
and final packages checked again before publishing; four public assets
with SHA-256 digests and the source tag verified. Occasional Windows releases;
no other Linux alpha is started and no issues are closed because of this port.
[Sources, execution and limits](docs/WINDOWS.md).


## Published and verified: alpha.36 — interface review

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.36). Closed #177/#181/#182/#186–#191,
including the new #191: literal search and compact options, fonts inside
Properties, split Copy/Paste rows, recovery of the tab space,
stability, Structure menus and an exact marker for dragging images.

#171 remains partial and open. The gallery uses real thumbnails over each new
document; the Flatpak uses the read-only XDG templates folder by default.
Choosing another folder is disabled in the sandbox because the portal
grants write access and refuses to drop that permission. Host access is not widened.

211 Rust tests, nine Python tests and 892 complete en/es messages; optimized
native suite, GNOME 49 and GNOME 51 prefix correct. Same validated bytes,
without rebuilding; tag and thirteen public files verified.
[Source, tests and limits](docs/history/ALPHA36_INTERFACE.md).

Seven remain: #1/#52/#99/#107/#152/#171/#185. David's deferrals and the
engine integration blockers are kept. The new public screenshots
remain David's responsibility; this release already fixes #186/#191.

## Store listing and release notes — #185

Prepared the approved English/Spanish description, the light/dark colors and
a common source of release notes for AppStream and GitHub. The previous
screenshots are kept with immutable references. It is not a new executable version.
FlatPark PR #1074 awaits authorization/acceptance from its maintainers. The new
screenshots can now be prepared after fixing #186/#191; checking the published
catalog in Bazaar is still pending.
[Process, editorial source and limits](docs/STORE_METADATA.md).

## Published and verified: alpha.35 — accepted proposals

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.35). Implemented and closed the thirteen issues
#170–#182: next style, personal templates, paste without formatting,
moving sections, format painter, advanced search, document
fonts and external images. [PR #183](https://github.com/frandavid100/leaf/pull/183)
and [lifecycle fix #184](https://github.com/frandavid100/leaf/pull/184).

205 Rust tests (fifteen new regressions), four Python tests and 879 en/es messages;
real controls and gestures at 1/2 scale, GNOME 49 installation and FlatPark
GNOME 51 prefix passed. The same validated bytes published, without rebuilding;
tag, identity and thirteen public files checked.
[Source, execution, scope and limits](docs/history/ALPHA35_WORKFLOWS.md).

A modified next style requires DOCX when the ODT exporter would lose
the choice. The templates folder is read without modifying the originals.
#84 was already checked and closed by David in the testing conversation.

Updated: November 8, 2026. [Current status](PROJECT_STATUS.md) ·
[Decisions](docs/DECISIONS.md) · [History](docs/history/README.md).
This is the current list; old archived plans do not add tasks.

## Reports predating the new proposals

#1/#99/#107/#152 await the integration of the fixes proposed in
WordCraft. #52 is deferred by David's decision. #84 was already tested and closed
in the validation conversation. The new #170–#182 were initially closed with alpha.35; #171 is kept
open after reviewing the design and permissions. The other reopened ones were verified
with alpha.36.
[Historical audit of the blockers](docs/history/REMAINING_AFTER_ALPHA34.md).

## Completed: alpha.33 — review of #9/#99/#156

[Published and verified](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.33)
on 7/11/2026. Closed #156 after two native before/after reproductions, full
suite and GNOME 49/51 installations. The same bytes promoted; tag and twelve
public files verified. #9 keeps the edited copy awaiting confirmation, with
controls and explanation already delivered. #99 has a public reproduction of the
engine and is still awaiting a fix. [Evidence](docs/history/ALPHA33_VALIDATION.md).

## Completed: alpha.32 — update and performance

[Published and verified](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.32)
on 7/11/2026: WordCraft 0.16.0, better ODT preservation, pinned language data
and HTML/RTF paste. Measured client optimization and clearer clipboard
indexes/code. Native suite, GNOME 49/51 and publication of the same bytes
completed; twelve public files verified. The later review closes
#100, already implemented and checked in that release; the executable does not change.
[Review of the nine pending issues](docs/history/ISSUE_TRIAGE_AFTER_ALPHA32.md).
[Scope, measurements and evidence](docs/WORDCRAFT_016_REVIEW.md).

## Completed: automatic alpha.31

[Published and verified](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.31)
on 7/11/2026 without another approval, in line with David's authorization. Latest
closures: #135/#149/#150/#151/#155; merges the previous candidates. Full native
suite, GNOME 49 and FlatPark/GNOME 51 passed; the same bytes promoted and
the twelve public files verified. [Evidence and limits](docs/history/ALPHA31_REMAINING.md).

Currently pending: #1/#2/#9/#52/#84/#99/#107/#152. Columns #1/#2 are worked on
separately in WordCraft; #152 reproduces a defect in the upstream model. Keep
open the cases without sufficient reproduction and the engine/host limitations.

## Historical: third alpha.31 candidate

Completed #121/#124/#126/#127 with explicit object and caption associations,
reversible Show Caption, shared properties, navigation and temporary
decoration. Full native suite and GNOME 49/51 installations checked;
English/Spanish, light/dark and 1×/2× scale. The eleven checksums of the download are
verified. [Implementation, identity and limits](docs/CAPTION_OWNERSHIP.md).

This earlier candidate required approval; the authorization for the last
pass allows publishing automatically after the checks. #132 closed on 7/11/2026 after David's check
with the original document, without attributing an unidentified cause or fix. After that candidate #135 was reopened and #149–#152 appeared; they were not
resolved by those four closures. The current section covers the later closures.

The second interface candidate remains as [historical evidence](docs/history/ALPHA31_UI_VALIDATION.md).
The first, which only closed #133, keeps its [historical record](docs/history/ALPHA31_VALIDATION.md).
Their issue lists describe those releases and do not replace the current status.

## Completed: alpha.30

[Published](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.30)
automatically under David's authorization. Accumulates the alpha.29 candidate and closes
#122, #123, #125, #128, #129, #130 and #131 after validating code and GNOME 49/51
installations. The same bytes of the candidate promoted; 12 public files
verified by SHA-256. [Evidence and limits](docs/history/ALPHA30_VALIDATION.md).

#121, #124, #126 and #127 keep pending work; the eight old reviewed issues
remain open. Resolve the canonical object/caption association before offering
a reversible Show Caption or a decoration that implies that ownership. This release
does not update WordCraft or start Flathub, Weblate or new Orca tasks.

## Completed: alpha.28

Authorized by David. Includes per-section zoom and thumbnails (#89, #94, #95),
floating images (#53), footnotes (#55), automatic table of contents (#54), captions,
cross-references and new public screenshots. DOCX/PDF, history and
interaction are checked. [Published](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.28)
and verified in Flatpak GNOME 49 and FlatPark GNOME 51 archive. The ODT
guards are kept; the separate FlatPark listing requires an external update.

## Completed: alpha.27 for FlatPark

Published relocatable archive, checksum and log of the installed test with
GNOME 51, with translations and spell checking verified. The GNOME 49 bundle
and the native suite also passed their checks. [Release](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.27).
Inclusion in the catalog is pending FlatPark's packaging.

## Completed: alpha.23

Rulers within the four borders, breaks menu and navigation by sections, independent even headers and footers, and native Properties tabs.

## Completed: alpha.22

[Published and verified](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.22).
Includes the formatting review, comments in the sidebar, shared Insert
and the printing and optimization changes. Columns (#1, #2),
inner rulers (#64), breaks (#65) and dictionaries (#69) remain pending.

## Completed: WordCraft 0.14 integration and client cleanup

Authorized by David after alpha.21: investigate the new features and clean up code,
structure and formatting. Dedicated printing API, separation of responsibilities
and removal of the hidden-styles traversal; no new features or release.
[Investigation and validation](docs/WORDCRAFT_014_REVIEW.md).

## Completed: sixth review and alpha.21 — WordCraft 0.14.0

Published and verified. Engine 0.14.0 with formats re-copied and the four ODT
patches reapplied. Four closures (#18, #48, #51, #61): revised theme selector,
transient thin white rulers, history with native AdwSplitButton and bounded
Font/Size lists. Also fixed: minimums for large fonts and click capture
by hidden rulers, with real regressions.

141 Rust tests, four Python tests, 678 gettext messages and [native/installed CI](https://github.com/frandavid100/leaf/actions/runs/41496070439)
complete. Public assets, source, checksums and installed screenshots checked.
Final status: **50 closed / 10 open**. #1, #2, #7, #9, #37 and #52–#56
keep their requirements; an engine update does not amount to advanced authoring.
[PR #62](https://github.com/frandavid100/leaf/pull/62) ·
[Scope](docs/history/ISSUE_SWEEP_ALPHA21.md) · [Evidence](docs/history/ISSUE_SWEEP_ALPHA21_VALIDATION.json).
Weblate #57 remains separate; no Flathub, new Orca stages or change of
public screenshots. Earlier histories and tags kept.

## Fifth review: alpha.20 (historical)

Six verified closures (#49–#51, #58–#60): cursor, history/Redo and sidebar
presentation. Ruler (#48) and GNOME selector (#18) implemented and tested automatically;
the installed visual comparison remained pending when the environment disconnected and was
completed during alpha.21.
141 Rust tests, four Python tests, 678 messages and native/installed CI correct.
ODT is protected against four additional structural losses; #53–#56 are not
considered complete. Status: 47 closed / 12 open.
[Scope](docs/history/ISSUE_SWEEP_ALPHA20.md) · [evidence](docs/history/ISSUE_SWEEP_ALPHA20_VALIDATION.json).
Engine/dependencies untouched; Weblate, Flathub, Orca and the corpus are not expanded.

## Completed: fourth review and alpha.19

Published and verified: nine closures (#25, #26, #28, #34, #35 and #44–#47).
128 Rust tests, four Python tests, 588 gettext messages and native/installed
Flatpak CI complete. #35 was reproduced in the installed runtime and verified
after fixing input and focus. [Scope](docs/history/ISSUE_SWEEP_ALPHA19.md) ·
[evidence](docs/history/ISSUE_SWEEP_ALPHA19_VALIDATION.json).

18 reports remained open at the end; check GitHub before choosing
another task. Do not automatically expand with new features, engine changes,
Weblate, Flathub or new Orca stages.

## Consolidation in main and Weblate preparation

Merge of the ApexDesk and Leaf histories into `main`, with an archive of the
previous main and `wordcraft-saving` kept. Development, CI,
contribution and prepared Weblate configuration references updated. It is not a
new release. [Review and validation](docs/MAIN_MIGRATION.md) ·
[Weblate activation handled by David](docs/WEBLATE_SETUP.md).

## Completed: third review and alpha.18

Published and verified. Ten reports closed out of the initial fourteen: #5, #6,
#14–#16, #30 and #40–#43. 123 Rust tests, four Python tests and native/installed
Flatpak CI complete. [Scope and pending items](docs/history/ISSUE_SWEEP_ALPHA18.md) ·
[evidence](docs/history/ISSUE_SWEEP_ALPHA18_VALIDATION.json).

Historical status after alpha.18: 32 closed and 20 open, out of 52. The four
pending from the initial pass were #1, #2, #7 and #9. The nine new ones (#44–#52)
and seven reopened ones (#18, #25, #26, #28, #34, #35, #37) kept their requirements
for later reviews.
Those extensions were not added to alpha.18 nor closed based on tests of
earlier requirements. WordCraft engine/dependencies,
Weblate, Flathub and main were not updated.

## Completed: maintenance and ease of contribution

**Authorized by David:** review structure and clarity, separate responsibilities,
make onboarding of people and new chats easier, and update this plan.
The reorganization is validated: 99 Rust tests, four Python tests, formatting,
translations, links, GTK/HiDPI and automated Orca. [Report](docs/MAINTAINABILITY_REVIEW.md).
[Code map](docs/ARCHITECTURE.md) · [Continuity](docs/CONTINUING.md).

## Completed: cumulative alpha.14 — annotations and sidebar proofing

Published with bookmarks/comments from Insert and context in the third
sidebar tab, and threads preserved in DOCX. Leaf protects ODT saving with
comments against the exporter's omission. [Details](docs/BOOKMARKS_COMMENTS.md).

Spelling and grammar occupies the fourth sidebar tab with a navigable list;
check-as-you-type is in Preferences. [Details](docs/PROOFING_SIDEBAR.md).
105 Rust tests, full native CI and installed Flatpak, update
from alpha.13, portals and automated Orca have passed. [Log](docs/BOOKMARKS_COMMENTS_VALIDATION.json).

General protection against external changes remains deferred. Do not start
new Orca stages.

## Completed: second review and alpha.17

Published and verified: 18 reports resolved (#12, #18, #19, #22, #25–#29,
#31–#39). The release includes word and Shift selection, text input,
usable links, proofing menus, navigation and native refinements.
117 Rust tests and native/installed Flatpak CI complete, including update,
portals and automated Orca. [Result](docs/history/ISSUE_SWEEP_ALPHA17.md) and
[evidence](docs/history/ISSUE_SWEEP_ALPHA17_VALIDATION.json).

When that release was finished, 14 issues remained open, including #40–#43.
The third review and the current status are described above. The branch for that release was `wordcraft-saving`; current development is in `main`.

## Completed: issue review and alpha.16

Published and verified after reviewing 32 issues (24 initial and eight new).
14 closed; 18 open. Sync #12 improves the main controls,
but remains open because of the column editor and the remaining coverage. Columns,
advanced tables, odd/even, the spacing case, proofing inheritance and the
eight new reports keep their pending tasks.
[Result and checks](docs/history/ISSUE_SWEEP_ALPHA16.md).

## Completed: panel and metadata refinement for alpha.15

Bookmarks integrated under Navigation headings, one bubble per comment
thread and sidebar proofing for navigation only. Remove the proofing entry
from the document menu. Add editable title, subject, author, keywords and category
in Properties. Published after 108 Rust tests, native validation and
installed Flatpak, update from alpha.14, portals and Orca/AT-SPI regression.
[Checks](docs/SIDEBAR_REFINEMENT_VALIDATION.json).

## Priority order after this review

These are proposals; they require a concrete bug or a request before acting.
The review and release are completed. Open issues keep their tasks; do not start another expansion automatically.

| Priority | Work | What allows considering it resolved |
| --- | --- | --- |
| 1 | Reproducible bugs involving content loss, saving or crashes. | Small reproduction, fix in Leaf and relevant regression test. If the defect is in WordCraft, an upstream proposal. |
| 2 | Interaction, import/saving or performance problems reported by users. | Before/after comparison and documented limits; engine untouched in Leaf changes. |
| 3 | Usability improvements chosen by David: e.g. advanced table or image editing. | Agreed scope, GNOME integration and the feature's own checks. Do not implement the whole list. |

While there are no reports, maintenance and automated checks
can continue without requiring a tester base. Do not invent issues:
check the current issues on GitHub before choosing a bug.

## Deferred or dependent on another person

| Topic | Current decision |
| --- | --- |
| New Orca extensions | Deferred by David. Keep the current support and automated test. |
| Human testing with Orca/braille, printers and GNOME OS | Wait until users are available; do not block other tasks or present them as already done. |
| Public screenshots | Main screenshot provided by David and added to the README; keep provenance and do not replace with CI images. |
| Hosted Weblate | David will complete the account/authorization. PO catalogs and contributions via PR already available. |
| Flathub | Later, with a review of requirements and human collaboration. GitHub Flatpak distribution already exists. |
| WordCraft update/optimization | Only with specific authorization. Propose engine changes there; do not mix them with Leaf optimizations. |
| Full DOCX/ODT corpus | Manual/optional when justified; not mandatory before each version. |

## Completed foundation

| Release | Result | Evidence |
| --- | --- | --- |
| Alpha.15 | Navigation with bookmarks, comment bubbles, simplified proofing and metadata. | [Validation](docs/SIDEBAR_REFINEMENT_VALIDATION.json) |
| Alpha.14 | Bookmarks/comments and sidebar spell checking. | [Annotations](docs/BOOKMARKS_COMMENTS.md), [proofing](docs/PROOFING_SIDEBAR.md) |
| Alpha.13 | Diagnosis and optimization limited to the Leaf host. | [Report](docs/HOST_PERFORMANCE_AUDIT.md) |
| Alpha.12 | Initial access to text with Orca. | [Scope and test](docs/ACCESSIBLE_DOCUMENT_TEXT.md) |
| Alpha.11 | Independent check with Writer and ODT geometry. | [Interoperability](docs/INTEROPERABILITY_AUDIT.md) |
| Alpha.10 | Sections, spell checking and text language. | [Options and language](docs/PAGE_OPTIONS_PROOFING.md) |
| Alpha.9 | Headers, footers and editable numbering. | [Usage and limits](docs/HEADERS_FOOTERS.md) |
| Alpha.8 | English/Spanish and dark document. | [Localization](docs/LOCALIZATION_DARK_MODE.md) |
| Alpha.6 | GTK lifecycle fixes. | [Audit](docs/GTK_LIFECYCLE_AUDIT_2026-10-31.md) |
| Alpha.5 | Printing and keyboard navigation. | [Printing/keyboard](docs/KEYBOARD_PRINTING.md) |
| Alpha.1–4 | Flatpak, columns, HiDPI and ODT preservation fixes. | [History](docs/history/README.md) |

Basic and formatting editing, styles, inline tables/images, properties,
PDF, tabs, search and recovery are part of that foundation. Their limits
remain in [PROJECT_STATUS](PROJECT_STATUS.md); "completed" does not mean universal
fidelity or human testing of every environment.
