#!/usr/bin/env python3
"""Generate original CC0 rectangle glyphs, never an installed or catalog font."""
from pathlib import Path
import sys
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

target = Path(sys.argv[1])
target.parent.mkdir(parents=True, exist_ok=True)
builder = FontBuilder(1000, isTTF=True)
glyphs = [".notdef", "space"] + [f"uni{n:04X}" for n in range(33, 127)]
builder.setupGlyphOrder(glyphs)
builder.setupCharacterMap({32: "space", **{n: f"uni{n:04X}" for n in range(33, 127)}})
outlines = {}
for name in glyphs:
    pen = TTGlyphPen(None)
    if name != "space":
        pen.moveTo((80, 0))
        pen.lineTo((500, 0))
        pen.lineTo((500, 680))
        pen.lineTo((80, 680))
        pen.closePath()
    outlines[name] = pen.glyph()
builder.setupGlyf(outlines)
builder.setupHorizontalMetrics({name: (600, 80) for name in glyphs})
builder.setupHorizontalHeader(ascent=800, descent=-200)
builder.setupNameTable({
    "familyName": "Leaf279 QA", "styleName": "Regular",
    "uniqueFontIdentifier": "Leaf279-QA-Original-1", "fullName": "Leaf279 QA Regular",
    "psName": "Leaf279QA-Regular", "version": "Version 1.0",
    "copyright": "Original Leaf test fixture, CC0 1.0 Universal",
    "licenseDescription": "CC0 1.0 Universal",
    "licenseInfoURL": "https://creativecommons.org/publicdomain/zero/1.0/",
})
builder.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
builder.setupPost()
builder.save(target)
print("FONT279_QA_FIXTURE_CREATED", target.name)
