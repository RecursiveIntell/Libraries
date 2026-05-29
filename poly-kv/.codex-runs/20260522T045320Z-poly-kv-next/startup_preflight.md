== poly-kv preflight ==
cwd: /home/sikmindz/Coding/Libraries/poly-kv
date_utc: 2026-05-22T04:53:52Z
git_root: /home/sikmindz/Coding/Libraries/poly-kv
git_head: f2d992f4eca6940a1d16a18deb5b5a44b32bd7c0
git_status:
 D .codex/hooks.json
 D .codex/hooks/post_tool_use_receipt.py
 D .codex/hooks/pre_tool_use_policy.py
 D .codex/hooks/stop_final_gate.py
 D .codex/hooks/user_prompt_submit.py
 M BUNDLE_MANIFEST.json
 M README.md
 M codex/prompts/MASTER_PROMPT.md
 D docs/CODEX_INPUT_BRIEF.md
 M docs/README_DRAFT.md
?? .codex-runs/
?? Cargo.lock
?? Cargo.toml
?? README_BUNDLE.md
?? codex/manual-injections/
?? codex/prompts/PHASE_00_PREFLIGHT_AND_INVENTORY.md
?? codex/prompts/PHASE_01_SHAPE_AND_CONTRACTS.md
?? codex/prompts/PHASE_02_RECEIPTS_AND_ACCOUNTING.md
?? codex/prompts/PHASE_03_CORE_API_COMPAT.md
?? codex/prompts/PHASE_04_PYTHON_SIDECAR_SKELETON.md
?? codex/prompts/PHASE_05_PYTHON_INTEROP_RECEIPTS.md
?? codex/prompts/PHASE_06_BENCHMARK_HARNESS.md
?? codex/prompts/PHASE_07_DOCS_AND_CLAIMS.md
?? codex/prompts/PHASE_08_VALIDATION.md
?? codex/prompts/PHASE_09_FINAL_AUDIT.md
?? crates/
?? docs/BENCHMARK_AND_HARNESS_SPEC.md
?? docs/CURRENT_STATE_AUDIT.md
?? docs/HOSTILE_AUDITOR_HANDOFF_TEMPLATE.md
?? docs/ISSUE_MATRIX.md
?? docs/PY_SIDECAR_SPEC.md
?? docs/ROLLBACK_AND_QUARANTINE_PLAN.md
?? docs/SOURCE_OF_TRUTH_MAP_NEXT.md
?? docs/TARGET_FINAL_STATE.md
?? docs/codex-runs/
?? poly-kv-generic-rust-next-codex-context-20260520.codex-archive.json
?? poly-kv-generic-rust-next-codex-context-20260520.excluded.json
?? poly-kv-generic-rust-next-codex-context-20260520.findings.json
?? poly-kv-generic-rust-next-codex-context-20260520.manifest.json
?? poly-kv-generic-rust-next-codex-context-20260520.report.md
?? poly-kv-generic-rust-next-codex-context-20260520.zip
?? scripts/assert_no_boundary_drift.py
?? scripts/assert_python_sidecar_layout.py
?? scripts/assert_realized_accounting.py
?? scripts/assert_receipt_integrity.py
?? scripts/preflight_next_pass.sh
?? scripts/run_next_validation.sh
?? z.py
rustc: rustc 1.93.0 (254b59607 2026-01-19) (Fedora 1.93.0-1.fc43)
cargo: cargo 1.93.0 (083ac5135 2025-12-15) (Fedora 1.93.0-1.fc43)
python: Python 3.14.2
Cargo manifests:
./Cargo.toml
./crates/poly-kv/Cargo.toml
./crates/quant-codec-core/Cargo.toml
Path dependencies pointing outside workspace:
./crates/poly-kv/Cargo.toml:10:quant-codec-core = { path = "../quant-codec-core" }
Codex/agents files:
./.agents/skills
./.agents/skills/recursiveintell-final-audit
./.agents/skills/recursiveintell-final-audit/SKILL.md
./.agents/skills/recursiveintell-phase-gate
./.agents/skills/recursiveintell-phase-gate/SKILL.md
./.agents/skills/recursiveintell-public-claim-boundary
./.agents/skills/recursiveintell-public-claim-boundary/SKILL.md
./.agents/skills/recursiveintell-rust-crate-boundary
./.agents/skills/recursiveintell-rust-crate-boundary/SKILL.md
./.agents/skills/recursiveintell-startup-preflight
./.agents/skills/recursiveintell-startup-preflight/SKILL.md
./AGENTS.md
schema ok: docs/POLY_KV_SCHEMA_PROPOSAL.json
public claim boundary ok
Preflight complete.
