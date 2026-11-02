# Theme selector

The grouped GtkCheckButton layout and stylesheet are adapted from GNOME Text
Editor, Copyright 2021 Christian Hergert and contributors, GPL-3.0-or-later.
Sources inspected on 2 November 2026:

- https://github.com/GNOME/gnome-text-editor/blob/main/src/editor-theme-selector.ui
- https://github.com/GNOME/gnome-text-editor/blob/main/src/style.css
- https://github.com/GNOME/gnome-text-editor/blob/main/src/editor-theme-selector.c
- https://github.com/GNOME/gnome-text-editor/blob/main/COPYING

Leaf scopes the selectors under `.leaf-theme-selector`, uses its existing
preferences/StyleManager callbacks and synchronizes grouped choices across
windows. Native checkbutton indicators, theme colors and 44 logical-pixel
previews replace the former custom Cairo circles.

PanelThemeSelector was considered first. Libpanel is absent from the supported
native build and existing locked dependency graph; adding it would require a
new library/binding and packaging change for this one control. The established
GNOME Text Editor implementation is used with its attribution and license,
without changing the engine, dependency pins or native minimum versions.
