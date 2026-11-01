# Translate Leaf

Leaf uses GNU gettext catalogs. The source language is English; the initial English and Spanish catalogs are complete for the native host. Initial translations were AI-assisted and welcome human review. Document content and user-defined style names are never translated. Diagnostic fixture strings and upstream WordCraft error text are outside the host catalog.

## Contribute now

You can edit `po/es.po` on GitHub or submit a pull request targeting `wordcraft-saving`. To start another language, copy `po/leaf.pot` to `po/LANGUAGE.po`, set the Language and Plural-Forms header, and fill the translations. Use a GNU gettext editor such as GNOME Translation Editor, Poedit or a compatible online service. Follow standard GNOME terminology. Preserve complete `{name}` and `{}` placeholders and keyboard shortcuts; named placeholders may change order. Do not translate document examples, action identifiers or engine commands.

Run `python tools/update_translations.py --compile` to update the extraction template, immutable runtime message index and translated desktop/AppStream files, compile catalogs and validate placeholders. It requires Python 3 and GNU gettext (`msgfmt`). Commit PO files and generated text files; `po/locale/` is generated and ignored. CI checks that generated files are current and English/Spanish remain complete. Other languages can be partial: untranslated strings fall back to English.

After adding strings in code, mark literals with `wordcraft_gnome::t!("English message")`, regenerate the POT, and use `msgmerge --update po/LANGUAGE.po po/leaf.pot` to update catalogs before translation. Use `wordcraft_gnome::i18n::interpolate` for messages containing named values; never concatenate translated sentences. The gettext plural API supports language-specific Plural-Forms for future count messages.

The application follows the system locale and LANGUAGE fallback list. Restart Leaf after changing the language. For a source build, compile catalogs before launching. `LEAF_LOCALEDIR` can select a catalog directory for development. The installed Flatpak loads catalogs from `/app/share/locale`, without network access. The source adapter currently targets GNU/Linux's gettext runtime.

## Weblate status

The repository and catalogs are ready for Hosted Weblate, but **no Leaf project has been activated there yet**. The owner must sign in or create an account, accept any service terms, authorize the GitHub integration and request the public Libre hosting plan. Do not advertise an unverified Weblate project URL.

After activation, contributors will be able to translate in a browser and Weblate can open translation pull requests. See [the exact setup](WEBLATE_SETUP.md). Until then, GitHub contributions above are available.
