# Alpha35: accepted workflows

Issues [170–182](https://github.com/frandavid100/leaf/issues?q=is%3Aissue+%22Accepted+by+David+in+the+planning+conversation%22) record the accepted proposals. All thirteen were completed after code verification and release asset verification.

WordCraft owns the document, model, protection, revision, history, layout and rendering. Leaf uses public commands and two registered engine commands for atomic block movement and font declaration substitution. It does not modify the engine or formats snapshot. The direct regex-automata dependency reuses the already locked version to expand captures using the engine's public pattern compiler/replacement parser. The CI boundary check compares alpha34’s existing direct dependencies, all non-Leaf lock packages, release profiles, overrides, pinned language revision and vendor/patch files. It allows this frontend reuse of an already locked crate without disabling the optional performance-baseline audits.

Fifteen headless regressions cover mixed block moves, note/image references, refusal rollback, scoped replacement, font substitution, template immutability and round trips. Real pointer checks additionally transfer a file from a separate native GTK process, use the external image clipboard, exercise Ctrl+Shift+V and drag a heading. Installed checks first assert the raw host image is unreadable and require GTK’s portal transfer.

New headless coverage: `tests/workflows_alpha35.rs`; native and installed controls: `--workflows35-ui-test`, `tools/workflows35-pointer-test.sh` and `tools/workflows35-installed-test.sh`. Existing regression drivers, including the original interaction pointer test, remain unchanged. New checks run in English and Spanish, with native HiDPI, installed GNOME 49 and the relocated FlatPark GNOME 51 prefix.

Regex capture expansion retains the original paragraph context, including anchors and tracked-deleted gaps, before any edits. Special-code and regex UI modes are mutually exclusive. Rich HTML/RTF clipboard formats retain priority when an external provider also advertises an image.

The shipped special-character control also has a native lifetime regression: it replaces selected entry text with the chosen code and releases its popup when the owner is dropped. Signal callbacks retain only a weak popup reference.

## Format painter rules

The ordered source selection’s first character (or the caret) supplies the appearance, including inherited paragraph/character style attributes. Source content, links and revision IDs are not copied. A caret destination receives word and paragraph formatting. A selected range receives text formatting; paragraph properties also apply when it starts at offset zero or spans paragraphs, following WordCraft’s public paste-format command. One-use clears after applying; repeated mode remains until Escape.

## Moving sections

A drag inserts the complete source section before the indicated heading. Drops
inside the source subtree are no-ops. The labelled Up/Down buttons also move
whole sections; Down can place a section after its final peer at the document
end. Heading levels are retained. TOC and navigation refresh from the canonical
moved blocks through the normal content-change path.

## Behavioral limits

- The pinned ODT exporter omits paragraph next-style choices. Leaf compares the effective choice with the importer’s built-in defaults and asks for DOCX when ODT would lose it. Custom style/DOCX round trips and the atomic ODT refusal are covered.
- A heading inside a table cannot itself define a movable section. A complete table inside a section travels intact.
- Moves crossing page-section breaks and moves while tracking changes are rejected atomically. Font substitution also requires tracking off.
- Selection search captures the current logical range. Leaf rebases its end after its own replacement; unrelated edits invalidate it and ask for a new selection.
- WordCraft omits empty regex hits. The adapter also refuses replacement of any empty hit before editing.
- Missing-font fallbacks come from the actual layout, not a guessed substitute.
- Templates use read-only access to the system XDG Templates folder or the folder portal, and open as new unsaved tabs. The source is never a save target. The bundle and relocated-prefix test declare the read-only XDG Templates grant; an existing external FlatPark manifest without that grant can use the folder chooser. No external store manifest update is claimed.
- Clipboard reads are cancellable and compare document identity, revision and selection. Drops use GDK FileList. Neither requires broad home access.

## Automatic delivery

The candidate waits up to 20 minutes for its exact native executable while the mandatory native suite continues. It still requires full native success and compares the stripped executable with the final validated artifact before publication. The package job has a 60-minute ceiling to include this bounded wait and the two installed runtimes.

The publisher also verifies that its candidate remains the current main source, so a superseded run cannot race a newer correction.

## Verification record

[Alpha.35](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.35) published automatically on 8 November 2026, after
[full native validation](https://github.com/frandavid100/leaf/actions/runs/42429643080), [installed GNOME 49/51 checks](https://github.com/frandavid100/leaf/actions/runs/42429972501)
and [promotion of the same bytes](https://github.com/frandavid100/leaf/actions/runs/42432915547) succeeded.

| Identity | Verified value |
| --- | --- |
| Release source/tag target | `1847466203cdaca2788fa15dcc2781788990732e` |
| Native source | `a738b76fd49b12acd649859626669e05561d7ca6` |
| Native run | `42429643080` |
| Candidate run | `42429972501` |
| Publisher run | `42432915547` |
| Public assets | 13, including 12 internally checksummed files and `SHA256SUMS` |

205 Rust tests, including 15 new workflow regressions, four Python tests and all
879 English/Spanish messages pass. Native controls and real pointer/clipboard
checks pass at scale 1/2; installed GNOME 49 and relocated GNOME 51 repeat the
portal file transfer, clipboard image, plain-paste shortcut, heading drag,
Undo/Redo, template restart and control checks. All existing mandatory native
and installed regressions also pass. This records automated testing, not human
GNOME OS validation.

The actual special-character popup leaked before the Weak callback fix; the
same shipped-control test fails before and passes afterwards. It additionally
verifies insertion over selected entry text. The final native source includes
this regression, rather than reusing the earlier executable.

The candidate compares its packaged executable byte-for-byte with the final
fully validated native artifact. The publisher verifies its exact source,
current-main identity, native success and every internal SHA-256 before creating
the prerelease without rebuilding. Release tag, candidate metadata and all
thirteen public asset digests were then checked against the verified files.

Issues #170–#182 were completed with individual implementation/verification
comments. The engine remains WordCraft 0.16.0. The older upstream issues and
David's deferred #52 are independent of this delivery.
