# Store presentation and release news

Issue [#185](https://github.com/frandavid100/leaf/issues/185) tracks the approved
English/Spanish description, light `#E4F2E8` and dark `#1B3327` brand colours,
versioned news and the separate FlatPark catalogue.

## Editorial source

[data/releases/news.json](../data/releases/news.json) is the reviewed source for
new release highlights and material limitations in English and Spanish. The
recent entries were backfilled from the corresponding published GitHub release
bodies, linked by each entry's `evidence` field. Alpha.35 uses its actual
publication date, 8 November 2026. Historical public assets are unchanged.

Before packaging a version, add its actual changes and run:

```sh
python3 tools/release_metadata.py
python3 tools/update_translations.py --compile
python3 tools/release_metadata.py --check --check-urls
python3 tools/update_translations.py --check --compile
appstreamcli validate --no-net data/io.github.frandavid100.leaf.metainfo.xml
```

The first generator updates AppStream release descriptions in the template and
`packaging/RELEASE_NOTES.md`. The existing publish workflow uses that Markdown
for the GitHub release. Translation generation handles individual paragraphs
and list items, preserving AppStream's `xml:lang` rules. The packaging check
rejects missing news for the actual app/package version. A focused metadata CI
job also checks the referenced screenshots online and validates AppStream.

The general description is translated with the existing gettext catalogs;
release prose is translated once in the news file. Never maintain a second
hand-written release summary in the packaging directory.

## Screenshots and store clients

The existing alpha.28 gallery is retained at an immutable commit. It has not
been replaced by CI captures. David's new screenshots remain pending the
visual fixes in #186 and #191; take and approve them one at a time.

Branding follows the AppStream specification. The placement and exact use of
these colours depend on the client. Metadata validation does not establish
that Bazaar has refreshed or uses the expected origin.

## FlatPark synchronization

FlatPark installs its own catalogue-facing metainfo. Updating Leaf's embedded
file alone does not update the store. The prepared FlatPark change opts Leaf's
resolver into importing the approved summary, description, colours, screenshots
and release notes from the exact packaged version, while retaining FlatPark's
IDs, URLs, categories, controls, licensing and installation/permission prose.

Future updates read the immutable release tag. A metadata-only correction to
an already published version may name an explicitly reviewed immutable commit
in `presentation-sources.json`; it must still contain that exact version as its
latest release. Never fetch presentation from `main` or use a later version's
capabilities for an older binary.

The proposed pin updater prepares and validates the combined result before
writing either file. Same-version presentation updates remain possible without
changing archive digests; mismatched IDs/versions, empty notes or invalid
AppStream fail before writes. The import has regression tests and does not
change the runtime, wrappers, sandbox permissions or application bytes.

The [ready proposal and validation record](FLATPARK_STORE_PR.md) preserve the
complete external patch and uploaded fork branch. [FlatPark PR #1074](https://github.com/flatpark/flatpark/pull/1074) is now open;
its CI is awaiting maintainer authorization for the fork. The FlatPark change
still needs acceptance and publication by its maintainers. After publication,
verify the served catalogue and the actual FlatPark origin in Bazaar: translated
description, colours, gallery, version and nonempty news. Do not mark #185
complete while new captures, distribution acceptance or client verification
remain pending.
