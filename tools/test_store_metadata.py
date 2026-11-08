import copy
import gettext
import json
import pathlib
import tempfile
import unittest
import xml.etree.ElementTree as ET

from metainfo_translations import LANG, text_nodes, translate
from release_metadata import check_screenshots, generate, load_news, notes_markdown, release_element


class StoreMetadataTests(unittest.TestCase):
    def test_list_entries_are_translated_without_localizing_the_list_container(self):
        root = ET.fromstring('<component><summary>Write</summary><description><p>Body</p><ul><li>Item</li></ul></description></component>')
        class Spanish(gettext.NullTranslations):
            def gettext(self, text):
                return {'Write': 'Escribe', 'Body': 'Texto', 'Item': 'Elemento'}[text]
        self.assertEqual([n.text for n in text_nodes(root)], ['Write', 'Body', 'Item'])
        translate(root, {'es': Spanish()})
        self.assertEqual(root.find('description/ul/li[@' + LANG + ']').text, 'Elemento')
        self.assertNotIn(LANG, root.find('description/ul').attrib)

    def test_generation_and_github_notes_share_reviewed_prose(self):
        entries = load_news()
        template = '<component><releases/></component>'
        output = generate(entries, template)
        self.assertEqual(output, generate(entries, output))
        xml = ET.fromstring(output)
        self.assertEqual(len(xml.findall('releases/release')), len(entries))
        notes = notes_markdown(entries[0])
        for language in ('en', 'es'):
            for text in entries[0][language]['highlights'] + entries[0][language]['limitations']:
                self.assertIn(text, notes)
                self.assertIn(text, ''.join(xml.find('releases/release').itertext()))

    def test_release_has_details_and_localized_paragraphs_and_items(self):
        entry = load_news()[0]
        release = release_element(entry)
        self.assertEqual(release.get('type'), 'development')
        self.assertEqual(release.find('url').text, entry['url'])
        self.assertTrue(release.findall('.//li[@' + LANG + '="es"]'))
        self.assertFalse(release.findall('.//ul[@' + LANG + ']'))

    def test_empty_translation_and_wrong_release_url_are_rejected(self):
        for mutate in (lambda x: x['es'].update(highlights=['']), lambda x: x.update(url='https://example.org/wrong')):
            with self.subTest(mutate=mutate), tempfile.TemporaryDirectory() as temp:
                entry = copy.deepcopy(load_news()[0])
                mutate(entry)
                file = pathlib.Path(temp) / 'news.json'
                file.write_text(json.dumps({'releases': [entry]}))
                with self.assertRaises(ValueError):
                    load_news(file)

    def test_screenshot_must_be_immutable(self):
        for ref, valid in [('main', False), ('a' * 40, True), ('v0.1.0-alpha.35', True)]:
            xml = ET.fromstring(f'<component><screenshots><screenshot><image>https://raw.githubusercontent.com/frandavid100/leaf/{ref}/docs/screenshots/test.png</image></screenshot></screenshots></component>')
            if valid:
                check_screenshots(xml)
            else:
                with self.assertRaises(ValueError):
                    check_screenshots(xml)


if __name__ == '__main__':
    unittest.main()
