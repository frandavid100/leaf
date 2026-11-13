# Current decisions

## 13/11/2026 — alpha.46 candidate prepared, not authorized for publication

David asks for a release candidate with the review's merged changes. It is
built with `[candidate-alpha]` only, without `[auto-publish-alpha]`: it waits
for his testing and approval of this specific candidate.
[Scope and limits](ALPHA46_REVIEW.md).

## 13/11/2026 — diagnostic code stays in the published app

Decision (1) of 12/11 is revised. CI runs about 25 diagnostic checks on the
installed Flatpak that gets published; removing them would require validating a
package different from the published one and redoing CI and the RC process.
The measured saving is about 1.3 MB out of 47.8 MB. David decides to keep it and
achieve clarity by gathering all diagnostics in `src/gnome/diagnostics/`, with a
single entry point for the hooks inside the product code.

## 12/11/2026 — review of size, speed and clarity

After a code review, David decides: (1) [revised on 13/11, see above] the published
app will not include the graphical diagnostics code; (2) only the `es_ES` dictionary
will be included by default, and `es_MX`/`es_AR` can be added from Preferences;
(3) contributor documentation will move to English, except `AGENTS.md` and
`CONTINUING.md`. It will be applied in small PRs; none of them involves publishing.

## Published and automatically verified: 0.1.0-alpha.45

The same bytes as the candidate are published after passing all native and
installation tests. Source, tag and thirteen public files are verified before
closing #259, #262, #278, #280. [Identity and limits](LATEST_RELEASE.md).

## 11/11/2026 — alpha.45 in validation, direct publication authorized

David asks for a new release and for closing the resolvable issues, prioritizing
the most recent one. It includes #278/#280, the catalog expansion #259 and the
column controls #262. #279 adds detection and download/opening in the system
viewer, with a mandatory installed test; it remains open because of the store
permissions and pending validations. Engine and upstream contributions are kept
separate. [Scope, tests and limits](history/ALPHA45_MAINTENANCE.md).


## 11/11/2026 — resuming the FlatPark listing

