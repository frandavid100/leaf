# Alpha.42 — controls, window closing and approved icon

David explicitly authorizes direct publication on 10 November 2026, after
validation. This delivery is an exception to the normal RC-first procedure.
The previous alpha.41 authorization is not reused.

## Scope

- #235: independent toolbar entry points reset the stack without a Back animation;
  genuine nested navigation keeps its normal transition.
- #238/#239: Start previews use the document page's resting top, including the tab
  row, and the recovery discard target is inset inside its paper preview.
- #240: one native Save Changes list per multi-document window, all dirty documents
  initially checked, explicit unchecked-discard explanation and distinct paths for
  Cancel, Discard All and Save. Revision/scope checks guard asynchronous saves.
  Failed/cancelled saves preserve open work; discard happens only at final commit.
- #241/#242: upward bottom dropdowns and Style / Text / Paragraph / Elements order,
  including Preferences. Existing within-group user choices are retained.
- #243/#244: wide sidebar preference is separate from a collapsed overlay; search
  options are inside the native search field, with a bundled Nautilus symbol.
  Adaptive breakpoints belong to each tab so their conditions cannot replace one
  another; resizing checks revisit both tabs with opposite visibility preferences.
- #245: Save / Save As / Export / Print remain together, then Document Properties.
- #248: one shared Text / Paragraph / Elements overflow with per-tool pinned,
  adaptive and More-only placement. Pinned shortcuts scroll without moving the
  fixed menu; native preferences keep ordering and migrate the old switches.
- #246: exact approved blue-quill SVG and symbolic sources; Windows' source icon is
  regenerated too. This does not create a new Windows release or store gallery.

#171 is closed after David accepts the system Templates folder and withdraws the
custom-folder requirement. #52 is closed pending further information, not fixed.
#185 is explicitly paused, including description changes and public screenshots.
Routine release news is still generated from the common bilingual source.

## Validation and publication conditions

Run the full existing native regression matrix and installed GNOME 49/51 checks.
The existing toolbar41/workflows40 probes additionally cover actual menu geometry,
independent-entry transitions, search bounds/icon availability, sidebar visibility
across resize, inset discard bounds, Start/page alignment and combined close.
The close probe exercises Cancel, disabled Save with no selection, repeated close,
a failing destination and saving one document while discarding another.
These are automated checks, not a claim of human testing on GNOME OS.

Keep #231 open: the maintainer's intermittent report does not yet have an isolated
trigger. Passing the existing clean-close and interrupted-session recovery probes
is not proof that this report is resolved.

WordCraft remains pinned at 0.17.0. #1/#99/#152 are not in the closure plan.
Updating the proposed upstream branches is blocked by GitHub connection write
permissions; prepared upstream work and fresh local results are recorded separately.

The publisher must verify the candidate source, optimized executable, tag and all
public asset digests before closing the explicit closure plan. No publication or
successful final native/installed result is claimed by this preparation document.
The publication workflow records the verified outcome in LATEST_RELEASE.md.

## GTK used for formatting navigation

The Ubuntu 24.04 host has GTK 4.14, which predates the upstream
[GNOME/gtk#6817 fix in GTK 4.17.6](https://github.com/GNOME/gtk/blob/4.18.0/NEWS).
The captured crash is in `gdk_surface_request_motion` from native layout after
a popup surface disappears. The unchanged format136 pointer test therefore runs
the exact optimized Leaf executable with GNOME Platform 49, the supported runtime.
The helper checks GTK >= 4.18 and records the immutable runtime commit. No input
assertion is skipped, and no GTK interposer or Leaf workaround is shipped.
All other native probes and installed GNOME 49/51 validation remain required.
