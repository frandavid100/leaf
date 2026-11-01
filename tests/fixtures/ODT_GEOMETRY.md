These three tiny ODT files are synthetic Leaf regression fixtures, generated for
this project on 2026-11-01. They contain no third-party documents or personal
data. They are distributed under the repository's GPL-3.0-or-later licence.

`odt-geometry-standard.odt` uses only standard ODF namespaces: page-edge header
distance 21pt, footer distance 27pt, 72pt body margins, dynamic header/footer
spacing and Roman page numbering. Its structure follows the properties exposed
by Writer's DOCX-to-ODF conversion; there are no Leaf extension attributes.

`odt-geometry-disabled.odt` retains header/footer style definitions but disables
their master-page contents. Body margins remain 72pt rather than being enlarged
by unused header/footer reservations. ZIP timestamps are fixed for reproducibility.

`odt-geometry-stale.odt` has the same standard properties as the first case, plus
deliberately stale Leaf extension attributes. Those attributes must not override
geometry or a number format changed independently in a different editor.
