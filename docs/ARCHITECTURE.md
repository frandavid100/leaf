# Leaf architecture and code map

Leaf is a GTK4/libadwaita host for the pinned WordCraft document engine.
The active branch is **main**. Start with [current state](../PROJECT_STATUS.md)
and [contribution instructions](../CONTRIBUTING.md); past release details live
in [history](history/README.md), not in the active roadmap.

## Layers

| Layer | Source | Responsibility |
| --- | --- | --- |
| Executable entry | [src/bin/gnome.rs](../src/bin/gnome.rs) | Calls the private native frontend; keeps the existing Cargo target name. |
| Native application | [src/gnome/mod.rs](../src/gnome/mod.rs), [application.rs](../src/gnome/application.rs) | Shared imports/facade, application ID/version, CLI and startup. |
| Window and tab ownership | [workspace.rs](../src/gnome/workspace.rs), [state.rs](../src/gnome/state.rs), [tab.rs](../src/gnome/tab.rs) | Window owners, one session per tab, widget construction and cleanup. |
| Native views/controllers | Files under [src/gnome](../src/gnome) | GTK widgets, signals, focus, dialogs and disposable view caches. |
| GTK-independent adapters | [src/lib.rs](../src/lib.rs) and the other top-level src modules | Public WordCraft commands and layout/format adapters. Testable without GTK. |
| Upstream model/layout/render | Pinned Cargo git dependencies; existing [formats exception](../patches/README.md) | Canonical document, edits, history, pagination, rendering and file formats. |
| Graphical diagnostics | [src/gnome/diagnostics](../src/gnome/diagnostics) | Opt-in real-widget tests, synthetic workloads and screenshot/demo setup. |

The GTK frontend now uses real private Rust modules, not source pasted into the
binary with include!. Child visibility is limited to the frontend (`pub(super)`)
and the binary does not expose a public library API. The facade supplies common
GTK imports and shared helpers to those modules. `Ui` still coordinates panels:
this is not a claim that every panel is an independent library. Prefer explicit
imports when adding new cross-module dependencies; avoid restoring the old
include-based namespace.

Selection-dependent formatting uses `src/text_properties.rs`; native immediate
controls and undo coalescing live in `src/gnome/text_controls.rs`. Link edits use
`src/links.rs` through public command registration and the native dialog in
`src/gnome/links.rs`. Issue-sweep controls are exercised in the opt-in
`src/gnome/diagnostics/issues.rs`; no parallel text or history model is added.

## Ownership and data flow

`Workspace` owns the window, tabs and strong `Rc<Ui>` owners. `Ui.host` points
back weakly. `Current` owns that tab's WordCraft `Session`, logical page sizes,
arrangement and disposable GTK raster/overlay caches. Widget callbacks, workers
and timers must preserve the weak/cancellable ownership established by the
[lifecycle audit](GTK_LIFECYCLE_AUDIT_2026-10-31.md).

The normal edit path is:

1. GTK input or an action calls a handler in [commands.rs](../src/gnome/commands.rs),
   context or a panel.
2. A top-level adapter invokes the public engine command on the current Session.
3. `after_content_change` updates geometry, invalidates the necessary view caches,
   schedules coalesced thumbnails/search/outline and derived-reference updates,
   and updates the tab state.
4. [selection_view.rs](../src/gnome/selection_view.rs) updates overlays and controls;
   [canvas.rs](../src/gnome/canvas.rs) synchronizes the derived accessible text.

Navigation and pending formatting can be selection/view changes without editing
the document. Preserve that distinction: dirty state and undo history belong to
WordCraft, not to a second GTK model. Printing/PDF, previews and background
proofing use isolated copies where needed and must not save or mutate the live
session while inspecting it.

## Where to make a change

