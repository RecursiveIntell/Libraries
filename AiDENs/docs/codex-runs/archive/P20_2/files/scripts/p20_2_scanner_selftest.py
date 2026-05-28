#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def run_expect_failure(name: str, command: list[str], cwd: Path | None = None) -> str:
    result = subprocess.run(
        command,
        cwd=cwd or ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if result.returncode == 0:
        print(f"{name}: expected failure but command passed", file=sys.stderr)
        print(result.stdout, file=sys.stderr)
        raise SystemExit(2)
    return result.stdout


def require_contains(name: str, output: str, needle: str) -> None:
    if needle not in output:
        print(f"{name}: expected output to contain {needle!r}", file=sys.stderr)
        print(output, file=sys.stderr)
        raise SystemExit(2)


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def selftest_package_integrity(work: Path) -> None:
    root = work / "package"
    root.mkdir()
    write(root / "MANIFEST.txt", "missing.txt\n")
    write(root / "MANIFEST.json", '{"file_count": 1, "files": ["missing.txt"]}\n')
    output = run_expect_failure(
        "package-integrity",
        [sys.executable, str(ROOT / "scripts/p20_2_scan_package_integrity.py"), str(root)],
    )
    require_contains("package-integrity", output, "missing_required")
    require_contains("package-integrity", output, "manifest_missing")
    require_contains("package-integrity", output, "missing.txt")


def selftest_testkit_purity(work: Path) -> None:
    root = work / "testkit"
    write(
        root / "crates/aidens-testkit/Cargo.toml",
        """
[package]
name = "aidens-testkit"
version = "0.0.0"
edition = "2021"

[dependencies]
aidens-runner = { path = "../aidens-runner" }
serde = "1"
""",
    )
    write(root / "crates/aidens-testkit/src/lib.rs", "use aidens_runner::AiDENsRunner;\n")
    write(
        root / "crates/aidens-integration-tests/Cargo.toml",
        """
[package]
name = "aidens-integration-tests"
version = "0.0.0"
edition = "2021"
""",
    )
    output = run_expect_failure(
        "testkit-purity",
        [
            sys.executable,
            str(ROOT / "scripts/p20_2_scan_testkit_purity.py"),
            str(root),
            "--require-integration-crate",
        ],
    )
    require_contains("testkit-purity", output, "forbidden_dependencies")
    require_contains("testkit-purity", output, "forbidden_source_imports")
    require_contains("testkit-purity", output, "aidens-runner")


def selftest_provider_overclaim(work: Path) -> None:
    root = work / "provider"
    write(root / "README.md", "OpenAI native tool calling is supported and ready.\n")
    output = run_expect_failure(
        "provider-overclaim",
        [
            sys.executable,
            str(ROOT / "scripts/p20_scan_aidens.py"),
            "--root",
            str(root),
            "--out",
            str(root / "scan"),
            "--require-phase-reports-through",
            "0",
            "--aidens-overlay-only",
            "--fail-on-blocking",
        ],
    )
    require_contains("provider-overclaim", output, "Blocking findings:")


def selftest_shadow_ownership(work: Path) -> None:
    root = work / "shadow"
    write(root / "crates/aidens-contracts/src/lib.rs", "pub struct EvidenceRecordV1;\n")
    output = run_expect_failure(
        "shadow-ownership",
        ["bash", str(ROOT / "scripts/assert_no_shadow_truth.sh"), str(root)],
    )
    require_contains("shadow-ownership", output, "SHADOW_SEMANTICS")


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="aidens-p20-2-scanner-selftest-") as temp:
        work = Path(temp)
        selftest_package_integrity(work)
        selftest_testkit_purity(work)
        selftest_provider_overclaim(work)
        selftest_shadow_ownership(work)
    print("P20.2 scanner self-test OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
