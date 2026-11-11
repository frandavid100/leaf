#!/usr/bin/env python3
"""Package the pinned Leaf release as a relocatable, unsigned macOS Apple Silicon preview.

Run only on a macOS arm64 runner with Homebrew GTK4/libadwaita/dylibbundler.
The bundle is a test build: automated CLI checks do not replace real Mac testing.
"""
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import textwrap

ROOT = Path(__file__).resolve().parents[1]
VERSION = subprocess.check_output(
    [str(ROOT / "target/release/wordcraft-gnome"), "--version"], text=True
).strip().removeprefix("Leaf ")
assert VERSION.startswith("0.1.0-alpha."), VERSION
PREVIEW = VERSION + "-macos.1"
DIST = ROOT / "dist/macos"
APP = DIST / "Leaf.app"
MACOS = APP / "Contents/MacOS"
FRAMEWORKS = APP / "Contents/Frameworks"
RESOURCES = APP / "Contents/Resources"
SHARE = RESOURCES / "share"
LIB = RESOURCES / "lib"

def run(*args, **kwargs):
    print("+", *map(str, args), flush=True)
    return subprocess.run(list(map(str, args)), check=True, **kwargs)

def brew(name):
    return Path(subprocess.check_output(["brew", "--prefix", name], text=True).strip())

def copy_tree(src, dst):
    if src.is_dir():
        shutil.copytree(src, dst, dirs_exist_ok=True, symlinks=False)

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

if sys.platform != "darwin" or os.uname().machine != "arm64":
    sys.exit("macOS arm64 is required")
shutil.rmtree(DIST, ignore_errors=True)
for d in (MACOS, FRAMEWORKS, SHARE, LIB):
    d.mkdir(parents=True, exist_ok=True)
shutil.copy2(ROOT / "target/release/wordcraft-gnome", MACOS / "leaf-bin")

# Compile the actual application icon from the checked-in SVG.
icon = ROOT / "data/icons/hicolor/scalable/apps/io.github.frandavid100.leaf.svg"
iconset = DIST / "Leaf.iconset"
iconset.mkdir()
for points in (16, 32, 128, 256, 512):
    for scale in (1, 2):
        px = points * scale
        name = f"icon_{points}x{points}" + ("@2x" if scale == 2 else "") + ".png"
        run("rsvg-convert", "-w", px, "-h", px, "-o", iconset / name, icon)
run("iconutil", "-c", "icns", "-o", RESOURCES / "Leaf.icns", iconset)
shutil.rmtree(iconset)

with (APP / "Contents/Info.plist").open("wb") as f:
    plistlib.dump({
        "CFBundleName": "Leaf",
        "CFBundleDisplayName": "Leaf",
        "CFBundleIdentifier": "io.github.frandavid100.leaf",
        "CFBundleExecutable": "leaf",
        "CFBundleIconFile": "Leaf",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": VERSION,
        "CFBundleVersion": "1",
        "LSMinimumSystemVersion": "15.0",
        "NSHighResolutionCapable": True,
        "NSHumanReadableCopyright": "Leaf contributors — GPL-3.0-or-later",
    }, f, sort_keys=True)

# Native GTK resources and runtime dependencies.
copy_tree(ROOT / "data/icons", SHARE / "icons")
copy_tree(ROOT / "data/dictionaries", SHARE / "hunspell")
copy_tree(ROOT / "po/locale", SHARE / "locale")
for name in ("gtk4", "libadwaita", "glib"):
    prefix = brew(name)
    copy_tree(prefix / "share/glib-2.0/schemas", SHARE / "glib-2.0/schemas")
    # Keep GNOME interface translations available without bloating with every locale.
    for lang in ("en", "es", "es_ES"):
        copy_tree(prefix / "share/locale" / lang, SHARE / "locale" / lang)
copy_tree(brew("adwaita-icon-theme") / "share/icons/Adwaita", SHARE / "icons/Adwaita")
copy_tree(brew("hicolor-icon-theme") / "share/icons/hicolor", SHARE / "icons/hicolor")
copy_tree(brew("shared-mime-info") / "share/mime", SHARE / "mime")
run("glib-compile-schemas", SHARE / "glib-2.0/schemas")
lic = SHARE / "licenses"
(lic / "Leaf").mkdir(parents=True)
shutil.copy2(ROOT / "LICENSE", lic / "Leaf/LICENSE")
for rel in ("data/icons/ICON_DEV_KIT_LICENSE.md", "data/THEME_SELECTOR_SOURCES.md"):
    source = ROOT / rel
    if source.is_file():
        shutil.copy2(source, lic / "Leaf" / source.name)
for p in (ROOT / "vendor/wordcraft-formats").glob("LICENSE-*"):
    (lic / "WordCraft").mkdir(exist_ok=True)
    shutil.copy2(p, lic / "WordCraft" / p.name)
# WordCraft's embedded language data includes independent licensing details.
with (lic / "Leaf/Language-data.txt").open("w") as f:
    run(MACOS / "leaf-bin", "--language-data-licenses", stdout=f)

# The same Contents/share convention is used by Linux/Windows resource discovery.
(APP / "Contents/share").symlink_to("Resources/share", target_is_directory=True)

