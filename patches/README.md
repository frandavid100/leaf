# WordCraft formats compatibility delta

## Current: WordCraft 0.16.0

Upstream: https://github.com/storytold/wordcraft at
`02a802cbafe45b5c8fecc00490c8f6b78c5ba251`, crate wordcraft-formats 0.16.0
(MIT OR Apache-2.0). The complete formats crate was recopied from that revision;
its licenses are retained. Engine, document, layout and rendering crates are
unmodified upstream dependencies at the same revision.

The four 0.14 patches below are historical and are **not** applied to this tree.
Their replacements are the smaller source delta in
[wordcraft-formats-0.16-compat.patch](wordcraft-formats-0.16-compat.patch).
The standalone Cargo manifest resolves upstream workspace dependencies at the
same WordCraft revision and preserves their ranges/features, including the new
ODT crypto dependencies. It is recorded separately from the source patch.

To reproduce: copy `crates/formats` at the revision above, copy LICENSE-MIT and
LICENSE-APACHE from the repository root, generate the standalone manifest, and
run `patch -p1 < ../../patches/wordcraft-formats-0.16-compat.patch` from the vendor
crate. No unrelated importer/exporter or engine code is patched.

The residual compatibility changes preserve:

- Editable header/footer links, disabled variants and otherwise unreferenced
  parts. ODF master pages receive the inherited content; link metadata is only
  restored if the exported copy still agrees with its predecessor. Independently
  edited content stays independent; duplicate parts alone are discarded.
- Writer-compatible dynamic header/footer reservations, standard geometry taking
  priority over stale cached distances, and exceptional overlapping nominal
  margins. Ordinary geometry uses standard ODF properties.
- No Proof overrides and the original language of unchecked text. Standard
  `fo:language="zxx"` interoperates with ODF readers; supplemental metadata retains
  the language for re-enabling review and accepts files from earlier Leaf releases.

WordCraft 0.16 now supplies the other original preservation work, including page
parts with tables/images and native page fields. The existing small regressions
remain enabled. Additional tests check retained ODT objects, custom column widths,
bookmark ranges and comment replies. Guarded features are listed in
[the integration review](../docs/WORDCRAFT_016_REVIEW.md); do not infer complete
ODF parity from these targeted checks.

## Historical delta for WordCraft 0.14.0

# Isolated WordCraft formats compatibility patch

Upstream: https://github.com/storytold/wordcraft at 5cd53f184aab11def13b7fced18727159b9bc96a, crate wordcraft-formats 0.14.0 (MIT OR Apache-2.0).

Cargo overrides only this crate with vendor/wordcraft-formats. The snapshot retains upstream source and license notices; the ODT exporter/importer, the common flow adapter for page fields and the standalone Cargo manifest differ. All other engine crates stay on the existing Git revision. This is an explicit exception to the earlier unmodified-engine state, justified by confirmed ODT data loss. It fixes all engine save paths, including autosave, without replacing the engine's atomic save or session bookkeeping.

wordcraft-formats-odt.patch records the exporter change. Cargo.toml expands inherited workspace version, license and dependencies into a standalone manifest with the same pinned doc/geom sources and dependency ranges. The standalone manifest follows upstream dependencies, including wordcraft-metafile in 0.14.0.

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

## WordCraft 0.14.0 upgrade — alpha.21

The formats source was recopied from v0.14.0, commit
`5cd53f184aab11def13b7fced18727159b9bc96a`. All four patches above apply cleanly,
in the documented order; their ODT changes are still absent upstream. The
standalone manifest now includes upstream's wordcraft-metafile dependency.
The lock diff changes only WordCraft packages (0.12.0 to 0.14.0), with the same
Rust 1.95 minimum and craft-libs PDF revision. The 141 Leaf Rust tests pass,
including ODT properties/geometry, fields, DOCX preservation and history.
