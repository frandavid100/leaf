#!/usr/bin/env python3
"""Optional one-off interoperability audit; run with system Python + python3-uno.

Creates its own non-confidential fixtures. Opens them in a separate LibreOffice
profile via UNO, edits actual document text, saves in the original format, and
reopens with Leaf's real file adapters. Reports losses; success means the audit
completed, not that every comparison passed. Never a per-release requirement.
"""
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import traceback
import zipfile
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "interop-results"
WORKER = ROOT / "target/release/examples/interop_audit"


def run(*args):
    try:
        return subprocess.run(list(map(str, args)), check=True, text=True, capture_output=True, timeout=100).stdout
    except subprocess.CalledProcessError as error:
        raise RuntimeError(f"{args[0]}: {error.stderr.strip()}") from error


def prop(name, value):
    from com.sun.star.beans import PropertyValue
    p = PropertyValue()
    p.Name, p.Value = name, value
    return p


def dump(path, data):
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")


def uno_state(doc):
    def get(obj, name):
        try:
            return obj.getPropertyValue(name)
        except Exception:
            return None
    page_styles = doc.StyleFamilies.getByName("PageStyles")
    styles = []
    for name in page_styles.ElementNames:
        p = page_styles.getByName(name)
        if not p.isInUse():
            continue
        names = ["Width", "Height", "TopMargin", "BottomMargin", "LeftMargin", "RightMargin", "HeaderIsOn", "HeaderHeight", "HeaderBodyDistance", "HeaderIsShared", "FirstIsShared", "FooterIsOn", "FooterHeight", "FooterBodyDistance", "FooterIsShared", "NumberingType"]
        item = {key: get(p, key) for key in names}
        item["name"] = name
        for key in ["HeaderText", "HeaderTextLeft", "HeaderTextFirst", "FooterText", "FooterTextLeft", "FooterTextFirst"]:
            text = get(p, key)
            item[key] = text.String if text is not None else None
        styles.append(item)
    probes = []
    for word in ["English", "Spanish", "UNCHECKED_TOKEN"]:
        search = doc.createSearchDescriptor()
        search.SearchString = word
        found = doc.findFirst(search)
        locale = get(found, "CharLocale") if found else None
        probes.append({"token": word, "language": locale.Language if locale else None, "country": locale.Country if locale else None})
    fields = doc.TextFields.createEnumeration()
    field_info = []
    while fields.hasMoreElements():
        f = fields.nextElement()
        field_info.append({"services": list(f.SupportedServiceNames), "presentation": f.getPresentation(False)})
    return {"body_text": doc.Text.String, "pages": doc.CurrentController.PageCount, "styles": styles, "language": probes, "fields": field_info}


def inspect(path, folder):
    run(WORKER, "inspect", path, folder)
    return json.loads((folder / "state.json").read_text())


def without_leaf_extensions(source, target):
    # Preserve every standard attribute, text and package member. Strip only our
    # namespace, including LibreOffice's regenerated prefix for that namespace.
    with zipfile.ZipFile(source) as zin, zipfile.ZipFile(target, "w") as zout:
        for entry in zin.infolist():
            data = zin.read(entry.filename)
            if entry.filename in ["styles.xml", "content.xml"]:
                root = ET.fromstring(data)
                for node in root.iter():
                    for key in list(node.attrib):
                        if key.startswith("{https://github.com/frandavid100/leaf/ns/odt/1.0}"):
                            del node.attrib[key]
                # The engine's XML reader uses standard prefixes; retain them.
                for _, (prefix, uri) in ET.iterparse(source_xml(data), events=["start-ns"]):
                    if prefix != "xmlns" and not prefix.startswith("ns"):
                        ET.register_namespace(prefix, uri)
                data = ET.tostring(root, encoding="utf-8", xml_declaration=True)
            zout.writestr(entry, data)


def source_xml(data):
    import io
    return io.BytesIO(data)


