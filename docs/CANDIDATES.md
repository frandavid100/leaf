# Candidates and publication

The usual procedure agreed with David on 9/11/2026 is to first generate and
validate an RC, deliver it to him for testing, and publish once he approves that
specific candidate. If it is approved without changes, the same files and frozen
notes are promoted, without rebuilding or repeating the test suite already
passed. On publication, the source, tag and checksums of the downloaded files
are verified.

If failures appear that require fixes, another RC is generated with a new
source, run and checksums; it is validated and delivered again for testing.
Preparing an RC does not by itself authorize its publication. Automatic
authorizations from earlier deliveries belong to their history and do not apply
to future versions.

Each candidate contains the source revision, the run identifier, the release
notes and the SHA-256 checksums of the packages. Its internal version is the
planned Alpha; "candidate 1", "candidate 2", etc. distinguish the builds by
their run and hash, without changing the package before publishing it.

## Alpha.32 published

[Candidate 42372618858](https://github.com/frandavid100/leaf/actions/runs/42372618858)
passed GNOME 49/51 and the complete native test suite 42371069496. The
[automatic promotion](https://github.com/frandavid100/leaf/actions/runs/42374275590)
published the same files on 7/11/2026. The tag and the twelve public files were
verified; source `2524dbc789cfbfa0f3b6e153fde16f1df5887b8a`.
[Update, measurements and limits](WORDCRAFT_016_REVIEW.md).

## Alpha.31 published

[Candidate 42361339472](https://github.com/frandavid100/leaf/actions/runs/42361339472)
passed the GNOME 49/51 installations and required the success of the entire
native test suite 42360443648. The [automatic promotion](https://github.com/frandavid100/leaf/actions/runs/42362957503)
published the same files on 7/11/2026. The twelve public digests were verified.
[Source, checksums and scope](history/ALPHA31_REMAINING.md).

## Generating and testing

The integration commit on `main` carries `[candidate-alpha]`. The same marker in
the title of its PR skips only the optional performance comparison; it does not
skip native or installed functional tests. The native test suite and the
installed checks generate the `leaf-flatpak-alpha` artifact.
The usual marker is `[candidate-alpha]` alone, without `[auto-publish-alpha]`:
the packages become available for testing and wait for David's approval.
The `[auto-publish-alpha]` mechanism is kept for a later exception expressly
authorized for a specific delivery, not as the usual procedure.

The integration commit may add `[bundle-only] [native-run:ID]`, with the actual
identifier of a native run from the same repository/workflow. Source constants
in the workflow are not changed for each candidate.

A native build may hand its executable over to packaging after passing the new
controls and the context menu, pinning and Paragraph regressions, run only once
before creating the reusable artifact, while it continues with the earlier
regressions. The workflow checks source identity and equivalence; it applies the
same optimized strip, validates the installations in parallel and requires the
complete success of the native run before accepting the candidate. Finally, it
compares both executables byte by byte. The manifest also keeps the native
source and run. The promotion checks that result again. A failed test does not
produce an approvable candidate.

To install the downloaded bundle:

```sh
flatpak install --user ./Leaf-0.1.0-alpha.32-x86_64.flatpak
flatpak run --user --branch=alpha io.github.frandavid100.leaf
```

When testing it, confirm that "About Leaf" shows `0.1.0-alpha.32`, especially if
you have windows of several versions open. The complete package includes
`SHA256SUMS`; `sha256sum -c SHA256SUMS` verifies the downloaded files.

The same application identifier and its data are kept. A candidate replaces the
version of that same branch installed from a bundle. The FlatPark branch may
also be installed; using `--branch=alpha` explicitly selects the candidate. To
go back, reinstall the previous published bundle.

The candidate's checks must cover the changes listed in its notes and the
documents where the problems appeared. A failure produces another candidate,
with a new source, run and checksums.

## Promoting a validated candidate

The `Publish a validated Alpha candidate without rebuilding` workflow is run
manually with the identifier of the RC approved by David. Automatic promotion is
reserved for the express exception described above. It checks that the run
finished successfully, belongs to the candidate branch and workflow of this
repository, and that the source, the manifest and all the checksums match. It
publishes those files and their frozen notes, creating the tag on the source
that produced the candidate.

The promotion reuses the approved artifact and its complete tests, with the
identity and integrity checks of the publication. It does not run the build or
the functional test suite of that same candidate again. Do not replace the
packages of a release that has already been published: any later fix generates
a new version.
