# #99: table-cell note below the footer

This is an independently reproduced engine defect, tracked in
[Leaf #99](https://github.com/frandavid100/leaf/issues/99). The fixture contains
only invented text; the supplied private document is neither needed nor
published. WordCraft 0.16.0 `02a802cbafe45b5c8fecc00490c8f6b78c5ba251`.
The relevant table-flow, notes and page sources are unchanged on main
`729cd0a5fc953bbedbfdec4f6af18ad6d1780b3b`.

## Reproduce

Run the [public diagnostic](../../examples/pending_notes_audit.rs):

```sh
bash tools/prepare-language-data.sh
export CRAFT_I18N_DIR="$PWD/.build-deps/craft-i18n" CRAFT_I18N_REQUIRED=1
LEAF_NOTE_REPRO_DIR=/tmp/leaf-note-repro cargo run --locked --example pending_notes_audit
```

The export directory receives a newly generated synthetic DOCX, PDF and PNG.
The optional positional DOCX input is only measured in memory; it is not exported.
The audit prints observations rather than asserting that a known defect passes.

The smallest currently isolated public case uses A4 (595.3 × 841.9 pt), 28.35 pt
margins on all sides, two columns with a 14.15 pt gap, and a 24 × 2 table.
Each cell holds five repetitions of “Ordinary table cell content. ”.
The footer is “An ordinary footer.”, 28.35 pt from the page edge.
A real footnote “A short footnote.” is referenced in zero-based row 12, cell 0,
paragraph 0, offset 20.

On the local reference run, the first-page note text ends at 823.2219 pt,
while the footer text starts at 799.5813 pt: about 23.64 pt of reversed order.
The rendered PNG and PDF show the separator crossing the footer and the note
below it. These coordinates describe this font environment, not portable
expected constants. The invariant is that note text/separator stay above the
actual footer.

Footer distances 35.45 and 60 pt preserve the same relative defect. Growing the
note, Undo/Redo, and a completely fresh Session layout also reproduce it.
Ordinary paragraph-only controls with one/three footer lines do not expose
this table case. Warm and cold geometry match: this is not evidence of stale
GTK textures.

Measurements of a continued paragraph use the placed fragment's origin:
`y + last_line.top - first_line.top + last_line.height`. Using the entire
paragraph's last-line top without subtracting the fragment's first-line top
would incorrectly report overflow on continuation pages.

## Suspected reservation order

In `crates/layout/src/table_flow.rs`, `place_table` calls `place` before
`table_notes` for both the last row fragment and split fragments.
`table_notes` then lowers `PageBuilder::bottom` to reserve the note area.
`flush_notes` in `page.rs` uses the maximum of that boundary and
`body_floor()`; already placed table rows can force the note below the footer.

This is a cause hypothesis supported by the code and reproduction, not a tested
engine correction. A fix needs preflight reservation and repagination/splitting
while keeping reference and note paired. Moving the note upward over body text,
clipping it, or changing the user's footer/margins would conceal the defect.

Related upstream work was checked: #450 / #483 fixed missing table-cell notes;
#3974 / PR #5293 concern an empty section-closing paragraph, absent here;
#4273 concerns general note/reference pagination. No equivalent table-reservation
fix was found in those proposals. No engine patch is added to Leaf.

## Submission boundary

Creating the upstream report through the connected GitHub integration returned
HTTP 403, “Resource not accessible by integration”. No upstream issue was
created. The public reproduction and these findings remain available in Leaf;
#99 stays open until the engine correction can be validated.

