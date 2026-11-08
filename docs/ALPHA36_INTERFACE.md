# Alpha.36 — revised document workflows

The candidate covers #171, #177, #181, #182 and #186–#191. Publication and
individual issue closure require the native suite and both installed runtimes.
No engine, renderer, vendor, compatibility patch or dependency pin changes.

## Behavior

- New blank documents offer real asynchronously rendered template thumbnails
  above the editable page. Choosing fills the same untitled tab. First committed
  input, including IME and paste, permanently dismisses that gallery; Undo and
  late preview completion cannot revive it. Preferences chooses or resets the
  folder. Missing/disabled XDG entries never fall back to the entire home.
- The default sandbox permission remains `xdg-templates:ro`. A chosen portal
  directory is narrowed to read-only before its returned path is persisted.
  Preview/cache data stays outside source storage. Native operation only reads
  ordinary files; the system user retains their existing filesystem permissions.
- Search options form one contiguous group with scope last. Tabs and line
  endings are pasted literally. Plain clipboard data cannot distinguish a soft
  break from a paragraph mark: line-ending queries find both, replacements
  create paragraphs. Literal ^p/^t/$1 retain their ordinary meaning. The public
  pinned pattern compiler and canonical positions remain authoritative; the
  multiline adapter uses disposable read-only text spans, never another editor.
- Fonts are embedded in Document Properties. Inventory and known fallback
  information remain read-only; explicit substitution is undoable. Owned font
  changes update the Properties lease so pending metadata can still be applied.
- One-tab windows reclaim the complete tab row. The sidebar selector remains
  independently usable without reserving a blank main-area row.
- Structure folds retain the visible parent offset, with the natural adjustment
  clamp at the bottom. Section separators appear only between sections; heading
  move actions live in an accessible three-dot menu.
- Context submenus follow Style, Paragraph, Text, Elements. Native split buttons
  keep ordinary Copy/Paste direct and variants separate. Back closes the nested
  popover without cascading to its parent. Primary and variant sensitivity are
  independent.
- External image drags paint a passive insertion caret without moving the typing
  selection. The actual drop uses that displayed position. Edge scrolling
  updates the marker; cancellation and obsolete async targets do not edit.

## Validation

Focused headless regressions cover pasted multiline search/replacement,
selection scope, tracked-deletion/section boundaries, real cached previews and
localized/disabled XDG directories. Native GTK diagnostics cover actual control
allocations, scope capture, font navigation/substitution with pending metadata,
outline offsets, split menu Back and IME/rapid-input gallery dismissal.

`tools/workflows36-pointer-test.sh` uses a separate native GTK file source,
external clipboard and real X11 gestures. The native delayed-read case uses a
FIFO and edits the document before releasing bytes, proving stale rejection.
`tools/workflows36-template-folder-test.sh` selects an actual external directory
through Preferences, checks thumbnails and same-tab creation, restarts the
process and verifies denied source writes inside the sandbox and unchanged
host bytes. Both installed runtime layouts retain the real D-Bus/portal session
and normal runner UID. The original interaction pointer regression is retained.

The unvalidated executable becomes available after the alpha35/36 workflow
checks so installed validation can run in parallel with older regressions.
Publication still requires complete native success and final byte comparison;
this changes scheduling, not the validation gate.

English/Spanish news is in the shared `data/releases/news.json`; generated
AppStream and GitHub notes use the store pipeline already merged separately.
#185 remains incomplete pending David's new captures, FlatPark acceptance and
actual catalogue/client verification. #1/#99 await upstream PR integration;
#107/#152 are merged upstream but absent from the pinned published 0.16.0.
#52 remains deferred. Automated checks do not establish human GNOME OS testing.
