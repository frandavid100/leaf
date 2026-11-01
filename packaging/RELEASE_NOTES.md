Leaf 0.1.0-alpha.16 addresses the open-issue review requested by the maintainer.

- Keep an unpaired final page in the left slot. Round screen page corners without changing pagination or exports.
- Leave header/footer editing by clicking outside its region; remove the extra mode bar. Insert page numbers at the original cursor, including body text, without position/alignment controls.
- Apply valid paragraph, text and table measurements immediately, with validation and grouped undo. Keep pinned formatting synchronized with the current selection and mixed values.
- Compact editable Font/Size selectors: font previews at the current size and common plus effective document sizes, including fractional sizes and other text stories.
- Navigate from comment anchors to their threads, including overlaps, while preserving drag selection.
- Persistent System/Light/Dark choices in the main menu; compact zoom and per-document Editing/Reviewing modes in the document menu. Reviewing records subsequent edits as tracked changes.
- Save an opened writable document directly with Ctrl+S or Save. Keep Save As separate and recovery enabled by default.
- Configure the author of new comments/revisions in Preferences; default to the system account and preserve previous authorship.
- Create/edit a selection's link from its context menu while preserving text and formatting. Mixed links are explicit.
- Enable manual page dimensions only for Custom paper sizes.

Columns, advanced mouse table interactions, odd/even story independence, the reported document-specific spacing case and coordinated language/proofing inheritance remain open. Engine, dependencies, vendor and patches are unchanged. ODT still cannot preserve comments/category with this exporter; Leaf offers DOCX.

Install Leaf-0.1.0-alpha.16-x86_64.flatpak over alpha.15. Experimental, AI-assisted development; automated checks do not replace testing important documents in another editor.
