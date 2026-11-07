#!/usr/bin/env python3
"""Compare optimized GUI binaries with matched inputs and fresh profiles.

Use AB/BA order on one runner. Report timing and RSS without noisy thresholds;
assert deterministic work reductions and canonical-document preservation.
"""
import json
import os
from pathlib import Path
import statistics
import subprocess
import sys
import tempfile

SIZES = (120, 600, 1800)
ORDERS = (("before", "after"), ("after", "before"))
LIMITS = (
    "Synthetic ordinary body text, Xvfb/Cairo, fresh profiles, same runner and "
    "WordCraft revision. Inclusive spans overlap; boundary calls include adapter "
    "overhead. No general speed/memory or cold-disk claims."
)


def run_case(binary, paragraphs, repeat, label, output):
    stem = f"{paragraphs}-{repeat}-{label}"
    report = (output / f"{stem}.json").resolve()
    with tempfile.TemporaryDirectory(prefix="leaf-perf-profile-") as profile:
        environment = os.environ | {
            "XDG_CONFIG_HOME": f"{profile}/config",
            "XDG_CACHE_HOME": f"{profile}/cache",
            "XDG_STATE_HOME": f"{profile}/state",
            "LANGUAGE": "en",
            "GSK_RENDERER": "cairo",
            "LEAF_PERF_PARAGRAPHS": str(paragraphs),
            "LEAF_PERF_REPORT": str(report),
        }
        for directory in ("config", "cache", "state"):
            Path(profile, directory).mkdir()
        with (output / f"{stem}.log").open("w") as log:
            subprocess.run(
                ["xvfb-run", "-a", "timeout", "180s", str(binary), "--performance-ui-test"],
                env=environment, stdout=log, stderr=subprocess.STDOUT,
                check=True, timeout=190,
            )
    result = json.loads(report.read_text())
    assert result["canonical_document_unchanged"]
    return result | {"label": label, "repeat": repeat}


def summarize(results):
    rows = []
    for size in SIZES:
        group = [result for result in results if result["paragraphs"] == size]
        assert len({result["pages"] for result in group}) == 1
        for name in (phase["phase"] for phase in group[0]["phases"]):
            row = {"paragraphs": size, "pages": group[0]["pages"], "phase": name}
            for label in ("before", "after"):
                phases = [
                    next(phase for phase in result["phases"] if phase["phase"] == name)
                    for result in group if result["label"] == label
                ]
                row[label] = {
                    "median_ms": statistics.median(phase["wall_ms"] for phase in phases),
                    "counts": phases[0]["counts"],
                    "median_rss_kib": statistics.median(phase["rss_kib"] for phase in phases),
                }
            before, after = row["before"]["counts"], row["after"]["counts"]
            if name == "cursor_right_80":
                assert after["overlay_queue"] <= 160
                assert before["toc_lookup_scans"] > 0
                assert after.get("toc_lookup_scans", 0) == 0
                assert after["ruler_refresh"] < before["ruler_refresh"]
            elif name == "thumbnail_cache_hits_80":
                # The same-engine control already has the older thumbnail optimization.
                assert after["thumbnail_bounds"] <= before["thumbnail_bounds"]
            elif name == "inactive_search_refresh_4":
                assert after.get("search_rows", 0) == 0
            rows.append(row)
    return rows


def main():
    output = Path("performance-results")
    output.mkdir(exist_ok=True)
    binaries = {label: Path(path).resolve() for label, path in zip(("before", "after"), sys.argv[1:])}
    assert len(binaries) == 2, "usage: performance_compare.py BASELINE CANDIDATE"
    for binary in binaries.values():
        binary.chmod(0o755)
    results = [
        run_case(binaries[label], size, repeat, label, output)
        for size in SIZES
        for repeat, order in enumerate(ORDERS)
        for label in order
    ]
    summary = summarize(results)
    document = {
        "schema": 2, "order": "AB/BA", "repeats_per_size": len(ORDERS),
        "baseline_source": os.environ.get("LEAF_PERF_BASELINE_SOURCE"),
        "candidate_source": os.environ.get("LEAF_PERF_CANDIDATE_SOURCE"),
        "summary": summary, "limits": LIMITS,
    }
    (output / "comparison.json").write_text(json.dumps(document, indent=2) + "\n")
    print("LEAF_PERFORMANCE_COMPARISON_OK", flush=True)
    for row in summary:
        if row["phase"] in ("cursor_right_80", "derived_refresh_16", "thumbnail_cache_hits_80"):
            print(json.dumps(row), flush=True)


if __name__ == "__main__":
    main()
