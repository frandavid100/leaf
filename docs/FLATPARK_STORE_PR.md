# Ready FlatPark proposal

The complete change is preserved in
[packaging/flatpark/store-sync.patch](../packaging/flatpark/store-sync.patch),
based on FlatPark `03438192ca16eee4343f26301d1a4c2bb24e7ec1`.
The complete tested tree is now pushed to
[frandavid100/flatpark:fix/leaf-store-sync](https://github.com/frandavid100/flatpark/tree/fix/leaf-store-sync)
at `8f54ac57f042702e0ed3b6f56af0c72bd05ce0e0`.
The integration rejects creating a PR on the external FlatPark repository (403),
and the browser backend cannot start. The PR has not been created or published.
Open the [comparison](https://github.com/flatpark/flatpark/compare/main...frandavid100:fix/leaf-store-sync?expand=1)
using the title and body below.
Changes to shared automation require a FlatPark maintainer's review.

## Proposed title

fix(leaf): synchronize exact-version store presentation and release news

## Proposed body

Leaf's catalogue entry still uses an older description and one screenshot, and
recent releases have empty news. This change imports the approved upstream
presentation and English/Spanish news for the exact packaged version, while
retaining FlatPark's identity, category, links, controls, licences and separate
installation/permission text.

The initial update pins the already published alpha.35 archive. Its presentation
comes from reviewed immutable Leaf commit
`29d13fbd8d4f8f78c6f56fa94954cb45cf8fa5bf`, which corrects metadata for that same
version. Later versions use their release tags. The existing approved gallery
is retained; new maintainer captures are pending visual fixes in Leaf #186/#191.
There is no runtime, wrapper, permission or application-code change.

The optional pin-updater import checks component ID, version, nonempty news and
final AppStream validity before writing either manifest or metainfo. It can
refresh same-version presentation without re-pinning unchanged application
bytes. Other resolvers retain their existing behavior.

Validation completed:

- Four import/integration tests, including initial/same-version updates,
  idempotence, preservation of distributor fields/history and rejection of
  invalid IDs, versions, empty news and malformed prose without partial writes.
- Existing `tests/test_update_pins.sh` passed.
- Live resolver updated alpha.34 to alpha.35 using the release asset API's
  digest/size; a second complete updater invocation made no changes.
- Registry descriptor validation/audit, Bash/Node syntax, whitespace checks and
  AppStream 1.0.2 validation of the final registry metainfo passed.

The alpha.35 application archive already has upstream native/installed GNOME
49/51 validation recorded in its public release. This metadata session did not
build or launch another installed Flatpak: Flatpak is unavailable here. The
signed catalogue and its appearance in Bazaar still need verification after
maintainer acceptance/publication. No claim is made that a local metadata
update has already refreshed users' store clients.

This change was prepared with AI assistance for Leaf's maintainer. Related:
https://github.com/frandavid100/leaf/issues/185 and
https://github.com/frandavid100/leaf/pull/192.
