# Leaf for Windows — experimental

The first Windows x86_64 package is being prepared from alpha.36. This page does
not claim it is published until the release and checks have been verified.

## Use

The setup executable installs for the current user without administrator access.
The ZIP includes the same runtime: extract it and open `bin/leaf.exe`. No separate
GTK, Rust, GNOME, MSYS2 or Linux installation is required. Both preserve Leaf's
existing interface. The installer adds a Start menu entry and an optional desktop
shortcut; it does not take over default document associations.

Preferences and imported dictionaries are stored in `%APPDATA%/Leaf`, and recovery
in `%LOCALAPPDATA%/Leaf/recovery`. The ZIP also uses these folders; it is a portable
application package, not a self-contained user profile. Removing the installation
preserves these user files. Set your full author name in Preferences. The default
template folder is `%APPDATA%/Microsoft/Windows/Templates` when it exists; choose
another folder through Preferences when needed.

## Build and release

[Windows workflow](../.github/workflows/windows.yml) installs GTK/libadwaita from
MSYS2 UCRT64 and uses Rust's GNU Windows target. WordCraft and Cargo.lock remain
pinned. The optimized binary is assembled by
[package_windows.py](../tools/package_windows.py) with transitive native DLLs,
icons, gettext catalogs, schemas, dictionaries, runtime licenses and a package
manifest. [Inno Setup definition](../packaging/windows/leaf.iss) wraps those same
files in a per-user installer.

[test_windows.ps1](../tools/test_windows.ps1) tests a ZIP extracted under a Unicode
path with spaces, removes developer tools from PATH and checks real GTK windows
in English/Spanish. The package diagnostic decodes its SVG icon, finds its bundled
dictionaries, saves/reopens/overwrites DOCX and ODT in Unicode paths, exports PDF
and persists preferences. The workflow tests installation, the installed program
and removal. A separate Linux job checks formatting, catalogs, Python/Rust tests,
optimized GUI compilation and localized windows before publication.

Use **Actions → Leaf — experimental Windows package → Run workflow** when a new
Windows build is wanted. Enable `publish` to release its verified artifacts. Ordinary
Linux releases do not trigger Windows builds. During setup, changes to this Windows
packaging workflow also run its checks; `[windows-release]` on such a main commit
explicitly enables publication after those checks. Artifacts include source/run
identity and SHA-256 sums. Releases use a separate `-windows.N` tag and do not move
Linux tags. Signing is not configured.

## Limits

Automated success is not human validation on a personal Windows computer.
Physical printing, assistive technology and GPU-specific behavior are unverified.
The packaged checks use GTK's Cairo renderer. File chooser, clipboard transfers
with third-party apps and real printer drivers still need interactive testing.
Use copies of important documents; the existing DOCX/ODT preservation limits apply.
An unsigned build may show a Windows reputation warning. Runtime package versions
are recorded in BUILD.json, with licenses and source links in the package.
