# Alpha35: accepted workflows

Issues [170–182](https://github.com/frandavid100/leaf/issues?q=is%3Aissue+%22Accepted+by+David+in+the+planning+conversation%22) record the accepted proposals. They stay open until the corresponding code passes verification and is shipped.

WordCraft owns the document, model, protection, revision, history, layout and rendering. Leaf uses public commands and two registered engine commands for atomic block movement and font declaration substitution. It does not modify the engine or formats snapshot. The direct regex-automata dependency reuses the already locked version to expand captures using the engine's public pattern compiler/replacement parser.

Fifteen headless regressions cover mixed block moves, note/image references, refusal rollback, scoped replacement, font substitution, template immutability and round trips. Real pointer checks additionally transfer a file from a separate native GTK process, use the external image clipboard, exercise Ctrl+Shift+V and drag a heading. Installed checks first assert the raw host image is unreadable and require GTK’s portal transfer.

New headless coverage: `tests/workflows_alpha35.rs`; native and installed controls: `--workflows35-ui-test`, `tools/workflows35-pointer-test.sh` and `tools/workflows35-installed-test.sh`. Existing regression drivers, including the original interaction pointer test, remain unchanged. New checks run in English and Spanish, with native HiDPI, installed GNOME 49 and the relocated FlatPark GNOME 51 prefix.

Regex capture expansion retains the original paragraph context, including anchors and tracked-deleted gaps, before any edits. Special-code and regex UI modes are mutually exclusive. Rich HTML/RTF clipboard formats retain priority when an external provider also advertises an image.

## Format painter rules

The ordered source selection’s first character (or the caret) supplies the appearance, including inherited paragraph/character style attributes. Source content, links and revision IDs are not copied. A caret destination receives word and paragraph formatting. A selected range receives text formatting; paragraph properties also apply when it starts at offset zero or spans paragraphs, following WordCraft’s public paste-format command. One-use clears after applying; repeated mode remains until Escape.

## Behavioral limits

- The pinned ODT exporter omits paragraph next-style choices. Leaf compares the effective choice with the importer’s built-in defaults and asks for DOCX when ODT would lose it. Custom style/DOCX round trips and the atomic ODT refusal are covered.
- A heading inside a table cannot itself define a movable section. A complete table inside a section travels intact.
- Moves crossing page-section breaks and moves while tracking changes are rejected atomically. Font substitution also requires tracking off.
- Selection search captures the current logical range. Leaf rebases its end after its own replacement; unrelated edits invalidate it and ask for a new selection.
- WordCraft omits empty regex hits. The adapter also refuses replacement of any empty hit before editing.
- Missing-font fallbacks come from the actual layout, not a guessed substitute.
- Templates use read-only access to the system XDG Templates folder or the folder portal, and open as new unsaved tabs. The source is never a save target. The bundle and relocated-prefix test declare the read-only XDG Templates grant; an existing external FlatPark manifest without that grant can use the folder chooser. No external store manifest update is claimed.
- Clipboard reads are cancellable and compare document identity, revision and selection. Drops use GDK FileList. Neither requires broad home access.

## Verification record

In progress. All 205 local Rust tests pass, including 15 workflow regressions; four Python tests, all 879 en/es messages, document links and native GTK controls/real gestures at scale 1/2 pass. Template restart and the ODT preservation guard are covered. Exact source, CI run identities, release assets and final results are recorded here after publication.
