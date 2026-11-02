# Install and update Leaf

Leaf is distributed as an experimental x86_64 Flatpak bundle on [GitHub Releases](https://github.com/frandavid100/leaf/releases). It is not currently published on Flathub. Use copies of important documents while testing the alpha.

## Install with GNOME Software

Download the file ending in `-x86_64.flatpak` from the release you want to try. Open it with GNOME Software and choose Install. Check the release's included `INSTALL.md` for instructions specific to that version.

The bundle needs the GNOME 49 runtime from Flathub. If it is missing, the runtime is downloaded separately; the total first-install download can be considerably larger than the Leaf bundle.

## Install from a terminal

Open a terminal in the folder containing the downloaded bundle. Add the runtime source if needed:

```sh
flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
```

Install the bundle by passing its actual filename to `flatpak install --user`. You can drag the downloaded file into the terminal to insert its path. Then launch Leaf from your application menu or run:

```sh
flatpak run io.github.frandavid100.leaf
```

The app uses the same application ID across alpha releases. To update, install a newer bundle over the current version. Do not uninstall or remove app data to update. These bundles do not include an automatic Leaf update feed.

## Files and preferences

Open documents through the native file selector or your file manager's Open With action. Leaf does not change your default editor automatically. Preferences and recovery data are stored under `~/.var/app/io.github.frandavid100.leaf`.

English and Spanish interfaces follow the system language. The document view follows the light or dark appearance setting. [Testing guide](TESTING.md) · [Report a problem](https://github.com/frandavid100/leaf/issues/new/choose).
