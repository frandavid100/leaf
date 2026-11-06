# Alpha.31 second candidate: native sidebar and formatting

David explicitly requested completing #134–#136 without reintroducing the font
input failure. The first Alpha.31 candidate and its #133 evidence remain in
[ALPHA31_VALIDATION](ALPHA31_VALIDATION.md). This pass does not authorize a
release, dependency update or replacement of public screenshots.

## Confirmed input defect and isolated correction

The native ruler's capture-phase motion/margin gestures processed events from
an anchored popover's GDK surface. With the aligned headers, the Font editable
overlapped the document ruler band. The ruler claimed the next click and took
keyboard focus; paste/backspace could then reach the document instead.

| Source / run | Result |
| --- | --- |
| `4f65aaf` / [42236717993](https://github.com/frandavid100/leaf/actions/runs/42236717993) | Navigation dropdown alone passes installed input. |
| `cb7bc67` / [42238593822](https://github.com/frandavid100/leaf/actions/runs/42238593822) | Aligned headers still fail despite retaining the mapped Font page. |
| `2eb16a8` / [42240273736](https://github.com/frandavid100/leaf/actions/runs/42240273736) | Passive focus trace identifies `document-ruler` taking focus, followed by the canvas receiving Backspace. |
| `e9ece6a` / [42240792846](https://github.com/frandavid100/leaf/actions/runs/42240792846) | The same installed input preconditions and assertions pass after ruler surface guards. |

`document_pointer_surface` is shared with the ruler. Motion, margin capture,
marker dragging and ruler scrolling reject foreign popup surfaces. This is
separate from popover page retention. The original font/paste regression and
its driver retain their assertions and input sequence; passive traces are not
shipped. Document, engine, renderer, locked dependencies and patches are intact.

## Delivered scope

* #134: native primary HeaderBars and secondary-row size groups align the full
  regions. Document-tab autohiding remains native; the sidebar selector retains
  full height. Bounds are checked with one and multiple tabs.
* #135: a native Pages/Structure dropdown, with a separate Structure content
  filter, per-document browsing/expansion and independent scroll restoration.
* #136: actual flat AdwSplitButton property/repeat controls; correct contained
  page titles and Back/Escape paths; Size's icon/label inside the full field;
  an immediate searchable font list with readable previews; searchable Language
  choices and reachable footer; native embedded colour palettes and custom
  editors; scrollable Style pages and accurate Elements property titles.

Font retains the existing editable, validation and debounced command controller.
Reopening the mapped Font page does not detach the editable. The canonical
current font and language markers synchronize without rebuilding focused native
fields, including pinned selection changes; partial searches do not edit the document.

The real public GtkColorChooserWidget supplies palette and arbitrary-colour
editing inside the page. It is deprecated but available in the supported GTK 4
runtimes; GtkColorDialog would require the separate modal chooser the maintainer
rejected. Read-only synchronization is suppressed and an initial opening never
applies a colour. Continuous valid edits use the existing coalesced formatting
transaction and preserve their editable's focus and selection.

Numeric controls ignore GTK's initial mapping output. Unset/mixed inputs remain
blank, and proportional line-spacing values are never reused as point values in
the Exact field. Merely opening that page therefore preserves the document and
selection; subsequent native input keeps the existing immediate/debounced edits.

Native highlight palette colours retain the engine's highlight property and
palette command handler. Arbitrary RGB text backgrounds use canonical character
shading through a registered Session command. They keep caret-only pending
typing, native history/repeat, protection and DOCX export; no parallel document
or history is introduced. The No Highlight action clears direct backgrounds.

Style retains its gallery, normalized previews and management actions. There
are no Style colour-dialog launchers to replace. Elements retains existing
target-sensitive properties before insertion, disabled no-target actions and
tooltips. Its existing table colour workflow uses the shared embedded chooser;
this does not claim to complete the separate caption/object ownership issues.

## Final acceptance — 6 November 2026

#134, #135 and #136 are closed. This is the **second unpublished Alpha.31
candidate**, retaining #133. Alpha.30 remains the latest public release.

| Evidence | Accepted identity / result |
| --- | --- |
| Candidate source | `79292491050943559df21d9082657fe38ee46254` |
| Native executable source | `179ef4bec1609c355c3dae325c3358fd6bb948a8` |
| [Complete native run](https://github.com/frandavid100/leaf/actions/runs/42263872786) | 178 Rust tests, four Python tests, 824 source messages and complete English/Spanish catalogs; full native interactions, lifetime, keyboard, print and visual checks pass. |
| [Installed candidate run](https://github.com/frandavid100/leaf/actions/runs/42265503961) | GNOME 49 sandbox/upgrade/desktop/printing/accessibility and FlatPark GNOME 51 real extra-data relocation pass. Fully validated optimized executable matches the packaged binary. |
| [Explicit bilingual installed acceptance](https://github.com/frandavid100/leaf/actions/runs/42269593968) | Both runtime jobs pass using these exact candidate files; English/Spanish titles are asserted literally. |
| Locale-check source | `126d5f647e5c912d7d1f7c77afe1c728ec914743`; verification scripts only, application not rebuilt. |
| [Candidate artifact](https://github.com/frandavid100/leaf/actions/runs/42265503961/artifacts/13527444066) | ZIP SHA-256 `18f50e90fa3c71da71ba9fd0edb2b3bcc15e8cef9c237f9f95e0575956a444af` independently verified after download. |

All eleven entries in `SHA256SUMS` were independently verified, together with
`candidate.json`, both source identities and run IDs. The machine-readable
[acceptance record](ALPHA31_UI_VALIDATION.json) retains their exact sizes and hashes.

| Deliverable | Bytes | SHA-256 |
| --- | ---: | --- |
| `Leaf-0.1.0-alpha.31-x86_64.flatpak` | 11251200 | `c1475a7c8e4f6c1529d4ca0158004f68425158f9d6c0efaf92db9c9d26932b28` |
| `Leaf-0.1.0-alpha.31-flatpark-x86_64.tar.xz` | 11487988 | `ae622d43563694c0eeedaf75228dce7589426075dc5994be4fe9ed5a57356c96` |

## Interaction and visual coverage

The four-family pointer regression visits 18 property pages, with 38 native
activations, mixed/repeated forward/Back navigation, pinned/unpinned states,
real custom hexadecimal ink/highlight edits, canonical targets and one-step
Undo. Merely opening a page preserves document and selection. Native flat
split components, independent successive Repeat targets, direct font list,
readable previews, language grouping and reachable final action are checked.

Header allocation checks cover the complete primary/secondary regions in
Review, Pages and Structure, one/multiple tabs, normal and overlay sidebar
layouts. The dropdown also retains per-document representation, independent
scroll/expansion/filter state and editing targets across Review/Search,
close/reopen and tab changes; mouse and keyboard choices are exercised.

Actual installed screenshots were inspected from both runtimes, including
English/light/normal UI and Spanish/dark/480px/16pt/2× cases. Font, Language,
custom Highlight, Paragraph spacing, Style creation, Table properties and
Navigation/Review header regions were reviewed. Internal QA captures do not
replace the public gallery. This is automatic X11 coverage, not a human
Wayland/IME/device test.

The original font/paste interaction source and driver are unchanged from the
first candidate. Their original input and assertions pass in native and
installed checks. Three added headless regressions cover custom RGB DOCX
round-trip and Undo/Redo/repeat, caret-only pending typing, native/custom
highlight switching, clearing and rejection of invalid input.

## Test-harness corrections retained transparently

The installed visual pass initially labelled an English fallback as a Spanish
case: `LANGUAGE=es` alone was ignored under the C locale. The application’s
explicit localization test already passed. The supplemental run sets
`LC_ALL=en_US.UTF-8` and asserts literal `Fuente` / `Idioma` page titles; both
runtimes pass. Future installed wrappers retain that explicit locale.

A supplemental first run also exposed a transition race in the diagnostic's
programmatic switch between popup families. The driver now waits 350ms for
the old popup grab to settle before its single activation, then retains the
same pointer coordinates, button sequence and all target/document/history
assertions. No click retry or weakening of the original font/paste driver was
introduced. Both complete bilingual installed jobs pass with that driver.

The prior Font width check now compares allocated outer bounds in a common
popover coordinate space; GtkEntry's content width excluded its padding.
The pinned paragraph diagnostic focuses its mapped active field before two
edits, preserving its exact values, target and one-step Undo assertions;
a stale unmapped control is correctly rejected after panel reconstruction.

## Remaining scope and publication

Thirteen issues remain open: #1, #2, #9, #52, #84, #99, #100, #107, #121, #124,
#126, #127 and #132. The original-document overlap #132 is not claimed fixed;
caption/object ownership requirements remain separate. WordCraft, its renderer,
locked dependencies and patches are unchanged.

No release or tag was published. David must test and approve promotion of these
exact candidate files without rebuilding. The first candidate's
[validation record](ALPHA31_VALIDATION.md) remains historical and identifies
different bytes.
