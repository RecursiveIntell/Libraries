#!/usr/bin/env python3
"""P30 static guard for hostile-audit regression patterns.

This is intentionally conservative. It is not a replacement for tests; it is a tripwire.
"""
from __future__ import annotations
import argparse, pathlib, re, sys, json

HARD_PATTERNS = [
    ("PARSER_DROP_FILTER_MAP", r"filter_map\s*\(\s*\|\s*call", ["crates/aidens-runner/src/provider_tool.rs"]),
    ("TOOL_RESULT_EMPTY_ON_SERIALIZE", r"to_string\s*\(\s*&\s*request\.tool_results\s*\)\.unwrap_or_default\s*\(", ["crates/aidens-runner/src/provider_tool.rs"]),
    ("EXECUTABLE_PERMISSIVE_REPAIR", r"parse_json_boundary\s*\([^\n]+permissive_degraded_repair\s*\(", ["crates/aidens-runner/src/provider_tool.rs"]),
    ("PATCH_READ_EMPTY_ON_ERROR", r"std::fs::read_to_string\s*\(\s*&\s*path\s*\)\.unwrap_or_default\s*\(", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("ROLLBACK_ERROR_IGNORED", r"let\s+_\s*=\s*write_file_atomically", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("PROCESS_LOCAL_ARTIFACT_COUNTER", r"static\s+GENERATED_ARTIFACT_COUNTER", ["crates/aidens-contracts/src/lib.rs"]),
    ("PUBLIC_GENERATED_ARTIFACT_ID", r"pub\s+fn\s+generated_artifact_id\s*\(", ["crates/aidens-contracts/src/lib.rs"]),
    ("CONSTANT_TOOL_EXPOSURE_ID", r"ArtifactId::new\s*\(\s*\"tool-exposure\"\s*\)", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("ADVISORY_MARKED_SUCCEEDED", r"VerificationAttemptState::Succeeded", ["crates/aidens-runner/src/lib.rs"]),
    ("AMBIENT_PATH_REINJECTED", r"\.env\s*\(\s*\"PATH\"\s*,\s*std::env::var\s*\(\s*\"PATH\"\s*\)", ["crates/aidens-tool-kit/src/lib.rs"]),
    ("STALE_SOURCE_BASIS_20260426", r"libraries-source-clean-20260426\.zip", ["crates/aidens-cli/src/package.rs"]),
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
    for p in repo.rglob("*.rs"):
        rel = p.relative_to(repo).as_posix()
        if any(part in SKIP_DIRS for part in rel.split("/")):
            continue
        yield p, rel


def check_hard(repo: pathlib.Path):
    findings=[]
    for name, pat, files in HARD_PATTERNS:
        rx=re.compile(pat)
        for file in files:
            p=repo/file
            if not p.exists():
                continue
            text=p.read_text(errors="replace")
            for m in rx.finditer(text):
                line=text.count("\n",0,m.start())+1
                findings.append({"level":"hard","name":name,"path":file,"line":line,"match":m.group(0)[:160]})
    return findings


def check_broad(repo: pathlib.Path, fail_broad: bool):
    findings=[]
    for p, rel in iter_rs(repo):
        text=p.read_text(errors="replace")
        for name, pat in BROAD_PATTERNS:
            rx=re.compile(pat)
            for m in rx.finditer(text):
                line=text.count("\n",0,m.start())+1
                findings.append({"level":"broad" if fail_broad else "warn","name":name,"path":rel,"line":line,"match":m.group(0)[:160]})
    return findings


def main():
    ap=argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--fail-broad", action="store_true")
    args=ap.parse_args()
    repo=pathlib.Path(args.repo).resolve()
    findings=check_hard(repo)+check_broad(repo,args.fail_broad)
    if args.json:
        print(json.dumps({"repo":str(repo),"findings":findings}, indent=2))
    else:
        for f in findings:
            print(f"{f['level'].upper()} {f['name']} {f['path']}:{f['line']} {f['match']}")
        print(f"findings={len(findings)} hard={sum(1 for f in findings if f['level']=='hard')}")
    hard=sum(1 for f in findings if f['level']=='hard')
    broad_fail=sum(1 for f in findings if f['level']=='broad')
    if hard or broad_fail:
        sys.exit(1)

if __name__ == "__main__":
    main()
