# Alpha.43 — focused formatting and contextual tools

## Scope and closure plan

Direct publication is explicitly authorized by David for this delivery; there is no
human RC approval step. All functional and installed checks remain required.
Closure after verified publication: #50, #236, #238, #240, #249, #250, #251,
#253, #254, #255, #256, #257, #258, #260, #263, #264, #266.

Start caches are tied to document identity rather than formatting revisions; stale
loads still check identity and revision before replacing live work. Sidebar width
is fixed at 240 logical pixels. Start's alignment references the resting single
page origin independently of a tab strip. Native combined close retains checked
save/discard semantics, with unique untitled labels and adaptive presentation.

Native split buttons expose last-result actions and independent non-mutating
options. Pinned panels retain their page, editing focus and scroll. Compact
underline/strike samples and the combined text-position chooser show current and
mixed values. Font search filters without applying on focus/typing; an explicit
choice applies. Installed-font previews use actual Pango glyph availability, and
missing document fonts preserve their identity. Installed families move between
the document and all-font groups, keeping one cached row per family. Language includes effective
Default, native/localized search, missing-dictionary explanation and footer tools.

All color properties reuse one compact 48-color palette, native custom editor and
bounded shared recent choices. Automatic text color and None highlight remain
semantic values. Picker controllers survive their builders without owning their
root widget cyclically; callbacks keep weak owners.

Repeat tracks bounded per-document supported formatting intents, newest first.
Choosing an older boundary freezes and replays its chronological suffix through
public engine commands, as one undoable user operation. Replaying an explicit
format does not toggle it. A completed batch becomes one stable recipe, avoiding
recursive growth. Document/revision/generation leases reject stale menus. Real
Redo takes precedence. File operations, clipboard contents and unsupported
structural operations are excluded; this is not universal command replay.

Ordinary Copy retains an immutable supported appearance payload only while Leaf
owns that exact clipboard offer. Paste Formatting transfers safe character and
resolved paragraph appearance, clears destination-only character properties,
preserves links and existing painter state, and inserts no text. Source list,
style and section identities are not transplanted across documents.

Paragraph controls separate alignment/direction, absolute/relative indents,
line/paragraph spacing, independent pagination toggles, sorting, tabs, view marks
and reset. Tab editing preserves inherited records until an explicit override.
Measurements follow Leaf's centimeter ruler (there is no alternate unit preference).
Formatting marks change only view state. Paragraph borders use a registered public
Session adapter so per-edge edits preserve other edges/shading and participate in
engine history. Engine/dependency/render pins and vendor patches are unchanged.

## Newly included proposals that remain open

| Issue | Delivered in this alpha | Remaining work / verified limit |
| --- | --- | --- |
| #252 | Compact line samples, current/mixed state, separate color | WordCraft models words-only as a single underline variant, so independent combinations with other line styles are unavailable. |
| #259 | Expanded stable Text/Paragraph/Elements catalog, direct property access, sizing and Copy → Paste Formatting | Complete primitive insertion/deletion/list sub-action inventory and exhaustive placement/entry parity still need completion. The overall issue stays open. |
| #261 | Three galleries, no-toggle reapply, explicit removal, level, compatible continuation, restart/start-at and native list-definition editor | Pinned engine toggles/restarts when converting between existing numbered designs. Unsafe conversion is disabled with an explanation; custom definitions are an explicit whole-definition operation. Custom-design reapply and individually configurable list sub-actions remain. |
| #262 | Fixed visible scope, five presets, persistent matches/custom status, actual page/margins preview, safe destination reapply | Full mixed-section presentation and immediate-detail interaction need another pass. Imported custom widths are preserved on opening. |
| #265 | Paragraph placements first, shared color on demand, style/width and per-edge preservation | Public table.borders changes the whole table and clears cell overrides; no independent inside-horizontal/vertical controls or scoped table preview are claimed. |
| #267 | Focused existing groups, immediate centimeter sizing/spacing, original size, visual non-destructive crop, transforms, alt/decorative metadata, distinct caption | Replacement policy/position organization and complete appearance controls remain. Border/transparency commands bake image pixels instead of preserving all editable properties. Canvas AT does not yet expose image descriptions as semantic children. |
| #268 | Scoped mixed dimensions, centimeter fields, exact/minimum row mode, table alignment/indent, distribution and separate vertical cell alignment/pagination | Cell-margin model data has no public editing command; table border scope above remains. Primitive toolbar access, full structure review and actual table-description accessibility integration are incomplete. |

An automatic height mixed with an explicit row height requires an explicit mode
and a confirmed numeric height; selecting a mode alone does not silently assign a
fabricated height. Exact rows keep their mode during value editing. Opening and
refreshing table properties do not mutate the document.

List continuation resolves compatible instances in the current story/container
and preserves the destination level. Choosing the active built-in design is a
no-op, never a toggle or restart. Removing a list preserves authored indents,
style, shading and text. A starting number is one Undo step and leaves preceding
items intact. List counters remain owned by WordCraft.

Cropping edits the engine's normalized source rectangle and display dimensions,
without changing original media bytes. The native draft supports pointer drag and
numeric alternatives, explicit confirmation/cancel and document/object revision
leases. Crop, transforms and alt/decorative metadata are verified through DOCX
save/reopen; this does not imply native assistive-technology integration.

## Validation and publication

Required: cargo fmt for the Leaf package, all Rust tests, Python tools tests,
complete English/Spanish gettext catalogs, current documentation links and an
optimized GUI build. CI must pass its five existing native regression groups and
installed GNOME 49/51 checks, including --alpha43-ui-test in both languages.

The new native probe exercises Start identity stability, current versus remembered
formatting, pinned pages, shared picker lifetime and real color activation,
Copy → Paste Formatting ownership, image group borrowing/restoration, crop draft
cancel/commit, alt/decorative save, list no-toggle/start/custom state, read-only and
mixed table properties, and the actual cumulative Repeat menu target. Headless
regressions cover repeat ordering/rollback/no-op/history, cross-document appearance,
paragraph edge preservation, list numbering and image DOCX roundtrips.

The publication workflow verifies optimized executable identity, source/tag and all
thirteen public asset digests before closing the explicit plan above. This report
records implementation and required checks; published identity and final workflow
results are recorded in LATEST_RELEASE.md after verification. Automated evidence
is not a claim of human GNOME OS, Wayland, Orca, braille or device testing.

## Existing external work and paused work

WordCraft remains 0.17.0 at 5574b32600aa7d782c64f5a6c6bb24696152220d.
#1/#99/#152 remain blocked on upstream integration; #231 is intermittent and
lacks an isolated trigger. The upstream permission limitation and prepared
contributions are retained from alpha.42. No vendor fork is introduced to bypass
these gaps. #185, description/public captures, Weblate and additional accessibility
projects retain David's explicit postponement. No Windows package is generated.
