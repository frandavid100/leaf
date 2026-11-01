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
import re
import socket
import subprocess
import tempfile
import time
import traceback
import zipfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "interop-results"
WORKER = ROOT / "target/release/examples/interop_audit"


def run(*args):
    return subprocess.run(list(map(str, args)), check=True, text=True, capture_output=True, timeout=100).stdout


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
    check("Language recognised independently by LibreOffice", [p["language"] for p in lo["language"]] == ["en", "es", "zxx"], str(lo["language"]))
    check("Section count retained", len(before["sections"]) == len(after["sections"]), f'{len(before["sections"])} -> {len(after["sections"])}')
    for key in ["header", "footer", "marginTop", "marginBottom", "pageNumFormat", "pageNumStart", "titlePage"]:
        a = [s[key] for s in before["sections"]]
        b = [s[key] for s in after["sections"]]
        equal = len(a) == len(b) and all(abs(x-y) < .2 if isinstance(x, (int,float)) and not isinstance(x,bool) and isinstance(y,(int,float)) else x == y for x,y in zip(a,b))
        check("Section property: " + key, equal, f"{a} -> {b}")
    fields = json.dumps(after["parts"])
    check("Leaf reopens PAGE and NUMPAGES fields", '"PAGE"' in fields and '"NUMPAGES"' in fields)
    check("LibreOffice recognises automatic page fields", any("PageNumber" in str(f["services"]) for f in lo["fields"]) and any("PageCount" in str(f["services"]) for f in lo["fields"]))
    return checks


def main():
    import uno
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
            results = []
            for path in sorted((OUT / "input").iterdir()):
                folder = OUT / (path.stem + "-" + path.suffix[1:])
                folder.mkdir(exist_ok=True)
                before = inspect(path, folder / "leaf-original")
                doc = desktop.loadComponentFromURL(path.as_uri(), "_blank", 0, (prop("Hidden", True), prop("UpdateDocMode", 3), prop("MacroExecutionMode", 4)))
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
                record = {"file": path.name, "checks": checks}
                results.append(record)
                dump(folder / "checks.json", record)
                print(path.name, sum(c["passed"] for c in checks), "/", len(checks), flush=True)
            dump(OUT / "results.json", {"source_commit": os.environ.get("GITHUB_SHA"), "libreoffice": run("libreoffice", "--version").strip(), "cases": results})
            lines = ["# Leaf ↔ LibreOffice: focused one-off audit", "", "Synthetic fixtures generated through Leaf; real independent open/edit/save, then Leaf reopen/save/reopen. A completed workflow is not an all-clear for document compatibility.", "", "| File | Check | Result | Detail |", "|---|---|---|---|"]
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
