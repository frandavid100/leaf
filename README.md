# Leaf

**Create and edit documents.**

Leaf is an experimental GTK4/libadwaita document editor for Linux, powered by [WordCraft](https://github.com/storytold/wordcraft). It is an independent project designed for GNOME.

**The active project is on [`wordcraft-saving`](https://github.com/frandavid100/leaf/tree/wordcraft-saving).** This default branch contains an older ApexDesk prototype and is retained as history. Do not build this branch to try the current editor.

[Read the current README and see screenshots](https://github.com/frandavid100/leaf/blob/wordcraft-saving/README.md).

## Status

Alpha.4 repairs the ODT header/footer and repeating-row defects found in our one-off audit. [Follow-up results and remaining limits](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/ODT_PRESERVATION_2026-10-31.md). Older alpha releases still have those defects.

The current development version implements DOCX/ODT editing and saving, PDF export, native printing, section columns, styles, tables, inline images, tabs and sidebar search/replacement. Complex document fidelity is still being tested: use copies of your files.

**[Download the current x86_64 Flatpak alpha](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.10).** Open the `.flatpak` with GNOME Software or follow the release's INSTALL.md. The GNOME 49 runtime is downloaded from Flathub if required. Leaf itself is not on Flathub. This is experimental: use copies of your documents.

Alpha.5 adds native printing (document menu / Ctrl+P), F6 pane navigation and clearer accessible control names. [Validation and remaining accessibility/printing limits](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/KEYBOARD_PRINTING.md). The custom document canvas does not yet provide full screen-reader text access.

Alpha.6 fixes retained GTK tab/window state, explicitly disconnects timers/controllers and adds focused lifetime regressions. Both native and installed Flatpak checks passed; measurements, source revisions and limits are documented in the [GTK ownership/memory audit](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/GTK_LIFECYCLE_AUDIT_2026-10-31.md).

Alpha.8 adds English/Spanish localization, installed gettext catalogs and document pages/thumbnails that follow GNOME’s dark appearance using WordCraft’s existing renderer. [Implementation and validation](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/LOCALIZATION_DARK_MODE.md). [Help translate Leaf](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TRANSLATING.md): PO contributions are welcome on GitHub; Hosted Weblate activation is pending the owner’s account setup. Public screenshot updates will be supplied by the maintainer.

Alpha.9 adds direct header/footer editing on the document canvas, automatic page numbers and preservation of basic PAGE/NUMPAGES fields in ODT. The document menu’s Insert submenu contains all three actions. [Usage, validation and limits](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/HEADERS_FOOTERS.md). Native and installed Flatpak checks passed, including upgrade from alpha.8.

Alpha.10 adds section-specific first/odd-even page options, numbering, header/footer distances and independent/linked copies, plus local spelling review (F7/context menu), text languages and bundled Spanish dictionaries. [Usage, verified CI and ODT extension limits](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/PAGE_OPTIONS_PROOFING.md). All 87 root tests, native GUI and installed Flatpak checks passed, including upgrade from alpha.9.

## Get involved

- [Testing guide](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TESTING.md)
- [Report a bug or document compatibility result](https://github.com/frandavid100/leaf/issues/new/choose) — English and Spanish welcome.
- [Development status](https://github.com/frandavid100/leaf/blob/wordcraft-saving/PROJECT_STATUS.md)
- [Release preparation](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/RELEASE_CHECKLIST.md)
- [AI assistance disclosure](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/AI_ASSISTANCE.md)

Current frontend development uses substantial AI assistance; scope and validation limits are documented above. WordCraft is pinned, with isolated ODT preservation fixes in wordcraft-formats; other engine crates and the renderer remain unchanged. Current Leaf source is GPL-3.0-or-later.

Pull requests for the current editor should target `wordcraft-saving`.
