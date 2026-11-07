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
removal and unchanged document state. Native CI installs a Czech Hunspell package
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

## Dictionary dialect limit found during validation

The first native run installed Ubuntu's real `hunspell-nl` 2:2.20.19-2, but
WordCraft 0.16 rejected line 54: `KEEPCASE Kc` appears before `FLAG long`. A direct
public-API probe reproduces this without GTK or Leaf. Leaf must not advertise
that unreadable dictionary as available. No engine code is changed here.
The native positive control uses Ubuntu `hunspell-cs` 1:24.2.1-1, whose `cs_CZ`
dictionary loads correctly through the same public APIs. The portal fixture
continues to use its invented Italian words.

## Local native selector check

The native folder/reload/cancel/owner-close/restart/forget diagnostic passed
locally through a TCP Xvfb display, GTK 4.14 and an isolated profile. Synthetic
character-by-character path entry plus Return interacted with GTK's inline
completion, returning a different path; a standalone GtkFileDialog reproduces
that without Leaf. The test driver pastes its synthetic path atomically and
activates the translated accept mnemonic. This exercises the real chooser and
resource loader; it does not replace the installed Flatpak portal checks.

## Portal chooser control correction

The first installed run passed its general sandbox and spacing/navigation tests,
but the new folder driver tried to confirm GTK3's Recent mode while its accept
button was disabled. A standalone GTK3 chooser reproduces this without Leaf.
The Flatpak-only driver now enters the folder browser with Alt+Home before
pasting its requested path and activating the accept mnemonic. A standalone
GTK3 control verifies the exact selected source. It never accepts the Home
folder. The native branch of the driver is unchanged byte for byte.

The binary is reused from the fully successful native run: all compiled inputs
are still required to match; only this uncompiled pointer driver is excluded
from the source comparison. The portal test runs before the longer installed
regressions, which remain mandatory. Actual installed portal success is still
pending and must be recorded before publication.

## Focus-triggered reload correction

The next installed run loaded the dictionary through its real document-portal
path, then stalled during window-focus handling after the source file changed.
A captured main-thread stack enters Leaf from GTK's focus notification; the
background threads are waiting normally. The manager no longer reloads global
resources from that notification. Refresh remains explicit and language-list
opening still rescans sources. Native and installed checks must validate this
change before publication. Temporary early native artifacts retain symbols for
diagnostics; final native and packaged executables are stripped as before.
