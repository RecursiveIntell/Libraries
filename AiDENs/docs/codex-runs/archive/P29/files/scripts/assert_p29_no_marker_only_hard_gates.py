#!/usr/bin/env python3
"""Fail if p29_verify delegates a hard gate to a marker-only assertion script."""

from pathlib import Path
import argparse
import re
import sys
import tempfile


MARKER_ONLY_PATTERNS = [
    re.compile(r"markers? present", re.IGNORECASE),
    re.compile(r"Missing .*markers?", re.IGNORECASE),
]
MARKER_ONLY_SUBSTRINGS = [
    "src = \"\\n\".join(p.read_text",
    "src = '\\n'.join(p.read_text",
]


def verifier_python_scripts(verify_script: Path) -> list[Path]:
    scripts: list[Path] = []
    for line in verify_script.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        parts = stripped.split()
        if len(parts) >= 2 and parts[0].endswith("python3") and parts[1].startswith("scripts/"):
            scripts.append(verify_script.parent.parent / parts[1])
    return scripts


def marker_only_reason(path: Path) -> str | None:
    if path.name == "assert_p29_no_marker_only_hard_gates.py":
        return None
    text = path.read_text(encoding="utf-8", errors="ignore")
    for pattern in MARKER_ONLY_PATTERNS:
        if pattern.search(text):
            return f"{path}: marker-only pattern `{pattern.pattern}`"
    for needle in MARKER_ONLY_SUBSTRINGS:
        if needle in text:
            return f"{path}: marker-only source scan `{needle}`"
    return None


def run_check(verify_script: Path) -> int:
    failures = []
    for script in verifier_python_scripts(verify_script):
        if not script.exists():
            failures.append(f"missing verifier script: {script}")
            continue
        reason = marker_only_reason(script)
        if reason:
            failures.append(reason)
    if failures:
        print("P29 marker-only hard gate check FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("P29 marker-only hard gate check OK")
    return 0


def self_test() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        scripts = root / "scripts"
        scripts.mkdir()
        marker = scripts / "assert_p29_fake_marker.py"
        marker.write_text(
            "from pathlib import Path\n"
            "src = '\\n'.join(p.read_text() for p in Path('crates').glob('**/*.rs'))\n"
            "print('fake markers present')\n",
            encoding="utf-8",
        )
        verify = scripts / "p29_verify.sh"
        verify.write_text("python3 scripts/assert_p29_fake_marker.py\n", encoding="utf-8")
        failed = run_check(verify) != 0
        if not failed:
            print("self-test did not reject marker-only verifier script", file=sys.stderr)
            return 1
    print("P29 marker-only hard gate self-test OK")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--verify-script", default="scripts/p29_verify.sh")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    return run_check(Path(args.verify_script))


if __name__ == "__main__":
    raise SystemExit(main())
