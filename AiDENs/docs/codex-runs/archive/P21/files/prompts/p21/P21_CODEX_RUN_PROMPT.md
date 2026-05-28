# P21 Codex Run Prompt — Usable Agent Builder Proof + Cross-App Extraction Superpass

You are Codex operating in the AiDENs repository.

You have been given a complete P21 handoff bundle. This run is not a small patch. It is a gated superpass intended to take AiDENs as far as possible without breaking canonical ownership or inventing local truth.

## Absolute mission

Make AiDENs usable as an agent-builder layer:

- package integrity clean;
- build/test/clippy certified;
- `run-test-agent` command works;
- generated `coding-agent` project runs;
- profiles/plans/provider/tool/agency surfaces are operator-usable;
- Recall/Recall-Coding patterns are extracted safely;
- release archive replay is verified;
- stretch work proceeds only after mandatory gates pass.

## Required context

Read before changing code:

1. `AGENTS.md`
2. `docs/p21/P21_SCOPE.md`
3. `docs/p21/P21_ACCEPTANCE_GATES.md`
4. `docs/p21/P21_IMPLEMENTATION_PLAYBOOK.md`
5. `docs/p21/P21_OWNERSHIP_SOURCE_OF_TRUTH_MAP.md`
6. `docs/p21/P21_RECALL_RECALL_CODING_EXTRACTION_PLAN.md`
7. `docs/p21/P21_PROVIDER_TOOL_CAPABILITY_POLICY.md`
8. `docs/p21/P21_AGENCY_GOVERNANCE_V02.md`
9. `audit/P21_SOURCE_BASIS_AND_CODE_FIRST_AUDIT.md`

## Phase protocol

You must execute phases in order.

After each phase:

1. write `handoffs/p21/PHASE_NN_REPORT.md`;
2. list commands run and outputs;
3. list files changed;
4. list invariant checks performed;
5. stop and wait for the operator to paste the next global + phase-specific injection.

Do not continue phases without operator injection.

## Phase order

1. Phase 00 — package/source closure.
2. Phase 01 — build certification.
3. Phase 02 — `run-test-agent` CLI.
4. Phase 03 — generated agent project proof.
5. Phase 04 — profile + plan-kit usability.
6. Phase 05 — provider/tool capability certification.
7. Phase 06 — agency governance v0.2.
8. Phase 07 — Recall/Recall-Coding extraction.
9. Phase 08 — release archive replay.
10. Phase 09 — guarded stretch work.
11. Phase 10 — final hostile audit.

## Do not fake progress

If a gate fails, fix the gate or report the blocker. Do not weaken tests, delete fixtures, delete eval cases, bypass verification, or mark unsupported providers/features as supported.

## Canonical ownership rules

Use canonical libraries for canonical semantics. AiDENs may own orchestration, profiles, product DTOs, CLI commands, receipts routing, and policy application. AiDENs may not own canonical memory/evidence/kernel/verification/repair truth.

## Required final state

At minimum, these should pass:

```bash
bash scripts/p21_verify.sh
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml
cargo run -p aidens-cli -- new coding-agent target/demo-agent
cargo run -p aidens-cli -- run --config target/demo-agent/aidens.toml "read README"
cargo run -p aidens-cli -- doctor --config target/demo-agent/aidens.toml
cargo run -p aidens-cli -- provider-check --config target/demo-agent/aidens.toml
cargo run -p aidens-cli -- tools inspect --config target/demo-agent/aidens.toml
bash scripts/p21_verify_release_archive.sh target/p21/aidens-v0.1-candidate.zip
```

## Final report

Produce `handoffs/p21/FINAL_AUDIT_REPORT.md` and `target/p21/audit/` with all logs. Include unsupported/deferred features clearly.
