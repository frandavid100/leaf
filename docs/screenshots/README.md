# Screenshot provenance

## Current README gallery: 11 November 2026

David supplied and individually approved these four screenshots for the public
README, including the underline menu in preference to a styles gallery. They are
reproduced byte for byte: no cropping, recolouring, generated UI or retouching.
They show the English interface editing an illustrative fourteen-page
`Portland Head Light Conservation Plan.docx`. The exact installed build and
display scale were not recorded; these images do not establish a release-specific
validation result. The case study's findings, quantities, costs and dates are
fictional, as its cover states.

| Repository image | Original attachment | Pixels | SHA-256 |
| --- | --- | --- | --- |
| leaf-2026-11-main.png | `image(20261111-053702).png` | 1074 × 890 | 2fd6be2a3cc104107b5e58636ab148ec848a5c749f7711cda1bbe05a59001a08 |
| leaf-2026-11-layout.png | `image(20261111-053921).png` | 1074 × 890 | 85c51dcfcb4668f2c6b48d1e742536ff08b05e7050f4c3167899fa217777a2f0 |
| leaf-2026-11-review.png | `image(20261111-054422).png` | 1074 × 890 | 87f34321e6e29c879a11458cd9bfa1c9b2691d66e18e4a9b5c449e144257484b |
| leaf-2026-11-format.png | `Untitled(1).png` | 1074 × 890 | ace3a8a7d9c3c71b49548b31d8a5bef8d68eb28d1629413d0fd345523d81da96 |

Scenes: cover and thumbnails; structure navigation and columns with a wrapped
photograph; Review with a comment and language suggestions; and the native
underline menu over a multilevel list and table. The previous page fragment in
the layout scene is intentional, following David's preference.

The document credits its lighthouse photographs to **Rapidfire**. Source images:
[Portland-Head-Lighthouse.jpg](https://commons.wikimedia.org/wiki/File:Portland-Head-Lighthouse.jpg)
(2009) and
[Portland-head-lighthouse.jpg](https://commons.wikimedia.org/wiki/File:Portland-head-lighthouse.jpg)
(2007), by Rapidfire, available under
[CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/).
The photographs are scaled and partly cropped by the document layout/viewport;
the screenshot files themselves are unchanged. Their photographic content
retains that license; no endorsement by the photographer is implied.

The same approved gallery is now referenced in upstream AppStream and prepared
for FlatPark. FlatPark publication and verification in an installed software
client remain pending under
[issue #185](https://github.com/frandavid100/leaf/issues/185).

## Historical public gallery: alpha.28

The four `leaf-alpha28-*.png` images are unchanged captures from the installed
GNOME 49 Flatpak, source `d2ba113d9ed2bc0dbb4794732c6035a195cba755`.
[Workflow run](https://github.com/frandavid100/leaf/actions/runs/42057994674),
[artifact 13430554090](https://github.com/frandavid100/leaf/actions/runs/42057994674/artifacts/13430554090).
They were generated on 5 November 2026 using `packaging/capture-gallery.sh`,
Xvfb on Ubuntu 24.04, the Cairo renderer, an English interface and display scale 1.
The 1080 × 800 images capture the composited window rectangle, including native
popup surfaces. No controls, document text or window chrome have been retouched.

| Image | Scene | SHA-256 |
| --- | --- | --- |
| leaf-alpha28-main.png | Full page, thumbnails and footnote at 80 % zoom | `193e5070feb5e2ada593e3b54ccea95b2d66c8abe2999cb77389a0709cd3bd6e` |
| leaf-alpha28-notes.png | On-page footnote and separator | `a9cd835f27e774d6347a978e444c3ebba45abde734653c82c7679f27a8d0e062` |
| leaf-alpha28-review.png | Expanded comment thread | `64e35937bf26c9308c29c686f2b7cd46c68e81e4e0d8892f94845101bc41849d` |
| leaf-alpha28-format.png | Native paragraph tools | `0660b433e93718bf7b32c217c757342b368bacd8d6386f37538ddc8bac3aaf6b` |

David explicitly authorized this new public gallery during alpha.28 work. The
Open Studio Handbook text and comment identities are synthetic test data. These
are actual application captures, not generated mockups or a claimed human GNOME
OS session. Fonts and pagination can vary with the runtime's available fonts.
The fixture is available only through a diagnostic flag, not normal startup.

## Earlier maintainer screenshot

`leaf-showcase.png` was supplied by David on 2 November 2026 as
`image(20261102-001247).png` and is reproduced unchanged. It shows Leaf editing
`Leaf-showcase.docx`, with the page-thumbnail sidebar visible. The exact build
version and display settings were not recorded; this capture is not evidence
that every document feature has been verified.

The document photograph is **Tomato Garden** by **Elaine Casap** (2016),
[available on Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Tomato_Garden_%28Unsplash%29.jpg)
under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/).
The screenshot is an actual application capture, not a generated UI mockup.

## Earlier CI captures

The files below are retained as historical development images. They are no
longer featured in the public README.

These older files are unmodified PNG captures from GitHub Actions run [41144295677](https://github.com/frandavid100/leaf/actions/runs/41144295677), source commit `c744dac3f553b0304e631dd80dde7fbe3669e1cd`.

Artifact ID: `13020294789`, `wordcraft-gnome-fase3d-linux`.
Original artifact ZIP SHA-256: `3e46a27293358c284d989ae42fc8de0ce4efe862eb0e093d0ada96935b566568`.

| Repository image | Original artifact file | Scene |
| --- | --- | --- |
| editor.png | vista-gnome.png | Paginated editor and thumbnails |
| search.png | vista-buscar.png | Sidebar search and replacement |
| dark.png | vista-oscura.png | Dark window chrome, white document pages |

The screenshots use the synthetic Open Studio Handbook test fixture under Xvfb on Ubuntu 24.04, with Spanish interface strings. They are not photographs of a GNOME OS session and do not demonstrate Flatpak installation. The black area outside the window belongs to the virtual display. No window chrome, text or controls have been retouched.

These captures predate Leaf branding integration; the application/window may still use its development labels. The maintainer-supplied November 2026 gallery above is now featured in the public README. Screenshots and the test document are not additional shipped sample documents in normal startup.
