# Leaf ↔ LibreOffice exchange — November 1, 2026

One-off audit authorized by the user. The files are synthetic and generated
automatically; no private documents were requested and the full 24-file corpus
was not repeated. This check remains optional.

## Method

`examples/interop_audit.rs` creates four files through WordCraft's public APIs
and Leaf's real adapters: DOCX/ODT, with one section and with three. They
include a different first page, odd/even pages, independent and inherited
headers, PAGE/NUMPAGES, numbering restart at VII, header distance 21pt/footer
distance 27pt, body margins of 72pt, British English, Spanish and a fragment
marked "do not proof". All pages are traversed, not only the first.

`tools/interop_audit.py` starts Writer 24.2.7 on Ubuntu 24.04 through UNO, with
its own temporary profile and macros disabled. It opens each file, replaces a
text marker in each section, saves in the same format and exports a PDF. Leaf
opens that result, saves it and reopens it. The states of both applications,
the files, PDFs, extracted text and an internal screenshot are kept.

The additional check converts the DOCX files to ODT through Writer and removes
only the Leaf namespace attributes before importing them. It is also done with
the ODT files saved by Writer. This distinguishes standard geometry from
recovery through Leaf's own metadata. The complete run is not interpreted as
all criteria having passed: each difference is stated explicitly in
`results.json` and `REPORT.md`.

## Findings and fix

In the previous version, an ODT header set 21pt from the edge appeared in
Writer at 72pt; the footer at 27pt appeared at 72pt. Writer kept the private
distance attributes and Leaf recovered their values on reopening, hiding the
visible difference. With those attributes removed, Leaf also did not correctly
interpret the distances and margins of a standalone ODT created by Writer.

The fourth isolated formats patch writes standard ODF reservations and dynamic
spacing and converts the measurements on import. The header/footer minimum
includes the spacing to the body, as in files exported by Writer. Only
reservations for the active parts of the master page are applied. Standard
geometry and common numbering formats take precedence over stale private
values. The engine and renderer revisions do not change.

The tests in `tests/odt_geometry.rs` use three small files included in the
repository: standard, disabled parts and deliberately stale metadata. They
also check preservation of the live document and nominal margins that overlap
the header/footer areas. The four patches applied on top of the pinned
WordCraft reproduce the vendored `src` code exactly.

## Result of the corrected rerun

Code tested: `125b6ab96db54169e9f652517d2e255039a66e36`.
[Complete run](https://github.com/frandavid100/leaf/actions/runs/41299085581),
artifact `13080537464`. Downloaded independently; SHA256 of the ZIP:
`5578d93f0088e1407dd495d5afc4bf41a338fb3cdd8f440624a2e8c1c385740c`.

| Check | DOCX, one section | DOCX, three sections | ODT, one section | ODT, three sections |
|---|---|---|---|---|
| Text after edit/save/reopen | Preserved | Preserved | Preserved | Preserved |
| Header variants and automatic fields | Preserved | Preserved | Preserved | Preserved |
| English and Spanish recognized | Yes | Yes | Yes | Yes |
| "Do not proof" fragment | Writer removes it | Writer removes it | Preserved | Preserved |
| Links between sections | Not applicable | Writer creates copies | Not applicable | Writer creates copies |
| Distances/margins and ODT without Leaf attributes | Correct | Correct | Correct | Correct |

The 108 criteria produce 100 correct results and eight warnings. The warnings
correspond to the proofing exclusion in DOCX and to the links in the
multi-section samples; they have not been omitted from the report to obtain an
overall result without differences. The corrected internal Writer screenshot
has been reviewed.

Previous uncorrected reference: code
`f1da96ba6a5c39f5a54b23aca36fd5a714a6193c`,
[run](https://github.com/frandavid100/leaf/actions/runs/41298485536),
artifact `13080586471`, SHA256
`6184493bdf54fbd31d7d54a181e777b2e8feed44293b0e40f18ae792d216c85e`.
The number of criteria grew afterward to measure the links explicitly; do not
compare aggregate percentages of both passes as an overall improvement.

The fix ships in [alpha.11](https://github.com/frandavid100/leaf/releases/tag/v0.1.0-alpha.11). 91 tests, the optimized GTK build, interface regressions and checks of the Flatpak installed/updated from alpha.10 passed. The hashes of the downloaded artifact and of all published assets match.

## Confirmed limits

- Writer 24.2.7 removes `w:noProof` when saving the test DOCX files. The input
  contains that standard element; the output lacks it. English and Spanish are
  preserved; the fragment is no longer excluded from proofing. In ODT the
  exclusion is preserved through the standard language `zxx`.
- Writer materializes the linked parts of sections into independent copies, in
  both DOCX and ODT. The texts remain present and the layout is checked
  separately; editing one copy no longer propagates the change to the previous
  one. Links are not inferred from identical texts, because they can also be
  intentional independent copies.
- Not all ODF geometry is implemented: borders, fills, fixed height,
  non-dynamic spacing or continuous sections may require additional work. The
  distances of inactive parts, exotic formats and overlapping nominal margins
  keep the extensions needed for their own concepts.
- This sample contains no tables or images; the existing tests cover those
  elements. It does not replace a representative corpus of real documents,
  Microsoft Word, manual sessions on GNOME OS, Orca or a physical printer.

## Reproducing the check

On a machine with Rust, Writer, `python3-uno` and `poppler-utils`:

```sh
cargo build --release --locked --example interop_audit
/usr/bin/python3 tools/interop_audit.py
```

`interop-audit.yml` can also be run manually in GitHub Actions. The
`[interop-audit]` marker allows a deliberate run when modifying the worker or
the converter; `[interop-audit-only]` avoids repeating the GTK tests when only
the diagnosis is adjusted. Normal releases do not run it.
The output files are saved in `interop-results/`; the summarized evidence and
its identifiers are collected in `INTEROPERABILITY_VALIDATION.json`.
