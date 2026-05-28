#!/usr/bin/env python3
"""Final assertion for the FibQuant paper-core pass."""
from __future__ import annotations
import argparse, os, re, subprocess, sys
from pathlib import Path

REQUIRED_FILES = [
    "fib-quant/Cargo.toml",
    "fib-quant/src/lib.rs",
    "fib-quant/src/error.rs",
    "fib-quant/src/profile.rs",
    "fib-quant/src/digest.rs",
    "fib-quant/src/rotation.rs",
    "fib-quant/src/spherical_beta.rs",
    "fib-quant/src/beta_inv.rs",
    "fib-quant/src/directions.rs",
    "fib-quant/src/codebook.rs",
    "fib-quant/src/lloyd.rs",
    "fib-quant/src/bitpack.rs",
    "fib-quant/src/codec.rs",
    "fib-quant/src/metrics.rs",
    "fib-quant/src/receipt.rs",
    "fib-quant/tests/profile_digest.rs",
    "fib-quant/tests/spherical_beta_sampler.rs",
    "fib-quant/tests/paper_k2_radius_closed_form.rs",
    "fib-quant/tests/direction_generators.rs",
    "fib-quant/tests/codebook_determinism.rs",
    "fib-quant/tests/lloyd_refinement.rs",
    "fib-quant/tests/bitpack_indices.rs",
    "fib-quant/tests/encode_decode_roundtrip.rs",
    "fib-quant/tests/corruption_rejection.rs",
    "fib-quant/tests/paper_smoke_regression.rs",
    "docs/compression/FIBQUANT_SOURCE_BASIS.md",
    "docs/compression/FIBQUANT_MATH_CONFORMANCE.md",
    "docs/compression/FIBQUANT_BENCHMARK_PLAN.md",
    "docs/compression/FIBQUANT_ROLLBACK_PLAN.md",
]

FORBIDDEN_CHANGE_PREFIXES = [
    "semantic-memory/src/",
    "turbo-quant/src/",
    "AiDENs/crates/aidens-contracts/src/",
]

FORBIDDEN_TEXT = [
    "zero accuracy loss",
    "guaranteed lossless",
    "beats turboquant",
    "production ready",
]

def run(cmd, cwd):
    try:
        return subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    except FileNotFoundError:
        return None

def git_changed(repo: Path):
    proc = run(["git", "diff", "--name-only", "HEAD"], repo)
    if proc is None or proc.returncode != 0:
        proc2 = run(["git", "status", "--porcelain"], repo)
        if proc2 is None or proc2.returncode != 0:
            return []
        paths = []
        for line in proc2.stdout.splitlines():
            if len(line) > 3:
                paths.append(line[3:].strip())
        return paths
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]

def root_default_members_contains_fib(root_toml: str) -> bool:
    m = re.search(r"default-members\s*=\s*\[(.*?)\]", root_toml, re.S)
    if not m:
        return False
    return "fib-quant" in m.group(1)

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    args = ap.parse_args()
    repo = Path(args.repo).resolve()
    errors = []
    warnings = []

    for rel in REQUIRED_FILES:
        if not (repo / rel).exists():
            errors.append(f"missing required file: {rel}")

    root = repo / "Cargo.toml"
    if root.exists():
        text = root.read_text(encoding="utf-8", errors="replace")
        if '"fib-quant"' not in text and "fib-quant" not in text:
            errors.append("root Cargo.toml does not include fib-quant workspace member")
        if root_default_members_contains_fib(text):
            errors.append("fib-quant appears in default-members; expected default-off crate")
    else:
        errors.append("missing root Cargo.toml")

    changed = git_changed(repo)
    for p in changed:
        for prefix in FORBIDDEN_CHANGE_PREFIXES:
            if p.startswith(prefix):
                errors.append(f"forbidden source surface changed: {p}")

    scan_roots = [repo / "fib-quant", repo / "docs/compression"]
    for sr in scan_roots:
        if not sr.exists():
            continue
        for path in sr.rglob("*"):
            if path.is_file() and path.suffix in {".rs", ".md", ".toml"}:
                data = path.read_text(encoding="utf-8", errors="replace").lower()
                for needle in FORBIDDEN_TEXT:
                    if needle in data:
                        errors.append(f"forbidden claim text '{needle}' in {path.relative_to(repo)}")

    # Non-failing cargo availability note.
    cargo = run(["cargo", "--version"], repo)
    if cargo is None:
        warnings.append("cargo not found; cannot run compile/test checks in assertion script")

    print("FibQuant final assertion")
    if changed:
        print("Changed files:")
        for p in changed:
            print(f"  {p}")
    for w in warnings:
        print(f"WARN: {w}")
    for e in errors:
        print(f"ERROR: {e}")
    return 1 if errors else 0

if __name__ == "__main__":
    raise SystemExit(main())
