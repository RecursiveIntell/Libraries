#!/usr/bin/env python3
"""P30 static guard for hostile-audit regression patterns."""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

HARD_PATTERNS = [
    (
        "PARSER_DROP_FILTER_MAP",
        r"filter_map\s*\(\s*\|\s*call",
        ["crates/aidens-runner/src/provider_tool.rs"],
    ),
    (
        "TOOL_RESULT_EMPTY_ON_SERIALIZE",
        r"to_string\s*\(\s*&\s*request\.tool_results\s*\)\.unwrap_or_default\s*\(",
        ["crates/aidens-runner/src/provider_tool.rs"],
    ),
    (
        "EXECUTABLE_PERMISSIVE_REPAIR",
        r"parse_json_boundary\s*\([^\n]+permissive_degraded_repair\s*\(",
        ["crates/aidens-runner/src/provider_tool.rs"],
    ),
    (
        "PATCH_READ_EMPTY_ON_ERROR",
        r"std::fs::read_to_string\s*\(\s*&\s*path\s*\)\.unwrap_or_default\s*\(",
        ["crates/aidens-tool-kit/src/lib.rs"],
    ),
    (
        "ROLLBACK_ERROR_IGNORED",
        r"let\s+_\s*=\s*write_file_atomically",
        ["crates/aidens-tool-kit/src/lib.rs"],
    ),
    (
        "PROCESS_LOCAL_ARTIFACT_COUNTER",
        r"static\s+GENERATED_ARTIFACT_COUNTER",
        ["crates/aidens-contracts/src/lib.rs"],
    ),
    (
        "PUBLIC_GENERATED_ARTIFACT_ID",
        r"pub\s+fn\s+generated_artifact_id\s*\(",
        ["crates/aidens-contracts/src/lib.rs"],
    ),
    (
        "CONSTANT_TOOL_EXPOSURE_ID",
        r"ArtifactId::new\s*\(\s*\"tool-exposure\"\s*\)",
        ["crates/aidens-tool-kit/src/lib.rs"],
    ),
    (
        "ADVISORY_MARKED_SUCCEEDED",
        r"VerificationAttemptState::Succeeded",
        ["crates/aidens-runner/src/lib.rs"],
    ),
    (
        "AMBIENT_PATH_REINJECTED",
        r"\.env\s*\(\s*\"PATH\"\s*,\s*std::env::var\s*\(\s*\"PATH\"\s*\)",
        ["crates/aidens-tool-kit/src/lib.rs"],
    ),
    (
        "DIRECT_CHILD_KILL_ONLY",
        r"child\.kill\s*\(\s*\)",
        ["crates/aidens-tool-kit/src/lib.rs"],
    ),
    (
        "STALE_SOURCE_BASIS_20260426",
        r"libraries-source-clean-20260426\.zip",
        ["crates/aidens-cli/src/package.rs"],
    ),
]

BROAD_PATTERNS = [
    ("UNWRAP", r"\.unwrap\s*\("),
    ("EXPECT", r"\.expect\s*\("),
    ("PANIC", r"panic!\s*\("),
    ("TODO", r"todo!\s*\("),
    ("UNIMPLEMENTED", r"unimplemented!\s*\("),
    ("DYNAMIC_JSON_VALUE", r"serde_json::Value"),
    ("JSON_MACRO", r"json!\s*\("),
    ("LINT_ALLOW", r"#\s*\[\s*allow\s*\("),
    ("RANDOM_UUID", r"Uuid::new_v4\s*\("),
]

SKIP_DIRS = {"target", ".git", "docs/codex-runs/archive", "input_evidence"}


def iter_rs(repo: pathlib.Path):
    for path in repo.rglob("*.rs"):
        rel = path.relative_to(repo).as_posix()
        if any(part in SKIP_DIRS for part in rel.split("/")):
            continue
        yield path, rel


def check_hard(repo: pathlib.Path):
    findings = []
    for name, pattern, files in HARD_PATTERNS:
        rx = re.compile(pattern)
        for file_name in files:
            path = repo / file_name
            if not path.exists():
                continue
            text = path.read_text(errors="replace")
            for match in rx.finditer(text):
                line = text.count("\n", 0, match.start()) + 1
                findings.append(
                    {
                        "level": "hard",
                        "name": name,
                        "path": file_name,
                        "line": line,
                        "match": match.group(0)[:160],
                    }
                )
    return findings


def check_broad(repo: pathlib.Path, fail_broad: bool):
    findings = []
    for path, rel in iter_rs(repo):
        text = path.read_text(errors="replace")
        for name, pattern in BROAD_PATTERNS:
            rx = re.compile(pattern)
            for match in rx.finditer(text):
                line = text.count("\n", 0, match.start()) + 1
                findings.append(
                    {
                        "level": "broad" if fail_broad else "warn",
                        "name": name,
                        "path": rel,
                        "line": line,
                        "match": match.group(0)[:160],
                    }
                )
    return findings


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=".")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--fail-broad", action="store_true")
    args = parser.parse_args()

    repo = pathlib.Path(args.repo).resolve()
    findings = check_hard(repo) + check_broad(repo, args.fail_broad)
    if args.json:
        print(json.dumps({"repo": str(repo), "findings": findings}, indent=2))
    else:
        for finding in findings:
            print(
                f"{finding['level'].upper()} {finding['name']} "
                f"{finding['path']}:{finding['line']} {finding['match']}"
            )
        hard = sum(1 for finding in findings if finding["level"] == "hard")
        print(f"findings={len(findings)} hard={hard}")

    hard = sum(1 for finding in findings if finding["level"] == "hard")
    broad_fail = sum(1 for finding in findings if finding["level"] == "broad")
    if hard or broad_fail:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
