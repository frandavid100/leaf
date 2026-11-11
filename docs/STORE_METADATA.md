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

The active gallery now uses David's four approved 11 November 2026 captures:
cover, document structure/layout, Review and the native underline menu. The
1074 × 890 PNGs retain alpha transparency, native rounded corners and compositor
shadows. They are referenced at immutable commit
`233d5b9c1e0201c55c79c62ec8b6a56c12970b2c`, without retouching or substituting CI
captures. English and Spanish captions describe the actual scenes.
[Provenance](screenshots/README.md) and [FlatPark change](FLATPARK_GALLERY_2026_11.md).

Branding follows the AppStream specification. The placement and exact use of
these colours depend on the client. Metadata validation does not establish
that Bazaar has refreshed or uses the expected origin.

## FlatPark synchronization

FlatPark installs its own catalogue-facing metainfo. Updating Leaf's embedded
file alone does not update the store. [PR #1074](https://github.com/flatpark/flatpark/pull/1074)
merged on 9 November 2026 with a static presentation update only. The proposed
generic importer was removed after maintainer review; there is **no automatic
upstream presentation import** in the accepted package.

The existing resolver currently emits only the version/date and archive URL.
The normal pin updater therefore prepends empty release descriptions. The
separate Leaf-only release-notes proposal remains available for maintainers at
[proposal/leaf-release-notes](https://github.com/frandavid100/flatpark/tree/proposal/leaf-release-notes).
Changes to an existing resolver or shared automation are maintainer-owned;
do not add them to a presentation PR.

Prepare static updates from the upstream metainfo for the **exact packaged
version**, preserving FlatPark's identity, licensing, categories, controls,
installation and permission paragraphs. Backfill empty release entries from
that version's reviewed upstream news. Keep already published release bytes and
pins unchanged. Screenshots use immutable URLs, and exported icons must be
copied explicitly into FlatPark's registry.

The [11 November revision](FLATPARK_GALLERY_2026_11.md) updates only FlatPark's
metainfo and two icons. Its revised bilingual copy follows the refreshed README,
with capabilities first and alpha/AI disclosures at the end of the product text. It keeps alpha.44's archive pin/runtime/permissions and
resumes the gallery work authorized by David after approving the four captures.
David approved the final revision and [PR #1139](https://github.com/flatpark/flatpark/pull/1139)
is open. FlatPark's CI requires maintainer authorization; review and catalogue
publication remain pending.

After maintainer acceptance and publication, verify the served catalogue and
the actual FlatPark origin in Bazaar: translated description, colours, four
images with captions, current icon, correct version and nonempty news. Keep
[#185](https://github.com/frandavid100/leaf/issues/185) open until that client check
has been completed. A merged metadata PR alone is not end-to-end verification.
