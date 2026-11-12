# Fixed sidebar mode bar — issue #208

David reports that the alpha.37 Navigation/Review tabs scroll out of view. The
original #200 wording incorrectly included these bottom mode tabs among the
scrolling selectors. Its clarified scope keeps only upper choices and filters
inside their respective scroll content.

`tab.rs` adds the existing mode bar to the common sidebar `AdwToolbarView` as a
flat bottom bar, outside the Pages/Structure/Review and Search stacks. Structure
and Review no longer construct proxy bars inside their result lists. Both modes
reactivate their selected view when returning from Search, even if the chosen
mode tab was already active below the search view.

This is frontend widget composition only. WordCraft, dependencies, document data,
undo history, upper selector placement and hover styling are unchanged.

## Validation

The alpha37 native diagnostic now checks stationary bottom-bar bounds while upper
selectors scroll, removes the old assertion requiring the mode bar to scroll,
and checks real pointer switching from long Pages and Review views, Search and an
empty document. Geometry is measured relative to the actual window/sidebar,
including start/middle/end scrolling, with read-only document/history assertions.
Existing view diagnostics also reject a mode bar inside any scroll container.
The annotation diagnostic locates Review in the common bar instead of the old
page stack; its comment/bookmark behavior assertions are retained.
The normal workflow repeats these checks in English and Spanish at scale 1/2.

Merged in [PR #210](https://github.com/frandavid100/leaf/pull/210) as
`ef703952d0d9b65318688cb48cfb69cc2d5eb779`. The verified source head was
`94103bc19f681cac0d43cd1b296f1c04448b1c6c`, tree
`53156c3940e2f0184cf90ed470a57bec9bf2e7e6`; the merge preserves that tree.

The [complete native CI](https://github.com/frandavid100/leaf/actions/runs/42513401094)
and store metadata checks passed. Local validation also passed 214 Rust tests,
nine Python tests, 893 complete English/Spanish messages, Leaf formatting,
project links and release metadata checks. Debug and optimized GUI builds passed;
optimized pointer checks passed with the normal driver deadline in English at
scale 1 and Spanish at scale 2, and complete annotation checks passed in both
languages. CI additionally covered the native regression suite, window sizes,
dark theme, raster and geometry at 100%/200% scaling.

An earlier CI run exposed the annotation diagnostic's obsolete mode-bar lookup;
the final source corrects that lookup without dropping behavior assertions.
Installed Flatpak and portal checks were not rerun for this source-only fix.
No new package is published by this change.

## Scope

The #207 start-view redesign and #209 upward dismissal animation remain separate
requests. General visual refinements have not been implemented as part of this
urgent correction. Public store screenshots remain David's captures.

## Included in alpha.38

The source-only correction above subsequently ships in [alpha.38](history/ALPHA38_START_VIEW.md),
with complete native and installed GNOME 49/51 validation, executable comparison
and public-asset verification. #207/#209 are completed in the same release.
