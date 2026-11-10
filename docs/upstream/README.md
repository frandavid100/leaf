# WordCraft contribution refresh — 10 November 2026

These are proposed changes for WordCraft, not Leaf engine patches. Leaf retains
its stable 0.17.0 dependency and existing formats compatibility snapshot unchanged.
No patch in this folder is applied by a Leaf build or release.

Base for all three proposed trees: upstream main
`2922fe566b575794f896077467cf92825ccba7c9`. Apply a selected patch to that base
with `git apply`, then inspect its diff and run the project's complete gates before
requesting final review. The original upstream MIT/Apache licensing applies;
these proposals do not relicense WordCraft as Leaf application code.

| Leaf report | Upstream state | Prepared work |
| --- | --- | --- |
| #1 | [PR #5891](https://github.com/storytold/wordcraft/pull/5891), unmerged; previous head `26401a789d0571eff8f18af4d12057bfaaa019ab` | Existing @PanSHIgg #2422 continuation merged with current main locally. Progress-document conflict resolved retaining both histories and authorship. The first regression now measures visible content rather than a hanging trailing-space caret endpoint and checks contiguous coverage of all text. |
| #99 | [PR #5935](https://github.com/storytold/wordcraft/pull/5935), unmerged; previous head `ce3a36372dd1fcaae1f8e19971a0554addec2626` | Current-main merge prepared locally. Existing nested Can't Split integration from the last reviewed revision is preserved. Eight engine regressions pass, including long/multiline notes, editing/Undo/Redo, DOCX and nested rows. |
| #152 | Empty-paragraph [PR #5929](https://github.com/storytold/wordcraft/pull/5929) is merged, but whole non-empty paragraph deletion still fails | Narrow follow-up preserves the following properties/mark when both ordered range endpoints are paragraph starts, while retaining partial-merge behavior. Unit tests cover both directions and multiple removed paragraphs; engine tests cover Delete/Backspace, lists, marks/runs, outline, history and DOCX. |

## Executed checks and limits

- #152: the new whole-paragraph unit regression fails without the model change.
  With it, 284 model unit tests, seven model integration tests and all three
  engine `empty_paragraph_merge` tests pass. The previous partial-merge control
  now retains at least one leading character; a whole-paragraph range has its
  own new regression instead of incorrectly expecting the old failure.
- #99: all eight `table_footnote_clearance` engine tests pass after the merge.
- #1: all seven unequal-column tests pass. Temporarily disabling width relay
  makes the focused geometry regression fail; the production source is restored.
  The initial full layout run exposed three missing-Language-data failures.
  After preparing exact upstream `craft-i18n.rev`
  `1e515422afdda2f317af430c1f9d68a5f83935b4`, the full layout suite passes
  **717 tests, zero failures and two previously ignored tests**.
- Leaf's native and installed gates are separate. These local checks do not
  establish that any of the three fixes is included in the released Leaf app.
- Full upstream `cargo xtask ci`, native Word comparison and installed Leaf
  integration are not claimed. Existing public reproductions remain available.

## Connection limitation

The GitHub connection returns HTTP 403 `Resource not accessible by integration`
when posting to WordCraft and when creating trees in `frandavid100/wordcraft`.
The proposed branch updates and #152 follow-up therefore could not be submitted
through this connection. No new upstream issue or PR is claimed. These complete
patches preserve the work for a connection with the needed repository access.
Keep Leaf #1/#99/#152 open until upstream integration and Leaf verification.
