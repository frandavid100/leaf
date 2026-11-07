Leaf 0.1.0-alpha.34 — dictionary integration test candidate

- Open Dictionary Settings from Text → Language to choose an additional Hunspell folder through the native file chooser, refresh changed dictionaries, remove that source or import a matching .aff/.dic pair.
- Combine imported, selected-folder, bundled and readable native system dictionaries. Keep document language and Undo unchanged when resources change.
- Show dictionary availability and add discovered languages recognized by WordCraft. Native Ubuntu may offer its installed Language Support application; Flatpak uses the folder portal and does not request broad host access.
- Include a separate synthetic dictionary ZIP with Spanish test instructions. Its invented Italian words are test data, never installed automatically.

Issue #84 remains open pending David’s GNOME OS test. Automated native and installed GNOME-runtime checks are recorded with this candidate; these do not claim human GNOME OS validation. WordCraft remains pinned at 0.16.0.
