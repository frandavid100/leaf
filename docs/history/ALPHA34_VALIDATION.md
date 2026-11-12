# Alpha.34 — dictionary integration test candidate

Published as a prerelease on 7 November 2026 under David's existing automatic
publication authorization. Issue #84 remains open pending his GNOME OS test.

| Evidence | Source / result |
| --- | --- |
| Native source | `65e8b6d2406372bc7443e3c0fd090cdbb1ed2948` |
| Published candidate source | `df4a22988216700c4771604d5e1ffeb5eaca74ab` |
| Full native checks | [Run 42403961297](https://github.com/frandavid100/leaf/actions/runs/42403961297) — success |
| Installed GNOME 49 and relocated GNOME 51 | [Run 42410141026](https://github.com/frandavid100/leaf/actions/runs/42410141026) — success |
| Promotion without rebuilding | [Run 42412187391](https://github.com/frandavid100/leaf/actions/runs/42412187391) — success |
| Published files | 12 internal checksums and 13 public asset sizes/digests verified; tag points to the candidate source |

193 Rust tests, four Python tests, package formatting, documentation links and
832 complete English/Spanish messages passed. The existing interaction driver
and all previous native/installed regressions were retained. The packaged
optimized executable was compared with the final fully validated native binary.

## Dictionary evidence

The native test installs a real Ubuntu Czech Hunspell package after startup,
then activates Refresh and verifies its actual system source and new language.
Native and installed controls exercise the real folder chooser, fresh dictionary
contents, cancellation, owner cleanup, restart persistence and stop using a
folder. Document content, selection, language, dirty state and Undo stay intact.
Installed checks run in English at scale 1 and Spanish at scale 2 in both runtime
layouts, with the raw host source inaccessible before the actual folder grant.
No broad host filesystem permission is added.

The separate `Leaf-0.1.0-alpha.34-dictionary-test.zip` contains an invented
Italian fixture and Spanish manual instructions. It is never automatically
installed as a production dictionary.

The language catalog checks readable file presence without parsing unrelated
large runtime dictionaries. WordCraft validates spelling data when that language
is used. Explicit refresh remains available; focus changes do not trigger reloads.

## Limits

David has not yet tested this prerelease on GNOME OS. Native Ubuntu's graphical
Language Support installation flow also remains unverified. Dictionary resource
precedence applies to matching filenames; WordCraft retains its locale/filename
fallback rules. Its Dutch KEEPCASE/FLAG-order parser limitation is documented in
the [integration report](ISSUE84_DICTIONARIES.md); no engine/dependency/vendor
changes are included. No additional issue is marked resolved by this candidate.

[Release and downloads](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.34) ·
[Public asset proof](ALPHA34_ASSET_VERIFICATION.json) ·
[Manual test](../../tests/fixtures/dictionary84/README.md).
