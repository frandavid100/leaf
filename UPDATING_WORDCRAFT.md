# Updating the WordCraft base

Leaf consumes public WordCraft crates at a pinned git revision. Frontend changes
belong in Leaf; proposed engine/render optimizations belong upstream. An engine
upgrade is a separate, explicitly authorized task, not part of routine cleanup.

## Current boundary

- Engine/doc/render: `165d5725dc25b4a7887d8025c3b1fe506ccc3dcc` in [Cargo.toml](Cargo.toml).
- PDF backend patch: the craft-libs revision declared in Cargo.toml.
- Existing compatibility exception: [vendor/wordcraft-formats](vendor/wordcraft-formats),
  with four isolated ODT preservation/properties patches documented in
  [patches/README.md](patches/README.md). Other engine crates/render are unmodified.
- GTK lives in [src/gnome](src/gnome); the top-level [src library](src/lib.rs)
  contains GTK-independent adapters. [Architecture](docs/ARCHITECTURE.md) maps each feature.

The source of truth is Session/Document. Public commands own mutation, protection,
dirty state and history; DocLayout owns geometry. Leaf controls native input,
views, caching, worker/dialog lifecycle and when to request those public APIs.
Do not introduce a second editable model or patch the engine for UI convenience.

## Authorized upgrade procedure

1. Inspect upstream changes and its MSRV/krilla requirements before choosing a revision.
2. Update WordCraft revisions together in Cargo.toml and vendor/wordcraft-formats/Cargo.toml.
3. Recopy the formats crate from that exact revision and reapply the four patches
   in the recorded order, or remove patches now fixed upstream. Keep licenses and
   provenance. Never carry an unrelated stale vendored crate across the upgrade.
4. Regenerate Cargo.lock and inspect the dependency diff. A revision bump is not
   presumed source-compatible; adapt public API changes on the Leaf side.
5. Run relevant [tests/build checks](CONTRIBUTING.md), including affected format,
   selection/history and GTK paths. Use small roundtrip reproductions for altered
   import/export behavior. The independent corpus/Writer audits are optional
   when justified, not mandatory for every release.
6. Check recovery/explicit discard, independent tabs, focus, clipboard and async
   completion after closing/replacing a tab. Preserve the invariants below.
7. For an actual release, follow the [release checklist](docs/RELEASE_CHECKLIST.md).
   Update any explicitly pinned native-artifact reuse guard to the newly validated
   source/run; never bypass its source-identity comparison.

## Integration contracts to recheck

| Area | Contract | Evidence / affected checks |
| --- | --- | --- |
| Tables | Existing-cell navigation uses pure caret/selection commands; upstream text.tab mutates when adding a final row. Preserve dirty/redo boundaries. | tables tests and real keyboard Tab/Shift+Tab. |
| Styles | Preview on an isolated sample/session, never the live document. CRUD uses public commands. | styles tests and native popover diagnostics. |
| Headers/fields | Edit the story displayed on the requested page; preserve page hints, body return selection, linked-part semantics and existing footer content when inserting a number. | [Headers](docs/HEADERS_FOOTERS.md), header_footer/page_options/odt_geometry tests. |
| Formatting/page setup | Preserve selection/pending format and use atomic undo/rollback for multi-section operations. | document_tools/page_options/advanced_formatting tests. |
| Search/clipboard | Identity/revision checks before applying stale async results; native clipboard provider ownership controls rich fragments. | recovery_search tests, real contextual row activation. |
| HiDPI | Logical geometry remains separate from raster density; keep the allocation cap and premultiplied buffers. | hidpi tests, native scale 1/2 and installed check. |
| Screen appearance | Screen-only RenderOptions.dark; file export/printing keep document colors. | [Dark-mode report](docs/LOCALIZATION_DARK_MODE.md). |
| Accessible text | Derived active-story index; Unicode offsets map to engine bytes. Old-delete/new-insert notifications must not hold RefCell borrows. | accessible_text tests and [AT-SPI/Orca](docs/ACCESSIBLE_DOCUMENT_TEXT.md). |
| Printing/PDF | Export an isolated document copy; retain temporary resources/application hold until native callbacks finish. | printing tests and portal success/cancel/error harness. |
| Lifetimes | Workspace owners, weak callbacks/back references, removable/cancellable workers and explicit tab teardown. | [GTK audit](docs/GTK_LIFECYCLE_AUDIT_2026-10-31.md). |

## Frontend details that are not engine work

The context menu's deferred unparenting preserves native action dispatch.
Popover pinning never changes modality on a mapped surface; contained contextual
pages restore borrowed controls before the host is detached. Preserve explicit
focus return and scroll position. AdwSplitButton internals and the GtkPopoverMenu
stack hierarchy are small GTK/libadwaita integration boundaries to recheck when
upgrading those libraries.

The [host performance report](docs/HOST_PERFORMANCE_AUDIT.md) explains bounded
thumbnail rasters, pre-allocation fallback, previous/new overlay invalidation and
inactive search. Keep those policies in Leaf. Measure matched inputs/conditions;
API-boundary spans include adapter overhead and are not exclusive engine CPU time.

The release profile uses speed optimization, ThinLTO, one codegen unit and symbol
stripping. Preserve panic unwinding because upstream uses catch_unwind. Ship
optimized release binaries. Current package identity/runtime/permissions and
validation are in [PROJECT_STATUS](PROJECT_STATUS.md), not hardcoded historical
run IDs in this guide.

[Earlier detailed integration notes](docs/history/UPDATING_WORDCRAFT_BEFORE_REORGANIZATION.md)
remain archived with their revisions. They can explain a past decision, but
current locations and policies are those linked above.
