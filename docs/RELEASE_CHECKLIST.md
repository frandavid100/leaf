# GitHub alpha release checklist

The latest published version and verified evidence are in [PROJECT_STATUS](../PROJECT_STATUS.md).
Leaf already has a GitHub Flatpak distribution; [first-release preparation](history/FIRST_RELEASE_CHECKLIST.md)
is historical. A source/documentation cleanup does not by itself require a release.

## For a new binary release

- [ ] Define the actual changes and known limitations; retain AI-assistance/license disclosure.
- [ ] Update APP_VERSION in [native module](../src/gnome/mod.rs), version assertions,
  package filenames/scripts, release notes and AppStream metadata together.
- [ ] Preserve the pinned engine/dependencies or document an explicitly authorized upgrade.
- [ ] Check formatting, documentation links and gettext generation; run cargo test --locked
  and build the optimized GUI. Run interaction regressions appropriate to the change.
- [ ] Identify the source revision/native workflow run. Reuse a native artifact only after
  the configured git diff proves source identity; never silently reuse an old binary.
- [ ] Validate desktop/AppStream data and the installed sandbox, upgrade from the previous
  release, launcher/document portal, data persistence, relevant features and existing Orca check.
- [ ] Review internal validation output and distinguish it from human GNOME OS, braille,
  printer or real-document testing. Do not invent confirmations.
- [ ] Publish a prerelease only after package validation, with Flatpak, instructions,
  checksums, exact source, runtime and permissions. Verify tag and downloaded assets.
- [ ] Update current status, roadmap and landing README. Store detailed evidence in a
  feature report/JSON; avoid copying the whole release narrative into active planning.

Extensive corpus, independent Writer and performance audits are optional when
justified. They are not automatically mandatory for each release. Public screenshots
will be supplied by David; do not replace them with internal CI captures.

## Distribution details

Application ID `io.github.frandavid100.leaf`, GNOME 49 runtime, x86_64 standalone
bundle. The existing build uses Flatpak build-init/export/bundle on a proven
native release executable. It is not a Flathub source manifest or submission.
The bundle has no automatic Leaf update feed. Keep the portal/minimal-permission
boundary and ensure installation instructions match the actual published package.

## Human participation / later work

Manual GNOME OS, printer, accessibility and real-document checks depend on
available users and must stay distinct from automated CI. Hosted Weblate and
public screenshots remain with the maintainer. Reddit promotion was already
performed; no new post is part of a release by default.

Flathub is a separate later task. Review its official requirements at that time
and arrange human packaging/maintenance participation; do not assume the GitHub
bundle authorizes or satisfies submission. Do not automate publication interactions.
