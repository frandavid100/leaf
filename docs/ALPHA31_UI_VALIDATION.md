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

## Implemented scope under validation

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

## Validation status

The complete headless suite passes (178 tests), including the three new tests: custom RGB DOCX round-trip with one Undo and
Redo/repeat; caret-only typing without changing the surrounding word; switching
between custom/native colours, clearing and rejecting invalid input.

The full four-family native pointer regression exercises forward/Back paths,
repeated visits, real custom-colour hexadecimal entry, canonical targets and
Undo, at normal and enlarged/HiDPI UI sizes. Existing pointer checks now visit
the actual direct font list and native palette instead of removed intermediate
widgets. Text repeat is exercised on two successive targets.

Deterministic metadata checks report 824 source messages with complete English
and Spanish catalogs. Sidebar checks also cover the narrow dark enlarged-text
presentation; the normal layout retains the one/multiple-tab bounds checks.

The wide four-family pointer pass succeeds at `9f5066b` in
[42256123317](https://github.com/frandavid100/leaf/actions/runs/42256123317), including
all 38 native activations, arbitrary ink/highlight entry, one-step Undo and every
Back path. The narrow formatting fixture dismisses the modal overlay sidebar
before opening editor menus, matching the existing narrow-input regression.
Sidebar geometry/representation checks separately keep that overlay open.

Full native and installed GNOME 49/FlatPark 51 validation, visual acceptance,
final source identity and checksums must be recorded before issue closure and
candidate delivery. #134–#136 remain open until then.
