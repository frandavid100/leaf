# Spelling, grammar and language

The third revision replaces the global check-as-you-type switch. Preferences
offers a default language and default checking **only for new documents**;
Properties → Information changes the document's inheritance. Text → Language
offers Do Not Check, Default and the languages. Default clears the direct
properties; an explicit language enables checking in that run. Styles and
explicit runs are not rewritten when the document changes.

The fourth sidebar tab shows the list without initial instructions or a Refresh
button. It is refreshed with a debounce after edits or configuration changes,
discarding results from another identity/revision. A click only selects and
reveals the text; corrections are in the context menu. F7 opens the same list.
Automatic checking uses each run's effective properties. Changing Preferences
does not alter open documents, visible errors or their history.

DOCX keeps inheritance and exceptions; the pinned ODT exporter flattens
languages and loses certain false values. Leaf requires DOCX when checking is
disabled in the document or when an explicit exception enables it. Language
inheritance by itself remains a limitation of ordinary ODT.

The document configuration can be undone. Opening, analysis and navigation do
not change content or the modified state. Uninstalled dictionaries are marked
in the selector; no alternative engines are added.

[alpha.18 scope and validation](history/ISSUE_SWEEP_ALPHA18.md) ·
[Panel](../src/gnome/spellcheck.rs) · [Model](../src/proofing_defaults.rs) ·
[Preferences](../src/gnome/workspace.rs).
