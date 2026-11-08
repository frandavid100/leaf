# Fixed sidebar mode bar — issue #208

David reports that the alpha.37 Navigation/Review tabs scroll out of view. The
original #200 wording incorrectly included these bottom mode tabs among the
scrolling selectors. Its clarified scope keeps only upper choices and filters
inside their respective scroll content.

`tab.rs` adds the existing mode bar to the common sidebar `AdwToolbarView` as a
flat bottom bar, outside the Pages/Structure/Review and Search stacks. Structure
and Review no longer construct proxy bars inside their result lists. Navigation
reactivation retains its selected state, including returning from Search.

This is frontend widget composition only. WordCraft, dependencies, document data,
undo history, upper selector placement and hover styling are unchanged.

## Validation

The alpha37 native diagnostic now checks stationary bottom-bar bounds while upper
selectors scroll, removes the old assertion requiring the mode bar to scroll,
and checks real pointer switching from long Pages and Review views, Search and an
empty document. Geometry is measured relative to the actual window/sidebar,
including start/middle/end scrolling, with read-only document/history assertions.
Existing view diagnostics also reject a mode bar inside any scroll container.
The normal workflow repeats these checks in English and Spanish at scale 1/2.

Validation is in progress. No new package is published by this source change.

## Scope

The #207 start-view redesign and #209 upward dismissal animation remain separate
requests. General visual refinements have not been implemented as part of this
urgent correction. Public store screenshots remain David's captures.
