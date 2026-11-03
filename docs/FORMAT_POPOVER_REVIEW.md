# Format popover review

The font list inherits the menu text size and uses each family only for its
typeface. Font Size has its own natural width and room for localized decimal
values. Its popup still follows its control's allocation.

Text uses a single language label. Change Case and Clear Text Formatting live
in More Text Options, with Clear Text Formatting at the bottom. A divider
separates language inheritance choices from named languages. The dictionary
notice remains until an installation route can be verified in Flatpak.

Paragraph offers Columns in the toolbar and opens the existing editor for
presets and custom values. The compact layout retains direct indent actions
and access to their detailed settings. A native prototype grouped the indent
actions into a submenu. It kept the same menu width and added a click to
increase or decrease indentation, so the direct actions remain.

Styles uses WordCraft's public quick gallery when the document supplies it.
The current style remains included, search covers all styles, and More Styles
expands the list. A document without a gallery shows its full list. Labels use
the existing menu typography and show the paragraph or character kind. A
native prototype applied family, weight and italic at menu size. The fixture
made the emphasis variants italic but added little distinction among the
paragraph styles. The list keeps uniform labels and the details retain the
full preview.

Validation is tracked by the pull request's native workflow.

## Comments

New threads and replies use the sidebar composer. It holds the original
selection and revision until submission. Navigation changes the selection
without changing the revision. A later document edit makes the draft stale,
and replacing a document drops its draft. The thread body opens its anchor,
with separate Reply and overflow controls. The flat thread fill uses the
thumbnail selection colour and a stronger fill for the current thread.

## Remaining reproductions

- #1: in the unequal selected-text fixture, lines continue with the wide
  column's geometry when they should wrap for the narrow column.
- #2: a three-column continuous section has no separator rules on its first
  page. The next page has both rules.
- #65: a synthetic DOCX round trip retains section start types. ODT loses
  the inline column break, flattens a same-layout continuous section, and
  normalizes odd and even starts to next-page starts. This needs explicit
  format handling alongside navigation entries.
- #69: the bundle packages Hunspell files under /app/share/hunspell. The
  language chooser separator is present. The proposed settings destination
  must install dictionaries that WordCraft can read in that sandbox.
