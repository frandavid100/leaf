# Alpha.38 — start rows, recovery and upward dismissal

Candidate for #207/#209, also shipping the fixed sidebar bar from #208/PR #210.
Publication and new issue closures require complete optimized native CI,
installed GNOME 49/51 checks, final executable comparison and public-asset proof.

## Behavior and ownership

Each untouched blank tab/window offers labeled Templates and Documents to Recover
rows only when populated, followed by the Blank Document heading and actual
editable page. Hidden groups measure zero and reserve no space. All rows share
the page's vertical viewport, with horizontal scrolling inside optional rows.
Existing opened documents never receive this start UI. The separate recovery
menu/picker and startup toast are removed; help and user/testing guides point to
the inline location.

`src/recovery.rs` calls the public engine recovery APIs on isolated canonical
Sessions. Inventory/load work runs off GTK. Restoring returns a dirty untitled
Session holding its original recovery record; a stale result can be dropped
without discarding that record. The UI validates its original document identity,
revision and eligibility before accepting either a template or recovery. A result
for an unselected untouched tab cannot steal another tab's focus. Inventory
refreshes on mapping, focus return and recovery outcomes; overlapping requests
schedule a fresh scan rather than losing the update. There is no periodic disk
scan on an idle blank document. Errors preserve the blank page and pending data.

The first content edit cancels discovery and input eligibility immediately, then
uses GtkRevealer's 180 ms upward slide to reclaim the start area's height.
Continued typing, paste and IME commits remain ordinary engine edits. A frame
clock handler observes GTK's updated size request before layout and compensates
only the part already scrolled out of view. It stops compensating if the user
scrolls. Waiting for the previous frame's allocation would cause a visible
rebound. Completion, reduced animations, unmapping, replacement and closure
remove the departing content and weak signal handlers. A pagination rebuild of
the same edited document preserves the running transition; Undo cannot revive it.

Navigation/Review stays fixed in the shared sidebar ToolbarView from PR #210;
upper selectors and filters retain their scrolling behavior. Existing synthetic
and real-pointer checks for the stationary bar remain enabled.

## Validation

Local headless coverage checks pending inventory, actual restoration, a stale or
abandoned restore, live-record locking and unchanged original recovery JSON.
`tools/workflows38-pointer-test.sh` exercises all optional-row combinations in
new tabs and windows, zero-size hidden groups, actual shared-content allocations,
real recovery/template activation, editing before an asynchronous reply, focus
across tabs, retained source bytes, actual typing/paste, painted animation frames,
IME, reduced motion and closing during a transition. It then starts separate
producer/editor processes and verifies a real pending recovery after restart.
Fixtures are explicitly discarded only after their retention assertions, using
the public engine API. English and Spanish HiDPI are required in native CI and
both installed runtime layouts. The superseded recovery dialog leaves the older modal check with eight tools;
its replacement is covered by the start/recovery matrix and restart check.
Existing full regressions remain required;
only the aggregate installed-suite deadline grows to include these extra checks.

GTK changes follow the [native revealer](https://docs.gtk.org/gtk4/class.Revealer.html)
and frame-clock lifecycle. Local tests use the documented extracted sysroot and
local-only D-Bus workaround. Normal CI and installed portal checks remain the
packaging evidence. No engine, renderer, lock, dependency pin, compatibility patch
or sandbox permission changes. The separate Windows preview is retained.

The engine-dependent #1/#99/#107/#152, deferred #52, read-only folder portal #171
and store/catalog #185 remain outside these verified fixes. WordCraft's latest
published stable is still the pinned 0.16.0; FlatPark metadata PR #1074 is open.
Public captures and Weblate retain David's existing decisions.
