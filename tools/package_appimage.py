#!/usr/bin/env python3
"""Package the tested Leaf release executable as an experimental x86_64 AppImage.

Run on the same Ubuntu 24.04 build host as the native executable, with GTK4,
libadwaita, librsvg, adwaita-icon-theme, gettext, patchelf and squashfs-tools.
GTK, libadwaita and their libraries are bundled; glibc, graphics drivers,
fontconfig and the other base libraries in the AppImage exclude list come from
the host. The bundled libraries therefore require glibc 2.39 or newer.

Usage: package_appimage.py [TESTED_BINARY]   (default: target/release/wordcraft-gnome)
"""
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import textwrap
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
APP_ID = "io.github.frandavid100.leaf"
DIST = ROOT / "dist/appimage"
APPDIR = DIST / "Leaf.AppDir"
USR = APPDIR / "usr"
LIB = USR / "lib"
SHARE = USR / "share"
SYSLIB = Path("/usr/lib/x86_64-linux-gnu")

# Static AppImage type 2 runtime, pinned by release tag and digest.
RUNTIME_URL = "https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64"
RUNTIME_SHA256 = "2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d"

# Libraries every target system provides itself (AppImage project exclude list,
# https://github.com/AppImageCommunity/pkg2appimage/blob/master/excludelist).
# Bundling them breaks graphics drivers, fonts or the C runtime on the host.
EXCLUDED = {
    "ld-linux.so.2", "ld-linux-x86-64.so.2", "libanl.so.1", "libBrokenLocale.so.1",
    "libcidn.so.1", "libc.so.6", "libdl.so.2", "libm.so.6", "libmvec.so.1",
    "libnss_compat.so.2", "libnss_dns.so.2", "libnss_files.so.2",
    "libnss_hesiod.so.2", "libnss_nisplus.so.2", "libnss_nis.so.2",
    "libpthread.so.0", "libresolv.so.2", "librt.so.1", "libthread_db.so.1",
    "libutil.so.1", "libstdc++.so.6", "libGL.so.1", "libEGL.so.1",
    "libGLdispatch.so.0", "libGLX.so.0", "libOpenGL.so.0", "libdrm.so.2",
    "libglapi.so.0", "libgbm.so.1", "libxcb.so.1", "libX11.so.6",
    "libX11-xcb.so.1", "libwayland-client.so.0", "libasound.so.2",
    "libfontconfig.so.1", "libfreetype.so.6", "libharfbuzz.so.0",
    "libcom_err.so.2", "libexpat.so.1", "libgcc_s.so.1", "libgpg-error.so.0",
    "libICE.so.6", "libSM.so.6", "libusb-1.0.so.0", "libuuid.so.1", "libz.so.1",
    "libjack.so.0", "libpipewire-0.3.so.0", "libxcb-dri3.so.0",
    "libxcb-dri2.so.0", "libfribidi.so.0", "libgmp.so.10",
}


def run(*args, **kwargs):
    print("+", *map(str, args), flush=True)
    return subprocess.run(list(map(str, args)), check=True, **kwargs)


