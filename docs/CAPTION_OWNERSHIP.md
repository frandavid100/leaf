# Caption ownership and native element tools — validated candidate

Authorized scope: #121, #124, #126 and #127; investigate the original overlap #132.
No engine, renderer, dependency or existing formats-patch changes.

## Canonical ownership

Picture ownership uses a named native bookmark range containing that exact image.
Table ownership uses an unlocked standard OOXML whole-table content-control tag,
so deleting its first row does not erase the owner. A separately named bookmark
identifies the caption paragraph. Ambiguous, copied, missing or deleted anchors
are not matched to neighboring objects. These are canonical document structures,
preserved by DOCX, not a parallel GTK document model or a proximity heuristic.

Native references.caption creates the editable styled paragraph and SEQ field.
The Leaf command registers through Session, preserving history and protection.
Hidden captions retain their text and fields using native hidden run/mark
properties; original hidden formatting is retained in a document custom property
and restored on showing. DOCX retains both states; ODT loss guards remain active.
Repeating Show reuses the existing caption. Picture drops carry their owner range
inside the same history transaction as the native arrange operation. Nested-table
captions are placed after the actual nested table, in its parent cell.

Image/Table properties expose Show Caption above insertion. Header/context entry
points reuse the same controls. The standalone Caption menu item is removed.
Caption numbering is currently supported in the document body; the checkbox is
disabled with an explanation for objects in other stories.

## Derived navigation and decoration

Navigation uses caption text to label its owner, shows the related caption as a
distinct indented entry, and uses symbolic content-type icons. Explicit typed
activation distinguishes a table and an image sharing a paragraph position;
table cell selection identifies the actual table, including nested tables.

Hover/caret backing is derived from canonical ownership and native layout, cached
by document identity/revision. It is a translucent editor overlay; it does not
change layout, history, document shading or exports. Related pieces on separate
pages remain separate. A short connecting backing is allowed only for adjacent,
clear object/caption geometry on the same page.

## Validation so far

Nine focused headless regressions pass: Show/Hide with authored text and native
Undo/Redo after DOCX; two identical images and drag/reopen; whole-table ownership
after deleting the first row; no-target/deleted-object safety; protection with no
partial anchors/history; nested-table placement; typed read-only navigation; and
read-only cue geometry with hidden-caption exclusion; cross-references after
toggle/reopen and identical exported raster pixels before/after reading cues.
Real GTK pointer/keyboard checks pass in English light/1× and Spanish dark/2×,
including native typing, pinned off/on, Undo/Redo, switching identical images,
table-caption caret context, DOCX reopening, typed navigation and hover/caret
discovery. Full native and installed CI pass. This delivery completes #121, #124,
#126 and #127; #132 remains open.

Existing imported or earlier captions without explicit object anchors remain
ordinary editable caption paragraphs. They are not silently assigned to nearby
objects. Numbered caption creation remains limited to the document body by the
pinned public engine API.

### Accepted candidate and provenance

- Candidate source: `da5b3f96c672eceb41218855e73ffba0897c126d`.
- Native source: `486a8b2c2050f09a6ffd31ce47cacab48e5051f8`.
- [Full native success](https://github.com/frandavid100/leaf/actions/runs/42302407273):
  187 Rust tests, four Python tests, gettext and complete GTK regressions.
- [Installed candidate success](https://github.com/frandavid100/leaf/actions/runs/42304211527):
  GNOME 49 Flatpak, desktop/file/print portal and data-preserving upgrade checks,
  existing AT-SPI/Orca checks, and the exact FlatPark archive relocated under
  GNOME 51. Formatting and caption controls pass in English light/1× and Spanish
  dark/2× on both runtimes. Original font/clipboard/canvas assertions are unchanged.
- [Downloaded artifact](https://github.com/frandavid100/leaf/actions/runs/42304211527/artifacts/13547105434):
  ZIP SHA-256 `c965f39d7fb83400fd72d8672a6cfdc29c84e10510c291ff6d6737dbcd233a37`.
  All eleven entries in `SHA256SUMS` verify after download; `candidate.json`
  identifies the above sources and runs and requires maintainer approval.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `Leaf-0.1.0-alpha.31-x86_64.flatpak` | 11,467,528 | `b192c00f400682984fd9a64893bf1c2e6f6cd512f087cc124d335af609f68644` |
| `Leaf-0.1.0-alpha.31-flatpark-x86_64.tar.xz` | 11,550,064 | `7f265cbba09a65df648ce5f5d95fed7f4db2f646a80d0b91e55269037105c129` |

The final stripped executable is 37,649,248 bytes, SHA-256
`c57552a532472f9ee922d6be9323d679b9711379e88b4e56fe44a673e8ad28bb`.
CI compares the packaged native input with the fully validated optimized binary.
Independent extraction of the FlatPark archive matches the same stripped binary.
The early native artifact has SHA-256
`4956e19a4fd97b8d9d74380060065150bb9287d89bfc802284b378bb15964f44`;
applying the existing final `strip --strip-unneeded` reproduces the final hash.
No rebuild is needed or authorized when promoting these accepted files.

### Earlier attempts and input-driver correction

Native run 42283745462 exposed the table-helper regression that required an
existing table even for Insert Table; PR #148 restores independent creation and
the unchanged context/column-selection test. Native runs 42287362018 and
42295229994 then passed their complete suites. Earlier packages from sources
`97194123d1658b920d8feefc2261c892960c63f1`,
`fc5e51e94c482317a97c7dbd1277c7c1407ad3cd` and
`0153cd23368778f5065a2e9d0864f8c5f2ffe830` were not accepted for delivery.

A real GTK motion handshake alone did not make the first installed formatting
click reliable. Captures showed the Text menu still visible; canvas focus in an
assertion did not establish where a click was delivered. Extra logging made
isolated controls pass and was therefore investigative evidence, not acceptance.

PR #154 keeps final target motion adjacent to the single press, with no repeated
clicks or additional waits. A shell trace separately located a no-op `mousemove
--sync` stall before a caption click; removing that redundant XTest wait retains
the stronger actual GTK entry acknowledgment. Native typing, selection, history
and header assertions remain intact. Installed comparison 42302372784 passes two
independent workers, four full formatting sequences and two caption regressions
without input logging. The final full native and installed runs above also pass.
Hidden canonical ownership anchors stay out of the user bookmark chooser, while
an authored bookmark remains selectable after DOCX reopening.

This is an unpublished candidate for David to test. Automated X11 checks are not
human Wayland/IME validation. Existing publication approval requirements remain.

## #132 investigation

Original DOCX SHA-256:
`85d7e722d989657ce5941ef5410e84cd4acca1891a88cac75ccdc0dbf91216b7`.
The original private document and screenshot were inspected locally. Neither is
copied into the public repository or CI. The current line probe produces two
separate lines in paragraph 4 (baselines 11 and 27.773901 points), with matching
PNG/PDF output. Native Alpha.30 and Alpha.31 controls also display separate lines,
including a Flatpak-style Noto Color Emoji font and opening paragraph spacing.
The supplied screenshot establishes a real failure, but its exact triggering
editing state is still missing. #132 remains open; no engine cause is invented.
