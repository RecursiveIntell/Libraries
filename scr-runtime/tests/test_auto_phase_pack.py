from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def test_auto_phase_manifest_is_automatic() -> None:
    manifest = json.loads((ROOT / ".codex" / "prompt_manifest.json").read_text(encoding="utf-8"))
    assert manifest["manual_injections_required"] is False
    assert manifest["auto_injections_required"] is True
    assert manifest["master_prompt"] == ".codex/prompts/MASTER_AUTOMATED_COMPLETION.md"
    assert len(manifest["phases"]) >= 6
    for phase in manifest["phases"]:
        assert (ROOT / phase["prompt"]).exists()
        assert (ROOT / phase["auto_injection"]).exists()


def test_auto_phase_runner_dry_run(tmp_path: Path) -> None:
    receipt = tmp_path / "auto_phase_receipt.json"
    result = subprocess.run(
        [
            "python",
            ".codex/tools/auto_phase_runner.py",
            "--dry-run",
            "--phase",
            "phase_00",
            "--receipt",
            str(receipt),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    data = json.loads(receipt.read_text(encoding="utf-8"))
    assert data["manual_injections_required"] is False
    assert data["auto_injections_required"] is True
    assert data["phases"][0]["id"] == "phase_00"
    assert data["phases"][0]["status"] == "ok"
    assert data["phases"][0]["prompt_path"].endswith("phase_00_current_state_and_failure_proof.md")
    assert data["status"] == "ok"


def test_active_pack_validator() -> None:
    result = subprocess.run(
        ["python", "scripts/validate_codex_pack.py"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr


def test_archive_includes_codex_pack_files() -> None:
    archive = Path("/tmp") / "scr-runtime-automated-phase-certification.zip"
    result = subprocess.run(
        [
            "python",
            "scripts/zip_source_certifier.py",
            "--mode",
            "next-codex-context",
            "--no-archive-codex-runs",
            "--no-strict",
            "--output",
            str(archive),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    check = subprocess.run(
        ["python", "scripts/assert_archive_includes_codex.py", str(archive)],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert check.returncode == 0, check.stdout + check.stderr


def test_assert_codex_active_pack_validator() -> None:
    result = subprocess.run(
        ["python", "scripts/assert_codex_active_pack.py"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 0, result.stdout + result.stderr


def test_phase_receipts_are_emitted_for_each_phase(tmp_path: Path) -> None:
    manifest = json.loads((ROOT / ".codex" / "prompt_manifest.json").read_text(encoding="utf-8"))
    for phase in manifest["phases"]:
        receipt = tmp_path / f"{phase['id']}_receipt.json"
        result = subprocess.run(
            [
                "python",
                ".codex/tools/auto_phase_runner.py",
                "--phase",
                phase["id"],
                "--receipt",
                str(receipt),
            ],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        assert result.returncode == 0, result.stdout + result.stderr
        data = json.loads(receipt.read_text(encoding="utf-8"))
        assert data["phases"][0]["id"] == phase["id"]
        assert data["phases"][0]["status"] == "ok"
        assert phase["id"] in data["phases"][0]["prompt_path"]
        assert data["status"] == "ok"