| Feature | GTK implementation | Engine adapter / regression |
| --- | --- | --- |
| Mouse, keyboard, input methods | [pointer_input.rs](../src/gnome/pointer_input.rs), [keyboard_input.rs](../src/gnome/keyboard_input.rs), [selection_drag.rs](../src/gnome/selection_drag.rs) | [selection.rs](../src/selection.rs), [editing.rs](../src/editing.rs); selection/editing root tests. |
| Native selection appearance | [selection_style.rs](../src/gnome/selection_style.rs) | [selection_ink.rs](../src/selection_ink.rs); disposable public display-list ink, bounded raster cache, theme colors and no document mutation. |
| History and repeat | [history.rs](../src/gnome/history.rs) | [history.rs](../src/history.rs); real engine labels/snapshots, cumulative traversal, bounded formatting replay, stale targets and `tools/sweep20-pointer-test.sh`. |
| Viewport ruler | [ruler.rs](../src/gnome/ruler.rs) | [ruler.rs](../src/ruler.rs); public placed-line geometry and indent/tab commands, selection leases, ruler tests and real pointer/keyboard checks. |
| Edit/format refresh | [commands.rs](../src/gnome/commands.rs), [selection_view.rs](../src/gnome/selection_view.rs) | [formatting.rs](../src/formatting.rs); advanced_formatting tests. |
| Open/save/close confirmation | [files.rs](../src/gnome/files.rs), [workspace.rs](../src/gnome/workspace.rs) | [saving.rs](../src/saving.rs); saving/recovery_search tests. |
| Pages, zoom, thumbnails, HiDPI | [view.rs](../src/gnome/view.rs), [paintable.rs](../src/gnome/paintable.rs) | [viewer_layout.rs](../src/viewer_layout.rs); hidpi tests and `--host-cache-ui-test`. |
| Navigation representations | [navigation.rs](../src/gnome/navigation.rs), [tab.rs](../src/gnome/tab.rs), [view.rs](../src/gnome/view.rs) | One primary Navigation entry; per-tab Pages/Structure state and independent scroll; read-only switching and guarded target reveal. `--alpha30-ui-test` includes #131. |
| Search and headings | [search.rs](../src/gnome/search.rs), [navigation.rs](../src/gnome/navigation.rs) | [search.rs](../src/search.rs), [navigation.rs](../src/navigation.rs); `--search-ui-test`, `--nav-ui-test`. |
| Bookmarks and comments | Bookmarks in [navigation.rs](../src/gnome/navigation.rs); comment threads in [annotations.rs](../src/gnome/annotations.rs), [annotation_dialog.rs](../src/gnome/annotation_dialog.rs) | [annotations.rs](../src/annotations.rs); annotations tests and `--annotations-ui-test`. Save guard in saving.rs, recovery fallback in workspace.rs. |
| Unified Review | [review.rs](../src/gnome/review.rs), [annotations.rs](../src/gnome/annotations.rs), [spellcheck.rs](../src/gnome/spellcheck.rs) | [review.rs](../src/review.rs) and public revision snapshots/commands; filters and source leases; `--review29-ui-test`. |
| Shared Elements | [elements.rs](../src/gnome/elements.rs), [references.rs](../src/gnome/references.rs) | Shared native table/image bodies, contextual TOC controls and stable creation; restoration on close/teardown. |
| Styles | [styles.rs](../src/gnome/styles.rs) | [styles.rs](../src/styles.rs); styles tests and `--styles-ui-test`. |
| Paragraph/text panels | [paragraph.rs](../src/gnome/paragraph.rs), [format_controls.rs](../src/gnome/format_controls.rs), [adaptive.rs](../src/gnome/adaptive.rs), [text_palette.rs](../src/gnome/text_palette.rs) | [formatting.rs](../src/formatting.rs); formatting tests, `--paragraph-ui-test`. |
| Clipboard | [clipboard.rs](../src/gnome/clipboard.rs) | Public engine fragments plus HTML/RTF/plain text; one cancellable read per tab, weak ownership and identity/revision/selection leases. `--clipboard-ui-test` checks external providers, one-step Undo and stale reads. |
| Context, pinning and contained pages | [context.rs](../src/gnome/context.rs), [popovers.rs](../src/gnome/popovers.rs) | Context pointer, `--pin-ui-test`, `--adaptive-ui-test` in CI. |
| Tables | [tables.rs](../src/gnome/tables.rs), [table_pointer.rs](../src/gnome/table_pointer.rs) | [tables.rs](../src/tables.rs); tables tests, real Tab/Shift+Tab input and tools/table-pointer-test.sh (public engine hit targets, one edit per drag). |
| Footnotes, contents, captions and cross-references | [references.rs](../src/gnome/references.rs) and [adapter](../src/references.rs) | Canonical engine stories/fields, section-aware note options, note/reference navigation and DOCX/history regressions. |
| Images, page properties/metadata, PDF | [picture_controls.rs](../src/gnome/picture_controls.rs), [document_properties_dialog.rs](../src/gnome/document_properties_dialog.rs), [pdf_dialog.rs](../src/gnome/pdf_dialog.rs); [shared operations](../src/gnome/document_tools.rs) | [pictures.rs](../src/pictures.rs), [document_properties.rs](../src/document_properties.rs), [pdf_export.rs](../src/pdf_export.rs); document_tools tests. |
| Headers, numbering, columns | [headers.rs](../src/gnome/headers.rs), [header_options.rs](../src/gnome/header_options.rs), [columns.rs](../src/gnome/columns.rs) | [header_footer.rs](../src/header_footer.rs), [page_options.rs](../src/page_options.rs); headers/page_options/columns tests. |
| Local spelling and languages | [spellcheck.rs](../src/gnome/spellcheck.rs) | [proofing.rs](../src/proofing.rs), [proofing_defaults.rs](../src/proofing_defaults.rs); inheritance and proofing tests and `--proofing-ui-test` (sidebar, Preferences and stale worker rejection). |
| Printing | [printing.rs](../src/gnome/printing.rs) | [printing.rs](../src/printing.rs); [print harness](../tools/print_test.sh). |
| Accessible text and focus | [canvas.rs](../src/gnome/canvas.rs), [accessibility.rs](../src/gnome/accessibility.rs) | [accessible_text.rs](../src/accessible_text.rs); [AT-SPI/Orca audit](../tools/accessibility_audit.py). |
| Localization | [i18n.rs](../src/i18n.rs), [po](../po) | [update_translations.py](../tools/update_translations.py), [translation_source.py](../tools/translation_source.py). |
| CSS, widget composition | [appearance.rs](../src/gnome/appearance.rs), [tab.rs](../src/gnome/tab.rs) | Existing screenshots, narrow/wide/dark UI and focus regressions. |

