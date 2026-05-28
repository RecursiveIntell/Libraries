# Codex master prompt — P32-SCR-RUNTIME-SUPERPASS

You are executing a deterministic, auditable completion super-pass for `scr-runtime`.

## Starting posture

Use `/plan` first. Inspect current files before editing. Do not assume the prior P31 run completed. The current uploaded audit found that the package was well-formed but SCR completion was not proven, final P31 artifacts were missing, phase gates were inert/weak, and the evaluator behaved too much like a fixture-signal resolver instead of a proposed-action control evaluator.

## Mission

Complete SCR-P0A as a reference control runtime in one long, gated super-pass.

The output must be a repository state that a hostile auditor can verify with commands and receipts.

## Source hierarchy

1. Current repo files.
2. This super-pass bundle.
3. Existing repo docs/specs.
4. Project doctrine only when it does not contradict current repo evidence.
5. Public/current docs only for Codex mechanics, not SCR semantics.

## Absolute hard rules

- Do not invent external owner-crate integration.
- Do not scan opaque refs for control truth.
- Do not silently widen schemas or parsing.
- Do not update golden fixtures without `docs/P32_POLICY_CHANGE_RECEIPT.md`.
- Do not replace verification with README prose.
- Do not claim completion without command outputs.
- Do not hide unresolved blockers.
- Do not create local substitutes for stack-wide crates without an adapter seam and ambiguity record.
- Stop, quarantine, or report when ownership is unclear.

## Use scoped subagents before major edits

Spawn read-only subagents for:

1. Schema/Rust parity audit.
2. Proposed-action/evaluator semantics audit.
3. Control-pack/hook/phase-gate audit.
4. CLI/fixture/golden audit.
5. External crate boundary/source-owner audit.

Subagents must not edit files. Main agent integrates findings into one issue matrix and implementation sequence.

## Phase sequence

Execute these phases in order. At every boundary, run the phase gate and paste/store a phase report.

1. Phase 00 — Preflight/source truth/run identity.
2. Phase 01 — Control pack and completion-receipt enforcement.
3. Phase 02 — Kernel contract/schema parity.
4. Phase 03 — Proposed-action semantics and typed signal discipline.
5. Phase 04 — Authority/evidence/owner/rollback basis.
6. Phase 05 — Candidate trace, receipts, digests, canonical JSON.
7. Phase 06 — CLI separation, fixture discipline, negative tests.
8. Phase 07 — External crate adapter seams and docs.
9. Phase 08 — Validation, fresh unzip, hostile handoff.
10. Phase 09 — Final self-audit and next-pass minimization.

## Required commands

At minimum, before final claim:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/validate_strict_schemas.py
python3 scripts/scr_superpass_static_gates.py final
python3 scripts/scr_superpass_preflight.py final
bash scripts/scr_superpass_run_all.sh final
bash scripts/run_p31_completion_checks.sh || true
```

If packaging is generated:

```bash
python3 z.py
python3 scripts/verify_archive_manifest_parity.py <zip> <manifest.json>
python3 scripts/assert_required_archive_paths.py <zip>
rm -rf /tmp/scr-runtime-fresh
mkdir -p /tmp/scr-runtime-fresh
unzip -q <zip> -d /tmp/scr-runtime-fresh
cd /tmp/scr-runtime-fresh
bash scripts/scr_superpass_run_all.sh final
```

## Required final report shape

Create:

```text
docs/P32_COMPLETION_REPORT.md
docs/P32_COMMAND_RECEIPTS.md
docs/P32_CHANGED_FILES.md
docs/P32_UNRESOLVED_RISKS.md
docs/P32_HOSTILE_AUDITOR_HANDOFF.md
docs/P32_POLICY_CHANGE_RECEIPT.md
docs/P32_ROLLBACK_PLAN.md
```

Final response must summarize:

1. changed files,
2. commands run,
3. pass/fail/skipped checks with reasons,
4. invariants validated,
5. unresolved risks,
6. rollback path,
7. exact next pass, if any.
