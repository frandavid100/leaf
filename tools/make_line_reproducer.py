#!/usr/bin/env python3
"""Synthetic DOCX controls for #132; no private source document is copied."""
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED
from html import escape

root = Path(__file__).resolve().parents[1] / "tests/fixtures"
text = "Estaría bien que cada exposición durará aproximadamente entre 1 y 2 minutos, ¡si hacen más pues genial! 🙂"
for emoji in (True, False):
    parts = {
        "[Content_Types].xml": '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>',
        "_rels/.rels": '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>',
        "word/_rels/document.xml.rels": '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>',
        "word/styles.xml": '<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Arial" w:hAnsi="Arial"/><w:sz w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style></w:styles>',
        "word/document.xml": '<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:pPr><w:spacing w:after="240" w:before="240" w:lineRule="auto"/></w:pPr><w:r><w:t>' + escape(text if emoji else text.replace("🙂", "")) + '</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1680" w:bottom="1440" w:left="1440" w:right="1440"/></w:sectPr></w:body></w:document>',
    }
    with ZipFile(root / ("wrapped-emoji.docx" if emoji else "wrapped-control.docx"), "w", ZIP_DEFLATED) as z:
        for name, data in parts.items():
            z.writestr(name, data)
