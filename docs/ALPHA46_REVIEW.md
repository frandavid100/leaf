# Alpha.46: code review for speed and clarity

Candidate prepared on 13/11/2026 at David's request, after a review of size,
speed and clarity. It closes no issues. WordCraft, its revision, dependencies,
patches and the release profile are unchanged.

## Changes

| PR | Change | Measured effect |
| --- | --- | --- |
| [#289](https://github.com/frandavid100/leaf/pull/289) | Review, Outline and thumbnails refresh after a typing pause (300/250/400 ms trailing debounce) instead of after every keystroke | Main-loop time between 30 keystrokes, 600 paragraphs: Review ~4,800 → ~680 ms; Outline ~815 → ~410 ms; Pages ~1,490 → ~460 ms |
| [#290](https://github.com/frandavid100/leaf/pull/290) | Fields and the table of contents are recomputed on a worker thread after a typing pause; the result is installed only if document, selection and pending format are unchanged | Longest stall at a pause: 139 → 72 ms (44 pages), 461 → 51 ms (131 pages) |
| [#287](https://github.com/frandavid100/leaf/pull/287) | Autosave keeps the preference in memory and runs the ODT check only in document-file mode, cached by revision | ~2.0 → ~0.8 µs per tick; ODT check (0.08–0.58 ms) no longer runs on idle ticks |
| [#288](https://github.com/frandavid100/leaf/pull/288) | CI: the workflows35 pointer test waits for keyboard focus before pasting | 5 failures in 9 runs → 0 in 20 |
| [#294](https://github.com/frandavid100/leaf/pull/294), [#296](https://github.com/frandavid100/leaf/pull/296) | Diagnostics declared in one module with an index; product code reaches hooks only through `diagnostics::flag` | No behaviour change |
| [#286](https://github.com/frandavid100/leaf/pull/286), [#297](https://github.com/frandavid100/leaf/pull/297), [#298](https://github.com/frandavid100/leaf/pull/298) | Historical reports archived in `docs/history/`; contributor documentation translated to English | Documentation only |

## Validation

- Each PR passed the full native CI suite before merging.
- Every behaviour-changing PR (#287, #289, #290, #294) was compared locally
  against the previous build under Xvfb: the 29 GUI checks invoked by CI gave
  identical results. For #290, a dedicated check confirmed that the
  cross-reference and the table of contents update, the caret is kept and one
  undo step is added.
- `cargo test --locked`: 248 tests.

## Limits

- Speed figures come from synthetic documents on a two-core virtual machine
  with the cairo software renderer. They show the relative change, not the
  absolute time on real hardware.
- Undo and Redo still refresh derived fields synchronously.
- Accessible-text rebuilding was measured at 2–3% of the per-keystroke cost and
  was deferred; Orca behaviour is unchanged.
- Human testing on GNOME OS is pending: David tests this candidate before any
  publication.
