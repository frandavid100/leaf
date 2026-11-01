Leaf 0.1.0-alpha.13 reduces unnecessary work in the GTK frontend: caret animation invalidates its own page, selection/search changes repaint their old and new pages, thumbnail visibility uses indexed GTK bounds, and closed search panels stop recalculating results until reopened. Header/footer guides retain deliberate full invalidation.

WordCraft, the renderer, formats patches, Cargo configuration and dependencies are unchanged. The optional synthetic performance audit compares actual before/after release binaries on the same runner, using AB/BA order, alongside native correctness and UI regressions. Results and limitations will be recorded in docs/HOST_PERFORMANCE_AUDIT.md; no universal speedup or memory reduction is claimed.

Install Leaf-0.1.0-alpha.13-x86_64.flatpak over alpha.12. English/Spanish, dark documents, HiDPI and first-stage Orca access remain included. Experimental AI-assisted development. Public screenshots and Weblate setup remain with the maintainer.
