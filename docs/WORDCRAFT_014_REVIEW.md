# WordCraft 0.14 integration and frontend review

Requested by David on 2 November 2026, after the verified alpha.21 delivery.
Baseline: `5cf599294763a421b62343879da2d4425376850c` on `main`.
This is an integration/maintenance change, not a new release or an engine upgrade.
The last published package remains alpha.21 until another release is requested.

## Which upstream changes reach Leaf?

Reviewed the [official 0.14 release notes](https://github.com/storytold/wordcraft/releases/tag/v0.14.0)
and the actual [0.12→0.14 source difference](https://github.com/storytold/wordcraft/compare/165d5725dc25b4a7887d8025c3b1fe506ccc3dcc...5cd53f184aab11def13b7fced18727159b9bc96a).
Leaf already pins engine/doc/render and its fresh formats snapshot to
`5cd53f184aab11def13b7fced18727159b9bc96a`. A newer engine does not replace
Leaf's native GTK interface with WordCraft's egui interface.

| Upstream change | Leaf integration and decision |
| --- | --- |
| Printer PDF without link annotations ([engine print API](https://github.com/storytold/wordcraft/blob/5cd53f184aab11def13b7fced18727159b9bc96a/crates/engine/src/print.rs)) | **Change needed and implemented.** PrintFile used generic file.exportPdf. It now calls public Session::print_job_pdf on its isolated document snapshot, retaining markup=false and private temporary-file cleanup. Generic PDF export continues to retain clickable links. |
| Unified language proofing and expanded Hunspell affix handling ([proof crate](https://github.com/storytold/wordcraft/blob/5cd53f184aab11def13b7fced18727159b9bc96a/crates/proof/src/lib.rs)) | Already consumed through public paragraph_issues/review.suggestions and engine proof_runs. Keep the same proofing engine, language inheritance, bundled Spanish dictionaries and user-dictionary configuration. No duplicated checker/language table is needed. Other languages may still require installed dictionaries. |
| Heading/widow layout, fallback for undefined DOCX styles, page-parameter bounds and TOC PAGEREF export | Already inside the pinned layout/docx/command paths. No frontend workaround. Leaf's DOCX/ODT and page-option regressions remain relevant; improved upstream behavior is not proof that every open document-specific report is resolved. |
| Relative font resizing and caret/highlight fixes in the compared source ([format commands](https://github.com/storytold/wordcraft/blob/5cd53f184aab11def13b7fced18727159b9bc96a/crates/engine/src/cmd/format.rs)) | Leaf's Increase/Decrease Size and formatting palette already dispatch those public commands. Preserve the distinction between relative size actions and typing an absolute size. No duplicate sizing/highlighting implementation. |
| Chinese/Japanese word counting ([document count](https://github.com/storytold/wordcraft/blob/5cd53f184aab11def13b7fced18727159b9bc96a/crates/doc/src/text_walk.rs)) | Available in the engine, but Leaf has no native word-count surface. A future dialog/status feature should use review.wordCount, not whitespace splitting. No new count UI is added by this cleanup. |
| Macro dialog, shared updater dialog, keytips and other desktop UI changes | Belong to WordCraft's own frontend. Native integration would be separate feature work. No automatic updater or macro-management UI is claimed here. |
| HTML/CSV parser corrections | Included in the formats snapshot. Leaf's normal open filter remains DOCX/ODT; importing additional formats needs its own native file-flow and preservation assessment. |

The four existing ODT patches, dependency pins, Cargo.lock and supported native
baseline remain unchanged. The previous synthetic capability audit still defines
known ODT structural losses; do not remove save guards on the strength of release
notes. Floating images, TOC, footnotes and equations still lack full native authoring.

## Structure and auditability

- Split the former 838-line input module into pointer/input-method handling,
  keyboard/scroll dispatch and cancellable selection dragging. Public frontend
  entry points and weak callback/identity guards are preserved.
- Split the former 842-line document-tools module into inline picture controls,
  document-properties transaction/dialog and PDF destination picker. Shared units,
  identity and command-result handling remain in the small document_tools module.
- Separate paragraph glyph drawing from paragraph settings. Put shared tree lookup
  and box clearing in widgets.rs, rather than hiding them inside Styles/View.
- Group the private module facade by production code and opt-in diagnostics, with
  stable alphabetical discovery. This is still a shared coordinator, not a claim
  that every panel has become an independent library.
- Expand compressed border, phonetic-guide and strike callbacks and performance/
  thread-local macros into readable Rust. rustfmt alone does not expand every
  macro body. Preserve command arguments, focus and undo behavior.
- Replace deprecated CSS loading and allocation-width calls with documented APIs
  available in the existing GTK 4.14 baseline. Retain the named selection-color
  compatibility lookup: switching it to newer GTK APIs would raise the baseline
  or change the native selection-color contract. No warning suppression is added.
- Regenerate gettext source locations, preserving all 678 message identities and
  complete English/Spanish translations. Update the current architecture map.

## Bounded optimization

After a populated Style popover is closed, every caret move still walked all its
rows and recursively searched for the current style. Synchronization now checks
whether the **actual list is mapped**, so it also works when the list is borrowed
by the contextual menu. Reopening reconstructs current row state; visible/pinned
lists continue to synchronize. No derived document cache or second model is added.

The matched synthetic diagnostic populates 42 style rows, closes the panel and
moves the caret 80 times in an 11-page, 120-paragraph document. Three instrumented
local debug baseline runs each visited **3,360 hidden rows**; the optimized runs
visit **zero**. Timing and environment are recorded in
[the measurement](WORDCRAFT_014_PERFORMANCE.json); debug timings are not release
performance or an app-wide speedup claim. The existing host-cache regression now
requires zero hidden traversal and verifies visible traversal/current-row state.

GTK visibility/allocation choices follow the official
[mapping contract](https://docs.gtk.org/gtk4/signal.Widget.map.html),
[width replacement](https://docs.gtk.org/gtk4/method.Widget.get_allocated_width.html)
and [CSS string API](https://docs.gtk.org/gtk4/method.CssProvider.load_from_string.html).

## Validation and limits

The new PDF regression **failed before the integration change**, because printer
output still contained /Annots, and passes after it. The paired control requires
ordinary PDF export to retain /Annots and /Link. Existing private-directory,
unique-job, unsaved-snapshot, live-state preservation and cleanup checks remain.
Local native compilation and host-cache checks pass; full native CI is pending
while this review branch is validated. Final evidence will name the tested source.

The cleanup does not add features, close unrelated issues, start new Orca phases,
replace public screenshots, activate Weblate or publish to Flathub. It preserves
alpha.21 and earlier releases/tags, archive/saving histories and separate Weblate
PR #57. No human Wayland/IME/device validation or universal file fidelity is claimed.
