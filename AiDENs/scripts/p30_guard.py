#!/usr/bin/env python3
"""P30 static guard for hostile-audit regression patterns.

Coverage is mechanically exercised; this is not containment certification.
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
import tempfile

HARD_PATTERNS = [
    ("PARSER_DROP_FILTER_MAP", r"filter_map\s*\(\s*\|\s*call", ["crates/aidens-runner/src/provider_tool.rs"]),
    ("TOOL_RESULT_EMPTY_ON_SERIALIZE", r"to_string\s*\(\s*&\s*request\.tool_results\s*\)\.unwrap_or_default\s*\(", ["crates/aidens-runner/src/provider_tool.rs"]),
    ("EXECUTABLE_PERMISSIVE_REPAIR", r"parse_json_boundary\([^\n]+permissive_degraded_repair\(", ["crates/aidens-runner/src/provider_tool.rs"]),
    ("PATCH_READ_EMPTY_ON_ERROR", r"std::fs::read_to_string\s*\(\s*&\s*path\s*\)\.unwrap_or_default\s*\(", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("ROLLBACK_ERROR_IGNORED", r"let\s+_\s*=\s*write_file_atomically", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("PROCESS_LOCAL_ARTIFACT_COUNTER", r"static\s+GENERATED_ARTIFACT_COUNTER", ["crates/aidens-contracts/src/lib.rs"]),
    ("PUBLIC_GENERATED_ARTIFACT_ID", r"pub\s+fn\s+generated_artifact_id\s*\(", ["crates/aidens-contracts/src/lib.rs"]),
    ("CONSTANT_TOOL_EXPOSURE_ID", r"ArtifactId::new\s*\(\s*\"tool-exposure\"\s*\)", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("ADVISORY_MARKED_SUCCEEDED", r"VerificationAttemptState::Succeeded", ["crates/aidens-runner/src/lib.rs"]),
    ("AMBIENT_PATH_REINJECTED", r"\.env\s*\(\s*\"PATH\"\s*,\s*std::env::var\s*\(\s*\"PATH\"\s*\)", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("DIRECT_CHILD_KILL_ONLY", r"child\.kill\s*\(\s*\)", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("STALE_SOURCE_BASIS_20260426", r"libraries-source-clean-20260426\.zip", ["crates/aidens-cli/src/package.rs"]),
]
BROAD_PATTERNS = [("UNWRAP", r"\.unwrap\s*\("), ("EXPECT", r"\.expect\s*\("), ("PANIC", r"panic!\s*\("), ("TODO", r"todo!\s*\("), ("UNIMPLEMENTED", r"unimplemented!\s*\("), ("DYNAMIC_JSON_VALUE", r"serde_json::Value"), ("JSON_MACRO", r"json!\s*\("), ("LINT_ALLOW", r"#\s*\[\s*allow\s*\("), ("RANDOM_UUID", r"Uuid::new_v4\s*\(")]
SKIP_DIRS = {"target", ".git", "docs/codex-runs/archive", "input_evidence"}


def discover_roots(repo):
    roots = [repo]
    aidens = repo / "AiDENs"
    if (aidens / "Cargo.toml").is_file():
        roots.append(aidens)
    return roots


def target_path(repo, root, target):
    direct = root / target
    if direct.exists():
        return direct
    nested = repo / "AiDENs" / target
    if root == repo and nested.exists():
        return nested
    return None


def check_hard(repo):
    findings, missing = [], []
    for root in discover_roots(repo):
        for name, pat, files in HARD_PATTERNS:
            for file in files:
                path = target_path(repo, root, file)
                normalized = path.relative_to(repo).as_posix() if path else ((root / file).relative_to(repo).as_posix())
                if path is None:
                    missing.append(normalized)
                    continue
                text = path.read_text(errors="replace")
                for match in re.finditer(pat, text):
                    findings.append({"level": "hard", "name": name, "path": normalized, "line": text.count("\n", 0, match.start()) + 1, "match": match.group(0)[:160]})
    return findings, sorted(set(missing))


def iter_rs(repo):
    for path in repo.rglob("*.rs"):
        rel = path.relative_to(repo).as_posix()
        if any(part in SKIP_DIRS for part in rel.split("/")):
            continue
        yield path, rel


def check_broad(repo, fail_broad):
    findings = []
    for path, rel in iter_rs(repo):
        text = path.read_text(errors="replace")
        for name, pattern in BROAD_PATTERNS:
            for match in re.finditer(pattern, text):
                findings.append({"level": "broad" if fail_broad else "warn", "name": name, "path": rel, "line": text.count("\n", 0, match.start()) + 1, "match": match.group(0)[:160]})
    return findings


def self_test_hard_rules():
    return {name: re.search(pattern, _fixture(pattern)) is not None for name, pattern, _ in HARD_PATTERNS}


def _fixture(pattern):
    samples = {"read_to_string": "std::fs::read_to_string(&path).unwrap_or_default(", "filter_map": "filter_map(|call", "unwrap_or_default": "to_string(&request.tool_results).unwrap_or_default(", "permissive": "parse_json_boundary(x, permissive_degraded_repair("}
    for key, value in samples.items():
        if key in pattern:
            return value
    sample = re.sub(r"\\s[+*]", " ", pattern)
    return sample.replace("\\(", "(").replace("\\", "")


def build_receipt(repo, fail_broad=False):
    hard, missing = check_hard(repo)
    roots = discover_roots(repo)
    coverage = self_test_hard_rules()
    findings = hard + check_broad(repo, fail_broad)
    summary = {
        "hard_findings": sum(finding["level"] == "hard" for finding in findings),
        "broad_findings": sum(finding["level"] == "broad" for finding in findings),
        "warning_findings": sum(finding["level"] == "warn" for finding in findings),
    }
    blockers = []
    if missing:
        blockers.append("missing-configured-targets")
    if not all(coverage.values()):
        blockers.append("hard-rule-self-test-coverage-incomplete")
    if summary["hard_findings"]:
        blockers.append("hard-findings-present")
    if summary["broad_findings"]:
        blockers.append("broad-findings-promoted-by-fail-broad")
    return {
        "repo": str(repo),
        "discovered_roots": [
            "." if root == repo else root.relative_to(repo).as_posix() for root in roots
        ],
        "target_count": len(HARD_PATTERNS) * len(roots),
        "missing_targets": missing,
        "rule_coverage": coverage,
        "summary": summary,
        "release_gate": {
            "status": "blocked" if blockers else "pass",
            "blockers": blockers,
            "warning_policy": "advisory-inventory-not-release-blocking",
            "claim_scope": "mechanical-hostile-pattern-coverage-not-containment-certification",
        },
        "findings": findings,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=".")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--fail-broad", action="store_true")
    args = parser.parse_args()
    receipt = build_receipt(pathlib.Path(args.repo).resolve(), args.fail_broad)
    if args.json:
        print(json.dumps(receipt, indent=2))
    else:
        for finding in receipt["findings"]:
            print(f"{finding['level'].upper()} {finding['name']} {finding['path']}:{finding['line']} {finding['match']}")
        print(f"findings={len(receipt['findings'])} hard={sum(f['level']=='hard' for f in receipt['findings'])}")
    if receipt["missing_targets"] or any(f["level"] in {"hard", "broad"} for f in receipt["findings"]):
        sys.exit(1)

if __name__ == "__main__":
    main()
