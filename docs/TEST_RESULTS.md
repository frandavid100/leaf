# Real-document test results

Public GitHub alphas have been shared for testing, including on Reddit. The table records human reports separately from automated validation, which is described in PROJECT_STATUS.md and TESTING.md. A reported defect is not an independently reproduced or fixed defect.

| Build / commit | Environment | Format and originating editor | Checks performed | Outcome / issue | Tested by / date |
| --- | --- | --- | --- | --- | --- |
| Installed build not confirmed; “Alpha 2” refers to the columns checklist item | Desktop screenshots supplied; exact session/scaling not recorded | DOCX, originating editor not recorded | Apply columns to selected body text, unequal widths and separator, including three columns | Equal-width division works. Unequal widths fail in the reported case; separator is reported to appear only for two columns. One screenshot shows text extending beyond the right page margin. David reports similar behavior in WordCraft; not yet independently reproduced or reduced to a public fixture. | David / 2026-11-01 |

## Reported column layout problems

The pinned engine contains unequal-width layout and separator rules for arbitrary
column counts; this report does not establish that those features are universally
unsupported. Inspection shows separator rules are created at page creation,
whereas entering a continuous section updates column boxes without adding those
rules. This is a candidate explanation for separators missing in selected-text
sections, not a completed reproduction or the established cause of the width defect.

Leaf's existing column tests check command state, selected-text preservation,
undo/redo and file-property round-trips. They do not assert the visible widths or
separator geometry in all continuous-section cases. Passing those tests must not
be presented as confirmation that this reported visual case works. Keep engine
changes upstream; no engine or renderer modification accompanies this report.

Use invented descriptions rather than private document names. Link reports to GitHub issues. Record DOCX and ODT separately and distinguish initial rendering from saving/reopening. Confirm fixes against a named build before closing an issue.
