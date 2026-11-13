# Bookmarks and comments

## Authorized scope

Create from **Insert** and the context menu. Since alpha.15, bookmarks appear
in **Navigation** under the last preceding heading; they are hidden when it is
collapsed. Those before the first heading appear at the start, and documents
without headings still show their bookmarks. Creating one opens Navigation.
The third tab is dedicated to **Comments**. Each thread shares one bubble with
horizontal separators between messages and vertical guides for replies,
without separate cards. Model, history and serialization belong to WordCraft;
Leaf only adds adapters, controls and save protection.

- Bookmarks: unique visible name, creation at the cursor or a body selection,
  navigation to the range and deletion. Reserved names starting with `_` stay
  hidden and are not created from this interface.
- Comments: create on a selection/word, edit, reply, resolve/reopen and delete
  with confirmation. Deleting a thread also deletes its replies; document
  operations can be undone/redone.
- Navigation selects the referenced text without marking the document as
  modified. Replies inherit the anchor; comments without a visible anchor are
  shown without an active navigation button. Missing parents and imported
  cycles are tolerated.
- Modal GTK dialogs, cancel with Escape and submit text with Ctrl+Enter.
  Ctrl+Shift+F5 adds a bookmark; Ctrl+Alt+M adds a comment from the document.
- Views are rebuilt after changes or when the panel opens. Weak callbacks,
  document revision/identity, and closing the editor when a tab is released.

The author is still the WordCraft session's; no new identity preference is
added. Margin balloons and any further Orca stage are not implemented.

## Formats: important limit

**DOCX:** check bookmark names/ranges, comments, replies, Unicode text and
resolved state after saving/reopening.

**ODT with the pinned WordCraft:** the interchange model reduces a bookmark to
`Inline::Anchor`, written as `text:bookmark`; on import it is rebuilt as a
point bookmark. It keeps the name and location but loses the original range.
Comment objects have no representation in that model, and the ODT importer
omits annotations. Do not claim ODT comment compatibility.

Leaf refuses an ODT save if the session contains comments, before writing the
file or clearing the modified state. Save suggests the DOCX extension. The
same protected save serves direct autosave; when the ODT target is
incompatible, native recovery is kept and a warning is shown once per error.
The comments dialog warns that DOCX is needed.

### Reproduction ready for upstream

`tests/annotations.rs::odt_bookmark_locations_survive_but_comments_must_not_be_silently_saved`
builds a small document, creates a bookmark on «Café», exports ODT and reopens
it: the name remains, but the selection is now a point. It then adds a comment,
checks that Leaf protects the original file, and deliberately uses
`wordcraft_engine::io::save_bytes("upstream.odt", &session.doc)` to reproduce
the comment being dropped outside Leaf's save path.

Affected upstream code, revision `165d5725dc25b4a7887d8025c3b1fe506ccc3dcc`:
`crates/formats/src/model/from_doc.rs`, `model/to_doc.rs`, `odt/writer.rs` and
`odt/body.rs`. No engine, formats, patches, revisions or dependencies are
modified. The reproduction is prepared; no upstream issue has been opened from
here.

## Checks

The cumulative native validation of the alpha.14 base has passed: 105 Rust tests, four
Python tests, formatting, catalogs, links, optimized build and the full GTK round,
including English/Spanish, light/dark, narrow windows and HiDPI.
[CI for the final source](https://github.com/frandavid100/leaf/actions/runs/41341927920).
The local environment does not allow sockets for Xvfb; the graphical checks have
been run in CI. The [installed Flatpak](https://github.com/frandavid100/leaf/actions/runs/41343316411)
has passed annotations, review, update from alpha.13, reinstallation
with data, portals and the Orca/AT-SPI regression.
[Alpha.14 is published](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.14);
public download and checksums have been verified.
[Log](BOOKMARKS_COMMENTS_VALIDATION.json).

```sh
cargo test --locked --test annotations
cargo build --release --locked --features gui --bin wordcraft-gnome
LANGUAGE=es GSK_RENDERER=cairo xvfb-run -a ./target/release/wordcraft-gnome --annotations-ui-test
```

Five headless regressions cover history, duplicate names, Unicode, navigation,
editing restrictions, DOCX preservation and ODT protection.
The GTK check triggers the real Insert action, a context menu row, reply
buttons and edit/resolve actions, undo/redo, search/panel switching and
rejection of a stale dialog. It also checks that the editor is closed when the
document is released. CI runs English and Spanish and keeps internal
screenshots of wide, narrow and dark windows. Public screenshots remain David's
responsibility.

The alpha.15 revision adds a bookmark-ownership regression (nested headings,
preface and level changes/undo) and checks in GTK the collapsing, navigation
and a single bubble with separators per thread. The
[native CI and the installed Flatpak](https://github.com/frandavid100/leaf/actions/runs/41348755238)
have passed.
[Alpha.15 published](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.15);
[cumulative log](SIDEBAR_REFINEMENT_VALIDATION.json).

Map: [adapter](../src/annotations.rs), [panel](../src/gnome/annotations.rs),
[dialogs](../src/gnome/annotation_dialog.rs), [tests](../tests/annotations.rs),
[GTK diagnostics](../src/gnome/diagnostics/annotations.rs).