Shared widget lookup (`find_widget`) and box clearing live in
[widgets.rs](../src/gnome/widgets.rs), independently of the Styles panel.
Paragraph glyphs live in [paragraph_icons.rs](../src/gnome/paragraph_icons.rs);
paragraph.rs owns the settings form.
Graphical checks are grouped under diagnostics with the same existing CLI flags.
[register.rs](../src/gnome/diagnostics/register.rs) receives typed borrowed controls
and installs only the explicitly requested diagnostic callbacks. Startup/motion
checks remain at their original registration points.

## Invariants to preserve

- Keep engine revisions, Cargo.lock and existing formats patches unchanged for
  frontend work. Propose engine changes upstream; see [updating the base](../UPDATING_WORDCRAFT.md).
- Run document edits through Session commands. Selection, dirty state, redo and
  protection must not be bypassed by writing GTK-side copies back into a document.
- Keep coordinate systems distinct: layout/pointer/caret use logical units;
  [paintable.rs](../src/gnome/paintable.rs) handles the display-density raster.
- Selection/search invalidation must erase both previous and current pages.
  Header guides deliberately repaint all affected pages. Thumbnail bounds are
  read after allocation; coincident new-widget bounds are not all visible pages.
- Never hold a RefCell borrow while synchronously notifying GTK accessible text.
  Its text index is derived/read-only, not another editable document.
- Popover close/unparent and action dispatch have ordering constraints; preserve
  the deferred native-context cleanup and the shared pin/focus behavior.
- Validate identity/revision before applying async dialog/worker results. Cancel
  or ignore stale work after closing/replacing a tab; release all owned resources.
- Regenerate translation references after moving or formatting source. Message
  identity and placeholders must remain stable; multiline macros are supported.

## Validation and navigation

See [CONTRIBUTING](../CONTRIBUTING.md) for commands. Rust unit/integration tests
remain under [tests](../tests); opt-in GUI diagnostics live separately. The normal
CI builds an optimized GUI and exercises actual native controls. Optional corpus,
Writer, performance and Orca audits are described by their own reports/workflows;
they are not all a requirement for every change.

Historical reports refer to the source paths of their own commit. Use this map
for current locations. A later cleanup can further reduce the shared Ui/facade,
but should be done in tested, feature-sized changes rather than replacing the
host or altering engine semantics during unrelated work.

