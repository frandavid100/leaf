"""Extract Leaf's literal translation calls, including rustfmt's multiline form.

This deliberately handles Leaf's JSON-compatible string literals, not arbitrary
Rust syntax. Keep source-message identity stable when moving or formatting code.
"""
import json
import re

_MESSAGE = re.compile(
    r'(?<![\w])(?:wordcraft_gnome::)?t!\s*\(\s*("(?:\\.|[^"\\])*")'
)


def messages(source):
    """Yield (message, one-based source line) for each supported translation call."""
    for match in _MESSAGE.finditer(source):
        yield json.loads(match[1]), source.count("\n", 0, match.start()) + 1
