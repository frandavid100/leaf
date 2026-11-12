# Alpha.41: final interface revision

Published and verified: #107, #217, #226, #227, #228, #229, #230, #231, #232, #233, #234, #235, #236, #237 are completed. See [published identity and validation](../LATEST_RELEASE.md).

## Changes after the earlier RC

| Issue | Final behavior |
| --- | --- |
| #233 | One continuous Start list, recoveries first and templates second, newest first within each type. Navigation-sized paper previews and names below, without section headings/dividers. The Templates folder button stays fixed in the shared sidebar footer, even when empty; first-letter and empty-document transitions retain sidebar/page geometry. |
| #234 | One fixed-width native style dropdown with the native raised appearance inside the toolbar. Current-style operations remain available through the style picker; the extra style-actions launcher is removed. |
| #235 | Actual native MenuButtons own each shared popup, including underline options. Direct roots suppress internal stack transitions; overflow retains nested Back. Context menus retrieve canonical panels independently of the current toolbar launcher. |
| #236 | Six logical pixels separate independent controls and overflow launchers. Underline/strike use the same native AdwSplitButton hierarchy and classes as Undo, with their two halves kept together. |
| #237 | Theme-colored vector bullet/numbering/removal icons appear beside each List choice, independent of installed symbolic icon availability. |

WordCraft remains at the authorized 0.17.0 pin. No model, layout, renderer,
dependency, compatibility patch, Flatpak permission, public screenshot gallery or
Windows release is changed by this revision.

## Validation

Local checks passed 227 Rust tests, 15 Python checks, 903 complete gettext
messages, metadata generation and project links. The native toolbar pointer test
passed opening/second-click dismissal, drag reorder, direct roots, preference
switches and fixed sidebar geometry. The Start check additionally uses two
recoveries of different ages and six newer templates, checks each ordering and
scrolls the list while measuring the unchanged footer. CI results for the new
source must be recorded separately before publication.

The extended toolbar acceptance checks native style framing, real pointer opening
and second-click dismissal, direct-root transition state, correct popup ownership,
independent-control geometry, split-button structure, List glyph mapping,
persistent preferences, and the fixed Start/Navigation footer. It runs together
with the existing editing, text, document, lifecycle and appearance regressions,
and installed GNOME 49 / relocated GNOME 51 checks in English and Spanish at
100% / 200% scale. These are automatic checks, not a claim of human GNOME OS or
physical printer testing.

Before publication, the candidate workflow must pass the optimized default GUI
build and all native/installed gates. Promotion verifies source identity, checksums
and the same tested executable; it publishes the existing candidate bytes. The
publisher records the final run/source/checksum evidence in
[LATEST_RELEASE](../LATEST_RELEASE.md) after verifying the public release assets.

## Follow-up from the first remote run

The initial remote run exposed a one-pixel Start preview rounding difference and
a duplicate overflow preparation that triggered an extra stack transition. Start
now uses Navigation’s same f32 rounding, and existing show handlers alone prepare
overflow home. Local spacing and real text-color pointer checks pass after this
correction. The four Start inventory combinations use isolated fixture folders,
so an earlier template session cannot repopulate the next empty-folder fixture.
A second remote pass also exposed localized footer-height differences and generic
Elements navigation being overwritten by its launcher factory. Both footer pages
now remain measurable in their homogeneous GtkStack while mapping follows the
selected page. The folder button uses a single native ellipsized line, keeping
the footer height stable with translated labels. Programmatic Elements navigation
passes its requested page to the group factory before mapping; direct launchers
retain their own roots and temporary reflows retain borrowed content.
Local 200% toolbar geometry and the full caption pointer workflow now pass,
including native typing, pinning, Undo/Redo, image switching, table captions,
DOCX reopening and hover cues. The changed source must complete the full remote
pipeline before publication.

## Remaining reports

| Issue | Evidence / remaining work |
| --- | --- |
| #1 | Unequal-column automatic continuation remains dependent on upstream WordCraft issue #2382, still open during this review. |
| #52 | Files/DOCX thumbnail report remains explicitly deferred, without sufficient original/provider/cache evidence to verify a fix. |
| #99 | The existing headless notes audit still reproduces table-flow note overlap in WordCraft 0.17: synthetic-table:60:footer:28.35 has note bottom 823.2219 versus footer top 799.5813, with warm/cold layouts agreeing. |
| #152 | New public `examples/paragraph152_audit.rs` reproduces the exact user distinction without GTK: whole-paragraph deletion preserves following properties=false; text deletion followed by empty-paragraph deletion=true. Selection spans Body paragraph 0 offset 0 to paragraph 1 offset 0. The issue stays open with the five original screenshots. |
| #171 | Custom folder portal still lacks the required persistent read-only grant; the default read-only XDG Templates folder remains available. No write/host permissions are added. |
| #185 | FlatPark metadata PR #1074 is merged. Final public screenshots remain reserved for David and actual client listing validation remains separate; private CI screenshots are not substituted. |

The closure plan excludes #152 and adds #233–#237. Earlier RC closures remain
recorded, but no unresolved report is closed merely to empty the issue list.
