import unittest

from translation_source import messages


class TranslationSourceTests(unittest.TestCase):
    def test_rustfmt_multiline_and_qualified_calls_keep_unicode_and_placeholders(self):
        source = 'wordcraft_gnome::t!(\n    "Café: {name}"\n);\nt!("Second");'
        self.assertEqual(list(messages(source)), [("Café: {name}", 1), ("Second", 4)])

    def test_format_macro_suffix_is_not_a_translation_call(self):
        self.assertEqual(list(messages('format!("internal {x}");')), [])

    def test_repeated_messages_retain_all_reference_lines(self):
        self.assertEqual(list(messages('t!("Save");\n\nt!( "Save" );')), [("Save", 1), ("Save", 3)])

    def test_escaped_quotes_and_newlines_are_decoded(self):
        self.assertEqual(list(messages(r't!("Say \"hello\"\nNext");')), [('Say "hello"\nNext', 1)])


if __name__ == "__main__":
    unittest.main()
