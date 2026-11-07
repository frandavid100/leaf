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
appear in the language list. Choices without readable dictionary files are
dimmed. Finding files does not imply that their format has been validated;
WordCraft parses them when their language is actually used.

Native Ubuntu can offer its installed Language Support launcher when the OS,
executable and desktop entry are present. Flatpak never offers that launcher.
The actual Ubuntu graphical installation flow remains unverified.

## Test plan and current limits

Four adapter regressions cover precedence, changed/removed sources and
incomplete pairs, plus discovery without parsing dictionary contents. The real-widget diagnostic and pointer driver exercise
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
public-API probe reproduces this without GTK or Leaf. The picker reports only readable file presence; its tooltip explicitly leaves
format validation to WordCraft when that language is used. No engine code is
changed here.
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
are still required to match; only the two uncompiled Flatpak test drivers are excluded
from the source comparison. The portal test runs before the longer installed
regressions, which remain mandatory. Actual installed portal success is still
pending and must be recorded before publication.

## Reload and language catalog correction

Installed run 42402167877 successfully read the updated four-word fixture through
its real document-portal path, then stalled while rebuilding the language list.
The symbolized main-thread stack enters WordCraft's `AffixDict::build` through
Leaf's `additional_languages`, `dictionary_path` and `TextTools::page`. An
independent sandbox read succeeded. The stall comes from eagerly parsing large
unrelated runtime dictionaries while displaying choices, not portal access.

The language catalog now uses the public file-discovery metadata APIs and read
permissions only. Actual parsing remains with WordCraft when the document uses
the chosen language; import verification and the active-language diagnostics
still check the engine. Tooltips distinguish file presence from validation.
A regression covers discovery of readable but intentionally invalid contents.

The manager also no longer reloads resources from window-focus notifications;
explicit Refresh and opening the language list rescan the inventory. Removing
the focus reload alone did not solve the eager parsing stall. Both changes need
fresh native and installed validation before publication. Temporary early native
artifacts retain symbols for diagnostics; final native and packaged executables
are stripped as before.

## Isolated installed portal sessions

The identical binary passed native run 42403961297. Candidate run 42405334031
passed the installed GNOME 49 checks, including the real dictionary portal, restart, general smoke, pointer controls,
printing and AT-SPI/Orca. GNOME 51 then failed before Leaf received its selection:
`fusermount3` could not access `/run/user/1001/doc` and reported Permission denied.
The independent D-Bus sessions were sharing the runtime mount left by earlier
tests. This matches upstream [xdg-desktop-portal #2140](https://github.com/flatpak/xdg-desktop-portal/issues/2140),
whose controlled reproducer avoids the collision with a runtime per session.

The installed dictionary driver now gives each language run a private runtime,
shared by its folder-selection and reopen stages. It still uses the real portal
and verifies that the raw host folder is inaccessible beforehand; no filesystem
permissions are added. Runtime state is never recursively deleted while it could
contain a FUSE mount. All native compiled inputs are unchanged and the fully
successful native binary is reused. Installed checks must pass again before
publication; alpha.34 remains unpublished at this point.
