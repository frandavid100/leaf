# Leaf icons

The approved blue leaf-shaped quill over a sheet of paper replaces the former
folded-corner document for alpha.42 (#246). Original vector artwork prepared with
AI assistance and approved by David. GPL-3.0-or-later; see ../../LICENSE.

November 2026 revision, approved by David: the quill's midrib now runs to the tip
and the light/dark halves of the blade split along it; the veins start on the
midrib. The symbolic icon is redrawn as a portrait sheet with two text lines and
the same quill, so it no longer resembles the generic edit symbol.

- `hicolor/scalable/apps/io.github.frandavid100.leaf.svg`: exact approved 128×128
  full-color source, transparent, no embedded fonts, rasters or external resources.
- `hicolor/symbolic/apps/io.github.frandavid100.leaf-symbolic.svg`: exact approved
  16×16 monochrome companion, recolored by the native icon theme.
- `packaging/windows/leaf.ico`: rasterized from the approved full-color source;
  updating this source does not publish a new Windows package.

Source names now match the installed application ID, so development builds and
installed builds use the same theme lookup. The existing installer copies them
into its hicolor icon tree. PNG render checks at 32/64/128 pixels use the SVG;
small-size and light/dark native checks accompany the release validation.
No public store screenshots are generated or replaced by this change.

The separate Nautilus search filter symbol is credited in NAUTILUS_SOURCES.md.
GNOME guidance: https://developer.gnome.org/hig/guidelines/app-icons.html
