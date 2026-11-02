# Hosted Weblate setup for Leaf

Prepared settings for **main**, verified against the Weblate documentation on
2 November 2026. This file is a manual setup checklist, not an automatically
imported Weblate configuration. No hosted Leaf project is confirmed active.

## Component settings

| Setting | Value |
| --- | --- |
| Project / website | Leaf / https://github.com/frandavid100/leaf |
| Component | Application (suggested slug `application`) |
| Source code repository | `https://github.com/frandavid100/leaf.git` |
| Repository branch / pull-request base | `main` |
| Version control | GitHub (via Weblate GitHub app), imported through the connected account |
| Create pull requests | Enabled (`create_merge_request`); review before merging; automatic merge disabled |
| Push branch | `weblate-leaf-application` (translation work branch, never `main`) |
| Repository push URL | Leave empty for the GitHub App backend; it authenticates against the source repository |
| Source language | English (`en`) |
| File format | GNU gettext PO (bilingual) |
| Translation file mask | `po/*.po` |
| Template for new translations | `po/leaf.pot` |
| Monolingual base-language file | Empty; gettext stores English source in each `msgid` |
| Existing catalogs | `po/en.po`, `po/es.po`; English remains synchronized by the extraction tool |
| Translation license | GPL-3.0-or-later |
| New languages | Community requests; review initial contributions |
| Repository browser | `https://github.com/frandavid100/leaf/blob/main/{{filename}}#L{{line}}` |

The repository branch selects the translated source and PR destination. The push
branch above holds Weblate's proposed commits. Enable the msgmerge add-on using
`po/leaf.pot` so updated source strings reach translation catalogs. Preserve named
and positional placeholders and keyboard shortcuts. Local validation:

```sh
python tools/update_translations.py --check --compile
```

## Owner activation checklist

1. Sign in or create your Hosted Weblate account and accept the applicable terms.
   Request public Libre hosting at https://hosted.weblate.org/hosting/; approval
   is Weblate's decision. This preparation has not purchased a subscription.
2. Create/select a workspace and Leaf project. Use **Connect GitHub account** to
   install/authorize the Hosted Weblate app for **frandavid100/leaf only**, then
   import the repository. The app handles repository access, PRs and incoming
   notifications; a separate legacy webhook or invited push user is unnecessary.
3. Apply the component settings above. Confirm the import reads `main`, the POT
   and both PO files. Check component diagnostics and enable incoming hooks.
4. Make one small translation and push it through Weblate. Verify that GitHub
   receives a PR **into main**, that Leaf CI succeeds, and review/merge the PR.
   Check translation display in a build and in the next validated Flatpak.
5. Once the actual public translation page is confirmed, add its real URL to the
   README and About dialog. Until then those links lead to the contribution guide.

Account creation, terms, repository authorization, hosting approval and the first
live PR remain unverified. The previous attempt only reached sign-in; this task
has not retried login or created a hosted project. The hosting page returned HTTP
403 to an unauthenticated fetch, so account/project state cannot be inferred from
that response.

References: [GitHub integration](https://docs.weblate.org/en/latest/admin/code-hosting.html),
[component settings](https://docs.weblate.org/en/latest/admin/projects.html),
[gettext](https://docs.weblate.org/en/latest/devel/gettext.html).
