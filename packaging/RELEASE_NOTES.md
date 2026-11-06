Leaf 0.1.0-alpha.31 candidate improves Review continuity.

Applying a correction keeps valid remaining findings visible during rechecking.
Changed text offsets are remapped and revalidated; stale results and targets are
rejected. Unaffected correction, comment and tracked-change cards are reused.
The current filter, scrolling context and sensible focus are retained. Undo/Redo
and a 40-finding DOCX were checked with real native input.

This is a candidate for maintainer testing. The published Alpha30 header,
Navigation and formatting interface is retained. Experiments for #134–#136 are
not included because they did not pass the installed input regression, and those
issues remain open. Caption ownership, documented engine limits and unresolved
original-document/platform reports also remain open.

The original DOCX from #132 was inspected without changing its text or formatting.
Its overlap did not reproduce locally, so the report remains open. Small synthetic
geometry controls and an incremental Review regression were added.

WordCraft, renderer, dependency revisions and existing patches are unchanged.
The candidate records its source and SHA-256 sums. Automated success does not
imply human validation on every platform or document.
