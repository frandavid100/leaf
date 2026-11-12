#!/usr/bin/env python3
"""Compare the same Poppins DOCX exported by Leaf and LibreOffice, without pixels."""
from pathlib import Path
import re
import subprocess
import sys
import xml.etree.ElementTree as ET

leaf, reference = map(Path, sys.argv[1:])


def inspect(path):
    fonts = subprocess.check_output(["pdffonts", str(path)], text=True)
    for face in ("Regular", "Bold", "Italic", "BoldItalic"):
        assert re.search(r"Poppins[-,]?" + face + r"\b", fonts), (path, face, fonts)
    xml = subprocess.check_output(["pdftotext", "-bbox", str(path), "-"], text=True)
    root = ET.fromstring(xml)
    assert len(root.findall(".//{*}page")) == 1, (path, "pagination differs")
    words = root.findall(".//{*}word")
    lines = []
    for word in words:
        y = float(word.attrib["yMin"])
        if not lines or abs(y - lines[-1][0]) > 3:
            lines.append([y, [], []])
        lines[-1][1].append(word.text or "")
        lines[-1][2].append((float(word.attrib["xMin"]), float(word.attrib["xMax"])))
    return [(" ".join(text), max(b for _, b in positions) - min(a for a, _ in positions))
            for _, text, positions in lines]


actual, expected = inspect(leaf), inspect(reference)
assert [text for text, _ in actual] == [text for text, _ in expected], (actual, expected)
assert len(actual) == 4, actual
for (text, width), (_, baseline) in zip(actual, expected):
    assert abs(width - baseline) <= max(1.0, baseline * 0.02), (text, width, baseline)
    print(f"FONT279_REFERENCE_LINE_OK {text!r}: Leaf={width:.3f} Writer={baseline:.3f}")
print("FONT279_REFERENCE_LAYOUT_OK styles=4 pages=1")
