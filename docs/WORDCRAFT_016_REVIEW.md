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
The [complete measured summary](WORDCRAFT_016_PERFORMANCE.json) identifies control
`4869c0a37f94dd7f51d796b9fcce281834c66380` and final native source
`c55188c3eb6fa791a4d294fbae03a030408dcda0`. All twelve raw profiles preserve the
canonical document. At every size and repetition, 80 caret moves issue 80 ruler
refreshes instead of 160 and zero repeated TOC scans instead of 80; overlay work
is unchanged. The index is rebuilt after revision/identity changes, including
Undo/Redo. Existing thumbnail and inactive-search work bounds also pass.

| Paragraphs | Pages | Measured phase | Before median, ms | After median, ms |
| --- | --- | --- | --- | --- |
| 120 | 11 | 80 caret moves | 194.454 | 183.036 |
| 120 | 11 | 16 field-free reference refreshes | 0.960 | 0.016 |
| 600 | 55 | 80 caret moves | 236.921 | 213.652 |
| 600 | 55 | 16 field-free reference refreshes | 1.600 | 0.023 |
| 1800 | 164 | 80 caret moves | 330.667 | 314.709 |
| 1800 | 164 | 16 field-free reference refreshes | 3.087 | 0.073 |

These are inclusive phase times from two samples per binary/size, not a
statistical whole-application claim. Cursor medians improve by roughly 5–10% in
this workload. Field-free reference refresh becomes much cheaper but was already
a small absolute cost. Cursor-phase RSS is about 1.6–2.3 MiB higher; no reduction
in total client memory is claimed. Upstream's shared Undo data is a separate
engine improvement, not part of this same-engine comparison. Fresh profiles do
not clear the operating system's file cache; no cold-disk or general document
speedup is inferred.

## Validation and delivery

Passed: 189 Rust tests, four Python tests, formatting, current-document links,
810 complete English/Spanish gettext messages, optimized native build and the
[complete final native workflow](https://github.com/frandavid100/leaf/actions/runs/42371069496).
Native clipboard/menu/Undo/stale-result diagnostics, cache identity/revision,
original interaction/font/clipboard/canvas driver, pointer regressions, memory,
printing, localization and HiDPI checks pass. An earlier narrow review demo
failed before scenarios received independent profiles; its assertions are kept.

The [installed GNOME 49 / relocated GNOME 51 workflow](https://github.com/frandavid100/leaf/actions/runs/42372618858)
passes on the same optimized binary, including the original font/clipboard/canvas
driver, the new rich-clipboard diagnostic in both runtimes, real pointer input,
English/Spanish, 1×/2×, upgrade/reinstall persistence, document/print portals and
the existing AT-SPI/Orca check. This is automated Ubuntu CI evidence, not a claim
of human GNOME OS, braille, printer or real-document testing.

Two initial packaging attempts caught assumptions about downloaded executable
permissions and host GTK libraries while extracting the new licenses. The final
builder executes the staged binary inside its GNOME Flatpak runtime. Its embedded
license text matches the tested native output. Application inputs remain equal
to the fully validated native source.

## Published alpha.32

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.32),
published 7 November 2026 through the [automatic promotion](https://github.com/frandavid100/leaf/actions/runs/42374275590)
without rebuilding. The complete native workflow succeeds before acceptance;
packaging compares the stripped early/final native executables byte for byte.
The promotion rechecks native success and candidate identity.

- Published source and verified tag: `2524dbc789cfbfa0f3b6e153fde16f1df5887b8a`.
- Native source: `c55188c3eb6fa791a4d294fbae03a030408dcda0`, run `42371069496`.
- Candidate run `42372618858`, artifact `13575365590`.
- All eleven internal checksum entries pass; all twelve public asset sizes and
  SHA-256 digests match the downloaded validated candidate, including the
  checksum file itself. Source/version/run identity and archive licenses match.
- x86_64 Flatpak: 12,904,096 bytes,
  `60e01f8bd2e6d01171707186c07f13b52764121438f6176d72d52d5e15ee2ef4`.
- FlatPark archive: 13,196,204 bytes,
  `388c04a759eb9a22cd2f03310c965cf7909e7a6f2fba3fd548ffdc711b7b30b0`.

At publication the engine update itself closed no additional issues. Subsequent
[functional triage](ISSUE_TRIAGE_AFTER_ALPHA32.md) closes #100 on the same published
binary: labels and direct note navigation meet the current requirements, without
attributing an unidentified historical cause. Nine remain: #1, #2, #9, #52, #84,
#99, #107, #152 and #156. In particular,
the separately developed upstream column changes are not inferred to be in this
stable release, and generic picture-placement improvements do not prove the exact
#107 case; Leaf retains its rollback when the public arrange command cannot
honor the requested page. Unsupported ODT fidelity options remain guarded.
The original font/clipboard/canvas driver remains unchanged. No new Orca phase,
Weblate action, Flathub submission or public screenshot replacement is included.
