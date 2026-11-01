# Hosted Weblate setup for Leaf

Prepared configuration, not a claim that a hosted project already exists.

- Project name: Leaf; project website: https://github.com/frandavid100/leaf
- Component: Application
- Public source repository: https://github.com/frandavid100/leaf.git
- Repository branch: `wordcraft-saving`
- Source language: English (`en`)
- File format: GNU gettext PO
- Translation file mask: `po/*.po`
- Source-language file: `po/en.po`
- Template for new translations: `po/leaf.pot`
- Translation license: GPL-3.0-or-later, matching Leaf
- New-language policy: allow community requests; review initial contributions
- Commit workflow: GitHub pull requests to `wordcraft-saving`, reviewed before merging
- Setup path: https://hosted.weblate.org/hosting/

Use the Hosted Weblate GitHub App for the single Leaf repository when connecting the account; review the permissions it requests. Request the public Libre plan, subject to Weblate approval. No paid subscription is required by this preparation and none has been purchased. For the gettext template workflow, enable Weblate's msgmerge add-on so updated source messages reach language catalogs. Keep `po/en.po` synchronized with the POT using the extraction tool and gettext utilities.

When the project is active, verify its public translation page, add that real URL to the README and Acerca de/About link, and test one submitted translation through a PR and a built Flatpak. The current About link goes to the contribution guide so it remains useful before activation.

Account signup, service terms and GitHub authorization require the owner's participation. An automated browser reached the sign-in screen; no account, project, credentials, service agreement or repository authorization was created.
