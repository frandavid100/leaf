# Odd/even header and footer isolation: reduced API reproduction

2026-11-01, Leaf model/adapters at `f6181391db009f4487c5b564d35bb2b008015aef`,
pinned WordCraft `165d5725dc25b4a7887d8025c3b1fe506ccc3dcc`.

This is a headless reproduction of the public command path, not a claim to have
opened the maintainer's original document or tested all imported variants.

1. Create `Document::from_text("Page 1\nPage 2\nPage 3")` and a `Session`.
2. At `Pos::body(1,0)` and `Pos::body(2,0)`, run `para.pageBreakBefore` with
   `{"value":true}`. Assert three layout pages.
3. On page zero enter Header and Footer using `header_footer::enter`; insert
   `ODD HEADER` / `ODD FOOTER` via `text.insert`, then close each region.
4. Run `layout.differentOddEven` with `{"value":true}` and verify the flag.
5. Enter each region on pages zero and one, recording `session.sel.focus.story`.
   Insert `EVEN HEADER` / `EVEN FOOTER` on page one and close.
6. Save DOCX through `saving::save`, reopen and inspect both section slot sets.

Observed output from the executable reproduction:

```text
ISSUE7 Header odd=Part(1) even=Part(1) shared=true text="ODD HEADEREVEN HEADER"
ISSUE7 Footer odd=Part(2) even=Part(2) shared=true text="ODD FOOTEREVEN FOOTER"
ISSUE7 DOCX even_odd=true header_default=Some(1) header_even=None footer_default=Some(2) footer_even=None
```

The flag is applied, but an absent even variant falls back to the default story;
editing page two writes to that same story. Reopening preserves the flag and
shared/default allocation. This confirms the suspected public API path for new
absent even slots, independently for header and footer. It does not prove an
error when explicit distinct even parts already exist.

Issue #7 remains open: independent allocation, direct/menu entry, rendering,
PDF, undo/redo and distinct preexisting parts still require a complete solution.
No engine, renderer or dependency patch has been added. No new upstream issue
was created during this release review.
