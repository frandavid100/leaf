# Remaining issues after alpha.34

Reviewed on 7 November 2026 after David requested autonomous issue resolution
and automatic publication without another approval. The current release remains
[alpha.34](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.34).
No additional application fix was identified in this review; no issue was closed
and no duplicate binary release was produced.

| Leaf issue | Current evidence | What prevents closure |
| --- | --- | --- |
| [#1 — unequal columns](https://github.com/frandavid100/leaf/issues/1) | [WordCraft #5891](https://github.com/storytold/wordcraft/pull/5891) ports the existing fix to current main; its full upstream gate passed. PR remains open and unmerged. | Upstream integration, a compatible engine update and visible selected-text/continuation checks in Leaf. |
| [#99 — table footnotes/footer](https://github.com/frandavid100/leaf/issues/99) | [WordCraft #5935](https://github.com/storytold/wordcraft/pull/5935) has six regressions, visual synthetic checks and a successful full gate. PR remains open and unmerged. | Upstream integration, engine update and end-to-end Leaf validation. |
| [#107 — picture drag between pages](https://github.com/frandavid100/leaf/issues/107) | [WordCraft #5937](https://github.com/storytold/wordcraft/pull/5937) covers the remaining 120×60 pt wrapped-picture case, Undo/Redo and DOCX; its full gate passed. PR remains open and unmerged. | Upstream integration, engine update and actual GTK drag checks. |
| [#152 — deleting an empty heading paragraph](https://github.com/frandavid100/leaf/issues/152) | [WordCraft #5929](https://github.com/storytold/wordcraft/pull/5929) preserves following formatting, with model/command/DOCX/history coverage and a successful full gate. PR remains open and unmerged. | Upstream integration, engine update and Leaf checks for formatting, navigation and TOC behavior. |
| [#84 — dictionaries](https://github.com/frandavid100/leaf/issues/84) | Alpha.34 implements the folder/portal workflow; native and installed GNOME 49/51 checks passed. [Evidence](ALPHA34_VALIDATION.md). | David's GNOME OS check and the complete native Ubuntu Language Support graphical flow remain unverified. Synthetic data does not establish host dictionary provisioning. |
| [#52 — white previews in Files](https://github.com/frandavid100/leaf/issues/52) | No specific Leaf/system thumbnail defect is attributed. The actual affected originals, active provider and cached preview have not been inspected. | Obtain an affected DOCX and identify its system thumbnailer/cache before choosing the responsible layer. Exact-title searches did not locate the reported originals among the available files. Do not clear the user's thumbnail cache or claim that Leaf caused it. |

WordCraft's latest stable release is still 0.16.0, the version already pinned
by Leaf. Its checked main `da914ce674f18ebc4e08cca425cc8f8acca3372f` does not contain the four proposed fixes. The remote
PR states and latest release were checked directly; previous successful fork
checks are evidence for the proposals, not for inclusion in Leaf.

The four upstream implementations already exist and should not be duplicated.
No unmerged fork revision, engine patch, host layout workaround or forced issue
closure is introduced. The existing [engine boundary](../UPDATING_WORDCRAFT.md)
still applies. WordCraft maintainers control approval/integration of their PRs.

Automatic publication is authorized for the next verified code fixes. This
review adds documentation only; it does not require recompiling or publishing
a package under the project's contribution rules. The latest actual downloads
and their verified source remain [alpha.34](ALPHA34_VALIDATION.md).
