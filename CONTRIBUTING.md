# Contributing to Leaf

The active editor is on **wordcraft-saving**. Target pull requests at that branch;
main retains an older prototype. Start with [current state](PROJECT_STATUS.md),
[architecture and code map](docs/ARCHITECTURE.md), [current roadmap](ROADMAP.md)
and [project decisions](docs/DECISIONS.md). [AGENTS.md](AGENTS.md) has the same
working constraints for automated contributors. [Continuing guide](docs/CONTINUING.md)
is available in Spanish for moving work between conversations.

## Build and check

Linux, Rust 1.95 or newer, pkg-config, GTK 4.14+ and libadwaita 1.4+ development
libraries are required. Ubuntu CI installs libgtk-4-dev/libadwaita-1-dev, Xvfb,
gettext and the other tools declared by the workflow. GNOME OS users can test
the published Flatpak without installing development packages on the host.

```sh
cargo fmt --package wordcraft-gnome -- --check
python -m unittest discover -s tools -p 'test_*.py'
python tools/update_translations.py --check --compile
python tools/check_project_docs.py
cargo test --locked
cargo build --release --locked --features gui --bin wordcraft-gnome
./target/release/wordcraft-gnome
```

After changing Rust formatting, source locations or visible strings, run
`python tools/update_translations.py --compile`, update translations when needed,
and repeat the check. It preserves the same gettext catalog/message format.
Format the Leaf package only; do not reformat or modify vendor as part of a
frontend cleanup. The installed application is Leaf; the internal Cargo target
keeps its existing wordcraft-gnome name.

## Making a focused change

Use public WordCraft Session/layout/command APIs. Keep the engine, renderer,
locked revisions and existing compatibility patches intact for Leaf work.
Propose a reproducible engine defect upstream rather than adding a frontend
fork. Follow [UPDATING_WORDCRAFT](UPDATING_WORDCRAFT.md) for an authorized upgrade.

The GTK entry point is small; [src/gnome](src/gnome) contains private modules and
[diagnostics](src/gnome/diagnostics) contains real-widget checks. Reuse existing
adapters and weak/cancellable ownership patterns. Add a focused regression for
a real defect where useful; exercise actual GTK activation/input for interaction
changes. Documentation-only changes do not need a new executable or release.

Describe the problem, resulting behavior, tested source and material limits in
the PR. Existing full-corpus and independent Writer audits are optional when
justified, not mandatory per release. Do not convert automated success into a
claim that a person tested GNOME OS, Orca, braille or a physical printer.

## Reporting and translations

[Bug reports](https://github.com/frandavid100/leaf/issues/new/choose) and real
file tests are welcome in English or Spanish. Use the [testing guide](docs/TESTING.md)
and attach a minimal, invented/anonymized reproducer, never confidential files.

[PO contributions](docs/TRANSLATING.md) are available now; hosted Weblate setup
is pending the maintainer. Public screenshots are also reserved for the maintainer.
AI-assisted contributions should disclose affected parts and validation; tests
are not a substitute for understanding a change. There is no promised response
time. Lost content, saving failures and crashes take priority over appearance.
