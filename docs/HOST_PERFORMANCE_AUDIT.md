# Leaf host diagnosis and optimization

Request: measure performance and fix only the Leaf layer. Any possible
WordCraft change must be proposed upstream, without modifying it here.

## Method

The alpha.12 host is first instrumented without changing its behavior, and that
optimized executable is kept as the reference. It is compared with the candidate
on the same Ubuntu 24.04 runner, GTK/Xvfb and GSK Cairo, using AB/BA ordering
and two fresh processes per version and size. Each process uses a temporary
profile and only synthetic documents of 120, 600 or 1800 paragraphs with
accented text.

Fixture generation/saving is excluded from import timings. DOCX and ODT are
imported from freshly created files (warm disk cache), the first layout is
measured, and then the initial widgets/rasters. Cursor movements, scrolling,
typing/undo and active/closed search are checked. The canonical document must
end up identical to the initial one.

The `leaf.*` spans are inclusive. The `boundary.*` spans delimit calls to the
engine's public APIs through the adapters: they include their small Leaf cost,
and are not an exclusive CPU profile of WordCraft. The counters add a small
instrumentation cost to both versions; the timings describe this diagnosis, not
a normal session. Do not add up nested spans or attribute all the time of an
operation to a single component.

Besides time, repaint requests, GTK bounds queries and created search rows are
counted. These counters distinguish eliminated work from CI fluctuations. RSS
is recorded per phase: it includes this test's documents and verification
copies, and is not equivalent to the consumption of a normal session nor a
promise of lower memory. Rasters already had a cache limited to the visible
area; that policy is kept.

## Leaf-only changes

- The selection layer repaints the union of the previous and current pages;
  this removes the marks left when deselecting, closing search or changing
  page. During the cursor animation only its page is invalidated. Entering or
  leaving headers/footers keeps full invalidation of their guides.
- Thumbnails query the current bounds of the GTK widgets by binary search. No
  CSS heights that could become stale when the window changes are stored.
  Before allocation, the same approximate bounds as the previous version are
  used, precomputed when building the list. Still-matching bounds of newly
  added widgets are treated as unallocated; not all visible thumbnails are. A
  weak, coalesced callback updates the bounds after the panel's fit/size
  changes. Only the visible thumbnails and the small cache are traversed to
  evict the previous ones.
- Closed search keeps the query, but stops running the searcher and rebuilding
  its rows on edit. When reopened, it is recomputed against the current
  revision. Thumbnails hidden by search/headings are not rasterized.

`Cargo.toml`, `Cargo.lock`, `vendor/`, `patches/`, the WordCraft revision and
the renderer do not change. The workflow checks that boundary via `git diff`.

## Correctness

The non-GUI tests compare the binary search with a full traversal on lists
with variable heights, gaps and exact edges, plus a list of 10,000 entries with
a bounded number of queries. The GTK test compares the cache with all real
bounds at the start, middle and end of the panel; it checks repainting of the
previous selection, closing and reopening search after editing, and exact
restoration of the document through undo. Existing regressions for headers,
zoom/HiDPI, animation, search and lifecycle are kept.

## Results

The AB/BA comparison passes at all three sizes. Each version runs twice per
size on the same runner. Medians of the widget and initial raster construction
phase, **not including import or first layout**:

| Pages | Thumbnails initially rasterized | Initial phase, ms (before → after) |
| --- | --- | --- |
| 11 | 11 → 4 | 29.02 → 16.68 |
| 55 | 55 → 4 | 111.95 → 20.92 |
| 164 | 164 → 4 | 315.90 → 30.83 |

At 164 pages, 80 cursor movements: **13,120 → 80** operations (`overlay_queue`), 1.259 → 0.991 ms in this sample.

At 164 pages, 80 cached thumbnail updates: **13,120 → 1,440** operations (`thumbnail_bounds`), 2.581 → 0.324 ms in this sample.

At 164 pages, 4 requested updates with search closed: **288 → 0** operations (`search_rows`), 103.335 → 0.002 ms in this sample.

The first layout and the eight inserts/undos show no consistent improvement
that should be attributed to these changes. WordCraft is not modified or
internally sped up. The lower RSS observed on the large documents is
consistent with avoiding unnecessary initial rasters, but two samples do not
justify a general memory promise. The work counters are the most direct
evidence; do not multiply their ratios by the speed of the whole app.

No specific engine defect has been isolated that would justify an upstream
issue. The import/layout/rasterization phases are recorded for a future
reproduction if a problematic case appears; any change there will be proposed
to WordCraft, keeping this branch free of engine modifications.

[Durable evidence, revisions and per-phase results](HOST_PERFORMANCE_VALIDATION.json).


## Reproduce

```sh
cargo test --locked
cargo build --release --locked --features gui --bin wordcraft-gnome
XDG_CONFIG_HOME=$(mktemp -d) XDG_STATE_HOME=$(mktemp -d) \
  GSK_RENDERER=cairo xvfb-run -a \
  ./target/release/wordcraft-gnome --host-cache-ui-test
python tools/performance_compare.py /ruta/referencia /ruta/candidato
```

The `--performance-ui-test` flag enables metrics exclusively for that synthetic
diagnosis. No documents or metrics are recorded in normal use. It is not a new
per-version compatibility suite, nor does it repeat the 24-file corpus. Nor
does it replace the human Wayland/GNOME OS or GPU test.

GTK documentation consulted:

- https://docs.gtk.org/gtk4/drawing-model.html
- https://docs.gtk.org/gtk4/method.Widget.queue_draw.html
- https://docs.gtk.org/gtk4/method.Widget.compute_bounds.html

## Delivery

[Alpha.13](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.13)
published after passing the cache regression on the installed Flatpak, the
update from alpha.12 and the real AT-SPI/Orca test. 99 tests and full GTK
regressions. Artifacts downloaded/verified and eight assets cross-checked;
revisions, hashes and installed results are in the linked JSON.
The public screenshots have not been replaced and Weblate remains the user's
responsibility.
