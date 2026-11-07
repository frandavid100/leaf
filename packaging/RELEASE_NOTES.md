Leaf 0.1.0-alpha.33

- Keep Home/End and their Shift/Ctrl variants in the document editor before GTK scrolling bindings can consume them. Native text fields retain their own navigation.
- Prevent a pending zoom restoration from jumping back to an older page after a newer caret navigation request, including End at an existing line end.
- Explain how adjoining Before/After paragraph spacing values combine. Real native controls verify fractional and comma input, incrementing and values below 5.5 pt without changing document spacing semantics.

WordCraft remains pinned at 0.16.0. Issue #99 now has a public synthetic reproduction of table-cell notes below the footer; its engine correction remains pending. The exact edited Portland state in #9 is not claimed reproduced. No private document or screenshot is distributed. Source and binary are validated through the existing native and installed GNOME 49/51 checks before automatic publication.
