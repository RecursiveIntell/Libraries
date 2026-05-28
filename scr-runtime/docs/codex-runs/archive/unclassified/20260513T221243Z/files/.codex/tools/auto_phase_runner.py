#!/usr/bin/env python3
"""Automated phase runner for ClaimLedger Codex completion.

Default mode is safe: dry-run prompt assembly and receipt emission. Use --execute
only when running locally with Codex CLI installed and you intentionally want it
to call `codex exec` per phase.
"""
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / ".codex" / "prompt_manifest.json"


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat()


def load_manifest() -> dict[str, Any]:
    return json.loads(MANIFEST.read_text(encoding="utf-8"))


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def phase_prompt(manifest: dict[str, Any], phase: dict[str, Any], include_master: bool) -> str:
    chunks: list[str] = []
    if include_master:
        chunks.append("# Master Prompt\n\n" + read(manifest["master_prompt"]))
    chunks.append("# Phase Prompt\n\n" + read(phase["prompt"]))
    chunks.append("# Automatic Phase Gate\n\n" + read(phase["auto_injection"]))
    chunks.append(
        "# Required Commands\n\n```json\n"
        + json.dumps(phase.get("required_commands", []), indent=2)
        + "\n```"
    )
    return "\n\n---\n\n".join(chunks) + "\n"


def select_phases(manifest: dict[str, Any], phase: str | None, from_phase: str | None, to_phase: str | None) -> list[dict[str, Any]]:
    phases = manifest["phases"]
    if phase:
        matches = [p for p in phases if p["id"] == phase]
        if not matches:
            raise ValueError(f"phase not found: {phase}")
        return matches
    start = 0
    end = len(phases)
    ids = [p["id"] for p in phases]
    if from_phase:
        try:
            start = ids.index(from_phase)
        except ValueError as exc:
            raise ValueError(f"unknown from-phase: {from_phase}") from exc
    if to_phase:
        try:
            end = ids.index(to_phase) + 1
        except ValueError as exc:
            raise ValueError(f"unknown to-phase: {to_phase}") from exc
        if end <= start:
            raise ValueError(f"to-phase '{to_phase}' comes before from-phase '{from_phase}'")
    return phases[start:end]


def run_required(command: str) -> dict[str, Any]:
    result = subprocess.run(command, shell=True, cwd=ROOT, text=True, capture_output=True)
    return {
        "command": command,
        "returncode": result.returncode,
        "status": "ok" if result.returncode == 0 else "failed",
        "stdout_tail": result.stdout[-4000:],
        "stderr_tail": result.stderr[-4000:],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--phase")
    parser.add_argument("--from-phase")
    parser.add_argument("--to-phase")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--run-required", action="store_true")
    parser.add_argument("--print-prompts", action="store_true")
    parser.add_argument("--no-master", action="store_true")
    parser.add_argument("--receipt", default=".codex/runs/P0-completion/auto_phase_receipt.json")
    parser.add_argument("--codex-bin", default="codex")
    args = parser.parse_args()

    manifest = load_manifest()
    try:
        phases = select_phases(manifest, args.phase, args.from_phase, args.to_phase)
    except ValueError as exc:
        print(f"auto phase runner failed: {exc}")
        return 2

    receipt: dict[str, Any] = {
        "receipt_version": "AutoPhaseRunnerReceiptV1",
        "started_at": utc_now(),
        "manifest": str(MANIFEST.relative_to(ROOT)),
        "manual_injections_required": manifest.get("manual_injections_required"),
        "auto_injections_required": manifest.get("auto_injections_required"),
        "dry_run": args.dry_run,
        "execute": args.execute,
        "status": "ok",
        "phases": [],
        "errors": [],
    }

    codex_path = shutil.which(args.codex_bin)
    if args.execute and not codex_path:
        receipt["errors"].append(f"Codex CLI not found: {args.codex_bin}")
        args.execute = False

    for phase in phases:
        prompt = phase_prompt(manifest, phase, include_master=not args.no_master)
        phase_record: dict[str, Any] = {
            "id": phase["id"],
            "name": phase["name"],
            "prompt": phase["prompt"],
            "auto_injection": phase["auto_injection"],
            "prompt_path": str((ROOT / phase["prompt"]).resolve().as_posix()),
            "required_commands": phase.get("required_commands", []),
            "prompt_bytes": len(prompt.encode("utf-8")),
            "commands": [],
            "status": "ok",
        }
        if args.print_prompts:
            print(f"\n===== {phase['id']} {phase['name']} =====\n")
            print(prompt)
        if args.execute:
            result = subprocess.run([args.codex_bin, "exec", prompt], cwd=ROOT, text=True, capture_output=True)
            phase_record["codex_exec"] = {
                "returncode": result.returncode,
                "status": "ok" if result.returncode == 0 else "failed",
                "stdout_tail": result.stdout[-4000:],
                "stderr_tail": result.stderr[-4000:],
            }
            if result.returncode != 0:
                phase_record["status"] = "failed"
                receipt["errors"].append(
                    f"phase {phase['id']} codex exec failed: returncode={result.returncode}"
                )
                receipt["status"] = "failed"
        if args.run_required:
            failed_commands = 0
            for cmd in phase.get("required_commands", []):
                command_result = run_required(cmd)
                phase_record["commands"].append(command_result)
                if command_result["status"] == "failed":
                    failed_commands += 1
            if failed_commands:
                phase_record["status"] = "failed"
                receipt["errors"].append(
                    f"phase {phase['id']} had {failed_commands} failed required command(s)"
                )
                receipt["status"] = "failed"
        receipt["phases"].append(phase_record)

    if not args.execute and not args.run_required and not args.print_prompts:
        # keep compatibility: dry-run mode still emits structured status only
        pass

    if receipt["errors"]:
        print("auto phase runner completed with errors:")
        for error in receipt["errors"]:
            print(f"- {error}")

    receipt["finished_at"] = utc_now()
    receipt_path = ROOT / args.receipt
    receipt_path.parent.mkdir(parents=True, exist_ok=True)
    receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True), encoding="utf-8")
    print(f"auto-phase receipt: {receipt_path}")
    return 1 if receipt["errors"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
