#!/usr/bin/env python3
"""Check local links in the small, authoritative project/onboarding documents."""
from pathlib import Path
import re
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
DOCUMENTS = (
    "README.md", "AGENTS.md", "PROJECT_STATUS.md", "ROADMAP.md",
    "CONTRIBUTING.md", "UPDATING_WORDCRAFT.md", "docs/ARCHITECTURE.md",
    "docs/CONTINUING.md", "docs/DECISIONS.md", "docs/RELEASE_CHECKLIST.md",
    "docs/history/README.md",
)


def main():
    errors = []
    for name in DOCUMENTS:
        document = ROOT / name
        if not document.is_file():
            errors.append(f"Missing project entry point: {name}")
            continue
        for target in re.findall(r"!?\[[^\]]*\]\(([^)]+)\)", document.read_text()):
            target = target.strip().strip("<>")
            parsed = urlsplit(target)
            if parsed.scheme or parsed.netloc or not parsed.path:
                continue
            path = (document.parent / unquote(parsed.path)).resolve()
            if not path.is_relative_to(ROOT) or not path.exists():
                errors.append(f"{name}: broken local link {target}")
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"PROJECT_DOC_LINKS_OK documents={len(DOCUMENTS)}")


if __name__ == "__main__":
    main()
