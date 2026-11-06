# Maintainability review — 1 November 2026

Requested by David to make Leaf easier to modify and resume from another chat.
The review starts from `6221ecfcd3e709f22842f5deaf8c34d4e759d18a` (published
alpha.13 plus delivery notes). This is a source/documentation maintenance change;
no new Flatpak version or feature release is claimed.

## Changes

- Replace the GTK include-based shared namespace with private Rust modules under
  src/gnome. The executable entry falls from 1,986 lines to 7. Application startup,
  window/tab ownership, shared state, commands, persistence, view geometry/raster
  caching, input and feature panels have distinct files.
- Move real-widget tests, synthetic probes, startup demos and registration into
  src/gnome/diagnostics. Existing CLI flags, widget identities and commands remain.
  Typed borrowed controls make the diagnostic registration dependencies explicit.
- Apply standard Rust formatting to Leaf-owned source/tests/examples. Remove two
  unused private helpers and make widget lookup's shared name find_widget explicit.
  Preserve weak lifecycle guards and the renderer/model boundary.
- Support multiline translation calls emitted by rustfmt, with four focused Python
  regressions. All 562 existing message identities and placeholders are preserved.
- Replace chronological state/roadmap documents with short current documents.
  Preserve previous state, roadmap, integration notes, first-release checklist and
  usage notes under docs/history. Rewrite contribution, architecture, handoff,
  decisions, usage, upgrade and release instructions to match current behavior.
- Add CI checks for Leaf package formatting, translation extraction and local links
  in the authoritative onboarding documents. Update the default-branch landing,
  contribution and agent pointers to the active branch and guides.

Cargo.toml, Cargo.lock, vendor, patches and data remain unchanged from the baseline.
WordCraft, its renderer and formats compatibility patches have not been rewritten.
Public screenshots and Weblate setup remain with David. No new Orca features,
corpus rerun, dependency update or engine optimization is part of this review.

## Validation

Final compiled source: `3d5e6826ec6311b23dcaac638a580cf76be2d04e`.
Native CI [41320879043](https://github.com/frandavid100/leaf/actions/runs/41320879043),
job 124890615799: package formatting, four Python regressions, documentation links,
562-message English/Spanish catalogs, **99 Rust tests**, optimized GUI build,
headers/page options/proofing, cache invalidation, lifecycle, keyboard/printing,
context actions, pinning/adaptation, page/selection screenshots and HiDPI checks passed.
The unchanged renderer's costly debug/release benchmark and full corpus were not repeated.

Native artifact 13089771888 was independently downloaded and verified, ZIP SHA256
`5fad9ce579592c106611868164b18a04d10d5037a6132f66df25dbcc75284822`.
Its internal document screenshot was reviewed; public screenshots were not replaced.

Real AT-SPI/Orca audit [41321948699](https://github.com/frandavid100/leaf/actions/runs/41321948699),
job 124893854375, validates the same source-identical binary after the GTK classes
moved. Artifact 13089628134 was independently downloaded and verified, ZIP SHA256
`022dbf4329802f50c41086c9c28779dc1864c453b8eb8fdbde727ac9e9eb0029`.
Orca 46.1 generated paragraph, selection, typed-character and restored-text
announcements; the application checked exact document restoration after undo.
This remains automated evidence, not human listening or braille validation.

## Deliberate remaining boundaries

Ui is still a shared tab coordinator and the private facade still supplies common
imports/helpers. The panels are not independent libraries. Further reductions
should be feature-sized and tested, rather than an unrequested host rewrite.
Four pre-existing deprecated GTK API warnings remain (allocated_width and CSS
load_from_data); changing their geometry/compatibility behavior is separate work.
The [architecture](ARCHITECTURE.md) documents the ownership/focus/cache contracts.

Use [current state](../PROJECT_STATUS.md), [roadmap](../ROADMAP.md) and
[continuation guide](CONTINUING.md) to resume work. Historical snapshots explain
past decisions but do not reauthorize postponed tasks.
