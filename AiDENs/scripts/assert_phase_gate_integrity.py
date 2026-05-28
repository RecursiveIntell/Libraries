#!/usr/bin/env python3
import re
from pathlib import Path

ROOT = Path.cwd()
PHASE_DIR = ROOT / "phase_injections"

STALE_TOKENS = {
    "P22",
    "P23",
    "P24",
    "P25 current",
    "target/p22",
    "target/p23",
    "target/p24",
    "target/p25",
    "handoffs/p22",
    "handoffs/p23",
    "handoffs/p24",
    "docs/p22",
    "docs/p23",
    "docs/p24",
}

REQUIRED_TERMS = [
    "stop",
    "blocking human-in-the-loop gate",
    "do not proceed",
]

EXPECTED_GATES = {
    "P26_GLOBAL_PRE_PHASE_INVARIANT_REVALIDATION.md",
    "P26_GATE_AFTER_PHASE_01_BEFORE_PHASE_02.md",
    "P26_GATE_AFTER_PHASE_03_BEFORE_PHASE_04.md",
    "P26_GATE_AFTER_PHASE_05_BEFORE_PHASE_06.md",
    "P26_GATE_AFTER_PHASE_07_BEFORE_PHASE_08.md",
    "P26_GATE_AFTER_PHASE_09_BEFORE_FINAL.md",
}

STALE_RE = re.compile(r"(P(?:2[0-4]) current|P25 current)|(?:target/p(?:2[0-5]))|(?:handoffs/p(?:2[0-4]))|(?:docs/p(?:2[0-4]))")


def stale_hits(text: str) -> list[str]:
    hits: list[str] = []
    for token in sorted(STALE_TOKENS):
        if token in text:
            hits.append(token)
    for match in STALE_RE.finditer(text):
        token = match.group(0)
        if token not in hits:
            hits.append(token)
    return hits


def missing_required_terms(text_lc: str) -> list[str]:
    return [term for term in REQUIRED_TERMS if term not in text_lc]


def main() -> int:
    if not PHASE_DIR.exists():
        print("ERROR: phase_injections directory missing")
        return 1

    failed = False
    for name in sorted(EXPECTED_GATES):
        path = PHASE_DIR / name
        if not path.exists():
            failed = True
            print(f"ERROR: expected phase gate injection missing: {path}")

    active_paths = [PHASE_DIR / name for name in sorted(EXPECTED_GATES)]
    active_paths.extend(sorted(PHASE_DIR.glob("P26_*.md")))
    seen = set()
    for path in active_paths:
        if path in seen or not path.exists():
            continue
        seen.add(path)
        text = path.read_text(encoding="utf-8", errors="replace")
        text_lc = text.lower()
        stales = stale_hits(text)
        missing = missing_required_terms(text_lc)
        if stales or missing:
            failed = True
            print(f"ERROR: {path}")
            if stales:
                print(f"  stale tokens: {stales}")
            if missing:
                print(f"  missing required gate terms: {missing}")

    if failed:
        return 1
    print("OK: phase gate integrity passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
