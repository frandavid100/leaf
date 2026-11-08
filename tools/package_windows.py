#!/usr/bin/env python3
"""Assemble a relocatable Windows prefix and its transitive MinGW DLLs."""
from pathlib import Path
import json
import os
import re
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def run(*args):
    return subprocess.check_output(args, text=True, encoding='utf-8')


def main():
    prefix = Path(run('cygpath', '-m', os.environ['MINGW_PREFIX']).strip())
    version = re.search(r'const APP_VERSION: &str = "([^"]+)"',
                        (ROOT / 'src/gnome/mod.rs').read_text(encoding='utf-8'))[1]
    version += '-windows.1'
    output = ROOT / 'dist/windows'
    output.mkdir(parents=True, exist_ok=True)
    package = output / f'Leaf-{version}-x86_64'
    if package.exists():
        shutil.rmtree(package)
    (package / 'bin').mkdir(parents=True)
    shutil.copy2(ROOT / 'target/release/wordcraft-gnome.exe', package / 'bin/leaf.exe')
    # Schemas, icons, translations and SVG loaders must survive relocation.
    for relative in ['share/glib-2.0/schemas', 'share/icons/Adwaita',
                     'share/icons/hicolor', 'lib/gdk-pixbuf-2.0', 'lib/gio/modules']:
        source = prefix / relative
        if source.exists():
            shutil.copytree(source, package / relative, dirs_exist_ok=True)
    for relative in ['share/locale', 'share/hunspell', 'share/licenses/Leaf']:
        (package / relative).mkdir(parents=True, exist_ok=True)
    shutil.copytree(ROOT / 'po/locale', package / 'share/locale', dirs_exist_ok=True)
    shutil.copytree(ROOT / 'data/icons', package / 'share/icons', dirs_exist_ok=True)
    shutil.copy2(ROOT / 'data/icons/hicolor/scalable/apps/leaf.svg',
                 package / 'share/icons/hicolor/scalable/apps/io.github.frandavid100.leaf.svg')
    shutil.copytree(ROOT / 'data/dictionaries', package / 'share/hunspell', dirs_exist_ok=True)
    shutil.copy2(ROOT / 'LICENSE', package / 'share/licenses/Leaf/LICENSE')
    # Retain runtime library notices, as well as Leaf/dictionary and embedded pack licenses.
    shutil.copytree(prefix / 'share/licenses', package / 'share/licenses/runtime', dirs_exist_ok=True)
    for cache in (package / 'lib/gdk-pixbuf-2.0').rglob('loaders.cache'):
        text = cache.read_text(encoding='utf-8')
        text = re.sub(re.escape(str(prefix).replace('\\', '/') + '/'), '', text, flags=re.I)
        text = re.sub(re.escape(str(prefix).replace('\\', '\\\\') + '\\\\'), '', text, flags=re.I)
        cache.write_text(text, encoding='utf-8')
    available = {p.name.lower(): p for p in (prefix / 'bin').glob('*.dll')}
    system = Path(os.environ['SYSTEMROOT']) / 'System32'
    queue = [package / 'bin/leaf.exe', *package.rglob('*.dll')]
    visited = set()
    while queue:
        binary = queue.pop()
        if binary in visited:
            continue
        visited.add(binary)
        for name in re.findall(r'DLL Name:\s*(\S+)', run('objdump', '-p', str(binary))):
            key = name.lower()
            if key in available:
                destination = package / 'bin' / available[key].name
                if not destination.exists():
                    shutil.copy2(available[key], destination)
                    queue.append(destination)
            elif not key.startswith(('api-ms-win-', 'ext-ms-win-')) and not (system / name).exists():
                raise RuntimeError(f'Unresolved DLL {name}, required by {binary.name}')
    revision = run('git', 'rev-parse', 'HEAD').strip()
    manifest = {'version': version, 'source': revision,
                'packages': run('pacman', '-Q').splitlines(),
                'dlls': sorted(p.name for p in (package / 'bin').glob('*.dll'))}
    (package / 'BUILD.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    (package / 'README.txt').write_text(
        f'Leaf {version} — experimental Windows x86_64 build\n\n'
        'Launch bin/leaf.exe. No GTK, Rust or MSYS2 installation is required.\n'
        'Settings: %APPDATA%/Leaf; recovery: %LOCALAPPDATA%/Leaf/recovery.\n'
        'The portable ZIP stores user settings in these same folders.\n'
        'Use Preferences to enter your full author name and choose templates.\n'
        'This build is unsigned and Windows may show a reputation warning.\n'
        'Printing and assistive technology have not been verified on Windows.\n'
        'Test with copies of documents. Export PDF if direct printing is unavailable.\n\n'
        f'Exact Leaf source: https://github.com/frandavid100/leaf/tree/{revision}\n'
        'Leaf: GPL-3.0-or-later. See share/licenses and share/hunspell.\n'
        'Bundled runtime package versions: BUILD.json. Source packages and recipes:\n'
        'https://repo.msys2.org/mingw/sources/\n'
        'https://github.com/msys2/MINGW-packages\n', encoding='utf-8')
    (output / 'version.txt').write_text(version, encoding='ascii')
    archive = output / f'{package.name}-portable.zip'
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        for path in sorted(package.rglob('*')):
            if path.is_file():
                z.write(path, path.relative_to(output))
    print(f'WINDOWS_PACKAGE_OK version={version} dlls={len(manifest["dlls"])} archive={archive}')


if __name__ == '__main__':
    main()
