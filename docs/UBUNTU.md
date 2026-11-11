# Leaf for Ubuntu — experimental deb

This optional package targets **Ubuntu 26.10 on amd64 (Intel/AMD 64-bit)**.
It does not require Flatpak, Flathub or Snap. It uses Ubuntu’s GTK/libadwaita
libraries and the same optimized binary as the published Linux alpha.45.
The first package passed [Ubuntu package CI](https://github.com/frandavid100/leaf/actions/runs/43036417792).
It remains an experimental package for volunteer desktop testing.

## Install

Download `leaf_0.1.0.alpha.45-1ubuntu26.10.1_amd64.deb`
from [alpha.45](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.45).
In the download directory:

```sh
sudo apt install ./leaf_0.1.0.alpha.45-1ubuntu26.10.1_amd64.deb
```

APT obtains the required libraries from your configured Ubuntu repositories.
Launch Leaf from the applications menu or run `leaf`. Use copies of important
documents: Leaf and this distribution package are experimental. Other Ubuntu
releases, Debian and ARM systems are not validated by this package.

Install a newer deb with the same command to update. There is no Leaf APT
repository or automatic update feed. Windows and Flatpak releases are separate.
To remove the program: `sudo apt remove leaf`. Settings and recovery files are
retained in `~/.config/wordcraft-gnome` and `~/.local/state/wordcraft-gnome`;
these are separate from Flatpak’s application data. The native app runs with
your normal user permissions rather than inside the Flatpak sandbox.

## Packaging and verification

[Workflow](../.github/workflows/ubuntu-deb.yml) downloads the existing published
application archive, verifies its GitHub digests, source and candidate manifest,
then runs [packaging and installation checks](../packaging/ubuntu/test-deb.sh)
in a disposable Ubuntu 26.10 container with networking disabled. Required
Ubuntu packages are authenticated and downloaded on the runner using separate
APT lists/cache and the container’s package status; they are not installed on
the runner. The binary is never rebuilt or patched.

The application tree lives in `/opt/leaf`, keeping bundled dictionaries and
translations separate from Ubuntu’s packages. `/usr/bin/leaf` launches it, and
the desktop entry, app icons and AppStream metadata are exported under `/usr`.
ABI dependencies are computed with `dpkg-shlibdeps`. Upstream source identity,
packaging commit and license notices are included in the package.

Checks cover real installation and dependency resolution, launch and translated
GTK windows in English/Spanish, resources and SVG decoding, dictionaries,
DOCX/ODT save/reopen/overwrite in Unicode paths, PDF export, preferences and
removal/reinstallation preserving user settings. The existing diagnostic named
`--windows-package-test` is also valid on Linux; its Windows-specific storage
assertions are conditional. This does not establish human validation of an
Ubuntu desktop, clipboard interoperability, accessibility or physical printing.

## First package provenance

- Application: alpha.45, source `ebd1bb40efcaac8278ab6416df70d9dfd5e3dfb6`.
- Original release validation: [run 43023157112](https://github.com/frandavid100/leaf/actions/runs/43023157112).
- Input archive SHA-256: `138cdb67333384a086cd56190d5fe2f146c2202708fa6ac75751c8db31199203`.
- Unchanged application binary SHA-256: `8c5a1195734e1a830e13431f506f62b40e338926dfbefee850f002584a612d07`.
- Packaging source: `b0b8d99def5e4fd45e0cce799f352b7cb6ad1586`.
- [Ubuntu validation run 43036417792](https://github.com/frandavid100/leaf/actions/runs/43036417792): passed; all 44 upstream files compared unchanged after extracting the deb.
- Package: 15,957,676 bytes; SHA-256 `acec9b0a5185ee6f57c539b65a5fc7b8d53d031773501109557817076efe974d`.
- The matching `.deb.sha256` file is published separately; the original release assets/checksums are retained.

David explicitly requested this experimental deb be uploaded to GitHub for an
Ubuntu 26.10 volunteer tester. This authorizes this distribution artifact; it
does not change the usual RC approval procedure for future application releases.
