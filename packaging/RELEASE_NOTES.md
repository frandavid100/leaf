Leaf 0.1.0-alpha.27 adds the release archive requested by FlatPark.

- Publishes `Leaf-0.1.0-alpha.27-flatpark-x86_64.tar.xz` containing the application files, with `bin/` and `share/` at the archive root.
- Includes the archive in `SHA256SUMS` and attaches it to the GitHub release.
- Checks the archive installed at `/app/extra/leaf` on GNOME Platform 51, including English/Spanish translations and spelling review. Results are recorded in `flatpark-smoke.log`.

FlatPark has agreed to list Leaf once a release provides this archive. Catalog publication is handled by FlatPark. The archive is for packaging, and the regular installation instructions use `Leaf-0.1.0-alpha.27-x86_64.flatpak` with GNOME 49. Install it over alpha.26 to update.

Leaf is experimental and its development has used substantial AI assistance.
