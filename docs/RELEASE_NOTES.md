# Release and validation notes

These notes were moved from the README on 2 November 2026. They describe the
releases and checks recorded at that point; consult [GitHub Releases](https://github.com/frandavid100/leaf/releases)
and [project status](../PROJECT_STATUS.md) for the current release and work in progress.

## Recorded alpha updates

Alpha.16 adds immediate formatting and compact editable Font/Size selectors, selection-link editing, comment-anchor navigation, theme/zoom/editing-mode menus, user identity preferences, direct saving of opened files, current-cursor page numbers and page-view refinements. The issue review closed 14 reports and leaves 18 open, including eight added during the work and the partially improved selection-sync report. [Detailed results and validation](ISSUE_SWEEP_ALPHA16.md).

Alpha.15 integrates bookmarks under navigation headings, combines each comment thread into one bubble and makes spelling rows navigate directly to their text. Document Properties adds title, subject, author, keywords and category. DOCX preserves all five fields; ODT cannot retain category, so Leaf proposes DOCX when it is set. [Validation and limits](SIDEBAR_REFINEMENT_VALIDATION.json).

Alpha.14 adds bookmarks and threaded comments through Insert/context menus and a shared sidebar, plus a separate spelling/grammar sidebar with clickable issues. Check-as-you-type is in Preferences. DOCX preserves comment threads; Leaf blocks ODT saves containing comments because the pinned exporter omits them. [Annotations and format limits](BOOKMARKS_COMMENTS.md) · [Proofing sidebar](PROOFING_SIDEBAR.md).

Alpha.13 reduces unnecessary frontend work when opening longer documents, moving the caret, updating thumbnails and editing with search closed. WordCraft and dependencies are unchanged. [Measured results and limitations](HOST_PERFORMANCE_AUDIT.md). Alpha.12 adds initial Orca access to ordinary document text, cursor and selection on the visible canvas. Native and installed Flatpak tests confirm reading, selection announcements, typing and undo with real AT-SPI and Orca 46.1. [Scope, evidence and remaining accessibility limits](ACCESSIBLE_DOCUMENT_TEXT.md). Alpha.11 fixes standard ODF header/footer geometry after independent Writer edits. [Focused audit and confirmed limits](INTEROPERABILITY_AUDIT.md). Alpha.10 adds section-aware headers/footers, numbering controls, local spelling review and text languages. [Usage, validation and ODT interoperability limits](PAGE_OPTIONS_PROOFING.md). Alpha.9 adds direct header/footer editing and automatic numbering in the document menu’s Insert submenu. [Usage, validation and limits](HEADERS_FOOTERS.md). Alpha.8 provides English/Spanish localization and dark document pages that follow GNOME appearance. [Help translate Leaf](TRANSLATING.md). Hosted Weblate activation is pending the owner’s account setup; PO contributions through GitHub are available now. The alpha.6 lifetime fixes remain included; [audit and limits](GTK_LIFECYCLE_AUDIT_2026-10-31.md).

The installed package has passed automated launcher, interface, document-portal opening/saving and persistent-data upgrade/reinstall checks on Ubuntu CI. Manual testing on GNOME OS and with your own documents is still needed. This standalone bundle has no automatic Leaf update feed.

Experienced testers can build the development branch. Requirements: Rust 1.95 or newer, Git, pkg-config, GTK 4.14+ and libadwaita 1.4+ development libraries. The existing CI runs on Ubuntu 24.04; other host environments have not been comprehensively tested.

```sh
git clone --branch wordcraft-saving https://github.com/frandavid100/leaf.git
cd leaf
cargo test --locked
cargo build --release --locked --features gui --bin wordcraft-gnome
./target/release/wordcraft-gnome
```

The source-build target retains its internal name, `wordcraft-gnome`; the installed application is Leaf, ID `io.github.frandavid100.leaf`, and its installed executable is `leaf`. GNOME OS users should install the Flatpak rather than add development packages to the host.

