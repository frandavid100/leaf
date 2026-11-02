# Leaf

<img src="https://raw.githubusercontent.com/frandavid100/leaf/wordcraft-saving/data/icons/hicolor/scalable/apps/leaf.svg" width="96" height="96" alt="Leaf icon">

**Create and edit documents on GNOME.**

Leaf is a document editor for Linux with a native GTK4/libadwaita interface. Write and format documents, work with tables and images, and navigate longer files with page thumbnails and headings. Leaf uses the [WordCraft](https://github.com/storytold/wordcraft) engine for document editing, layout and rendering.

**[Download the Flatpak alpha](https://github.com/frandavid100/leaf/releases)** · [Installation](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/INSTALLING.md) · [Report a problem](https://github.com/frandavid100/leaf/issues/new/choose)

![Leaf editing an illustrated document, with page thumbnails in the sidebar](https://raw.githubusercontent.com/frandavid100/leaf/wordcraft-saving/docs/screenshots/leaf-showcase.png)

*A document open in Leaf, captured by the maintainer. [Screenshot details and photo credit](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/screenshots/README.md).*

Leaf is in **alpha development**. Use copies of important documents: compatibility with complex DOCX and ODT files is still being tested. It is an independent project designed for GNOME, not an official GNOME application.

## Work with your documents

- **Open and share:** open, edit and save DOCX and ODT files, export to PDF and print through the system dialog.
- **Shape the page:** use text and paragraph formatting, reusable styles, section columns, headers, footers and automatic page numbers.
- **Add structure:** insert tables and inline images; edit document information such as title, author and keywords.
- **Find your way:** use page thumbnails, heading navigation, bookmarks and search with replacement.
- **Review your writing:** add threaded comments and use the spelling review sidebar. Some annotations and document properties require DOCX to preserve them.
- **Make room to write:** work in multiple tabs or windows, use controls that adapt to the window width, and follow the system's light or dark appearance.

The interface is available in English and Spanish. Autosave and recovery are configurable in Preferences. [Read the usage guide in Spanish](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/USER_GUIDE_ES.md).

## Try Leaf

Download the **x86_64 `.flatpak` bundle** from [GitHub Releases](https://github.com/frandavid100/leaf/releases), open it with GNOME Software and choose Install. The GNOME 49 runtime is downloaded from Flathub if needed; Leaf itself is not yet on Flathub.

To update, download a newer bundle and install it over the existing version. These alpha bundles do not provide automatic Leaf updates. [Installation and command-line instructions](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/INSTALLING.md).

## Current limitations

Complex layouts, fonts, pagination and format preservation need broader testing. Images are currently inline; floating-image positioning is not exposed. Advanced table editing and some column layouts remain incomplete.

DOCX is required for comments and the document category: Leaf protects against saving those features to ODT while the exporter cannot preserve them. Screen-reader support currently covers ordinary document text, cursor and selection; richer document semantics remain incomplete.

[Current status and known limits](https://github.com/frandavid100/leaf/blob/wordcraft-saving/PROJECT_STATUS.md) · [Open issues](https://github.com/frandavid100/leaf/issues) · [Release and validation notes](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/RELEASE_NOTES.md)

## Help improve Leaf

Try the [15-minute testing guide](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TESTING.md) and [report a reproducible problem](https://github.com/frandavid100/leaf/issues/new/choose). English and Spanish reports are welcome. Share a small, anonymized example when a document does not open or save as expected.

Contributions to [translations](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/TRANSLATING.md), testing and development are welcome. Start with the [contribution guide](https://github.com/frandavid100/leaf/blob/wordcraft-saving/CONTRIBUTING.md), [architecture](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/ARCHITECTURE.md) and [roadmap](https://github.com/frandavid100/leaf/blob/wordcraft-saving/ROADMAP.md).

The current editor is developed on **[`wordcraft-saving`](https://github.com/frandavid100/leaf/tree/wordcraft-saving)**; direct pull requests there. The `main` branch retains an earlier prototype and this project presentation. [Build from source](https://github.com/frandavid100/leaf/blob/wordcraft-saving/CONTRIBUTING.md#build-and-check).

## Credits and development

Leaf is powered by [WordCraft](https://github.com/storytold/wordcraft), which provides the document engine. Leaf develops the native GNOME interface and integration separately, with isolated ODT compatibility patches documented in [patches](https://github.com/frandavid100/leaf/blob/wordcraft-saving/patches/README.md).

Development, tests, packaging, documentation and original icon artwork have used substantial AI assistance under the maintainer's direction. [Scope and validation limits](https://github.com/frandavid100/leaf/blob/wordcraft-saving/docs/AI_ASSISTANCE.md).

Leaf is free software under the [GNU General Public License v3.0 or later](https://github.com/frandavid100/leaf/blob/wordcraft-saving/LICENSE).
