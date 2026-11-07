# Alpha35: accepted workflows

Issues [170–182](https://github.com/frandavid100/leaf/issues?q=is%3Aissue+label%3Aenhancement) record the accepted proposals. They stay open until the corresponding code passes verification and is shipped.

WordCraft owns the document, model, protection, revision, history, layout and rendering. Leaf uses public commands and two registered engine commands for atomic block movement and font declaration substitution. It does not modify the engine or formats snapshot. The direct regex dependency reuses the already locked version to expand captures using the engine's public pattern compiler/replacement parser.

Thirteen headless regressions cover mixed block moves, note/image references, refusal rollback, scoped replacement, font substitution, template immutability and round trips. Real pointer checks additionally transfer a file from a separate native GTK process, use the external image clipboard, exercise Ctrl+Shift+V and drag a heading. Installed checks first assert the raw host image is unreadable and require GTK’s portal transfer.

New headless coverage: `tests/workflows_alpha35.rs`; native and installed controls: `--workflows35-ui-test`, `tools/workflows35-pointer-test.sh` and `tools/workflows35-installed-test.sh`. Existing regression drivers, including the original interaction pointer test, remain unchanged. New checks run in English and Spanish, with native HiDPI, installed GNOME 49 and the relocated FlatPark GNOME 51 prefix.

## Behavioral limits

- The pinned ODT exporter omits paragraph next-style choices. Leaf compares the effective choice with the importer’s built-in defaults and asks for DOCX when ODT would lose it. Custom style/DOCX round trips and the atomic ODT refusal are covered.
- A heading inside a table cannot itself define a movable section. A complete table inside a section travels intact.
- Moves crossing page-section breaks and moves while tracking changes are rejected atomically. Font substitution also requires tracking off.
- Selection search captures the current logical range. Leaf rebases its end after its own replacement; unrelated edits invalidate it and ask for a new selection.
- WordCraft omits empty regex hits. The adapter also refuses replacement of any empty hit before editing.
- Missing-font fallbacks come from the actual layout, not a guessed substitute.
- Templates use read-only access to the system XDG Templates folder or the folder portal, and open as new unsaved tabs. The source is never a save target.
- Clipboard reads are cancellable and compare document identity, revision and selection. Drops use GDK FileList. Neither requires broad home access.

## Verification record

In progress. Local full `cargo test --locked`, four Python tests, complete en/es catalogs and document link checks passed before native UI verification. Exact source, CI run identities, release assets and final results are recorded here after publication.
