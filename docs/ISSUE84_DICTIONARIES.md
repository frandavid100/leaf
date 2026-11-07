# Issue #84 — dictionary integration test candidate

Alpha.34 is [published](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.34)
and its native/installed checks passed. Issue #84 remains open: David has not yet
confirmed the behavior on GNOME OS. [Release evidence](ALPHA34_VALIDATION.md).

## Behavior and ownership

`src/dictionaries.rs` inventories matching readable Hunspell `.aff`/`.dic` pairs.
Its private temporary symlink index prefers explicit imports, then the chosen
folder, then bundled data for matching filenames. WordCraft retains its locale
fallback rules and discovers readable native system dictionaries. Source files
are never modified. Directory inventories are bounded; there is no recursive
host search. Removing a folder leaves bundled and imported fallbacks available.

`src/gnome/dictionaries.rs` owns the weak/cancellable folder chooser and import
controls, explicit Refresh and Stop Using. The selected folder persists in the
existing preferences. Flatpak uses the path granted by the actual folder portal;
no broad host permission is added. Resources update proofing across tabs and
windows without changing document content, language, selection, dirty state,
Undo or the default proofing language.

Language choices report readable file presence and add engine-recognized tags.
Finding files does not imply parser compatibility: WordCraft validates their
format when that language is used. The catalog uses public metadata APIs and
does not eagerly parse unrelated large runtime dictionaries. This avoids the
installed refresh stall captured in `AffixDict::build`. Focus notifications do
not reload global dictionaries; Refresh and opening the language list rescan.

Native Ubuntu offers its Language Support launcher only when the distribution,
executable and desktop entry are present. Flatpak offers folder/import controls.
The complete graphical Ubuntu installation flow remains unverified.

## Validation

The full [native run 42403961297](https://github.com/frandavid100/leaf/actions/runs/42403961297)
passed 193 Rust tests, four Python tests, complete 832-message English/Spanish
catalogs, formatting, documentation links and the previous interface regressions.
Four dictionary regressions cover source priority, changes/removal, incomplete
pairs and discovery without parsing dictionary contents. A real Ubuntu Czech
Hunspell package installed after startup is detected through Refresh and its
actual system path; the new language appears without restarting Leaf.

The [installed run 42410141026](https://github.com/frandavid100/leaf/actions/runs/42410141026)
passed GNOME 49 and the relocated GNOME 51 package. Both exercise actual folder
selection, changed-word reload, cancellation, chooser-owner cleanup, restart
persistence, forgetting the folder and unchanged document state, in English at
scale 1 and Spanish at scale 2. The raw host source is asserted unreadable before
the folder grant. General smoke, old pointer controls, printing and the existing
AT-SPI/Orca checks remain included. The packaged executable matches the final
fully validated native binary; all thirteen public release assets are verified.

The driver enters GTK's folder browser and pastes the synthetic path atomically
before activating the accept mnemonic, avoiding inline completion/Recent-mode
ambiguities reproduced in standalone GTK controls. CI sessions use private host
mount namespaces to avoid [portal mount collisions](https://github.com/flatpak/xdg-desktop-portal/issues/2140).
The runner binds fresh owned runtime storage at its unchanged standard path
inside a private host mount namespace, preventing inherited stale FUSE mounts.
It creates only that test environment with sudo and immediately restores
the original UID/GID before D-Bus, the portal or Leaf; the launcher asserts that
UID. Rootless user namespaces cannot mount FUSE on this runner. Application
permissions stay unchanged. Selection and restart share one test namespace.

## Manual test and limits

The separate `Leaf-0.1.0-alpha.34-dictionary-test.zip` contains a minimal synthetic
Italian dictionary and [Spanish instructions](../tests/fixtures/dictionary84/README.md).
It is never installed automatically as a production dictionary. David still
needs to test the workflow on GNOME OS, including an actual host dictionary
folder when available. Synthetic data validates the folder workflow, not GNOME
OS dictionary provisioning.

Readable files still need a Hunspell dialect supported by WordCraft. Ubuntu's
`hunspell-nl` 2:2.20.19-2 fails its KEEPCASE/FLAG-order parser; a direct public-API
probe reproduces that independently of GTK/Leaf. The positive native control
uses `hunspell-cs` 1:24.2.1-1. No WordCraft, dependency or vendor changes are
included. The original interaction pointer driver is unchanged. No private
documents or gallery updates are included; no additional issue is resolved
by this test candidate.
