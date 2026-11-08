#!/usr/bin/env python3
"""Generate AppStream release descriptions and GitHub notes from reviewed news."""
import argparse
import datetime
import json
import pathlib
import re
import sys
import urllib.request
import xml.etree.ElementTree as ET

ROOT = pathlib.Path(__file__).resolve().parents[1]
LANG = '{http://www.w3.org/XML/1998/namespace}lang'
META = 'data/io.github.frandavid100.leaf.metainfo.xml.in'


def load_news(path=None):
    entries = json.loads((path or ROOT / 'data/releases/news.json').read_text())['releases']
    seen = set()
    for entry in entries:
        version = entry['version']
        if not re.fullmatch(r'0\.1\.0-alpha\.[1-9][0-9]*', version) or version in seen:
            raise ValueError(f'Invalid or duplicate release version: {version}')
        seen.add(version)
        datetime.date.fromisoformat(entry['date'])
        if entry['type'] != 'development':
            raise ValueError(f'{version}: expected development status')
        if entry['url'] != f'https://github.com/frandavid100/leaf/releases/tag/v{version}':
            raise ValueError(f'{version}: release URL must match version')
        for language in ('en', 'es'):
            prose = entry[language]
            if not prose['title'].strip() or not 1 <= len(prose['highlights']) <= 5:
                raise ValueError(f'{version}/{language}: missing title or highlights')
            if any(not text.strip() for text in prose['highlights'] + prose['limitations']):
                raise ValueError(f'{version}/{language}: empty release prose')
    if not entries:
        raise ValueError('No reviewed release news')
    if entries != sorted(entries, key=lambda e: int(e['version'].rsplit('.', 1)[1]), reverse=True):
        raise ValueError('Release news must be newest first')
    return entries


def release_element(entry):
    release = ET.Element('release', {key: entry[key] for key in ('version', 'date', 'type')})
    ET.SubElement(release, 'url', {'type': 'details'}).text = entry['url']
    description = ET.SubElement(release, 'description')
    # AppStream requires translations on p/li, not description or ul.
    for language in ('en', 'es'):
        attrs = {} if language == 'en' else {LANG: language}
        ET.SubElement(description, 'p', attrs).text = entry[language]['title']
    bullets = ET.SubElement(description, 'ul')
    for language in ('en', 'es'):
        attrs = {} if language == 'en' else {LANG: language}
        for text in entry[language]['highlights']:
            ET.SubElement(bullets, 'li', attrs).text = text
    for language in ('en', 'es'):
        attrs = {} if language == 'en' else {LANG: language}
        for text in entry[language]['limitations']:
            ET.SubElement(description, 'p', attrs).text = text
    return release


def notes_markdown(entry):
    sections = []
    for language in ('en', 'es'):
        prose = entry[language]
        lines = [f"## Leaf {entry['version']} — {prose['title']}", '']
        lines += ['- ' + text for text in prose['highlights']]
        lines += ['', '\n\n'.join(prose['limitations'])] if prose['limitations'] else []
        sections.append('\n'.join(lines))
    return '\n\n'.join(sections) + '\n'


def generate(entries, template):
    root = ET.fromstring(template)
    releases = root.find('releases')
    if releases is None:
        releases = ET.SubElement(root, 'releases')
    managed = {entry['version'] for entry in entries}
    older = [entry for entry in releases if entry.get('version') not in managed]
    releases[:] = [release_element(entry) for entry in entries] + older
    releases[:] = sorted(releases, key=lambda e: int(e.get('version').rsplit('.', 1)[1]), reverse=True)
    ET.indent(root, space='  ')
    return '<?xml version="1.0" encoding="UTF-8"?>\n' + ET.tostring(root, encoding='unicode') + '\n'


def current_version():
    package = re.search(r'^version=(\S+)$', (ROOT / 'packaging/build-bundle.sh').read_text(), re.M)
    app = re.search(r'APP_VERSION:\s*&str\s*=\s*"([^"]+)"', (ROOT / 'src/gnome/mod.rs').read_text())
    if not package or not app or package[1] != app[1]:
        raise ValueError('Application and package versions disagree')
    return package[1]


def check_screenshots(root, online=False):
    images = root.findall('screenshots/screenshot/image')
    if not images:
        raise ValueError('No screenshot references')
    for image in images:
        url = image.text or ''
        if not re.fullmatch(r'https://raw\.githubusercontent\.com/frandavid100/leaf/(?:[0-9a-f]{40}|v[0-9][^/]+)/docs/screenshots/[^/]+\.png', url):
            raise ValueError('Screenshot must use an immutable commit or release tag: ' + url)
        if online:
            request = urllib.request.Request(url, method='HEAD')
            with urllib.request.urlopen(request, timeout=30) as response:
                if response.status != 200 or not response.headers.get('Content-Type', '').startswith('image/'):
                    raise ValueError('Unavailable screenshot: ' + url)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--check-urls', action='store_true')
    args = parser.parse_args(argv)
    entries = load_news()
    version = current_version()
    if entries[0]['version'] != version:
        raise ValueError(f'Add reviewed bilingual news for {version} before packaging')
    template = (ROOT / META).read_text()
    generated = generate(entries, template)
    check_screenshots(ET.fromstring(generated), args.check_urls)
    for relative, content in [(META, generated), ('packaging/RELEASE_NOTES.md', notes_markdown(entries[0]))]:
        path = ROOT / relative
        if args.check:
            if path.read_text() != content:
                raise ValueError(f'{relative} is outdated; run tools/release_metadata.py')
        else:
            path.write_text(content)
    print(f'RELEASE_METADATA_OK version={version} bilingual_releases={len(entries)}')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, ET.ParseError) as error:
        sys.exit(str(error))
