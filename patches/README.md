# Isolated WordCraft formats compatibility patch

Upstream: https://github.com/storytold/wordcraft at 165d5725dc25b4a7887d8025c3b1fe506ccc3dcc, crate wordcraft-formats 0.12.0 (MIT OR Apache-2.0).

Cargo overrides only this crate with vendor/wordcraft-formats. The snapshot retains upstream source and license notices; the ODT exporter/importer, the common flow adapter for page fields and the standalone Cargo manifest differ. All other engine crates stay on the existing Git revision. This is an explicit exception to the earlier unmodified-engine state, justified by confirmed ODT data loss. It fixes all engine save paths, including autosave, without replacing the engine's atomic save or session bookkeeping.

wordcraft-formats-odt.patch records the exporter change. Cargo.toml expands inherited workspace version, license and dependencies into a standalone manifest with the same pinned doc/geom sources and dependency ranges. No new runtime libraries are introduced.

On an engine update: compare the upstream ODT implementation first. If fixed upstream, remove the Cargo patch and vendor snapshot, regenerate Cargo.lock, and run tests/odt_preservation.rs plus the affected corpus cases. Otherwise recopy this crate at the new fixed revision, reapply/review both preservation patches, regenerate the standalone manifest/lock and validate. Never silently keep a vendor snapshot from a different engine revision.

The exporter shares one Writer across body and page parts so automatic-style and picture identifiers cannot collide. Both XML style scopes receive those definitions. Page parts include nested tables and picture resources. The ODT-only export view disables the common flow model's visual-header heuristic; actual RowProps.header flags remain. Other format converters are unchanged.

Unused imported header/footer stories are retained in unreferenced master pages. Disabled first/even content is also retained there, without enabling those modes; its disabled-slot association is an existing importer limitation. This is a targeted preservation fix, not full ODF page-style support.


## Automatic page fields — alpha.9

`wordcraft-formats-page-fields.patch` preserves unlocked PAGE and NUMPAGES fields with plain instructions. The flow adapter keeps the cached result and a small field marker, ODT writes standard text:page-number / text:page-count elements, and import reconstructs field objects with their formatting. Adjacent fields remain separate. Other converters continue to export the cached result through their existing text path. Locked fields, visible field codes and instructions with switches keep the earlier fallback behavior; this does not claim general ODF field support.

Run `cargo test --locked --test header_footer --test odt_preservation` after reapplying/removing these patches. The root tests exercise both DOCX and ODT through the engine IO boundary. No dependency revision or renderer changes are needed.

## Section options and text languages — alpha.10

Apply wordcraft-formats-properties-language.patch after the existing two patches. Standard ODF language attributes and page number formats preserve the selected text language and numbering. Inactive first/even parts remain attached without enabling their modes. Leaf extension attributes retain original unchecked-text languages, exact engine header/footer distances and per-variant section-link masks. Standard header/footer content is still exported for other editors; another editor can ignore or strip the additional attributes. This is not a promise of exact geometry or link semantics across every ODF application. No new engine revision or runtime library is introduced.

Import restores a link only when the exported copy matches its predecessor, and discards only those redundant copies, to avoid accumulating duplicate parts over repeated saves. Changed external content stays independent. Run page_options, proofing, header_footer and odt_preservation root tests after updates, including repeated roundtrips. The common flow model now carries language/no-proof metadata; unsupported foreign formats continue through their existing exporters.
