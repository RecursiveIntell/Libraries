#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path.cwd()
AUDIT_DIR = ROOT / "target" / "p26" / "audit"
MANIFEST_PATH = ROOT / "P26_STATUS_EVIDENCE_MANIFEST.json"

COMMANDS = [
    ("assert_phase_gate_integrity", "python3 scripts/assert_phase_gate_integrity.py"),
    ("assert_current_run_truth", "python3 scripts/assert_current_run_truth.py"),
    ("assert_support_claims", "python3 scripts/assert_support_claims.py"),
    ("assert_p26_agent_spec_contract", "python3 scripts/assert_p26_agent_spec_contract.py"),
    ("assert_p26_support_truth", "python3 scripts/assert_p26_support_truth.py"),
    ("agent_validate", "cargo run -q -p aidens-cli -- agent validate --spec examples/agents/local-coding-agent/agent.json"),
    ("agent_doctor", "cargo run -q -p aidens-cli -- agent doctor --spec examples/agents/local-coding-agent/agent.json"),
    ("agent_run", "cargo run -q -p aidens-cli -- agent run --spec examples/agents/local-coding-agent/agent.json --task examples/agents/local-coding-agent/task.md --sandbox-root examples/agents/local-coding-agent/sandbox --out target/p26/verifier/local-coding-agent"),
    ("agent_inspect", "cargo run -q -p aidens-cli -- agent inspect --run target/p26/verifier/local-coding-agent"),
    ("assert_p26_run_bundle_evidence", "python3 scripts/assert_p26_run_bundle_evidence.py"),
    ("assert_plan_act_verify_receipts", "python3 scripts/assert_p26_plan_act_verify_receipts.py"),
    ("memory_grounded_agent_run", "cargo run -q -p aidens-cli -- agent run --spec examples/agents/memory-grounded-agent/agent.json --task examples/agents/memory-grounded-agent/task.md --sandbox-root examples/agents/memory-grounded-agent/sandbox --out target/p26/verifier/memory-grounded-agent"),
    ("assert_memory_grounded_agent_lane", "python3 scripts/assert_p26_memory_grounded_agent_lane.py"),
    ("assert_coding_agent_v1_lane", "python3 scripts/assert_p26_coding_agent_v1_lane.py"),
    ("assert_abstention_repair_cases", "python3 scripts/assert_p26_abstention_repair_cases.py"),
    ("assert_run_bundle_v3_replay", "python3 scripts/assert_p26_run_bundle_v3_replay.py"),
    ("assert_no_shadow_truth", "bash scripts/assert_no_shadow_truth.sh"),
    ("assert_no_local_substitute_dependencies", "bash scripts/assert_no_local_substitute_dependencies.sh"),
]


def run(label: str, command: str, idx: int) -> dict:
    AUDIT_DIR.mkdir(parents=True, exist_ok=True)
    log_path = AUDIT_DIR / f"p26_verify_{idx:02d}_{label}.txt"
    with log_path.open("w", encoding="utf-8") as fp:
        fp.write(f"$ {command}\n")
        fp.flush()
        result = subprocess.run(
            command,
            cwd=ROOT,
            shell=True,
            text=True,
            stdout=fp,
            stderr=subprocess.STDOUT,
            executable="/bin/bash",
        )
    return {
        "label": label,
        "command": command,
        "status": "pass" if result.returncode == 0 else "fail",
        "return_code": result.returncode,
        "log": log_path.as_posix(),
    }


def main() -> int:
    AUDIT_DIR.mkdir(parents=True, exist_ok=True)
    records = [run(label, command, idx) for idx, (label, command) in enumerate(COMMANDS, 1)]
    failed = [record for record in records if record["status"] != "pass"]
    manifest = {
        "schema": "P26StatusEvidenceManifestV1",
        "run_id": "P26",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "audit_dir": AUDIT_DIR.as_posix(),
        "validation_results": records,
        "changed_files_basis": "see handoffs/p26 phase reports and git status",
        "support_claims": [
            "supported-local AgentSpecV1 CLI flow",
            "AiDENsRunBundleV3 operator evidence",
            "memory-grounded canonical-seam evidence remains delegated",
            "abstention and repair display evidence is explicit",
            "cloud/autonomy/V10 runtime geometry deferred"
        ],
        "invariants": {
            "consumer_only": True,
            "no_cloud_runtime": True,
            "no_broad_autonomy": True,
            "no_v10_runtime_geometry": True
        },
        "unresolved_risks": [
            "Final full workspace fmt/check/test/clippy/doc evidence is recorded in target/p26/audit/phase09_command_log_20260504T200000Z.json.",
            "Package validation and package self-replay evidence are recorded in target/p26/package and Phase 09 handoff reports.",
            "This pass remains not production-cloud-ready and does not implement broad autonomy or V10 runtime geometry."
        ],
        "failed_checks": failed,
    }
    MANIFEST_PATH.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": "pass" if not failed else "fail", "manifest": MANIFEST_PATH.as_posix(), "failed": len(failed)}, indent=2))
    return 0 if not failed else 1


if __name__ == "__main__":
    raise SystemExit(main())
