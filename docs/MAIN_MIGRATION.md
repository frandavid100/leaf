# Consolidation of Leaf on main — November 2, 2026

## Previous state and preservation

- Previous main: `58da57efcfd03e7b442b20629703d32a697274f4`, archived unchanged
  in [archive/apexdesk-main-2026-11-02](https://github.com/frandavid100/leaf/tree/archive/apexdesk-main-2026-11-02).
- Remote Leaf: `61f3045d63eb5c0f985c4adacc338f51172daa52`, preserved in
  [wordcraft-saving](https://github.com/frandavid100/leaf/tree/wordcraft-saving).
- Common ancestor: `6ab4b958c9ae78b87706fff8d8e629791e2e21de`.
  Main had 19 exclusive commits; Leaf, 473. The integration uses a merge commit
  with both tips as parents; it does not replace or squash their histories.
- Previous checkout clean. Its 12 local commits and the 12 divergent remote ones
  had the same final tree; the original checkout and the local branch
  `archive/local-wordcraft-saving-2026-11-02` are kept. Work isolated in another
  worktree.
- There were no open pull requests to redirect.
- Latest release: [v0.1.0-alpha.18](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.18),
  source/tag `a8207cbdb5b11f050373287a44aecd835aefd50c`. No other release is
  created and no tags are moved for this migration.

## Review of exclusive changes

The 19 commits specific to main and their complete diff from the ancestor were
compared: only six files changed. The merge of the rest of Leaf was automatic.

| Files from main | Resolution |
| --- | --- |
| README.md | Same current Leaf presentation on both branches; David's text, attribution and screenshot are kept. Relative links and images now resolve from main. |
| AGENTS.md and CONTRIBUTING.md | The main versions were redirection guides. The complete Leaf guides are kept, updated for developing, building and contributing on main. |
| Three issue forms/configuration | They match Leaf; they are kept. The active link to TESTING moves to main. |
| Old ApexDesk code, dependencies and binaries | Leaf's evolution from the common ancestor replaces the prototype. The exact previous state remains accessible in the archive branch and in the merge history. |

Exclusive main commits reviewed (chronological order):

```
c4b0be9 Point Leaf landing page to current development and enable testing issue forms [skip ci]
b131b30 Link first installable Leaf Flatpak prerelease from repository landing page [skip ci]
22e0d91 Point project landing page to columns alpha.2 [skip ci]
b68a406 Link HiDPI alpha.3 from repository landing page [skip ci]
3bc4704 Surface confirmed ODT preservation issue on landing page [skip ci]
bd220a6 Point Leaf landing page to alpha.4 and verified ODT fixes [skip ci]
444f18e Link alpha.5 printing release and validation from Leaf landing page [skip ci]
901ff79 docs: link alpha.6 and verified GTK memory audit
0362c8f docs: link alpha.8 and community translation instructions
ed5867a docs: point landing page to verified alpha.9 with editable page headers and numbering [skip ci]
9f87bef docs: point landing page to verified alpha.10 section options and local spelling review [skip ci]
623b0e5 docs: point landing page to verified alpha.11 and independent interoperability audit [skip ci]
c260bc7 docs: point landing page to alpha.12 and first-stage Orca support [skip ci]
fe04fc4 docs: link alpha.13 host performance release from landing page [skip ci]
952df58 docs: direct human and automated contributors to current Leaf modules and handoff guide [skip ci]
40488be Show current Leaf presentation on the repository landing page
846762c Clarify Leaf's engine capabilities and AI-generated implementation
885c701 Lead with Leaf's goal of advanced word processing in a focused GNOME interface
58da57e Condense Leaf presentation around focused editing, capabilities and transparent development
```

On the Leaf side, the current development tree is kept in full, except for the
migration adjustments: guides and active links, help/translation URL in About
and AppStream, branch filters and publication guards in Actions, and Weblate
preparation. Cargo.toml/Cargo.lock, vendor, patches, tests, catalogs,
screenshots and historical reports are kept. The About URL changes in a single
line, without altering gettext messages or their location in the code.

## Validation

Local checks: package formatting correct, four Python tests, 587 gettext
messages with complete and compiled catalogs, links in 14 documents and YAML of
five workflows correct. 123 Rust tests passed.
Optimized release GUI build passed (3 min 26 s); `--version` keeps alpha.18.
Ten pre-existing build warnings; no errors.
Integration commit published: `770a43cfa094db05308cef9980c5f0273e68a3e5`.
[CI of the integration on main](https://github.com/frandavid100/leaf/actions/runs/41410124180):
**passed**, completed at 08:25:56 UTC on November 2, 2026.
Native job `125161018239`: build and complete graphical regressions,
controls/mouse, languages/dark theme, headers, sections, comments, GTK memory,
keyboard/printing, debug/release comparison and 100/200% scaling.
The Flatpak publication and the optional base audit were skipped according to
their conditions. No other version was published.

The two parents and the identity of the local/remote tree were verified. The 17
tags and the 17 releases (including their assets) keep their identifiers and
content.
Flatpak publication still requires `[release-alpha]`. The later commit only
records this evidence in this report (`[skip ci]`); it does not modify code,
configuration, workflows or catalogs relative to the validated commit.
The optional audits keep their historical hashes, sources and conditions.

[Weblate configuration and pending steps](WEBLATE_SETUP.md).
