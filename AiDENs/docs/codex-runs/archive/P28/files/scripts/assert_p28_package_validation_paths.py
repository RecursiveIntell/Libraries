#!/usr/bin/env python3
"""P28 regression check for package manifest path/name validation."""

from pathlib import Path
import json
import os
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parents[1]
ASSERT_PACKAGE_VALIDATION = ROOT / "scripts" / "assert_package_validation.py"


def write_package_fixture(package_dir: Path, manifest_package: str) -> None:
    prefix = "AiDENs-p28-codex-context"
    package_dir.mkdir(parents=True)
    (package_dir / f"{prefix}.zip").write_bytes(b"fixture")
    (package_dir / f"{prefix}.report.md").write_text("# fixture\n", encoding="utf-8")
    (package_dir / f"{prefix}.excluded.json").write_text("[]\n", encoding="utf-8")
    (package_dir / f"{prefix}.findings.json").write_text(
        json.dumps({"error_count": 0, "warning_count": 0}) + "\n",
        encoding="utf-8",
    )
    (package_dir / f"{prefix}.manifest.json").write_text(
        json.dumps(
            {
                "package": manifest_package,
                "sidecars": {"package": manifest_package},
                "archive_sha256_semantics": "zip-byte-sha256-not-canonical-content-hash",
                "content_manifest_sha256": "0" * 64,
                "codex_archive": {"current_run": "P28", "errors": []},
            }
        )
        + "\n",
        encoding="utf-8",
    )


def run_validation(package_dir: Path) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["AIDENS_CURRENT_RUN"] = "P28"
    env["AIDENS_PACKAGE_DIR"] = str(package_dir)
    return subprocess.run(
        [sys.executable, str(ASSERT_PACKAGE_VALIDATION)],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
    )


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="aidens-p28-package-validation-") as tmp:
        bad_dir = Path(tmp) / "bad"
        write_package_fixture(bad_dir, "target/p27/package/AiDENs-p27-codex-context.zip")
        bad = run_validation(bad_dir)
        if bad.returncode == 0:
            print("FAIL: package validation accepted mismatched package path", file=sys.stderr)
            return 2

        good_dir = Path(tmp) / "good"
        write_package_fixture(good_dir, "target/p28/package/AiDENs-p28-codex-context.zip")
        good = run_validation(good_dir)
        if good.returncode != 0:
            print(good.stdout, end="")
            print(good.stderr, end="", file=sys.stderr)
            return good.returncode

    print("PASS: package validation rejects mismatched package paths")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
