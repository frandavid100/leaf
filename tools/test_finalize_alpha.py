import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from finalize_alpha import document_changes, verify


class PublishedAlphaVerification(unittest.TestCase):
    def setUp(self):
        self.candidate = dict(version='0.1.0-alpha.39', source='a' * 40, run_id=12,
                              native_source='b' * 40, native_run_id=11,
                              closed_issues=[212, 213, 214], publication='authorized')
        self.notes = 'Release notes\n'
        names = ['Leaf-0.1.0-alpha.39-x86_64.flatpak', 'Leaf-0.1.0-alpha.39-flatpark-x86_64.tar.xz',
                 'Leaf-0.1.0-alpha.39-dictionary-test.zip', 'INSTALL.md', 'source-commit.txt',
                 'runtime.txt', 'runtime-commit.txt', 'flatpak-metadata.txt',
                 'flatpak-smoke.log', 'flatpark-smoke.log', 'candidate.json', 'RELEASE_NOTES.md']
        hashes = {name: hashlib.sha256(name.encode()).hexdigest() for name in names}
        for name, text in [('source-commit.txt', self.candidate['source'] + '\n'),
                           ('candidate.json', json.dumps(self.candidate, indent=2) + '\n'),
                           ('RELEASE_NOTES.md', self.notes)]:
            hashes[name] = hashlib.sha256(text.encode()).hexdigest()
        self.manifest = ''.join(digest + '  ' + name + '\n' for name, digest in hashes.items())
        hashes['SHA256SUMS'] = hashlib.sha256(self.manifest.encode()).hexdigest()
        self.release = dict(tag_name='v0.1.0-alpha.39', prerelease=True, draft=False,
                            html_url='https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.39',
                            published_at='2026-11-08T00:00:00Z', body=self.notes,
                            assets=[dict(name=name, state='uploaded', size=10, digest='sha256:' + digest)
                                    for name, digest in hashes.items()])
        self.native = dict(id=11, head_sha='b' * 40, status='completed', conclusion='success',
                           path='.github/workflows/wordcraft-gnome.yml')
        self.run = dict(self.native, id=12, head_sha='a' * 40)

    def check(self, **changes):
        args = dict(release=self.release, tag='a' * 40, candidate=self.candidate,
                    manifest=self.manifest, notes=self.notes, native=self.native, run=self.run)
        args.update(changes)
        return verify(**args)

    def test_valid_complete_publication(self):
        self.assertEqual(self.check()['closed_issues'], [212, 213, 214])

    def test_wrong_or_missing_public_bytes_rejected(self):
        for asset_change in ['digest', 'missing', 'pending', 'empty']:
            release = copy.deepcopy(self.release)
            if asset_change == 'missing': release['assets'].pop()
            elif asset_change == 'digest': release['assets'][0]['digest'] = 'sha256:' + '0' * 64
            elif asset_change == 'pending': release['assets'][0]['state'] = 'new'
            else: release['assets'][0]['size'] = 0
            with self.subTest(asset_change=asset_change), self.assertRaises(AssertionError):
                self.check(release=release)

    def test_wrong_source_or_failed_validation_rejected(self):
        for change in [dict(tag='c' * 40), dict(native=dict(self.native, conclusion='failure')),
                       dict(run=dict(self.run, head_sha='d' * 40)), dict(run=dict(self.run, status='in_progress'))]:
            with self.subTest(change=change), self.assertRaises(AssertionError): self.check(**change)

    def test_manifest_and_notes_cannot_drift(self):
        for change in [dict(manifest=self.manifest + self.manifest.splitlines()[0] + '\n'),
                       dict(notes='Other notes\n')]:
            with self.subTest(change=change), self.assertRaises(AssertionError): self.check(**change)

    def test_candidate_issue_plan_cannot_be_changed_after_packaging(self):
        with self.assertRaises(AssertionError):
            self.check(candidate=dict(self.candidate, closed_issues=[1, 52]))

    def test_documentation_preserves_windows_and_existing_history(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'docs').mkdir()
            for name in ['PROJECT_STATUS.md', 'ROADMAP.md', 'docs/DECISIONS.md']:
                (root / name).write_text('# Project\n\n## Historical section\nKeep this.\n')
            windows = '**[Experimental Windows version](https://example.org/alpha.36-windows.1)**'
            (root / 'README.md').write_text('**[Try the Flatpak alpha](https://example.org/old)**\n' + windows)
            files = document_changes(self.check(), [1, 52, 213, 214], root)
            self.assertIn(windows, files['README.md'])
            for name in ['PROJECT_STATUS.md', 'ROADMAP.md', 'docs/DECISIONS.md']:
                self.assertIn('## Historical section\nKeep this.', files[name])
            self.assertIn('212', files['docs/LATEST_RELEASE.md'])
            self.assertIn('#52', files['PROJECT_STATUS.md'])
            self.assertIn('Permanecen abiertos: #1, #52.', files['PROJECT_STATUS.md'])
            self.assertIn('Permanecen abiertos: #1, #52.', files['docs/LATEST_RELEASE.md'])

    def test_maintenance_report_does_not_claim_empty_issue_closures(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'docs').mkdir()
            for name in ['PROJECT_STATUS.md', 'ROADMAP.md', 'docs/DECISIONS.md']:
                (root / name).write_text('# Project\n\n## History\nKeep this.\n')
            (root / 'README.md').write_text('Readme\n')
            (root / 'docs/ALPHA44_MAINTENANCE.md').write_text('# Maintenance\n\n## Scope\nKeep this.\n')
            proof = self.check() | {'closed_issues': [], 'release_report': 'docs/ALPHA44_MAINTENANCE.md'}
            files = document_changes(proof, [1, 99], root)
            report = files['docs/ALPHA44_MAINTENANCE.md']
            self.assertIn('claims no issue closures', report)
            self.assertNotIn(' are completed', report)
            self.assertIn('## Scope\nKeep this.', report)
