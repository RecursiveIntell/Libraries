#!/usr/bin/env python3
"""Validate proveKV/poly-kv derived-candidate integration boundaries."""

from __future__ import annotations

import argparse
import pathlib
import re
import subprocess
import sys

ALLOWED_DIRECT_COMPRESSION_CRATES = {
    "semantic-memory",
    "quant-governor",
    "scr-runtime-compression",
    "poly-kv",
    "poly-kv-core",
    "poly-kv-python",
    "compressed-scorer",
    "fib-quant",
    "turbo-quant",
    "turbo-quant-semantic-memory-harness",
    "provekv",
    "hnsw-bench",
    "quant-eval",
    "standalone-claim-repo",
}

COMPRESSION_DEP_NAMES = ("poly-kv", "fib-quant", "turbo-quant", "provekv")
FORBIDDEN_DIRECT_PATH_FRAGMENTS = ("/Recall/", "/Recall-Coding/")


def run(cmd: list[str], cwd: pathlib.Path) -> str:
    return subprocess.check_output(cmd, cwd=cwd, text=True, stderr=subprocess.STDOUT)


def package_name(cargo_toml: pathlib.Path) -> str:
    text = cargo_toml.read_text(errors="ignore")
    match = re.search(r"(?m)^\s*name\s*=\s*[\"']([^\"']+)[\"']", text)
    return match.group(1) if match else cargo_toml.parent.name


def check_no_recall_diff(root: pathlib.Path, errors: list[str]) -> None:
    try:
        diff = run(["git", "diff", "--name-only"], root)
    except Exception as exc:  # pragma: no cover
        errors.append(f"could not inspect git diff: {exc}")
        return
    for line in diff.splitlines():
        normalized = f"/{line}"
        if any(fragment in normalized for fragment in FORBIDDEN_DIRECT_PATH_FRAGMENTS):
            errors.append(f"forbidden Recall/Recall-Coding path changed: {line}")


def check_direct_deps(root: pathlib.Path, errors: list[str]) -> None:
    for cargo in root.rglob("Cargo.toml"):
        rel = cargo.relative_to(root)
        if any(part in {"target", ".git"} for part in rel.parts):
            continue
        text = cargo.read_text(errors="ignore")
        name = package_name(cargo)
        if name in ALLOWED_DIRECT_COMPRESSION_CRATES:
            continue
        for dep in COMPRESSION_DEP_NAMES:
            # Match TOML dep keys only, not prose in comments as much as practical.
            if re.search(rf"(?m)^\s*{re.escape(dep)}\s*=", text):
                errors.append(f"crate {name} has forbidden direct compression dep {dep} in {rel}")


def check_exact_rerank(root: pathlib.Path, errors: list[str]) -> None:
    text = (root / "semantic-memory" / "src" / "config.rs").read_text(errors="ignore") if (root / "semantic-memory" / "src" / "config.rs").exists() else ""
    combined = text
    lib = root / "semantic-memory" / "src" / "lib.rs"
    if lib.exists():
        combined += "\n" + lib.read_text(errors="ignore")
    if "ProveKvPoolCandidateOnly" not in combined:
        errors.append("DerivedVectorBackendPolicy::ProveKvPoolCandidateOnly is missing")
    if "exact" not in combined.lower() or "rerank" not in combined.lower():
        errors.append("semantic-memory proveKV pool policy does not visibly require exact rerank")


def check_docs_no_framework_kv_claim(root: pathlib.Path, errors: list[str]) -> None:
    suspicious = re.compile(
        r"provekv[^\n]{0,80}(reduces|saves|compresses)[^\n]{0,80}(provider|framework|inference)[^\n]{0,40}kv",
        re.IGNORECASE,
    )
    for path in list((root / "docs").rglob("*.md")) + list((root / "AiDENs" / "docs").rglob("*.md") if (root / "AiDENs" / "docs").exists() else []):
        text = path.read_text(errors="ignore")
        for line in text.splitlines():
            if suspicious.search(line):
                lowered = line.lower()
                if "no doc" in lowered or "not claim" in lowered or "must not" in lowered:
                    continue
                errors.append(f"possible forbidden framework/provider KV-cache claim in {path.relative_to(root)}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    args = parser.parse_args()
    root = pathlib.Path(args.root).resolve()
    errors: list[str] = []

    check_no_recall_diff(root, errors)
    check_direct_deps(root, errors)
    check_exact_rerank(root, errors)
    check_docs_no_framework_kv_claim(root, errors)

    if errors:
        print("proveKV integration boundary validation FAILED:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("OK: proveKV integration boundaries hold")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
