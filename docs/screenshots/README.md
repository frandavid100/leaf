# Screenshot provenance

## Current public gallery: alpha.28

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

These captures predate Leaf branding integration; the application/window may still use its development labels. The installed alpha.28 gallery above is now featured in the public README. Screenshots and the test document are not additional shipped sample documents in normal startup.