def compare(before, after, lo, pdf_text):
    checks = []
    def check(label, result, detail=""):
        checks.append({"check": label, "passed": bool(result), "detail": detail})
    expected = before["body_text"].replace("BODY_SENTINEL", "BODY_EDITED_IN_LIBREOFFICE").strip()
    check("Body text after independent edit", after["body_text"].strip() == expected)
    headers = {p["text"] for p in before["parts"] if "HEADER_S" in p["text"]}
    after_headers = {p["text"].strip() for p in after["parts"] if "HEADER_S" in p["text"]}
    check("All header variant contents retained", headers <= after_headers, str(sorted(headers - after_headers)))
    check("All header variants visible in LibreOffice PDF", all(text in pdf_text for text in headers), str(sorted(text for text in headers if text not in pdf_text)))
    check("Language and unchecked ranges retained", before["language"] == after["language"])
    check("English and Spanish recognised independently by LibreOffice", [p["language"] for p in lo["language"][:2]] == ["en", "es"], str(lo["language"][:2]))
    check("Unchecked range recognised independently by LibreOffice", lo["language"][2]["language"] in ["zxx", "none", ""], str(lo["language"][2]))
    active_headers = [s for s in lo["styles"] if s["HeaderIsOn"]]
    active_footers = [s for s in lo["styles"] if s["FooterIsOn"]]
    check("Header edge distance recognised independently", bool(active_headers) and all(abs(s["TopMargin"]*72/2540-before["sections"][0]["header"]) < .2 for s in active_headers), str([s["TopMargin"]*72/2540 for s in active_headers]))
    check("Footer edge distance recognised independently", bool(active_footers) and all(abs(s["BottomMargin"]*72/2540-before["sections"][0]["footer"]) < .2 for s in active_footers), str([s["BottomMargin"]*72/2540 for s in active_footers]))
    check("Section count retained", len(before["sections"]) == len(after["sections"]), f'{len(before["sections"])} -> {len(after["sections"])}')
    for kind in ["headers", "footers"]:
        links = lambda state: [[s[kind][slot] is None for slot in ["default", "first", "even"]] for s in state["sections"]]
        a, b = links(before), links(after)
        check("Previous-section link semantics: " + kind, a == b, f"{a} -> {b}")
    check("Displayed header sequence retained", [p["header"] for p in before["pages"]] == [p["header"] for p in after["pages"]])
    for key in ["header", "footer", "marginTop", "marginBottom", "pageNumFormat", "pageNumStart", "titlePage"]:
        a = [s[key] for s in before["sections"]]
        b = [s[key] for s in after["sections"]]
        equal = len(a) == len(b) and all(abs(x-y) < .2 if isinstance(x, (int,float)) and not isinstance(x,bool) and isinstance(y,(int,float)) else x == y for x,y in zip(a,b))
        check("Section property: " + key, equal, f"{a} -> {b}")
    fields = json.dumps(after["parts"])
    check("Leaf reopens PAGE and NUMPAGES fields", '"PAGE"' in fields and '"NUMPAGES"' in fields)
    check("LibreOffice recognises automatic page fields", any("PageNumber" in str(f["services"]) for f in lo["fields"]) and any("PageCount" in str(f["services"]) for f in lo["fields"]))
    return checks


