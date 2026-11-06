# Localization and dark documents — 1 November 2026

Leaf's native host now uses English source messages and complete initial English/Spanish GNU gettext catalogs. The extraction currently contains 520 messages across menus, toolbar options, dialogs, help, accessible labels, host errors and desktop/AppStream metadata. Initial translations are AI-assisted and welcome human review.

## Translation architecture

`src/i18n.rs` binds the `leaf` gettext domain at process startup, before GTK/engine workers. It uses the GNU/Linux runtime already present in the platform; no new Rust dependency or upstream engine change is required. Catalog strings are copied once into an immutable process-owned message table so GTK controls can borrow stable strings without retaining document/window objects. Language changes take effect after restarting the application.

Gettext provides the system locale, LANGUAGE fallback list and language-specific plural evaluation. English is the untranslated fallback. The runtime plural API is available for count messages; current labels mostly avoid dynamic counts. Named placeholders are inserted verbatim and can change order in translations. CI validates that translations preserve placeholders, and unit checks cover reordered arguments, escaped braces and braces in document names.

`tools/update_translations.py` extracts literal `t!()` calls and translatable metadata, generates the POT and runtime message index, validates/compiles all PO catalogs, and generates translated desktop/AppStream metadata. English and Spanish are required to be complete; additional languages may be partial. It reads PO files through GNU msgfmt and Python's GNUTranslations, supporting Weblate's multiline PO output. The installed Flatpak includes the catalogs in `/app/share/locale`; no runtime network service is needed. Source builds can use `LEAF_LOCALEDIR`.

[Contribution guide](TRANSLATING.md). Hosted Weblate has **not** been activated: its browser reached sign-in, and the owner must complete account access, service terms and GitHub authorization. [Prepared configuration](WEBLATE_SETUP.md). Until then, contributors can edit PO files through GitHub and submit PRs. Do not claim that a live Leaf Weblate translation portal exists.

## Dark document rendering

The existing pinned WordCraft `RenderOptions.dark` API handles screen colors, preserving image colors and adapting text/highlights. Leaf applies it to document pages and thumbnails according to libadwaita's dark appearance, including live changes. It clears cached renderings when appearance changes, uses a lighter caret in dark mode, and disconnects its weak theme subscription on tab closure. The document model, selection and undo history are preserved. Printing/PDF export and the ordinary headless rendering API retain document colors.

Headless regression: render a document containing a colored image in light/dark mode, verify equal geometry, different paper, retained image-colored pixels, unchanged document/selection/revision/history, then save and reopen DOCX/ODT and confirm light paper remains the document default. This is a focused check, not a rerun of the 24-file corpus.

Native and sandbox localization regressions launch separate processes for English and Spanish, verify actual menu labels and untitled document names, toggle dark/light live, inspect pixels in the actual GTK page/thumbnail paintables, and confirm unchanged document state. Existing lifetime, clipboard, interaction, keyboard/print and HiDPI tests remain required.

## Delivery and limits

Published: [Leaf alpha.8](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.8). Native source `eaa83c9f6acf88478e8eae9aa3c7aad36fddf6ac`, [run 41285353136](https://github.com/frandavid100/leaf/actions/runs/41285353136), job 124785422076: all checks passed. This includes the 520-message/catalog checks, headless regression, release GTK build, both language/live-texture tests, existing UI/keyboard/printing/HiDPI regressions and lifetime checks. No full 24-file compatibility audit was repeated.

Release/tag/package source: `23207f0be9c352c31cedcd4fa274251007847169`. [Final package run 41286130576](https://github.com/frandavid100/leaf/actions/runs/41286130576), job 124787688014, passed. Only packaging/publication rules changed after the native run: a source-identity check covered src, Cargo.toml, Cargo.lock, tests, vendor, patches and po before reusing its binary. Development screenshots were excluded from release assets; validation captures remain internal CI artifacts.

Inside the installed GNOME 49 Flatpak: English and Spanish actual menu/document-name checks, live dark/light page and thumbnail pixels, lifetime regression, upgrade from alpha.6, desktop launcher, external document portal, data persistence/reinstallation and print portal checks passed. Both native and package artifacts were independently downloaded and their ZIP digests matched GitHub. Every published SHA256SUMS entry passed locally. The native dark-document capture was inspected for validation; no new promotional screenshots were added.

Bundle `Leaf-0.1.0-alpha.8-x86_64.flatpak`: **10207640 bytes**, SHA-256 `1aec3ec70c9e310c1480b5468973b7ca4e206c2612dff652b521b92e7984c264`.
Native artifact 13074788467: ZIP SHA-256 `7c0a3ac7cde3a97c262827c4c9e44f45a60650ebbf9fcbf7091fefc9c93f9c6d`.
Package artifact 13075286512: ZIP SHA-256 `8461b14cfddb08c11708195462263810ac92da1d83d7d47c2084e6bed75c0cb4`.

The earlier alpha.7 candidate passed native and installed checks, but its publication failed with GitHub's workflow/target-commit permission guard after its branch head was superseded. It was not released. Alpha.8 adds the remaining validation messages and actual texture checks; superseded sources now skip publication. The final alpha.8 release is verified above. Screenshots for public promotion remain the maintainer's task; development captures are for internal validation. Automated tests do not constitute manual GNOME OS, Wayland/multi-monitor, Orca or physical-printer validation. Upstream error messages and document-defined style names retain their original language.
