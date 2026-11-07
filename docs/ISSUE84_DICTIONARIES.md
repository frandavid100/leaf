# Issue #84 — dictionary integration candidate

David authorized preparing a GNOME OS test candidate on 7 November 2026.
Issue #84 remains open. Alpha.34 is under validation; publication and actual
GNOME OS behavior are not yet verified.

## Behavior

`src/dictionaries.rs` combines explicit imports, one chosen folder and bundled
Hunspell pairs in a private temporary symlink index, in that priority order.
WordCraft still discovers readable native system dictionaries and owns all
spelling, grammar, loading and cache invalidation. Sources are never modified;
removing a folder keeps the bundled fallback. Matching files are scanned with
a bounded directory inventory; no recursive host search is performed.

`src/gnome/dictionaries.rs` offers the native folder chooser, refresh, stop using
and import controls. Flatpak stores the path returned by the file chooser's
folder portal, without adding host filesystem permissions. The chooser is
cancellable and owned by its tab/panel; callbacks use weak owners. Dictionary
changes update proofing across tabs and windows, not the document language,
selection, dirty state or Undo. Newly detected engine-supported language tags
appear in the language list; unavailable dictionaries are dimmed and explained.

Native Ubuntu can offer its installed Language Support launcher when the OS,
executable and desktop entry are present. Flatpak never offers that launcher.
The actual Ubuntu graphical installation flow remains unverified.

## Test plan and current limits

Three adapter regressions cover precedence, changed/removed sources and
incomplete pairs. The real-widget diagnostic and pointer driver exercise
chooser selection/cancellation/owner cleanup, file refresh, restart persistence,
removal and unchanged document state. Native CI installs a Dutch Hunspell package
after startup and verifies rediscovery. Installed Flatpaks exercise the actual
folder portal in English at scale 1 and Spanish at scale 2, on GNOME 49 and 51.
Automated results must be recorded after execution, not inferred from this plan.

The separate dictionary test ZIP contains only invented Italian words and the
[manual instructions](../tests/fixtures/dictionary84/README.md). It is not bundled
as a production dictionary. David must still test on GNOME OS, including an
actual host dictionary folder if one is available there. The synthetic test
proves the folder workflow, not the dictionary provisioning policy of GNOME OS.

WordCraft/dependencies/vendor and the original interaction pointer driver are
unchanged. No private documents or public screenshots are used.
