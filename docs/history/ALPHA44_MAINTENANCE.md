# Alpha.44 — frontend maintenance, native colors and Windows

Published and verified; this maintenance release claims no issue closures. See [published identity and validation](../LATEST_RELEASE.md).

## Authorized scope

David requests conservative frontend simplification with the existing appearance
and behavior preserved, then explicitly restores the standard GNOME color
selectors and authorizes a new release plus a Windows installer for friends.
The intentional visual change is the color chooser. WordCraft work continues
separately; its pinned revision, dependencies and vendor patches stay untouched.
Store screenshots and description (#185), Weblate and new accessibility projects
retain their postponement. This maintenance delivery claims no issue closures.

## Implementation for reviewers

Toolbar overflow builds its translated catalog once per group instead of once
per hidden tool. Menu filtering likewise reads the persisted placement list once
per group and reuses it across its rows. Ordering, stable preference identities,
visibility, native controls and document targeting are unchanged.

Diagnostic registration captures each common owner/callback signature in one
small runner. The original flags, delays, registration order and first-tab guards
are retained; arguments are read once for this registry. Every diagnostic remains
in the executable, so verification still exercises the exact distributed code.
No test removal or separate untested production configuration is introduced.

The custom 48-cell palette, application recent-color registry, painted swatches
and controller registry are replaced by GTK's standard ColorChooserWidget, as
used before alpha.43. GTK owns its palette and custom editor. Each property keeps
its Automatic/None choice and last-value semantics. Synchronization is guarded,
explicit same-color activation remains a user choice, and native callbacks retain
their small guards/action without retaining the root widget or controller.
Mixed selections remain described by the property state; previews do not edit.

The Windows workflow uses the current package version in its release notes.
It builds the optimized program with the same engine revision, assembles the
runtime and resources, checks a relocated ZIP without MSYS2 on PATH, then verifies
per-user installation, the installed program and removal before publication.
Windows packages use a separate `-windows.1` tag and retain the approved icon.

The publication finalizer now describes an empty closure plan accurately, with
an explicit regression ensuring a maintenance report never claims empty closures.

## Measurement and verification

[Local measurement data](ALPHA44_MEASUREMENTS.json): final executable
45,735,432 bytes, a reduction of 77,152 bytes (0.1684%) from the matched baseline.
Rust source is 58,000 lines, 285 fewer; all diagnostics remain available.
The settled default-document view is pixel-identical in the checked 1140×840
capture. Native color controls are the explicitly requested visual exception.
The final optimized native probe also passes in Spanish at scale 2.
These measurements are local, not an assertion of the public artifact’s exact
size on another compiler/platform.


The local baseline is alpha.43 source
`19baf2bb29b42153bfa6ec2d9f2046f8171c39ef`, compiled with the unchanged release
profile, Rust 1.99.1, GTK 4.18/libadwaita 1.7 on the same Linux environment.
Its executable is 45,812,584 bytes; the Rust source has 58,285 physical lines,
including 16,129 diagnostic lines. Source-line proportions do not predict binary
size, and installed Windows runtime size is a separate measurement.

The final baseline/candidate comparison runs both executable orders (AB/BA), fresh
profiles, 120 and 600 synthetic paragraphs, Xvfb/Cairo and the same engine/assets.
All eight cases preserve the canonical document and agree on page counts.
Individual phases move in both directions, so this does not claim a universal
speed improvement. The cursor-movement sample is not every editing workflow.

Reproduction uses the existing `--performance-ui-test` with
`LEAF_PERF_PARAGRAPHS` and `LEAF_PERF_REPORT`; use fresh config/cache/state folders,
matched inputs and both execution orders. Keep compiler, profile and inputs fixed when comparing; the candidate includes
both the structural maintenance and the requested native color restoration.

Local native checks cover color activation, reapplication, controller lifetime,
paragraph borders/shading, and all 41 Format136 pointer stages, including native
custom RGB entry and one-step Undo. Final release verification also requires all
Rust/Python checks, complete catalogs, the five native CI groups and installed
GNOME 49/51 validation. The Windows workflow checks both the portable package
and installer independently. Publication identity and digests are recorded only
after those gates succeed; no human/device testing is implied.
