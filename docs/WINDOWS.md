# Leaf for Windows — experimental

[Download 0.1.0-alpha.44-windows.1](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.44-windows.1), published and verified on 10 November 2026. Windows 10/11 x86_64 is the target;
automated checks ran on Windows Server 2022 x64. The installer is about 35 MB and
the ZIP about 51 MB. This is based on Linux alpha.44 and is an occasional release.

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

The [promotion workflow](../.github/workflows/publish-windows.yml) also accepts a
successful candidate run ID. It verifies source ancestry and candidate digests,
adds current notices, retains the executable unchanged, rechecks both final
packages and publishes those exact bytes. Increment `packaging/windows/sequence.txt`
for a second Windows package of the same Linux alpha.

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

## Latest verified release

Alpha.44 includes the frontend cleanup and restored standard GTK color selectors.
It uses the same application source as the Linux alpha.44 package.

- Executable source/tag: `534c86eab70e05ae5de8c4d1d9c95d075d8ae666`.
- [Build, 239 Windows/240 Linux Rust tests, portable/installed checks and publication](https://github.com/frandavid100/leaf/actions/runs/42900734179).
- 64 native DLLs; relocated and installed checks use the bundled runtime.
- Installer: 34,767,302 bytes; SHA-256 `13310436f06f184156003549f6d097baae5ee3b95027bb8566ded4e53a2caad6`.
- Portable ZIP: 50,677,234 bytes; SHA-256 `32ee1ea47c32f56ee361152d59d076f9d96760e63d9d00bb9a5987c63bf9a29d`.

The four public assets, GitHub digests and source tag were independently checked.
The downloaded build artifact's package hashes match the public asset digests and
`SHA256SUMS`; `BUILD.txt` identifies the source and successful workflow above.
These are automated checks, not a report of personal Windows testing.

## Verified first release

- Executable source/tag: `3f16eac843e91a9a725ddf27ff91faaa32ce5b4d`.
- [Build, 210 Windows/211 Linux tests, portable and installed checks](https://github.com/frandavid100/leaf/actions/runs/42496853724).
- [Final package checks and publication](https://github.com/frandavid100/leaf/actions/runs/42499295215), packaging source `54114e38f9134e6d8c6a08b40913ace4b35eaa0e`.
- 63 native DLLs; no separately installed GTK/MSYS2 was used by the relocated or installed checks.
- Final installer SHA-256: `6377a35c58b6104aff1279b5157e222bef303b85aa4259e0007a6ab66b8a2422`.
- Final ZIP SHA-256: `375b60ef91291b68355a4009f7adc0668d07a1a732eea0b9e948bbd7096867c5`.

The public release is a prerelease and does not replace the latest Linux release.
Its four assets, digests and source tag were confirmed through GitHub's API.
