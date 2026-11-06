# Caption ownership and native element tools — in validation

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
discovery. Full native/installed CI acceptance is pending. No issue is closed by
this checkpoint.

Existing imported or earlier captions without explicit object anchors remain
ordinary editable caption paragraphs. They are not silently assigned to nearby
objects. Numbered caption creation remains limited to the document body by the
pinned public engine API.

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
