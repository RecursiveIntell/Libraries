# P29 Codex Super Pass Prompt

You are working in AiDENs.

Execute **P29 — AiDENs Evidence Repair + v11A Local Release Candidate + v11B Executable Seed**.

## Goal

Produce a trustworthy P29 package that repairs P28 evidence/package failures, reaches v11A local release-candidate status for the declared supported-local agent path, and seeds v11B executable graph/region/subtraction surfaces without overclaiming.

## Start order

1. Read `P29_OPERATOR_PASTE_FIRST.md`.
2. Read `P29_MASTER_PACKET.md`.
3. Read `P29_PHASE_PLAN.md`.
4. Read `P29_ACCEPTANCE_GATES.md`.
5. Read `P29_CLAUDE_AUDIT_ABSORPTION.md`.
6. Read `matrices/P29_MASTER_ISSUE_MATRIX.csv`.
7. Execute phases in order.

## Required phase reports

Write each:

```text
handoffs/p29/PHASE_00_REPORT.md
...
handoffs/p29/PHASE_21_REPORT.md
```

## Manual gates

At the required points, stop and request the operator injection from `P29_MANUAL_PHASE_INJECTIONS.md`.

## Final outputs

Required:

```text
P29_STATUS_EVIDENCE_MANIFEST.json
docs/p29/P29_FINAL_AUDIT_REPORT.md
docs/p29/P29_KNOWN_LIMITATIONS_REGISTER.md
docs/p29/P29_SUPPORT_TRACEABILITY.md
handoffs/p29/FINAL_AUDITOR_HANDOFF.md
target/p29/package/AiDENs-p29-codex-context.zip
target/p29/package/AiDENs-p29-codex-context.report.md
target/p29/package/AiDENs-p29-codex-context.manifest.json
target/p29/package/AiDENs-p29-codex-context.findings.json
target/p29/package/AiDENs-p29-codex-context.excluded.json
```

## Final command bar

Run:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
bash scripts/p29_verify.sh
python3 scripts/assert_p29_package_self_replay.py --package target/p29/package/AiDENs-p29-codex-context.zip
```

Do not call the pass complete unless these commands pass or failures are explicitly classified as blocked with no release claim.
