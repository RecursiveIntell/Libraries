# Phase 01 — Restore Active Codex Control Pack

Goal: restore/generate the active `.codex/` control pack required by tests and release gates.

Required active files:

```text
.codex/config.toml
.codex/hooks.json
.codex/prompt_manifest.json
.codex/prompts/MASTER_AUTOMATED_COMPLETION.md
.codex/prompts/phase_00_current_state_and_failure_proof.md
.codex/prompts/phase_01_restore_active_codex_pack.md
.codex/prompts/phase_02_auto_phase_runner.md
.codex/prompts/phase_03_packaging_policy.md
.codex/prompts/phase_04_tests_and_release_gates.md
.codex/prompts/phase_05_fresh_unzip_certification.md
.codex/prompts/phase_06_hostile_audit_handoff.md
.codex/auto_injections/phase_00_gate.md
.codex/auto_injections/phase_01_gate.md
.codex/auto_injections/phase_02_gate.md
.codex/auto_injections/phase_03_gate.md
.codex/auto_injections/phase_04_gate.md
.codex/auto_injections/phase_05_gate.md
.codex/auto_injections/phase_06_gate.md
.codex/rules/safety.rules
.codex/tools/auto_phase_runner.py
.codex/tools/phase_prompt_builder.py
.codex/tools/inspect_codex_setup.py
```

Also ensure `.agents/skills/*/SKILL.md` exists and `.codex/skills/*/SKILL.md` mirrors those skills if validators expect it.

Rules:
- Do not put old manual injection files in the active workflow.
- If old `.codex/manual_injections/` exists, archive it under `docs/codex-runs/archive/manual-injections-legacy/` or leave it only if validators ignore it.
- Active workflow must use `.codex/auto_injections/`.

Run:

```bash
python scripts/assert_codex_active_pack.py
python scripts/validate_codex_pack.py
```
