# Alpha.37 — scrolling, focus and integrated dialogs

Published alpha.37 implements and closes #199–#204 after full optimized native CI,
installed GNOME 49 and relocated GNOME 51 checks, final executable comparison and
verification of the tag, release notes and all thirteen public assets.

## Behavior

- Templates and the blank page share one vertical viewport. The gallery retains
  horizontal scrolling. Canvas-local geometry accounts for its offset in page
  visibility, navigation, caret reveal, image anchors, the ruler and keyboard
  context positioning. Automatic blank-page refits retain gallery browsing;
  the first committed edit removes the gallery once and reclaims its height.
  Refit invalidation repaints visible pages without moving gallery browsing.
- Pages, Structure and Review own their scrolling selectors and mode controls.
  Independent adjustments are retained. Native dropdowns share state through
  GTK property bindings instead of moving an open popup between parents.
  Resting selector backgrounds are transparent; hover and native focus remain.
  Sidebar selectors no longer reserve a secondary header/footer band. Primary
  header alignment and tab-row autohide remain separately checked.
- App-owned modal tools use AdwDialog, PreferencesDialog, AlertDialog and
  AboutDialog within the correct parent. Keyboard Shortcuts uses searchable
  native preference rows and accelerator labels. File/folder/print choosers
  remain native system flows; formatting popovers and pins keep their workflows.
  The existing libadwaita crate enables its v1_5 API; the dependency version,
  Cargo.lock, WordCraft, renderer, vendor and compatibility patches are unchanged.
- Mapping Structure can temporarily clamp GTK's selected row to the top. The
  previous 220 ms restoration is replaced by stable native allocation frames.
  Generation/identity guards reject obsolete work, and new scrolling, pointer
  or keyboard input cancels a pending restore. Genuine target changes still reveal.
- Modal roots guard document/window actions. Native entry editing stays native;
  Escape dismisses a nested dropdown before its dialog and focus returns to the
  editor. Late close callbacks cannot clear a newer link/comment dialog. Modal
  table controls reject closed, hidden or replaced document targets; asynchronous
  folder/save results retain cancellation and document leases.
- First Save proposes safe, bounded Unicode from title, first heading or the
  localized generic name. A copy suffix is reserved for existing files and
  translated. DOCX requirements, direct Save and native overwrite confirmation
  are retained. Constructing the suggestion never changes document metadata,
  contents, selection or history.

## Validation

Headless filename tests cover metadata precedence, heading fallback, unsafe and
empty names, Unicode truncation, format guards and read-only history/selection.
The complete local and optimized native CI suites passed 214 Rust tests, including the three filename
cases. Python: 9 tests. Catalogs:
893 messages with complete English and Spanish translations.

`tools/workflows37-pointer-test.sh` checks real gallery/sidebar deltas, independent thumbnail rasterization and active-page
visibility in actual native bounds, repeated
hide/show, user-input cancellation, nine integrated dialog bounds at narrow
window sizes, stale table targets, native keyboard/mouse Escape, modal shortcuts,
focus, Undo and a real first-save chooser cancellation. It runs in native and
installed paths for English and Spanish HiDPI. Existing pointer/IME/template,
formatting, tab, font, navigation and lifecycle regressions remain required.

Synchronous legacy probes now wait for native presentation/closing rather than
GtkWindow visibility. libadwaita 1.5 opens an integrated sheet only after its
initial mapping ticks; a newly mapped widget alone is not interactive. New
workflow probes use asynchronous waits and retain their behavior assertions.

Local checks use an extracted GTK 4.14/adwaita 1.5 sysroot and a local-only D-Bus
workaround. The file chooser can emit local GTK cleanup warnings there. Normal
CI and installed portal checks are authoritative for packaged behavior; this
workaround is never included in distribution or CI.

## Limits

The new #207 inline recovery/template redesign was recorded after this release
was published and is pending a separate implementation.

The deferred #52, engine-dependent #1/#99/#107/#152 and read-only custom-folder
blocker #171 remain outside these fixes. Store/catalog work #185 continues in
its separate conversation. Public screenshots are not replaced with CI captures.
The published experimental Windows 0.1.0-alpha.36-windows.1 is preserved. The compiled-source freshness
guard also includes its new build.rs; existing source exclusions are unchanged.
No claim of a manual GNOME OS, Orca, printer or user-system test is made.

## Repaint regression found during validation

The installed localization check failed because a gallery-eligible refit cleared
page rasters while the branch preserving gallery browsing skipped repainting.
The same preference sequence reproduced the missing paintable locally. The native
layout callback now refreshes pages without scrolling; existing English/Spanish
dark/light pixel and unchanged-document assertions pass. The real gallery check
also requires a painted page and the same adjustment after a refit.

The historical dependency boundary permits exactly the existing libadwaita
v1_4-to-v1_5 API feature change. Crate versions, other existing dependency
specifications, engine packages, profiles and patches remain checked unchanged.

## Published identity

- Source/tag: [`4659acbc0730ebe980a97c9ca802c4666c7f6cdb`](https://github.com/frandavid100/leaf/commit/4659acbc0730ebe980a97c9ca802c4666c7f6cdb).
- Fully validated native source: [`9ff36ff794d058270c75f791e0a116dcfdd53feb`](https://github.com/frandavid100/leaf/commit/9ff36ff794d058270c75f791e0a116dcfdd53feb).
- [Full optimized native regression run](https://github.com/frandavid100/leaf/actions/runs/42506559955).
- [Installed candidate and final binary comparison](https://github.com/frandavid100/leaf/actions/runs/42506617001).
- [Promotion without rebuilding](https://github.com/frandavid100/leaf/actions/runs/42510045898).
- [Release and SHA256SUMS](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.37).

The public asset names, sizes, uploaded states and SHA-256 digests match the
validated candidate. The checksum manifest, source-commit text, candidate JSON
and bilingual release notes were checked separately.
