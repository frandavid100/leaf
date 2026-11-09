# FlatPark store presentation — revised proposal

Updated 2026-11-09 after [FlatPark's review](https://github.com/flatpark/flatpark/pull/1074#issuecomment-6729671898).

[PR #1074](https://github.com/flatpark/flatpark/pull/1074) now has only two changed data files,
with head `45f21e53acfd13ced20990ace0d416aa7ea6e7dc` and FlatPark main baseline
`3c7852ddc2fa7b4ebd4a1990485f2195982d1213`. GitHub reports no merge conflicts.
It remains open, unmerged and unpublished. Leaf issue #185 remains open.

The earlier [store-sync.patch](../packaging/flatpark/store-sync.patch) is retained
only as a historical proposal. It is superseded: do not apply its generic importer,
shared-script changes or alpha.35 pin. The source branch now matches the narrower
scope below. No new Leaf binary release is needed for this work.

The GitHub integration can update the fork branch but returns HTTP 403 when editing
the PR's title/body in FlatPark's repository. The existing PR already links this
page, which provides the corrected scope and validation record.

## Revised PR description

Leaf (`io.github.frandavid100.leaf`) currently has an older description, a single screenshot and empty release descriptions in FlatPark. This updates its store presentation and fills the release notes for the versions already in the catalog, using the published upstream alpha.38 metainfo.

### What changes

- English/Spanish summary and description, four existing upstream screenshots with localized captions and immutable commit URLs, and light/dark branding colors.
- Nonempty bilingual notes and details links for the existing alpha.38, .36, .34, .32, .30 and .28 entries. FlatPark's alpha.27 packaging announcement is retained.
- FlatPark's distribution/permissions paragraphs, app identity, licensing, categories and other packaging fields are preserved.

The final diff contains **only two data files**: `flatpark.yml` (summary) and `io.github.frandavid100.leaf.metainfo.xml`. It is based on FlatPark main `3c7852dd`. The alpha.38 archive pin, runtime, sandbox, wrapper, resolver, shared scripts, tests and documentation match that base. There is no application version bump in this PR.

### Automatic release notes — separate maintainer proposal

Following [the review](https://github.com/flatpark/flatpark/pull/1074#issuecomment-6729671898), the generic importer has been removed from this PR. A small Leaf-only resolver proposal is available separately at [d13cedb](https://github.com/frandavid100/flatpark/commit/d13cedb09ea9429f77227964fe5b967e388f9f34), on `proposal/leaf-release-notes`.

That proposal uses the existing `releaseNotes`/`releaseUrl` path and existing curl/jq dependencies. It reads the matching release description from the exact selected upstream tag. A failed metadata fetch, absent notes or an ambiguous match warns and still returns the binary update. The existing updater validates notes and drops invalid fragments without blocking the pin; its same-version short-circuit is unchanged.

**The resolver proposal is not part of this PR.** An existing resolver is maintainer-owned under the review rules, so a maintainer would need to review and apply that separate commit. This lets the presentation update be reviewed independently.

### Validation and limits

- `appstreamcli validate --no-net registry/io.github.frandavid100.leaf/io.github.frandavid100.leaf.metainfo.xml`: passed.
- `node scripts/audit-descriptor.mjs registry/io.github.frandavid100.leaf/flatpark.yml`: passed.
- `bash tests/test_update_pins.sh`: passed with AppStream validation enabled.
- Eight offline resolver-to-updater integration scenarios passed using the captured alpha.39 GitHub release response and tag metainfo: matching notes, HTTP failure, wrong version, missing/empty descriptions, duplicate matches, malformed metadata and invalid AppStream notes. Every scenario allowed the binary update, produced valid metainfo and left both files unchanged on a repeated run.
- `git diff --check`: passed. Confirmed the four image files exist at their pinned upstream commit through GitHub's contents API; direct HTTP HEAD checks timed out in this environment.

No Flatpak build, installation or application launch was performed for this metadata-only revision. The screenshots are the existing upstream captures, not newly captured images. Final store/Bazaar rendering and catalog publication remain unverified.

Submitted for Leaf's maintainer, with AI assistance.