# Package optional image loaders, including SVG used by the app theme.
loaders_root = brew("gdk-pixbuf") / "lib/gdk-pixbuf-2.0/2.10.0/loaders"
loaders = sorted(loaders_root.glob("*.so"))
dest_loaders = LIB / "gdk-pixbuf-2.0/2.10.0/loaders"
dest_loaders.mkdir(parents=True)
for p in loaders:
    shutil.copy2(p, dest_loaders / p.name)

# Rewrite Homebrew library dependencies to the app-local Frameworks directory.
# Fix plugin dependencies too: the loader modules are opened dynamically.
run("dylibbundler", "-od", "-b", "-x", MACOS / "leaf-bin",
    "-d", FRAMEWORKS, "-p", "@executable_path/../Frameworks/")
for p in sorted(dest_loaders.glob("*.so")):
    run("dylibbundler", "-od", "-b", "-x", p,
        "-d", FRAMEWORKS, "-p", "@executable_path/../Frameworks/")

# GdkPixbuf cache stores absolute plugin paths, so substitute at app launch.
if loaders:
    cache = subprocess.check_output(
        ["gdk-pixbuf-query-loaders", *map(str, dest_loaders.glob("*.so"))],
        text=True
    )
    cache = cache.replace(str(dest_loaders), "@GDK_PIXBUF_MODULEDIR@")
    (LIB / "loaders.cache.in").write_text(cache)

launcher = textwrap.dedent("""\
    #!/bin/sh
    set -eu
    HERE="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
    RES="$HERE/../Resources"
    export LEAF_LOCALEDIR="$RES/share/locale"
    export GSETTINGS_SCHEMA_DIR="$RES/share/glib-2.0/schemas"
    export XDG_DATA_DIRS="$RES/share:/usr/local/share:/usr/share"
    export GDK_PIXBUF_MODULEDIR="$RES/lib/gdk-pixbuf-2.0/2.10.0/loaders"
    if [ -f "$RES/lib/loaders.cache.in" ]; then
        CACHE="${TMPDIR:-/tmp}/leaf-pixbuf-$$.cache"
        sed "s|@GDK_PIXBUF_MODULEDIR@|$GDK_PIXBUF_MODULEDIR|g" "$RES/lib/loaders.cache.in" > "$CACHE"
        export GDK_PIXBUF_MODULE_FILE="$CACHE"
    fi
    "$HERE/leaf-bin" "$@"
    STATUS=$?
    if [ -n "${CACHE:-}" ]; then rm -f "$CACHE"; fi
    exit "$STATUS"
""")
# Let exec inherit signals and return codes; set -e is intentionally absent here
# because we need to clean up the generated loader cache.
launcher = launcher.replace("set -eu", "set -u")
(MACOS / "leaf").write_text(launcher)
(MACOS / "leaf").chmod(0o755)

# Verify the relocatable executable can initialize gettext and embedded data.
run(MACOS / "leaf", "--version")
run(MACOS / "leaf", "--language-data-licenses", stdout=subprocess.DEVNULL)
for binary in [MACOS / "leaf-bin", *FRAMEWORKS.glob("*.dylib"), *dest_loaders.glob("*.so")]:
    linked = subprocess.check_output(["otool", "-L", str(binary)], text=True)
    if "/opt/homebrew/" in linked or "/usr/local/opt/" in linked:
        sys.exit(f"Unbundled Homebrew dependency: {binary}\n{linked}")

run("codesign", "--force", "--deep", "--sign", "-", APP)
run("codesign", "--verify", "--deep", "--strict", APP)

(DIST / "README.txt").write_text(textwrap.dedent(f"""\
    Leaf {PREVIEW} — experimental macOS Apple Silicon build
    Requires macOS 15 or later. No installation of Homebrew or GTK is required.
    Open Leaf.app from the ZIP or drag it to Applications.
    This preview is ad-hoc signed, not Apple Developer ID notarized.
    macOS may block launch: use System Settings → Privacy & Security → Open Anyway.
    Only CLI-level checks have run. The main window, documents, printing, clipboard
    and fonts still require testing on a real personal Mac.
    Use COPIES of important documents; the editor is alpha software.
    Report problems: https://github.com/frandavid100/leaf/issues
    License: GPL-3.0-or-later. GTK/libadwaita: LGPL, WordCraft: see sources.
"""))
(DIST / "BUILD.json").write_text(json.dumps({
    "version": PREVIEW,
    "source": os.environ.get("GITHUB_SHA", "local"),
    "architecture": "arm64",
    "macos_minimum": "15.0",
    "signing": "ad-hoc (not notarized)",
    "brew": subprocess.check_output(["brew", "list", "--versions"], text=True).splitlines(),
}, indent=2) + "\n")
zipname = DIST / f"Leaf-{PREVIEW}-macos-arm64.zip"
run("ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", APP, zipname)
# DMG is a convenience container of exactly the same app.
run("hdiutil", "create", "-volname", "Leaf " + PREVIEW,
    "-srcfolder", APP, "-ov", "-format", "UDZO",
    DIST / f"Leaf-{PREVIEW}-macos-arm64.dmg")
with (DIST / "SHA256SUMS").open("w") as f:
    for p in sorted(DIST.glob("*")):
        if p.suffix in (".zip", ".dmg"):
            f.write(f"{digest(p)}  {p.name}\n")
(DIST / "VERSION").write_text(PREVIEW + "\n")
print("PACKAGING_OK", PREVIEW, flush=True)