def check_writer_authored_docx(desktop, macro_mode):
    """Open a Writer-authored document in Leaf, edit it, and check both outputs."""
    folder = OUT / "writer-authored"
    folder.mkdir(exist_ok=True)
    original = folder / "writer.docx"
    options = (prop("Hidden", True), prop("MacroExecutionMode", macro_mode))
    paragraphs = "Worksheet instructions\nAnswer the questions below.\nResults"
    cells = {"A1": "Task", "B1": "Points", "A2": "Reading", "B2": "10"}
    doc = desktop.loadComponentFromURL("private:factory/swriter", "_blank", 0, options)
    if doc is None:
        raise RuntimeError("LibreOffice could not create the worksheet fixture")
    try:
        from com.sun.star.text.ControlCharacter import PARAGRAPH_BREAK

        cursor = doc.Text.createTextCursor()
        for index, paragraph in enumerate(paragraphs.splitlines()):
            if index:
                doc.Text.insertControlCharacter(cursor, PARAGRAPH_BREAK, False)
            doc.Text.insertString(cursor, paragraph, False)
            cursor.gotoEnd(False)
        first = doc.Text.createTextCursor()
        first.gotoStart(False)
        first.ParaBottomMargin = 318  # Nine points, in hundredths of a millimetre.
        end = doc.Text.createTextCursor()
        end.gotoEnd(False)
        table = doc.createInstance("com.sun.star.text.TextTable")
        table.initialize(2, 2)
        doc.Text.insertTextContent(end, table, False)
        for address, value in cells.items():
            table.getCellByName(address).String = value
        doc.storeAsURL(original.as_uri(), (prop("FilterName", "Office Open XML Text"), prop("Overwrite", True)))
    finally:
        doc.close(True)

    run(WORKER, "edit", original, folder)
    reopened = desktop.loadComponentFromURL((folder / "leaf-edited.docx").as_uri(), "_blank", 0, options)
    if reopened is None:
        raise RuntimeError("LibreOffice could not reopen the Leaf-edited worksheet")
    try:
        body = reopened.Text.String
        tables = reopened.TextTables
        table_ok = tables.Count == 1 and all(
            tables.getByIndex(0).getCellByName(address).String == value
            for address, value in cells.items()
        )
        first = reopened.Text.createTextCursor()
        first.gotoStart(False)
        spacing_pt = first.ParaBottomMargin * 72 / 2540
    finally:
        reopened.close(True)

    geometry = json.loads((folder / "spacing.json").read_text())
    pdf = run("pdftotext", "-layout", folder / "leaf-edited.pdf", "-")
    checks = [
        ("Writer reopens the Leaf edit", body.startswith("LEAF_EDIT " + paragraphs), ""),
        ("Writer reopens the original table cells", table_ok, ""),
        ("Writer reads Leaf's 24-point paragraph spacing", abs(spacing_pt - 24) < .2, str(spacing_pt)),
        ("Leaf reflows an imported 9-point gap to 24 points",
         abs(geometry["edited"] - geometry["baseline"] - 15) < .4, str(geometry)),
        ("Leaf retains that geometry after DOCX reopen",
         abs(geometry["edited"] - geometry["reopened"]) < .2, str(geometry)),
        ("Leaf PDF contains the edit and table content",
         all(token in pdf for token in ["LEAF_EDIT", "Answer the questions", *cells.values()]), ""),
    ]
    return {
        "file": "writer-authored.docx",
        "checks": [{"check": name, "passed": passed, "detail": detail} for name, passed, detail in checks],
    }


