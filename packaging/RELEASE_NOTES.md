Leaf 0.1.0-alpha.31 candidate improves Review continuity and sidebar layout.

- Applying a correction keeps valid remaining findings visible during rechecking. Changed text offsets are remapped and revalidated; stale results and targets are rejected. Unaffected correction, comment and tracked-change cards are reused.
- Sidebar and editor headers share native sizing for their complete regions, including filters, representation choice and document tabs.
- Navigation uses a native Pages/Structure dropdown matching Review. It retains document-specific filters, expansion and independent browsing positions without moving the editing target.
- Text and Paragraph property rows reuse the actual native AdwSplitButton used by history. Back routes and page titles are corrected, Size has its label/icon within the field, and Language has direct search.

This is a candidate for maintainer testing, not an automatically published release. #136 remains partial: the direct Font list, embedded color choosers and the full Style/Elements audit are still pending. Caption ownership, the documented engine limitations and the unresolved original-document/platform reports remain open.

The original DOCX from #132 was inspected without altering its text or formatting. Its overlap did not reproduce in the local layout, PNG/PDF or Leaf window with the tested fonts, so the report remains open. Small synthetic wrapping controls and an incremental Review regression were added.

WordCraft, renderer, dependency revisions and existing format patches are unchanged. The candidate records its source and SHA-256 sums. Automated checks do not imply human testing.
