# Leaf architecture and code map

Leaf is a GTK4/libadwaita host for the pinned WordCraft document engine.
The active branch is **wordcraft-saving**. Start with [current state](../PROJECT_STATUS.md)
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
   schedules coalesced thumbnails/search/outline updates and updates the tab state.
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
| Mouse, keyboard, input methods | [input.rs](../src/gnome/input.rs) | [selection.rs](../src/selection.rs), [editing.rs](../src/editing.rs); selection/editing root tests. |
| Edit/format refresh | [commands.rs](../src/gnome/commands.rs), [selection_view.rs](../src/gnome/selection_view.rs) | [formatting.rs](../src/formatting.rs); advanced_formatting tests. |
| Open/save/close confirmation | [files.rs](../src/gnome/files.rs), [workspace.rs](../src/gnome/workspace.rs) | [saving.rs](../src/saving.rs); saving/recovery_search tests. |
| Pages, zoom, thumbnails, HiDPI | [view.rs](../src/gnome/view.rs), [paintable.rs](../src/gnome/paintable.rs) | [viewer_layout.rs](../src/viewer_layout.rs); hidpi tests and `--host-cache-ui-test`. |
| Search and headings | [search.rs](../src/gnome/search.rs), [navigation.rs](../src/gnome/navigation.rs) | [search.rs](../src/search.rs), [navigation.rs](../src/navigation.rs); `--search-ui-test`, `--nav-ui-test`. |
| Bookmarks and comments | Bookmarks in [navigation.rs](../src/gnome/navigation.rs); comment threads in [annotations.rs](../src/gnome/annotations.rs), [annotation_dialog.rs](../src/gnome/annotation_dialog.rs) | [annotations.rs](../src/annotations.rs); annotations tests and `--annotations-ui-test`. Save guard in saving.rs, recovery fallback in workspace.rs. |
| Styles | [styles.rs](../src/gnome/styles.rs) | [styles.rs](../src/styles.rs); styles tests and `--styles-ui-test`. |
| Paragraph/text panels | [paragraph.rs](../src/gnome/paragraph.rs), [format_controls.rs](../src/gnome/format_controls.rs), [adaptive.rs](../src/gnome/adaptive.rs) | [formatting.rs](../src/formatting.rs); formatting tests, `--paragraph-ui-test`. |
| Context, pinning and contained pages | [context.rs](../src/gnome/context.rs), [popovers.rs](../src/gnome/popovers.rs) | Context pointer, `--pin-ui-test`, `--adaptive-ui-test` in CI. |
| Tables | [tables.rs](../src/gnome/tables.rs) | [tables.rs](../src/tables.rs); tables tests and real Tab/Shift+Tab input. |
| Images, page properties, PDF | [document_tools.rs](../src/gnome/document_tools.rs) | [pictures.rs](../src/pictures.rs), [document_properties.rs](../src/document_properties.rs), [pdf_export.rs](../src/pdf_export.rs); document_tools tests. |
| Headers, numbering, columns | [headers.rs](../src/gnome/headers.rs), [header_options.rs](../src/gnome/header_options.rs), [columns.rs](../src/gnome/columns.rs) | [header_footer.rs](../src/header_footer.rs), [page_options.rs](../src/page_options.rs); headers/page_options/columns tests. |
| Local spelling and languages | [spellcheck.rs](../src/gnome/spellcheck.rs) | [proofing.rs](../src/proofing.rs); proofing tests and `--proofing-ui-test` (sidebar, Preferences and stale worker rejection). |
| Printing | [printing.rs](../src/gnome/printing.rs) | [printing.rs](../src/printing.rs); [print harness](../tools/print_test.sh). |
| Accessible text and focus | [canvas.rs](../src/gnome/canvas.rs), [accessibility.rs](../src/gnome/accessibility.rs) | [accessible_text.rs](../src/accessible_text.rs); [AT-SPI/Orca audit](../tools/accessibility_audit.py). |
| Localization | [i18n.rs](../src/i18n.rs), [po](../po) | [update_translations.py](../tools/update_translations.py), [translation_source.py](../tools/translation_source.py). |
| CSS, widget composition | [appearance.rs](../src/gnome/appearance.rs), [tab.rs](../src/gnome/tab.rs) | Existing screenshots, narrow/wide/dark UI and focus regressions. |

Shared widget lookup is named `find_widget`; it is not a styles-only operation.
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