def main():
    import uno
    from com.sun.star.document.MacroExecMode import NEVER_EXECUTE
    OUT.mkdir(exist_ok=True)
    run(WORKER, "generate", OUT / "input")
    # Private profile and an unused local port; never connect to a user's LO.
    with tempfile.TemporaryDirectory(prefix="leaf-lo-interop-") as profile:
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        process = subprocess.Popen(["libreoffice", "-env:UserInstallation=" + Path(profile).as_uri(), "--headless", "--norestore", "--nodefault", "--nofirststartwizard", f"--accept=socket,host=127.0.0.1,port={port};urp;StarOffice.ComponentContext"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            local = uno.getComponentContext()
            resolver = local.ServiceManager.createInstanceWithContext("com.sun.star.bridge.UnoUrlResolver", local)
            for attempt in range(100):
                try:
                    context = resolver.resolve(f"uno:socket,host=127.0.0.1,port={port};urp;StarOffice.ComponentContext")
                    break
                except Exception:
                    if process.poll() is not None or attempt == 99:
                        raise
                    time.sleep(.2)
            desktop = context.ServiceManager.createInstanceWithContext("com.sun.star.frame.Desktop", context)
            results = [check_writer_authored_docx(desktop, NEVER_EXECUTE)]
            for path in sorted((OUT / "input").iterdir()):
                folder = OUT / (path.stem + "-" + path.suffix[1:])
                folder.mkdir(exist_ok=True)
                before = inspect(path, folder / "leaf-original")
                doc = desktop.loadComponentFromURL(path.as_uri(), "_blank", 0, (prop("Hidden", True), prop("UpdateDocMode", 3), prop("MacroExecutionMode", NEVER_EXECUTE)))
                if doc is None:
                    raise RuntimeError(f"LibreOffice refused {path}")
                try:
                    doc.refresh()
                    initial = uno_state(doc)
                    dump(folder / "libreoffice-original.json", initial)
                    replace = doc.createReplaceDescriptor()
                    replace.SearchString, replace.ReplaceString = "BODY_SENTINEL", "BODY_EDITED_IN_LIBREOFFICE"
                    replacements = doc.replaceAll(replace)
                    if replacements != len(before["sections"]):
                        raise RuntimeError(f"Unexpected edit count: {replacements}")
                    doc.TextFields.refresh()
                    saved = folder / ("libreoffice-saved" + path.suffix)
                    filter_name = "writer8" if path.suffix == ".odt" else "Office Open XML Text"
                    doc.storeAsURL(saved.as_uri(), (prop("FilterName", filter_name), prop("Overwrite", True)))
                    if path.suffix == ".docx":
                        doc.storeToURL((folder / "libreoffice-reference.odt").as_uri(), (prop("FilterName", "writer8"), prop("Overwrite", True)))
                    doc.storeToURL((folder / "libreoffice.pdf").as_uri(), (prop("FilterName", "writer_pdf_Export"), prop("Overwrite", True)))
                    lo = uno_state(doc)
                    dump(folder / "libreoffice-edited.json", lo)
                finally:
                    doc.close(True)
                after = inspect(saved, folder / "leaf-reopened")
                final = inspect(folder / "leaf-reopened" / ("leaf-saved" + path.suffix), folder / "leaf-final")
                run("pdftotext", "-layout", folder / "libreoffice.pdf", folder / "libreoffice.txt")
                run("pdftoppm", "-f", "1", "-singlefile", "-scale-to", "1100", "-png", folder / "libreoffice.pdf", folder / "libreoffice-first")
                checks = compare(before, after, lo, (folder / "libreoffice.txt").read_text())
                checks.append({"check": "Final Leaf save/reopen preserves edited body", "passed": final["body_text"] == after["body_text"], "detail": ""})
                checks.append({"check": "Final Leaf save/reopen preserves language ranges", "passed": final["language"] == after["language"], "detail": ""})
                standard = folder / "standard-only.odt"
                source = saved if path.suffix == ".odt" else folder / "libreoffice-reference.odt"
                without_leaf_extensions(source, standard)
                standard_state = inspect(standard, folder / "leaf-standard-only")
                for key in ["header", "footer", "marginTop", "marginBottom"]:
                    a = [s[key] for s in before["sections"]]
                    b = [s[key] for s in standard_state["sections"]]
                    # A fresh LO export is the reference for DOCX; an ODT saved
                    # by LO is also checked with all Leaf extension data removed.
                    same = len(a) == len(b) and all(abs(x-y) < .2 for x,y in zip(a,b))
                    checks.append({"check": "Standard ODT reopen: " + key, "passed": same, "detail": f"{a} -> {b}"})
                record = {"file": path.name, "checks": checks}
                results.append(record)
                dump(folder / "checks.json", record)
                print(path.name, sum(c["passed"] for c in checks), "/", len(checks), flush=True)
            dump(OUT / "results.json", {"source_commit": os.environ.get("GITHUB_SHA"), "libreoffice": run("libreoffice", "--version").strip(), "cases": results})
            lines = ["# Leaf ↔ LibreOffice: focused one-off audit", "", "Synthetic fixtures generated through Leaf and Writer; each editor opens, edits or reopens files produced by the other. A completed workflow is not an all-clear for document compatibility.", "", "| File | Check | Result | Detail |", "|---|---|---|---|"]
            for case in results:
                for c in case["checks"]:
                    lines.append(f'| {case["file"]} | {c["check"]} | {"PASS" if c["passed"] else "LOSS / DIFFERENCE"} | {c["detail"].replace("|", "/")} |')
            (OUT / "REPORT.md").write_text("\n".join(lines) + "\n")
            desktop.terminate()
            print("INTEROP_AUDIT_COMPLETED", flush=True)
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()


if __name__ == "__main__":
    try:
        main()
    except Exception:
        OUT.mkdir(exist_ok=True)
        (OUT / "REPORT.md").write_text("# Audit incomplete\n\n```\n" + traceback.format_exc() + "```\n")
        raise
