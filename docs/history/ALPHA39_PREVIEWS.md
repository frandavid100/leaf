# Alpha.39 — start previews and parallel release validation

Published and verified: #212, #213, #214 are completed. See [published identity and validation](../LATEST_RELEASE.md).

## Resulting behavior

Recovery cards show the first page of the actual pending snapshot, including
unsaved edits. A bounded worker copies opaque snapshot files into a private
scratch directory and uses the public canonical recovery API on that copy.
It never locks, restores, writes, prunes or consumes the original record.
The temporary Session and files are dropped after rendering. There is no client
JSON document parser or parallel document model. Invalid, missing, changed or
oversized previews leave the original actionable document fallback in place.
Async application validates gallery ownership, document identity/revision and
the inventory generation, and cancellation cannot replace newer work or focus.

Template and recovery cards share a fixed preview slot, while the paper itself
retains its real first-page proportions. Long labels cannot stretch the paper
background. Raster previews use twice the logical card resolution and the same
public first-page renderer as document views. The supplied garden DOCX was
compared privately: the pre-fix raster already matched the opened first page;
the visible defect was allocation/background stretching by its long title.
The document and its rendered contents are not committed to the public project.

The fixed bottom Navigation/Review controls retain native checked backgrounds,
keyboard focus and mutually exclusive accessible toggle state. Hover-only CSS
continues to apply exclusively to the upper representation/filter controls.

## Validation and preserved scope

`tests/start_recovery.rs` compares the preview with the restored unsaved snapshot,
checking original bytes, inventory availability, stale timestamps and invalid IDs.
`tests/start_previews.rs` compares whole-page pixels, page proportions, warm cache,
file replacement, original bytes and unchanged document/history state.
The existing start-view real-input driver additionally checks recovery pictures,
allocated paper proportions with long names and selected-mode pixels in light/dark
styles, hover and focus. English and Spanish/HiDPI remain mandatory in native,
installed GNOME 49 and relocated-prefix GNOME 51 runs.

WordCraft 0.16.0, Cargo.lock, engine/render revisions, formats patches, permissions
and Windows alpha.36-windows.1 remain unchanged. The seven earlier blocked or
deferred reports are not closed by these preview/CI changes. Public screenshots,
Weblate and store integration retain David's existing decisions.

## Release validation graph

A single optimized GUI build supplies five isolated native regression runners:
editing, documents, text, lifecycle and appearance. The validated executable
artifact is produced only after all groups succeed, and is stripped from the
original compiled bytes. Installed validation compares its packaged executable
against that final artifact exactly as before.

GNOME 49 and relocated GNOME 51 use separate runners. The latter downloads the
exact already-built archive; it does not rebuild the app. Its complete previous
battery must finish successfully before the candidate checksum manifest can be
produced. Failure/cancellation is propagated, and each runner keeps its evidence.
All original native and installed commands are accounted for in the
[coverage comparison](ALPHA39_CI_COVERAGE.json). Repeated environment checks remain
where they exercise different packaging/runtime conditions; no assertion was
removed merely because it resembled a native check.

The broad screenshot sweep waits for explicit readiness instead of sleeping
seven seconds per scene. Readiness requires all registered demo setup callbacks
to finish, a painted document, completed pending layout and identical actual
window/popover/dialog rendering across consecutive frames. The helper is opt-in
for capture processes and retains weak ownership and bounded host-side deadlines.
Ordinary editing and diagnostics retain their existing scheduling.

After publishing the validated bytes, `tools/finalize_alpha.py` verifies native
and candidate identities, source/tag, all public asset digests, checksum manifest
and notes. Only then does it close the explicit candidate issue list and update
published status/history and the Linux download link. A non-forced ref update
preserves concurrent work; Windows links and historical decisions are retained.
The publication verifier has negative tests for failed validation, wrong sources,
missing/pending/empty assets, changed digests, manifests, notes and issue plans.

Alpha.38 baseline: native 38m52s, installed candidate 32m24s, overlapped validation
about 45 minutes, publication 17s. These are one-run observations; alpha.39 timings
must be measured after real CI completion, not inferred from the new graph.

## Isolated-runner resource correction

The initial split run detected missing generated gettext catalogs on the text
runner: Spanish correctly failed its existing translated-label assertion. Every
runner now checks and compiles the same catalogs before starting GUI checks.
The original assertion is retained. The stable `test` Rust cache namespace is
shared by the builder and runners; only the builder saves it, preserving warm
pre-split dependencies and avoiding competing cache uploads. A complete fresh
validation passed before publication; the failed candidate was not promoted.

## Completed validation and observed time

216 Rust tests, 15 Python tests and 891 complete English/Spanish messages passed.
Every native group and both installed runtimes passed, followed by the exact
executable comparison and verification of all thirteen public assets. Issues
#212/#213/#214 were closed after that verification.

- Final successful validation to publication: 32m32s; alpha.38: 45m24s.
- Same forty-scene desktop sweep: 2m00s; alpha.38: 5m21s.
- Corrected shared-cache build: 9m12s; the initial cold build was 12m52s.
- [Complete native run](https://github.com/frandavid100/leaf/actions/runs/42531382692).
- [Installed candidate](https://github.com/frandavid100/leaf/actions/runs/42531440034).
- [Verified publication and automatic closure](https://github.com/frandavid100/leaf/actions/runs/42534051115).
- [Observed timing data and limits](ALPHA39_CI_METRICS.json).

The initial configuration attempt correctly failed the Spanish-label check
because generated catalogs were missing on the isolated runner. It was discarded,
the setup was corrected, and a complete fresh validation passed. The final-run
duration above excludes that discarded attempt and implementation time. These
are individual CI observations, not a guaranteed time for every future release
or a claim of improved document editing performance.