Fit Width stores an effective scale per page, shared within each section. Current::page_zoom is the coordinate conversion used by rendering, overlays, pointer hit testing, rulers and navigation. Percentage zoom remains uniform. Thumbnails use their own fixed-width scale and cannot change the sidebar width. Picture drags preview in the overlay and commit one arrange.bounds command.

## Derived document fields and Alpha29 controls

`src/references.rs` invokes public reference commands on a disposable Session
copy, temporarily protecting non-derived fields. Only computed document values
are transferred back; source history is preserved. The native coalesced timer
in `src/gnome/references.rs` defers while a dialog or drag is active and is
cancelled on tab cleanup. Source undo/redo recomputes these values. Save As
prepares fields using its destination filename and restores the live document
on write failure. PDF and printing prepare isolated copies using the document
path. They never write back to the live session.

`src/gnome/picture_pointer.rs` owns a disposable drag preview and all eight
handles. Shared autoscroll updates the target using current page geometry.
`src/pictures.rs::drop_at` verifies that the public arrange command really
placed a floating image on the requested page, restoring the original snapshot
if the pinned engine cannot honor it. No engine/layout fork is introduced.

`src/equations.rs` and `src/gnome/equations.rs` expose basic UnicodeMath through
public equation commands. Imported math must survive a linear round-trip before
editing is enabled. The Alpha29 diagnostic and pointer driver exercise these
controls in English/Spanish and normal/HiDPI, including the installed Flatpak.
See [candidates](CANDIDATES.md) for the separate human approval/publication step.


## Alpha31 derived review state

SpellReview keeps a disposable snapshot only for finding remapping. A changed
paragraph's retained findings must match current public proofing output; an
overlapping or ambiguous changed span is discarded. Suggestions are recomputed
for changed paragraphs. Cards are derived widgets, not a second document model.
Their actions validate the live document id and the revalidated finding; cached
comments/revisions resolve their canonical targets without weakening asynchronous
dialog revision guards. Worker generations/revisions guard both success and error.

The current Alpha31 candidate retains aligned sidebar/document headers, uses a
native Pages/Structure dropdown, and shares the Text/Paragraph/Style/Elements
submenu framework. Earlier candidates and their issue lists are historical.
Review cards and findings are derived state; no parallel editable document
model is introduced.

## Explicit caption ownership

`src/captions.rs` uses canonical picture bookmark ranges, whole-table content
controls and caption paragraph bookmarks. Native hidden properties retain
authored caption text; the public Session command owns history and protection.
`src/gnome/caption_cues.rs` draws disposable theme-aware backing from native
layout, cached by document identity/revision. Navigation uses typed targets to
distinguish tables and their nested pictures. See [ownership and validation](CAPTION_OWNERSHIP.md).


## Alpha32 disposable indexes and clipboard

`reference_geometry.rs` retains only canonical TOC ranges, keyed by document id
and revision. Caret, hover and drawing lookups reuse the index; content changes,
Undo/Redo and replacement rebuild it on demand. Actual TOC activation still
validates the stored bookmark through the core adapter. Caption cues share an
immutable `Rc<[Cue]>` instead of copying every rectangle for each paint.
`refresh_selection` updates the ruler through `update_status` once. The derived
reference adapter checks body/table/part blocks for eligible fields before
creating its disposable Session copy.

`clipboard.rs` owns native format negotiation independently of context menus.
Internal fragments are reusable only while GTK owns that exact provider.
External HTML/RTF reads are asynchronous and bounded to 8 MiB; plain text uses
GDK decoding. A read holds a weak Ui, is cancelled on replacement/close or a new
paste, and validates document id, revision and selection before the public paste
command. Rich bytes are temporary input, never a parallel document or history.

## Dictionary resources

[src/dictionaries.rs](../src/dictionaries.rs) inventories readable Hunspell sources
and owns the disposable merged index. Language pickers inspect file metadata
and read permissions; they do not parse unrelated dictionaries. WordCraft owns
format validation and spelling when a language is used.
[src/gnome/dictionaries.rs](../src/gnome/dictionaries.rs) owns the weak/cancellable
folder and import UI; the native/Flatpak chooser returns the authorized path.
[Issue #84](ISSUE84_DICTIONARIES.md) records precedence, checks and GNOME OS limits.
