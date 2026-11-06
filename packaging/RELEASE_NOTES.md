Leaf 0.1.0-alpha.31 candidate improves Review continuity and sidebar layout.

- Applying a correction keeps valid remaining findings visible during rechecking. Changed text offsets are remapped and revalidated; stale results and targets are rejected. Unaffected correction, comment and tracked-change cards are reused.
- Sidebar and editor headers share native sizing for their complete regions, including filters, representation choice and document tabs.
- Navigation uses a native Pages/Structure dropdown matching Review. It retains document-specific filters, expansion and independent browsing positions without moving the editing target.

This is a candidate for maintainer testing, not an automatically published release. #136 remains open. Its experimental formatting changes were withdrawn from this candidate after installed font paste regressed while the published Alpha30 control passed on the same runtime. The published Text/Paragraph controls are retained. Caption ownership, the documented engine limitations and the unresolved original-document/platform reports remain open.

The original DOCX from #132 was inspected without altering its text or formatting. Its overlap did not reproduce in the local layout, PNG/PDF or Leaf window with the tested fonts, so the report remains open. Small synthetic wrapping controls and an incremental Review regression were added.

WordCraft, renderer, dependency revisions and existing format patches are unchanged. The candidate records its source and SHA-256 sums. Automated checks do not imply human testing.
