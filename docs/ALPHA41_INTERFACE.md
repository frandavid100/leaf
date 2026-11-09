# Alpha41 candidate: scope and evidence

Status: implementation and local checks in progress, not a published release.

The authorized WordCraft 0.17 base is
`d9f5e5353d4acf38f6b551714307ac9bcb356fe1`, pinned to upstream
`5574b32600aa7d782c64f5a6c6bb24696152220d`. Language data retains upstream's
unchanged pin. The formats snapshot was recopied and the residual ODT preservation
delta reapplied; see [compatibility patch](../patches/README.md). Other engine,
layout, document and renderer crates are unmodified upstream dependencies.

## Changes being verified

| Issue | Behavior and focused verification |
| --- | --- |
| #107 | The existing public picture adapter accepts the manual-page-break target. Root regression verifies page, one undo step, Undo/Redo and DOCX reopening. |
| #152 | Delete/Backspace before a heading retains its paragraph properties, Undo/Redo, DOCX and ODT. |
| #217 | Normalize the predominant contrasting neutral pair in Chrome wrappers before public HTML paste. Real Chrome/Firefox light/dark clipboard captures are preserved in test fixtures. Local red, highlights, bold, italic and links remain. The test checks one paste undo step, redo, DOCX/ODT and both screen-renderer themes. |
| #226 | The style launcher keeps a fixed text allowance and ellipsizes long names; adjacent style actions have separate targets. Native acceptance moves between Normal, Heading1 and a long custom name and measures the neighboring controls. |
| #227 | Shared panels derive their pointing rectangle and root page from the direct launcher; nested overflow navigation retains Back. Visible controls are omitted from overflow; List is one control. |
| #228 | Table, Image and Contents keep their insert action outside the scrollable options. Borrowed element bodies retain their own scroll/footer structure. |
| #229 | Pure toggles retain off/on/mixed state. Pickers use one click target. Underline and strikethrough keep separate options targets. Reapply-color commands live inside their picker. |
| #230 | Three native preference lists persist order and visibility and immediately refresh open windows. Dragging and keyboard actions reorder within a group. Narrow layouts do not overwrite saved choices; all-off groups remain reachable through overflow. |
| #231 | Independent recovery inventories exclude live locked snapshots. Normal save/discard remove their copies; interruption retains unsaved work. A superseded asynchronous scan is ignored before it can show stale recovery/discard rows. |
| #232 | Start-only previews use a 30×42 logical-unit envelope and 2 px corners; template names can wrap on two lines. Start uses the allocated sidebar width and a wrapping folder action. Navigation previews are unchanged. |

Replacing a picture now retains its width and follows the replacement's aspect
ratio, as explicitly specified by WordCraft 0.17. The former stretched-height
expectation is updated to that public command behavior, not worked around in Leaf.

## Validation and limits

225 Rust tests passed before the additional #231 regression; that focused file
passes all three tests, including live/normal-close/interruption semantics.
15 Python checks, gettext generation/check and project links passed locally.
Native and installed candidate validation are still required before declaring the
requested issues complete. The normal distribution build preserves the release
profile and panic unwinding. A local diagnostic executable may override host optimization and cross-crate
LTO; only the default release-profile CI bytes are eligible for distribution.

Browser capture provenance is recorded in the fixture metadata and
`tools/capture_clipboard217.py`. The actual public alpha.38 release was copied with
Chrome 157.0.8103.38 and Firefox 142.0.1. Office365 evidence is the reporter's
attached light/dark screenshots, not a claim of an independent signed-in Office
session. Clipboard HTML does not reveal which computed neutral colors were
intentionally authored, so the normalization policy is deliberately limited;
this is not a general proof of source-color intent.

GNOME references: [native split controls](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.SplitButton.html),
[native drag gestures](https://docs.gtk.org/gtk4/class.GestureDrag.html),
and [popover pointing rectangles](https://docs.gtk.org/gtk4/method.Popover.set_pointing_to.html).
Public store screenshots remain David's responsibility. No Flathub, Weblate,
new Orca scope, or occasional Windows release is started by this candidate.