David authorizes preparing the entry update with the four already reviewed
screenshots and the current icon. He then asks to renew the text as well: the
README's approach is adapted in EN/ES, with advantages and features first and
alpha status and AI at the end. The existing colors are kept. The external change is limited to metainfo and icons, keeping
pin, permissions, runtime, wrapper and resolver. The empty notes are recovered from the
upstream metainfo for the packaged alpha.44. David approves the result, and
[PR #1139](https://github.com/flatpark/flatpark/pull/1139) is opened on 11/11/2026. The branch
contains only the three planned data files and has no conflicts. Its CI
requires a maintainer's authorization; review and publication remain pending. Do not attribute publication or testing in Bazaar to a prepared branch.
[Scope and validation](FLATPARK_GALLERY_2026_11.md).

## 11/11/2026 — README presentation

David authorizes using the four reviewed screenshots with the new icon and updating
the text. The cover screenshot is the main one; layout, review and the preferred
underline menu form a visible gallery without a drop-down. The original PNGs and
their provenance are kept. At David's later request, the installation options and
the FlatPark badge go at the end, followed by the explicit explanation of vibe
coding.
No exact version is attributed to the screenshots, and no release or catalog
update is published because of this change. [Screenshots](screenshots/README.md).

## 10/11/2026 — Windows alpha.44 published and verified

The `0.1.0-alpha.44-windows.1` installer and ZIP use the same source
`534c86eab70e05ae5de8c4d1d9c95d075d8ae666` as the Linux alpha.44 release.
The build, portable package, installation and uninstallation tests
pass on [Windows CI](https://github.com/frandavid100/leaf/actions/runs/42900734179).
Tag and four public files are verified; the hashes of the downloaded artifact
match the public digests and `SHA256SUMS`. [Download, identity and limits](WINDOWS.md).
No human testing on Windows computers is attributed. Both publications are
completed without modifying WordCraft or closing partial requirements.

## Published and automatically verified: 0.1.0-alpha.44

The same bytes as the candidate are published after passing all native and
installation tests. Source, tag and thirteen public files are verified before
closing any. [Identity and limits](LATEST_RELEASE.md).

## 10/11/2026 — alpha.44: maintenance, GNOME colors and Windows

David authorizes applying the discussed cleanup, then publishing another release,
restoring the previous native pickers and generating an installer for friends
who use Windows. The new appearance is limited to the color pickers.
The checks and the engine are kept; the WordCraft changes being prepared in
another conversation are not mixed in. Incomplete requirements are not closed.
[Scope, measurements and delivery conditions](history/ALPHA44_MAINTENANCE.md).


## Published and automatically verified: 0.1.0-alpha.43

The same bytes as the candidate are published after passing all native and
installation tests. Source, tag and thirteen public files are verified before
closing #50, #236, #238, #240, #246, #249, #250, #251, #253, #254, #255, #256, #257, #258, #260, #263, #264, #266. [Identity and limits](LATEST_RELEASE.md).

## 10/11/2026 — alpha.43 with the new issues and direct publication

David adds the recent proposals and explicitly asks for a release, without an RC.
All validation is kept, and this delivery is published automatically once it passes.
Partial requirements are documented without closing them; the engine's limits do
not justify a fork or fake controls. The public listing remains postponed.
[Delivery plan and specific limits](history/ALPHA43_TOOLS.md).

## Published and automatically verified: 0.1.0-alpha.42

The same bytes as the candidate are published after passing all native and
installation tests. Source, tag and thirteen public files are verified before
closing #235, #238, #239, #240, #241, #242, #243, #244, #245, #246, #248. [Identity and limits](LATEST_RELEASE.md).

## 10/11/2026 — alpha.42 and template scope

David asks to review the WordCraft contributions, close what can be verified and
directly publish a new alpha. The exception applies only to this
delivery and keeps all verifications. The new #245/#246/#248 are added, including the common menu and the three
configurable placements per tool.
He accepts only the system templates folder (#171); permissions are not expanded.
#52 is closed without confirmed attribution. #185 remains on hold:
David will take new screenshots after the next alpha and review the description.
[Scope, tests and limits](history/ALPHA42_INTERFACE.md).


## Published and automatically verified: 0.1.0-alpha.41

The same bytes as the candidate are published after passing all native and
installation tests. Source, tag and thirteen public files are verified before
closing #107, #217, #226, #227, #228, #229, #230, #231, #232, #233, #234, #235, #236, #237. [Identity and limits](LATEST_RELEASE.md).

## 9/11/2026 — explicit exception: publish alpha.41 when finished

David asks to resolve the open issues and publish directly because he is going
to sleep. This instruction authorizes the automatic promotion of this delivery after
validating the new source, with the same checked bytes; it does not replace the usual
rule of RC and approval for future deliveries. Only the resolved reports are
closed: #233–#237 are added to the closing plan, #152 is removed from the plan.
The other five blockers and the public screenshots reserved for David are kept.
[Scope and limits](history/ALPHA41_FINAL.md).

## 9/11/2026 — #152 reopened after testing the RC

David provides five screenshots and distinguishes two operations: selecting and
deleting the whole paragraph with a triple click still imposes its formatting on
the next one; deleting only the text and then removing the now-empty paragraph
keeps the formatting. The fix and the earlier regression tests covered the second
case. #152 is reopened, with both paths, results and screenshots; the layer causing
the remaining case is not taken as confirmed without inspecting selection and model.
This update records the failure; it does not modify the executable or publish the
RC. [Issue](https://github.com/frandavid100/leaf/issues/152).

## 9/11/2026 — usual procedure: RC, testing and promotion without changes

David establishes as the usual procedure to first deliver a validated RC
for him to test. With his approval for that candidate and no fixes, publish
the same files and frozen notes, without recompiling or repeating the test suite
already passed. Source, tag and SHA-256 checks are kept when publishing.
If he finds failures and there are changes, prepare another validated RC to test again.
The earlier automatic authorizations were for those deliveries; they are not
a permanent authorization to publish future versions. Alpha.41 remains
a candidate until it receives his approval. [Procedure](CANDIDATES.md).

## 9/11/2026 — RC alpha.41 verified and ten closures

#107, #152, #217 and #226–#232 are closed after validating code, interface and
installed GNOME 49/51 packages. The executable from `2d9b1fd` is kept without
recompiling; candidate `d6adbe5` verifies source and byte equivalence.
The later adjustments affect only artifact downloading and the test drivers'
profiles/synchronization. The RC remains downloadable for testing; the
latest public alpha is still 40. The five pending reports and the deferrals
are kept. [Identity and evidence](history/ALPHA41_INTERFACE.md).

## 9/11/2026 — 0.17 candidate and new issues

David asks for an RC with WordCraft 0.17 and #217, adding the recent issues
#226–#232. The predominant neutral pair of Chrome's external HTML is adapted;
intentional local colors are kept. WordCraft remains the
model, the renderer and the history. Preferences only stores the layout
of controls, never document data. Copies of live documents are still
written but are not offered for recovery or for discarding. Do not publish
or close based on a partial validation; record the candidate's tests and limits.


## Published and automatically verified: 0.1.0-alpha.40

The same bytes as the candidate are published after passing all native and
installation tests. Source, tag and thirteen public files are verified before
closing #218, #219, #220, #221, #222. [Identity and limits](LATEST_RELEASE.md).

## 9/11/2026 — new review and alpha.40

David authorizes resolving the new verifiable issues and publishing another alpha.
#218 replaces the gallery over the page with a reversible Start in the sidebar;
#219–#222 return formatting to a bottom bar with four groups. The engine and the
permissions are kept. Discarding requires confirmation because the public API
does not offer a reversible retention that allows Undo with guarantees. The
public screenshots, occasional Windows builds and the external blockers keep their scope.
[Implementation, tests and limits](history/ALPHA40_INTERFACE.md).



## Published and automatically verified: 0.1.0-alpha.39

The same bytes as the candidate are published after passing all native and
installation tests. Source, tag and thirteen public files are verified before
closing #212, #213, #214. [Identity and limits](LATEST_RELEASE.md).

## 8/11/2026 — alpha.39: thumbnails and publishing process

David authorizes applying the CI speed-up, resolving the new issues and publishing
another alpha. The full checks and the bytes of the single build are kept;
jobs and runtimes are split up, and the screenshots wait for the rendered scene.
Recoveries are previewed on private copies through the public API;
the original records keep their bytes and availability. Publishing verifies
thirteen files before closing the explicit plan and updating documents.
[Scope and pending measurement](history/ALPHA39_PREVIEWS.md).

## 8/11/2026 — alpha.38 published and verified

David authorizes another release closing what can be verified and including PR #210.
#207/#209 are completed and #208 is distributed: conditional initial rows,
direct recovery and native 180 ms retraction when typing, with Navigation/Review
fixed. The page remains editable; loading, canceling or retracting the rows does
not discard pending copies. Stale results do not replace edits or take focus
away from another tab. The compensation observes GTK's size request before
assigning positions; it does not use the size of the previous frame.

Closures after native validation, GNOME 49/51, executable comparison
and verification of thirteen assets. The engine/portal/store blockers and
David's deferrals remain. The occasional Windows delivery is kept; this alpha
is Linux. [Source, tests and limits](history/ALPHA38_START_VIEW.md).

## 8/11/2026 — Navigation/Review fixed at the bottom of the panel

David clarifies #200 after observing regression #208: the two bottom tabs
must stay fixed; the top selectors do belong to the scrollable content.
A single bar is kept in the common AdwToolbarView, also reachable
from Search, without copies in the lists. The previous geometric test
wrongly expected it to scroll and is fixed together with the interface.
Merged into main through PR #210 after passing the full native test suite;
no new alpha is published. #209 separately records the animated upward
retraction of the transient rows when editing starts.
[Implementation and validation](SIDEBAR208.md).

## 8/11/2026 — alpha.37 published and verified

David authorizes creating the reports on Structure jumps, focus/continuity and
save name, resolving the recent bugs and publishing another alpha.
#199–#204 are closed after the full native test suite, GNOME 49/51,
executable comparison and verification of the tag and thirteen public files.

Gallery and page share scrolling; each panel keeps its own
controls and position. Restorations wait for GTK's size allocation
and are canceled if the user intervenes. Readjustments repaint the page without
moving the gallery. Custom dialogs are integrated into their window; portals
remain native and fields keep Undo/Redo.

For #201, only the v1_5 API of the existing libadwaita crate is enabled,
without changing its version, Cargo.lock, engine, profiles or patches. This
specific exception is checked in CI. Suggested names are a read
operation: they do not modify title, selection or history. Windows,
the earlier blockers and David's public screenshots are kept.
[Source, execution and limits](history/ALPHA37_INTERFACE.md).

## 8/11/2026 — Windows published and checked

Published `0.1.0-alpha.36-windows.1`: 210 Windows Rust tests, 211 Linux ones and
real checks of the portable and installed packages in en/es, including
moving to a Unicode path, save/reopen, SVG, PDF, preferences and windows.
The final warnings are completed without altering the executable; then the tests
of both packages are repeated and those bytes are published, with four assets and
tag verified. Physical printing, accessibility and real GPUs remain unvalidated.
[Identity, runs and digests](WINDOWS.md).

## 8/11/2026 — first version for Windows

David asks to prepare a first experimental version of the current state.
The GTK/libadwaita client, WordCraft and the pinned patches are kept.
Windows deliveries are generated occasionally, without requiring parity with each
Linux alpha. The installer is per-user, without administrative privileges;
the ZIP uses the same user configuration and recovery folders.
Publish only the bytes checked on Windows, recording the limitations
in printing, accessibility and human testing. [Details](WINDOWS.md).

## 8/11/2026 — alpha.36 interface review

The transient gallery replaces the templates dialog. Fonts lives inside
Properties; search receives pasted characters without a special picker.
Structure keeps visual anchors and uses menus to move headings. The drag
cursor is passive and its position is the only valid destination for the import.
Published and verified with nine closures. #171 remains open: `xdg-templates:ro`
is kept and the picker for another folder is disabled in Flatpak as long as
the portal does not allow the minimum required access. Do not replace it with write
access or a broad permission. [Source, verification and limits](history/ALPHA36_INTERFACE.md).


## 2026-11-08: listing, colors and what's new

David approves the description and the light `#E4F2E8` and dark `#1B3327` colors,
and authorizes implementing the what's-new notes and their synchronization. The
new screenshots remain his responsibility and wait for the title bar spacing to be
fixed; they will be reviewed one by one. No binary release is started because of
this change. The bilingual editorial source generates AppStream and the notes used
by the GitHub release. FlatPark must import metadata from the exact version
it distributes, keeping its policy and the installation details.
[Process and limits](STORE_METADATA.md).

## 2026-11-07: accepted proposals for alpha.35

David authorizes creating the thirteen issues #170–#182, implementing them, closing
only the verified ones and automatically publishing an alpha. The document-based
style workflow is kept; only the next style is added. Templates are
normal files in the XDG Templates folder or in a folder chosen through
the portal. Conflict protection and the symbol picker are postponed.

Structural and font operations are recorded as engine commands
through its public API: WordCraft keeps protection, transactions, review and
history. Neither the engine nor the formats snapshot is modified. Search
selection limits invalidate results after external changes. The new actions
are verified with headless tests, GTK controls and real gestures, also
in the GNOME 49/51 packages. [Details](history/ALPHA35_WORKFLOWS.md).


Alpha.35 published and verified on 8/11/2026; the thirteen reports are closed.
The control lifecycle test, the native and installed regression tests and the
executable comparison before promoting it without recompiling are kept.
Publishing rejects candidates superseded by another source on main.
[Identity and checks](history/ALPHA35_WORKFLOWS.md).

These decisions come from David and from work already accepted. They take
precedence over historical proposals; a roadmap does not by itself authorize new features.

| Topic | Decision | Consequence when modifying code |
| --- | --- | --- |
| Autonomous publication of pending fixes | David asks on 7/11/2026 to resolve the remaining issues and generate the next release without further approval. | Publish after verifying real fixes; keep integration or testing blockers open. The current review contributes no code changes. [Six pending](history/REMAINING_AFTER_ALPHA34.md). |
| Dictionaries #84 — alpha.34 published | David authorizes on 7/11/2026 preparing a candidate to test on his system. | Folder through the picker/portal, without expanding host permissions; keep import, bundled data, language/document and history. Issue open until his GNOME OS test. [Scope](history/ISSUE84_DICTIONARIES.md) and [delivery validation](history/ALPHA34_VALIDATION.md). |
| Alpha.33 published/verified | David asks to act on #9/#99/#156 and keeps the authorization for automatic publication after checking the delivery. | #156 closed for two native failures reproduced and fixed; #9 has an explanation/controls and #99 a public engine reproduction. Both remain open. Full test suite, GNOME 49/51 and twelve public files verified. [Sources and limits](history/ALPHA33_VALIDATION.md). |
| Post-alpha.32 review | David asks to review whether the ten remaining reports can be closed. | #100 closed for requirements verified in the existing release; do not attribute an unknown historical cause or require it as a closing condition. #152 is still reproducible in 0.16.0. [Evidence and nine pending](history/ISSUE_TRIAGE_AFTER_ALPHA32.md). |
| Alpha.32 published/verified | David authorizes on 7/11/2026 updating to the latest stable WordCraft, adapting Leaf, optimizing performance and clarity, and publishing another release. | Published with WordCraft 0.16.0, broader ODT support and HTML/RTF pasting, after the native test suite and GNOME 49/51. Pinned data, residual compatibility reviewed and comparison with the same engine; twelve public files checked. No additional issues are closed. [Review](WORDCRAFT_016_REVIEW.md). |
| Alpha.31 published/verified | David authorizes on 7/11/2026 resolving/closing what can be verified and publishing automatically without further approval. | Published on 7/11/2026 after all native/installed validation, with the same bytes and twelve public digests checked. Closed #135/#149/#150/#151/#155; replaces the pending approvals for this delivery. [Results and limits](history/ALPHA31_REMAINING.md). |
| Third alpha.31 candidate validated | Completed #121/#124/#126/#127 through canonical caption ownership, shared controls, typed navigation and temporary decoration. Full native test suite and GNOME 49/51 verified. | [Source, checksums and limits](CAPTION_OWNERSHIP.md). Keep text/history, do not infer ownership by proximity without attributing an unidentified fix to #132; David confirmed the original and it was closed on 7/11/2026. The earlier approval of that candidate was replaced by the later authorization of automatic publication after all checks. |
| Historical second alpha.31 candidate | In that delivery #134–#136 were closed; #135 was later reopened and closed again in the last pass. The ruler's event capture was isolated; it keeps #133. Full native test suite, GNOME 49/51 and actually translated menus verified on the same files. | [Source, checksums and limits](history/ALPHA31_UI_VALIDATION.md). The earlier approval of that candidate was replaced by the later authorization for the last alpha.31 pass; keep its historical evidence. |
| Historical first alpha.31 candidate | It only closed #133; the interface experiments were removed in that delivery. | [Historical identity](history/ALPHA31_VALIDATION.md). The second candidate replaces it; do not reuse its issue list as the current state. |
| Visual review #111–#121 | David authorizes adding the eleven new issues to the same candidate. Review gathers corrections/changes/comments; Edit/Suggest modes, format rows with mixed checkboxes and Elements in the header with shared creation/properties. | Keep pinned floating, selection, inheritance and public APIs. #121 replaces the name/location of Insert from #109, #116 the previous icon, #120 the previous grids; keep capabilities. |
| Navigation #131 | A single main Navigation entry, with Pages/Structure inside the panel; Review is the other main entry. Replaces the three previous main destinations. | Keep representation, filter, expansion and independent scrolling per tab; sync the real destination without altering selection/document when switching views. |
| Alpha.30 published/verified | Automatic publication authorized and completed on 6/11/2026; integrates candidate 29 and seven new closures. | [CI, source, checksums and limits](history/ALPHA30_VALIDATION.md). No recompiling when promoting; #121/#124/#126/#127 keep work. |
| Candidates and alpha.30 | David explicitly authorizes publishing this new pass automatically, without waiting for his approval. | Keep native and installed tests and promote the same bytes; `[auto-publish-alpha]` enables automatic promotion only after a successful candidate from main. Without that marker, the approval process is kept. |
| Alpha.28 published/verified | David authorizes on 5/11/2026 footnotes, index, cross-references, floating images, zoom/navigation and public gallery. | Reuse WordCraft stories, fields and commands, verify DOCX/PDF and history, and keep the ODT guards. The gallery authorization replaces the earlier deferral for this delivery. [Full native and installed CI](https://github.com/frandavid100/leaf/actions/runs/42057994674). |
| Development branch | `main` for development, building and PRs; both histories merged. | ApexDesk archived in `archive/apexdesk-main-2026-11-02`; keep `wordcraft-saving` and existing releases/tags. [Review](MAIN_MIGRATION.md). |
| Document model | WordCraft is the source of truth. | Use Session, layout and public commands; no hidden editor or parallel history. |
| Upstream boundary | Optimize Leaf; propose changes to its engine in WordCraft. | Do not modify engine/render or dependencies for a client adjustment. Keep the ODT exceptions already recorded in [patches](../patches/README.md). |
| Base updates | David authorizes on 2/11/2026 updating to the latest stable version: WordCraft 0.14.0, commit `5cd53f184aab11def13b7fced18727159b9bc96a`. | Engine and interface adjustments in separate changes; formats recopied and four ODT patches reapplied. Follow [UPDATING_WORDCRAFT](../UPDATING_WORDCRAFT.md) and validate before publishing. |
| Ruler | Transient scales limited to the visible sheet; more legible numbers and drags snapping to 0.5 cm (#118). | Keep keyboard, imported geometry and precision of numeric inputs. Margins on both axes, indents/tabs with their correct origin, preview, cancellation and a single Undo step. |
| Post-alpha.21 review | Integrate WordCraft 0.14's print-specific PDF; separate input/dialogs and clear formatting; avoid hidden style work. | [Research and checks](WORDCRAFT_014_REVIEW.md). Keep the engine/dependencies and the remaining behavior; it is not an authorization to implement the whole WordCraft interface. |
| Design | GTK4/libadwaita and GNOME conventions. | Keep native controls, the Style → Paragraph → Text order, wide/narrow adaptation and contained context menus. |
| Lifecycle | Explicit owners, weak callbacks and cleanup on close. | Do not reintroduce Rc/widget cycles; check closing of tabs/windows and workers. |
| Bookmarks and comments | Bookmarks under their headings in Navigation; comments and replies in Review alongside corrections and changes (#117/#119). Creation from Elements/context (#121). | WordCraft keeps data and history. Keep connectors and authorship; DOCX keeps the threads and ODT stays protected while its exporter omits them. |
| Spelling and grammar | Corrections in the Review panel with Apply/Ignore and extended actions. Preferences sets new documents; Properties sets inheritance and Text the exceptions. | Invalidate stale positions, coalesce the work and cancel it when the tab is closed. Edit/Suggest mode is independent. |
| Metadata | Edit title, subject, author, keywords and category in Properties → Information, using `file.properties`. | Do not overwrite other properties or the session identity. DOCX keeps the five fields; block ODT with a category to prevent its loss. Apply is a single undo step. |
| Alpha.23 / issues | Closed #64, #65, #7 and #37 after validating the installed alpha. | Save as DOCX for the breaks that the ODT exporter does not keep. Properties uses the inline picker from libadwaita 1.7 and keeps the previous picker on older compatible versions. |
| Alpha.22 / issues | Published with WordCraft 0.14.0, printing and optimization, seven closures and validation of the installed Flatpak. | Styles uses the quick gallery when it exists. Indents keep their direct controls after evaluating the grouped alternative. Keep #65 and #69 open until the limitations described in [the review](FORMAT_POPOVER_REVIEW.md) are resolved. |
| Alpha.21 / issues | Published and verified with WordCraft 0.14.0; four closures (#18, #48, #51, #61) and ten open. Thin transient rulers, AdwSplitButton history and bounded Font/Size. | [Scope](history/ISSUE_SWEEP_ALPHA21.md) and [evidence](history/ISSUE_SWEEP_ALPHA21_VALIDATION.json). Keep ODT guards and authorship limits; update the reuse reference only to validated source/CI. |
| Alpha.20 / issues (historical) | Published; six closures and 12 pending. Ruler and GNOME picker were left with a pending visual review, completed in alpha.21. Four additional ODT guards prevent structural loss. | [Scope](history/ISSUE_SWEEP_ALPHA20.md) and [evidence](history/ISSUE_SWEEP_ALPHA20_VALIDATION.json). Do not present the ODT block as complete advanced authoring; the pinned design of alpha.20 is replaced by #48. |
| Alpha.19 / issues | Published and verified; nine closures out of the nine authorized reports. #35 reproduced in Flatpak GNOME 49 and verified after fixing filter and focus. | [Scope](history/ISSUE_SWEEP_ALPHA19.md) and [validation](history/ISSUE_SWEEP_ALPHA19_VALIDATION.json). Keep real events, input draining before assertions and installed tests; do not claim human Wayland/IME coverage. |
| Alpha.18 / issues | Published and verified; ten closures from the initial pass and 20 open after new/reopened ones. | [Scope](history/ISSUE_SWEEP_ALPHA18.md) and [validation](history/ISSUE_SWEEP_ALPHA18_VALIDATION.json). Do not consider later requirements resolved with tests of the earlier specifications. |
| Alpha.17 / issues | Published and verified; 18 reports closed and 14 open at the end. | [Scope](history/ISSUE_SWEEP_ALPHA17.md) and [validation](history/ISSUE_SWEEP_ALPHA17_VALIDATION.json). New #40–#43 are outside this delivery; in that delivery main kept the prototype. |
| Alpha.16 / issues | Published; 32 reviewed, 14 closed and 18 open. #12 keeps pending partial work. | [Evidence and limits](history/ISSUE_SWEEP_ALPHA16.md); close only what is verified and keep the remaining tasks. |
| Explicit saving | Ctrl+S saves over the open file; Save As chooses another destination. | Recovery remains the default; failures keep the changes. |
| Formatting controls | Valid scalar changes are immediate; pinned panels follow the selection. | Validate input, avoid edits while syncing and group undo. Atomic exceptions documented in the review. |
| Edit/Suggest modes | Choose per document whether new edits are recorded as proposals (#117). | Switching modes does not accept/reject or discard existing revisions; their actions remain available in both modes and are independent of spelling/comments. |
| Identity | Preferences sets the author of new comments and revisions. | Initial value from the system account; do not rewrite earlier authorship or metadata. |
| Cumulative alpha.15 | Published with panel and metadata refinement; native and installed CI and download verified. | Keep [evidence and limits](SIDEBAR_REFINEMENT_VALIDATION.json); later proposals are not automatic tasks. |
| Cumulative alpha.14 | Published with annotations and side review after native/installed validation. | Keep the scope, evidence and limits of [the delivery](BOOKMARKS_COMMENTS_VALIDATION.json); new extensions require their own request. |
| Conflicts with external changes | Postponed to prioritize bookmarks/comments. | Do not start that extension now. |
| Orca | The first stage is kept; extensions postponed. | Keep the current regression tests without starting accessible tables/images on your own initiative. |
| Testing with people | Wait until users are available. | Distinguish automated success from human testing; do not block independent work. |
| Screenshots | Use real public screenshots provided by David; the main one is already included in the README. | Keep provenance; do not replace them with CI screenshots. Internal screenshots are for validation only. |
| Translations | English/Spanish available; collaboration through PO files. Weblate prepared for `main`; registration/authorization pending from David. | Do not repeat the remote access; keep messages/placeholders and gettext tools. |
| Corpus | Extensive manual checks when justified. | Do not impose the full corpus per version; use small regression tests for specific failures. |
| FlatPark | Publish the `/app` tree in `Leaf-<version>-flatpark-x86_64.tar.xz`, without a containing folder. | Resources relative to the executable; validate extraction under `/app/extra/leaf` with GNOME 51 before publishing. Notify FlatPark of desktop/icon/metainfo changes and new runtime requirements. Confirm publication in its catalog separately. |
| Distribution | Experimental GitHub Flatpak, GNOME 49, minimal permissions and portals. | A new release requires identifiable sources and installed tests. Do not start Flathub automatically. |
| License and authorship | GPL-3.0-or-later and declared AI-assisted development. | Do not change the license or attributions, or promise nonexistent human review. |

Captions use explicit ownership in the canonical document: bookmarks for
images and captions, unlocked OOXML controls for tables. Show/hide
retains the text and the native history; captions are not attributed by proximity.
Navigation and decoration derive from that data. #132 was closed on 7/11/2026 after
David's confirmation with the original document; the trigger and the specific
fix remain unidentified.
[Implementation and limits](CAPTION_OWNERSHIP.md).

The [history](history/README.md) keeps the reasoning and the earlier deliveries.
The proposals in the [plan](../ROADMAP.md) can be chosen in a later
conversation; the decisions on this page must remain visible when doing so.
