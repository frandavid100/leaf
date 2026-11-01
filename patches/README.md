# Isolated WordCraft formats compatibility patch

Upstream: https://github.com/storytold/wordcraft at 165d5725dc25b4a7887d8025c3b1fe506ccc3dcc, crate wordcraft-formats 0.12.0 (MIT OR Apache-2.0).

Cargo overrides only this crate with vendor/wordcraft-formats. The snapshot retains upstream source and license notices; the ODT exporter/importer, the common flow adapter for page fields and the standalone Cargo manifest differ. All other engine crates stay on the existing Git revision. This is an explicit exception to the earlier unmodified-engine state, justified by confirmed ODT data loss. It fixes all engine save paths, including autosave, without replacing the engine's atomic save or session bookkeeping.

wordcraft-formats-odt.patch records the exporter change. Cargo.toml expands inherited workspace version, license and dependencies into a standalone manifest with the same pinned doc/geom sources and dependency ranges. No new runtime libraries are introduced.

On an engine update: compare the upstream ODT implementation first. If fixed upstream, remove the Cargo patch and vendor snapshot, regenerate Cargo.lock, and run tests/odt_preservation.rs plus the affected corpus cases. Otherwise recopy this crate at the new fixed revision, reapply/review the four compatibility patches, regenerate the standalone manifest/lock and validate. Never silently keep a vendor snapshot from a different engine revision.

The exporter shares one Writer across body and page parts so automatic-style and picture identifiers cannot collide. Both XML style scopes receive those definitions. Page parts include nested tables and picture resources. The ODT-only export view disables the common flow model's visual-header heuristic; actual RowProps.header flags remain. Other format converters are unchanged.

Unused imported header/footer stories are retained in unreferenced master pages. The initial patch retained disabled first/even content there without enabling those modes; alpha.10 additionally preserves its disabled-slot association. This is a targeted preservation fix, not full ODF page-style support.


## Automatic page fields — alpha.9

`wordcraft-formats-page-fields.patch` preserves unlocked PAGE and NUMPAGES fields with plain instructions. The flow adapter keeps the cached result and a small field marker, ODT writes standard text:page-number / text:page-count elements, and import reconstructs field objects with their formatting. Adjacent fields remain separate. Other converters continue to export the cached result through their existing text path. Locked fields, visible field codes and instructions with switches keep the earlier fallback behavior; this does not claim general ODF field support.

Run `cargo test --locked --test header_footer --test odt_preservation` after reapplying/removing these patches. The root tests exercise both DOCX and ODT through the engine IO boundary. No dependency revision or renderer changes are needed.

## Section options and text languages — alpha.10

Apply wordcraft-formats-properties-language.patch after the existing two patches. Standard ODF language attributes and page number formats preserve the selected text language and numbering. Inactive first/even parts remain attached without enabling their modes. Leaf extension attributes retain original unchecked-text languages, exact engine header/footer distances and per-variant section-link masks. Standard header/footer content is still exported for other editors; another editor can ignore or strip the additional attributes. This is not a promise of exact geometry or link semantics across every ODF application. No new engine revision or runtime library is introduced.

Import restores a link only when the exported copy matches its predecessor, and discards only those redundant copies, to avoid accumulating duplicate parts over repeated saves. Changed external content stays independent. Run page_options, proofing, header_footer and odt_preservation root tests after updates, including repeated roundtrips. The common flow model now carries language/no-proof metadata; unsupported foreign formats continue through their existing exporters.

## Standard ODF header/footer geometry — alpha.11

Apply `wordcraft-formats-standard-geometry.patch` fourth, after all three earlier
patches. ODF page margins measure to the outside of the header/footer; WordCraft
stores body margins plus separate page-edge distances. Export writes standard
header/footer reservations and dynamic spacing; import reads active master-page
reservations, including independent Writer ODT files. Stale private distance and
number-format attributes cannot override supported standard properties changed
by another editor. No layout engine, renderer or dependency revision changes.

Normal header/footer positions and body margins no longer depend on Leaf
attributes. Distances for inactive headers and overlapping nominal body margins
still need supplemental metadata; exotic numbering and section links retain
their earlier limitations. This does not implement every ODF border, padding,
fixed-height or non-dynamic-spacing layout rule.

Run `cargo test --locked --test odt_geometry --test page_options --test proofing
--test header_footer --test odt_preservation`. The tiny synthetic ODT fixtures
include standard-only geometry, inactive parts and stale extension values.
The optional `interop-audit.yml` workflow performs independent Writer edits and
standard-only reopens; it is not required on every release.
