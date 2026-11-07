# WordCraft 0.16 integration and Leaf performance review

David explicitly authorizes upgrading the engine, adapting Leaf to benefit from
it, optimizing/reorganizing the client and publishing a new release after checks.
This is the authorized engine update described in UPDATING_WORDCRAFT.md; it does
not authorize a fork of upstream layout/render behavior.

## Pinned base and build inputs

Latest published stable release checked: [WordCraft 0.16.0](https://github.com/storytold/wordcraft/releases/tag/v0.16.0),
commit `02a802cbafe45b5c8fecc00490c8f6b78c5ba251`. The preceding Leaf release used
0.14.0 (`5cd53f184aab11def13b7fced18727159b9bc96a`). WordCraft's Rust minimum is
still 1.95. All WordCraft revisions move together; the krilla override follows
the release's craft-libs pin `c5a1cbfb5f9049f9270955caffb5bee5997aebf7`.

The formats snapshot is recopied from this exact release, with a reviewed
[compatibility-only delta](../patches/README.md). The old four patches are retained
as historical evidence, not reapplied to unrelated new source. `.gitattributes`
marks the vendor tree for reviewers; the authoritative small patch is separate.

Language data follows WordCraft's `craft-i18n.rev`, pinned to
`1e515422afdda2f317af430c1f9d68a5f83935b4`. The preparation script fetches that
exact commit; CI requires the embedded data instead of silently accepting the
no-data fallback. It does not add runtime network permissions. Embedded language
licenses are shipped with the packages and exposed in About Leaf.

## Integration choices

| Upstream change | Leaf integration and boundary |
| --- | --- |
| Shared style sheet/numbering in Undo snapshots | Leaf's registered proofing-default command and fixtures use `Arc::make_mut`; historical snapshots keep their independent values. The upstream memory improvement is not credited to Leaf. |
| Much broader ODT model | Enable ordinary saves of footnotes, comment threads/ranges, floating pictures and newly authored basic equations after roundtrip checks. Bookmark ranges are now retained. |
| Custom ODT columns | Verify explicit widths/gaps in both formats, and read the effective first gap from explicit widths in the native column controls. The old assertion that ODT must discard unequal widths is replaced with preservation checks. This alone does not close the older column-layout reports. |
| Remaining ODT limitations | Keep guards for caption ownership/hidden caption text, category, proofing inheritance, original/complex equations, picture effects, advanced fields/TOC options, tracked formats/moves and unsupported section options. Do not convert structured-object presence into a full-fidelity claim. |
| Proofing data moved to craft-proof | Prepare the pinned Language data for builds, retain bundled Hunspell dictionaries and package their respective licenses. Existing document/style/fragment review semantics stay in Leaf. |
| Other engine improvements | Consume upstream layout, DOCX preservation, PDF field refresh, image placement and safety changes through the public APIs. Native WordCraft ribbon/dialog features are not automatically Leaf features. |

The unmodified 0.16 formats crate first failed six existing preservation checks
for geometry, linked/dormant page parts and language semantics. The compatibility
delta retains those assertions; no failure was hidden by removing its test.
Older ODT files with `zxx` plus Leaf's saved language are checked explicitly.
Successful saves now stamp upstream file metadata; regressions exclude only those
four save stamps when comparing the otherwise unchanged canonical document.

## Performance and reviewer clarity

The engine-upgrade snapshot adds opt-in counters for repeated TOC scans and ruler
refreshes and a field-free derived-reference phase. It is the same-engine control
for subsequent client optimizations. Normal use records no metrics or documents.
The host now indexes canonical TOC ranges once per document id/revision for
selection, hover and drawing; it removes a duplicate ruler refresh, shares
immutable caption cue rectangles, and checks eligible fields before making a
disposable reference-update Session. No cached range is written into the model.
Revision/identity, Undo/Redo and unchanged-document diagnostics cover these paths.

Clipboard handling is extracted into one private module. Leaf offers upstream
RTF/HTML and single-equation MathML when copying; external HTML/RTF uses public
`edit.paste`, with a bounded asynchronous transfer, weak ownership, cancellation
and identity/revision/selection validation. Plain text and exact-provider internal
fragments retain their existing paths. Native/installed diagnostics check external
RTF/HTML/plain text, formatting, one-step Undo and stale/replaced destinations.

Matched optimized-build comparisons use the same engine/formats/language pins,
AB/BA order, two fresh-profile repetitions per size (120/600/1800 paragraphs),
Xvfb/Cairo on one runner, canonical document equality and deterministic operation
counts. Timing and RSS are reported without noisy pass thresholds; shared engine
and prior thumbnail improvements are not credited to this client change.
Actual results will be recorded here after validation. No whole-application speedup is inferred from a synthetic
workload or from upstream's published memory figures.

## Validation and delivery

In progress: local Rust/format checks, optimized native build, matched GUI
performance runs, full native regression and GNOME 49 / relocated GNOME 51 package
checks. Release identity, sources and asset hashes are recorded only after success.
The original font/clipboard/canvas driver remains unchanged. No new Orca phase,
Weblate action, Flathub submission or public screenshot replacement is included.