def digest(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def install(src, dst, mode=0o644):
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(src, dst)
    dst.chmod(mode)


def copy_tree(src, dst):
    shutil.copytree(src, dst, dirs_exist_ok=True, symlinks=False)


def dependencies(elf):
    """Direct (DT_NEEDED) dependencies of an ELF file, resolved as name -> path.

    Only direct ones: libraries needed solely by an excluded host library
    (for example graphite2 for harfbuzz) must keep coming from the host.
    """
    needed = subprocess.run(["patchelf", "--print-needed", str(elf)], text=True,
                            capture_output=True, check=True).stdout.split()
    out = subprocess.run(["ldd", str(elf)], text=True, capture_output=True, check=True).stdout
    resolved = {}
    for line in out.splitlines():
        line = line.strip()
        if "=>" not in line:
            continue
        name, target = (part.strip() for part in line.split("=>", 1))
        resolved[name] = target
    found = {}
    for name in needed:
        if name in EXCLUDED:
            continue
        target = resolved.get(name, "not found")
        if target.startswith("not found"):
            sys.exit(f"Unresolved dependency {name} of {elf}")
        found[name] = Path(target.split(" (", 1)[0])
    return found


def is_elf(path):
    with open(path, "rb") as f:
        return f.read(4) == b"\x7fELF"


if sys.platform != "linux" or platform.machine() != "x86_64":
    sys.exit("Linux x86_64 is required")
binary = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "target/release/wordcraft-gnome"
version = subprocess.check_output([str(binary), "--version"], text=True).strip().removeprefix("Leaf ")
assert version.startswith("0.1.0-alpha."), version
run(sys.executable, ROOT / "tools/release_metadata.py", "--check")
run(sys.executable, ROOT / "tools/update_translations.py", "--check", "--compile")

shutil.rmtree(DIST, ignore_errors=True)
(USR / "bin").mkdir(parents=True)
LIB.mkdir()
install(binary, USR / "bin/leaf", 0o755)

# Leaf's own resources, found relative to usr/bin/leaf exactly as in the Flatpak.
copy_tree(ROOT / "po/locale", SHARE / "locale")
install(ROOT / "data/io.github.frandavid100.leaf.desktop", SHARE / f"applications/{APP_ID}.desktop")
install(ROOT / "data/io.github.frandavid100.leaf.metainfo.xml", SHARE / f"metainfo/{APP_ID}.appdata.xml")
icons = ROOT / "data/icons/hicolor"
install(icons / "scalable/apps/io.github.frandavid100.leaf.svg", SHARE / f"icons/hicolor/scalable/apps/{APP_ID}.svg")
install(icons / "symbolic/apps/io.github.frandavid100.leaf-symbolic.svg",
        SHARE / f"icons/hicolor/symbolic/apps/{APP_ID}-symbolic.svg")
for icon in sorted((icons / "scalable/actions").glob("*-symbolic.svg")):
    install(icon, SHARE / "icons/hicolor/scalable/actions" / icon.name)
for dic in sorted((ROOT / "data/dictionaries").glob("*")):
    if dic.suffix in (".aff", ".dic"):
        install(dic, SHARE / "hunspell" / dic.name)

licenses = SHARE / f"licenses/{APP_ID}"
install(ROOT / "LICENSE", licenses / "LICENSE")
for rel in ("data/icons/NAUTILUS_SOURCES.md", "data/icons/ICON_DEV_KIT_LICENSE.md",
            "data/THEME_SELECTOR_SOURCES.md"):
    install(ROOT / rel, licenses / Path(rel).name)
for name in ("LICENSE.md", "GPLv3.txt", "README_hunspell_es.txt", "SOURCE.md", "SHA256SUMS"):
    install(ROOT / "data/dictionaries" / name, licenses / "dictionaries" / name)
for name in ("LICENSE-MIT", "LICENSE-APACHE"):
    install(ROOT / "vendor/wordcraft-formats" / name, SHARE / "licenses/wordcraft-formats" / name)
install(ROOT / "docs/AI_ASSISTANCE.md", SHARE / "doc/leaf/AI_ASSISTANCE.md")
with (licenses / "LANGUAGE_DATA.txt").open("w") as f:
    run(USR / "bin/leaf", "--language-data-licenses", stdout=f)
assert (licenses / "LANGUAGE_DATA.txt").stat().st_size > 0

# GTK runtime data: settings schemas (the file chooser aborts without them),
# the Adwaita icon theme for hosts without GNOME, and MIME information.
schemas = SHARE / "glib-2.0/schemas"
schemas.mkdir(parents=True)
for schema in sorted(Path("/usr/share/glib-2.0/schemas").glob("org.gtk.gtk4.*.gschema.xml")):
    install(schema, schemas / schema.name)
run("glib-compile-schemas", schemas)
copy_tree(Path("/usr/share/icons/Adwaita"), SHARE / "icons/Adwaita")
install(Path("/usr/share/icons/hicolor/index.theme"), SHARE / "icons/hicolor/index.theme")
for theme in ("Adwaita", "hicolor"):
    run("gtk4-update-icon-cache", "--quiet", "--force", "--ignore-theme-index", SHARE / "icons" / theme)

# Dynamically loaded modules: SVG/raster image loaders and GTK print backends.
pixbuf_src = SYSLIB / "gdk-pixbuf-2.0/2.10.0/loaders"
pixbuf_dst = LIB / "gdk-pixbuf-2.0/2.10.0/loaders"
copy_tree(pixbuf_src, pixbuf_dst)
loaders_cache = subprocess.check_output(
    [str(SYSLIB / "gdk-pixbuf-2.0/gdk-pixbuf-query-loaders"), *map(str, sorted(pixbuf_dst.glob("*.so")))], text=True)
(LIB / "gdk-pixbuf-2.0/2.10.0/loaders.cache.in").write_text(
    loaders_cache.replace(str(pixbuf_dst), "@LOADERS@"))
copy_tree(SYSLIB / "gtk-4.0/4.0.0/printbackends", LIB / "gtk-4.0/4.0.0/printbackends")

# Bundle the closure of shared libraries outside the exclude list.
modules = sorted(p for p in LIB.rglob("*.so") if p.parent != LIB)
pending = [USR / "bin/leaf", *modules]
bundled = {}
while pending:
    elf = pending.pop()
    for name, source in dependencies(elf).items():
        if name in EXCLUDED or name in bundled:
            continue
        target = LIB / name
        shutil.copy2(source.resolve(), target)
        target.chmod(0o755)
        bundled[name] = str(source)
        pending.append(target)
print(f"Bundled {len(bundled)} libraries", flush=True)

# Relative run paths instead of LD_LIBRARY_PATH, so programs Leaf starts on the
# host (font viewer, browser, file manager) keep using their own libraries.
run("patchelf", "--set-rpath", "$ORIGIN/../lib", USR / "bin/leaf")
for lib in sorted(LIB.glob("*.so*")):
    run("patchelf", "--set-rpath", "$ORIGIN", lib)
for module in modules:
    rel = os.path.relpath(LIB, module.parent)
    run("patchelf", "--set-rpath", f"$ORIGIN/{rel}", module)

apprun = APPDIR / "AppRun"
apprun.write_text(textwrap.dedent("""\
    #!/bin/sh
    # Leaf AppImage entry point: point GTK at the bundled data and modules.
    HERE="$(dirname "$(readlink -f "$0")")"
    export XDG_DATA_DIRS="$HERE/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
    export GTK_PATH="$HERE/usr/lib/gtk-4.0${GTK_PATH:+:$GTK_PATH}"
    LOADERS="$HERE/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders"
    CACHE_DIR="${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}}"
    CACHE="$(mktemp "$CACHE_DIR/leaf-pixbuf-loaders.XXXXXX")" || exit 1
    sed "s|@LOADERS@|$LOADERS|g" "$HERE/usr/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache.in" > "$CACHE"
    export GDK_PIXBUF_MODULEDIR="$LOADERS" GDK_PIXBUF_MODULE_FILE="$CACHE"
    "$HERE/usr/bin/leaf" "$@"
    STATUS=$?
    rm -f "$CACHE"
    exit "$STATUS"
"""))
apprun.chmod(0o755)
shutil.copy2(SHARE / f"applications/{APP_ID}.desktop", APPDIR / f"{APP_ID}.desktop")
shutil.copy2(SHARE / f"icons/hicolor/scalable/apps/{APP_ID}.svg", APPDIR / f"{APP_ID}.svg")
(APPDIR / ".DirIcon").symlink_to(f"{APP_ID}.svg")

# Checks on the unpacked tree: startup, embedded data and a self-contained closure.
run(apprun, "--version")
run(apprun, "--language-data-licenses", stdout=subprocess.DEVNULL)
for elf in [USR / "bin/leaf", *LIB.rglob("*.so*")]:
    if not is_elf(elf):
        continue
    for name, path in dependencies(elf).items():
        if name not in EXCLUDED and not path.resolve().is_relative_to(APPDIR.resolve()):
            sys.exit(f"{elf} loads {name} from the host: {path}")
if shutil.which("desktop-file-validate"):
    run("desktop-file-validate", APPDIR / f"{APP_ID}.desktop")

# Assemble: static runtime followed by the zstd SquashFS image of the AppDir.
runtime = Path(os.environ.get("APPIMAGE_RUNTIME", DIST / "runtime-x86_64"))
if not runtime.exists():
    print("+ download", RUNTIME_URL, flush=True)
    with urllib.request.urlopen(RUNTIME_URL, timeout=120) as response:
        runtime.write_bytes(response.read())
if digest(runtime) != RUNTIME_SHA256:
    sys.exit(f"Unexpected AppImage runtime digest: {runtime}")
squashfs = DIST / "Leaf.squashfs"
run("mksquashfs", APPDIR, squashfs, "-root-owned", "-noappend", "-comp", "zstd",
    "-Xcompression-level", "19", "-quiet")
appimage = DIST / f"Leaf-{version}-x86_64.AppImage"
with appimage.open("wb") as out:
    for part in (runtime, squashfs):
        with part.open("rb") as f:
            shutil.copyfileobj(f, out)
appimage.chmod(0o755)
squashfs.unlink()
if runtime.parent == DIST:
    runtime.unlink()

glibc = platform.libc_ver()[1]
(DIST / "README.txt").write_text(textwrap.dedent(f"""\
    Leaf {version} — experimental AppImage for Linux x86_64

    Make the file executable and run it:
        chmod +x Leaf-{version}-x86_64.AppImage
        ./Leaf-{version}-x86_64.AppImage
    In most file managers you can instead enable "Allow executing file as
    program" in the file's properties and open it with a double click.

    Requirements: an x86_64 distribution with glibc {glibc} or newer (for example
    Ubuntu 24.04, Fedora 40, Debian 13 or newer) and FUSE. If it reports a FUSE
    problem, install your distribution's FUSE package or run it with
    --appimage-extract-and-run.

    GTK, libadwaita and their libraries are included. Graphics drivers, fonts
    and the C library come from your system. The AppImage does not integrate
    with the launcher or file associations by itself and has no automatic
    updates: download a newer AppImage to update. Unlike the Flatpak it runs
    without a sandbox, with your user's normal file access.

    This build has only passed automated checks; use copies of important
    documents. Report problems: https://github.com/frandavid100/leaf/issues
    License: GPL-3.0-or-later. Bundled components keep their own licenses.
"""))
(DIST / "BUILD.json").write_text(json.dumps({
    "version": version,
    "source": os.environ.get("GITHUB_SHA", "local"),
    "architecture": "x86_64",
    "glibc_minimum": glibc,
    "runtime": {"url": RUNTIME_URL, "sha256": RUNTIME_SHA256},
    "bundled_libraries": dict(sorted(bundled.items())),
}, indent=2) + "\n")
(DIST / "VERSION").write_text(version + "\n")
with (DIST / "SHA256SUMS").open("w") as f:
    f.write(f"{digest(appimage)}  {appimage.name}\n")
print("PACKAGING_OK", appimage.name, flush=True)
