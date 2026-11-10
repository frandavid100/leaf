#!/usr/bin/env python3
"""Verify published candidate bytes before closing its explicit issues and updating docs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess


def api(path, payload=None, method=None):
    command = ['gh', 'api', path]
    if payload is not None:
        command += ['--method', method or 'POST', '--input', '-']
    result = subprocess.run(command, input=json.dumps(payload) if payload is not None else None,
                            text=True, check=True, capture_output=True)
    return json.loads(result.stdout)


def verify(release, tag, candidate, manifest, notes, native, run):
    version = candidate['version']
    assert re.fullmatch(r'0\.1\.0-alpha\.[1-9][0-9]*', version)
    source = candidate['source']
    assert re.fullmatch(r'[a-f0-9]{40}', source)
    assert tag == source
    assert release['tag_name'] == 'v' + version and release['prerelease'] and not release['draft']
    for record, identity, sha in [(native, candidate['native_run_id'], candidate['native_source']),
                                  (run, candidate['run_id'], source)]:
        assert record['id'] == identity and record['head_sha'] == sha
        assert record['status'] == 'completed' and record['conclusion'] == 'success'
        assert record['path'] == '.github/workflows/wordcraft-gnome.yml'
    rows = []
    for line in manifest.splitlines():
        match = re.fullmatch(r'([a-f0-9]{64})  ([A-Za-z0-9._-]+)', line)
        assert match, 'Invalid checksum manifest'
        rows.append(match.groups())
    expected_names = {f'Leaf-{version}-x86_64.flatpak', f'Leaf-{version}-flatpark-x86_64.tar.xz',
                      f'Leaf-{version}-dictionary-test.zip', 'INSTALL.md', 'source-commit.txt',
                      'runtime.txt', 'runtime-commit.txt', 'flatpak-metadata.txt',
                      'flatpak-smoke.log', 'flatpark-smoke.log', 'candidate.json', 'RELEASE_NOTES.md'}
    assert len(rows) == 12 and {name for _, name in rows} == expected_names
    expected = {name: digest for digest, name in rows}
    expected['SHA256SUMS'] = hashlib.sha256(manifest.encode()).hexdigest()
    assert len(release['assets']) == 13 and {a['name'] for a in release['assets']} == set(expected)
    for asset in release['assets']:
        assert asset['state'] == 'uploaded' and asset['size'] > 0
        assert asset['digest'] == 'sha256:' + expected[asset['name']], asset['name']
    assert expected['source-commit.txt'] == hashlib.sha256((source + '\n').encode()).hexdigest()
    assert expected['candidate.json'] == hashlib.sha256((json.dumps(candidate, indent=2) + '\n').encode()).hexdigest()
    assert expected['RELEASE_NOTES.md'] == hashlib.sha256(notes.encode()).hexdigest()
    assert release['body'].strip() == notes.strip()
    issues = candidate.get('closed_issues', [])
    assert len(issues) == len(set(issues)) and all(type(n) is int and n > 0 for n in issues)
    report = candidate.get('release_report')
    assert report is None or re.fullmatch(r'docs/ALPHA[0-9]+_[A-Z_]+\.md', report)
    return dict(release_report=report, verified=True, version=version, source=source, native_source=candidate['native_source'],
                native_run=candidate['native_run_id'], candidate_run=candidate['run_id'],
                release_url=release['html_url'], published_at=release['published_at'],
                closed_issues=issues, asset_sha256=expected)


def document_changes(proof, remaining, root):
    version, source = proof['version'], proof['source']
    closed = ', '.join('#' + str(n) for n in proof['closed_issues']) or 'ninguno'
    # GitHub's issue list can lag its successful close responses.
    remaining = sorted(set(remaining) - set(proof['closed_issues']))
    open_issues = ', '.join('#' + str(n) for n in remaining) or 'ninguno'
    report = f'''# Última release verificada: {version}

[Release y descargas]({proof['release_url']}).
Cerrados tras verificar la publicación: {closed}. Permanecen abiertos: {open_issues}.

- Fuente/tag: `{source}`.
- [Batería nativa completa](https://github.com/frandavid100/leaf/actions/runs/{proof['native_run']}).
- [Paquetes instalados, ambos runtimes y comparación del ejecutable](https://github.com/frandavid100/leaf/actions/runs/{proof['candidate_run']}).
- Trece archivos públicos, SHA-256, notas y fuente comprobados contra la candidata.
- [Prueba de identidad y digests](LATEST_RELEASE_ASSETS.json).

La validación es automática; no implica pruebas humanas con GNOME OS, impresoras o dispositivos físicos.
'''
    files = {'docs/LATEST_RELEASE.md': report,
             'docs/LATEST_RELEASE_ASSETS.json': json.dumps(proof, indent=2) + '\n'}
    block = f'''<!-- published-alpha:start -->
## Última publicación verificada: {version}

[Release]({proof['release_url']}). Issues cerrados: {closed}.
Permanecen abiertos: {open_issues}.
[Fuente, validación y límites](docs/LATEST_RELEASE.md).
<!-- published-alpha:end -->
'''
    report_path = proof.get('release_report')
    if report_path:
        report_text = (root / report_path).read_text()
        at = report_text.index('\n## ')
        completed = (f'Published and verified: {closed} are completed.' if proof['closed_issues']
                     else 'Published and verified; this maintenance release claims no issue closures.')
        files[report_path] = (report_text.splitlines()[0] + '\n\n' + completed +
                              ' See [published identity and validation](LATEST_RELEASE.md).\n' + report_text[at:])
    for name in ['PROJECT_STATUS.md', 'ROADMAP.md']:
        text = (root / name).read_text()
        text = re.sub(r'<!-- candidate-alpha:start -->.*?<!-- candidate-alpha:end -->\n', '', text, flags=re.S)
        if '<!-- published-alpha:start -->' in text:
            text = re.sub(r'<!-- published-alpha:start -->.*?<!-- published-alpha:end -->\n', block, text, flags=re.S)
        else:
            at = text.index('\n## ')
            text = text[:at] + '\n\n' + block + text[at:]
        files[name] = text
    readme = (root / 'README.md').read_text()
    files['README.md'] = re.sub(r'(\*\*\[Try the Flatpak alpha\]\()https://github.com/frandavid100/leaf/releases/tag/[^)]+',
                                r'\g<1>' + proof['release_url'], readme)
    decisions = (root / 'docs/DECISIONS.md').read_text()
    entry = f'''## Publicada y verificada automáticamente: {version}

Se publican los mismos bytes de la candidata tras superar todas las pruebas
nativas y de instalación. Se verifican fuente, tag y trece archivos públicos antes
de cerrar {closed}. [Identidad y límites](LATEST_RELEASE.md).

'''
    at = decisions.index('\n## ')
    files['docs/DECISIONS.md'] = decisions[:at + 1] + entry + decisions[at + 1:]
    return files


def finalize(dist, close=False, docs=False):
    repo = os.environ['GITHUB_REPOSITORY']
    assert repo == 'frandavid100/leaf'
    endpoint = 'repos/' + repo
    candidate = json.loads((dist / 'candidate.json').read_text())
    version = candidate['version']
    release = api(endpoint + '/releases/tags/v' + version)
    tag = api(endpoint + '/git/ref/tags/v' + version)['object']
    if tag['type'] == 'tag':
        tag = api(endpoint + '/git/tags/' + tag['sha'])['object']
    assert tag['type'] == 'commit'
    proof = verify(release, tag['sha'], candidate, (dist / 'SHA256SUMS').read_text(),
                   (dist / 'RELEASE_NOTES.md').read_text(),
                   api(endpoint + '/actions/runs/' + str(candidate['native_run_id'])),
                   api(endpoint + '/actions/runs/' + str(candidate['run_id'])))
    (dist / 'public-verification.json').write_text(json.dumps(proof, indent=2) + '\n')
    if close:
        for number in proof['closed_issues']:
            path = endpoint + '/issues/' + str(number)
            issue = api(path)
            assert 'pull_request' not in issue
            if issue['state'] == 'closed':
                continue
            comment = (f"Published in [{version}]({proof['release_url']}). Complete native validation "
                       f"and installed GNOME 49/51 checks passed; the final executable comparison, "
                       f"source/tag and all thirteen public asset SHA-256 digests were verified "
                       f"against candidate run {proof['candidate_run']}. See the release notes for "
                       "the resulting behavior. Closing as completed after publication verification.")
            api(path + '/comments', {'body': comment})
            result = api(path, {'state': 'closed', 'state_reason': 'completed'}, 'PATCH')
            assert result['state'] == 'closed' and result['state_reason'] == 'completed'
    if docs:
        assert close, 'Published status must follow completed issue closures'
        source = proof['source']
        current = api(endpoint + '/git/ref/heads/main')['object']['sha']
        if current != source:
            print('Documentation preserved: main advanced after publication; proof remains in artifacts.')
            return proof
        remaining = []
        page = 1
        while True:
            batch = api(endpoint + f'/issues?state=open&per_page=100&page={page}')
            remaining.extend(x['number'] for x in batch if 'pull_request' not in x)
            if len(batch) < 100: break
            page += 1
        files = document_changes(proof, remaining, Path.cwd())
        tree = api(endpoint + '/git/trees', {'base_tree': api(endpoint + '/git/commits/' + source)['tree']['sha'],
                   'tree': [dict(path=path, mode='100644', type='blob', content=text) for path, text in files.items()]})
        commit = api(endpoint + '/git/commits', {'message': f'[skip ci] Record verified {version} and completed issues',
                     'tree': tree['sha'], 'parents': [source]})
        # Non-forced update rejects a concurrent change instead of overwriting it.
        api(endpoint + '/git/refs/heads/main', {'sha': commit['sha'], 'force': False}, 'PATCH')
        print('DOCUMENTATION_UPDATED', commit['sha'])
    print('PUBLIC_ALPHA_VERIFIED', version, 'assets=13', 'closed=' + str(proof['closed_issues']))
    return proof


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dist', type=Path, default=Path('dist'))
    parser.add_argument('--close-issues', action='store_true')
    parser.add_argument('--update-docs', action='store_true')
    args = parser.parse_args()
    finalize(args.dist, args.close_issues, args.update_docs)
