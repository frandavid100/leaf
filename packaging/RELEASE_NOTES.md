Leaf 0.1.0-alpha.19 improves font-list readability, hyperlink editing, paragraph selection and navigation details.

- Font choices have a useful bounded width, one preview/name per family and full-name tooltips, including narrow and HiDPI windows and large current text sizes.
- Ctrl+K creates a link at the caret or edits an existing whole link. Right-clicking any part of a link edits its complete range, including visual line wraps and mixed formatting. Cancel leaves content intact; insertion is one undo step. Hover displays the actual destination as plain, wrapping text alongside Ctrl+click.
- Triple-click dragging extends by complete paragraphs in both directions, including autoscrolling across page boundaries. Existing character and word selection remain.
- Heading jumps leave preceding line context. Proofing menus are centered within their complete rows. Bookmark names use regular weight.

Font/Size input now preserves the editable delegate's text/cursor and focus during live formatting and page-set rebuilds. Leaf's validated parser receives both decimal separators instead of relying on GTK's numeric filter. Installed GNOME 49 testing reproduced slow `18,75` becoming `75` before this correction; native and installed regressions gate publication, including a long-document reflow case. This does not establish human Wayland/IME coverage.

WordCraft, dependency pins and compatibility patches are unchanged. Existing format limitations remain, including unequal columns, column separators, absent independent even headers/footers and ODT preservation limits. New ruler, history and document-authoring features are outside this delivery.

Install Leaf-0.1.0-alpha.19-x86_64.flatpak over alpha.18. Experimental, AI-assisted development. See docs/ISSUE_SWEEP_ALPHA19.md for scope, tested sources and validation limits. Native and installed checks gate publication; automated success does not establish human GNOME OS/Wayland/device testing.
