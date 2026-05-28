# Phase 4 — CLI, Replay, and Fixture Discipline

## Objective

Make CLI behavior honest, replayable, and auditable.

## Required actions

1. Split generation from verification.
   - `generate-fixtures` may write expected outputs.
   - `verify-fixtures` must compare actual receipts to expected outputs and fail on drift.
   - If keeping old `eval-fixtures`, make it an alias only if semantics are clear.
2. Add `evaluate-audit` command if prompts/specs require it, or update prompts/specs to match actual CLI.
3. Add `explain-receipt --receipt <path>`.
   - Must parse and validate receipt.
   - Must explain chosen action, candidate/action reasoning, policy hash, evaluator ID/hash semantics, authority/evidence basis.
   - Must not re-evaluate input or policy.
4. Add raw-input evaluation if needed.
   - Preserve `raw_input_hash` and `typed_input_hash` separately.
   - Unknown fields or parse failures must not disappear silently.
5. Add policy diff support if practical:
   - `policy-diff old.toml new.toml` or documented deferred issue.
6. Update README, specs, and scripts to match final CLI exactly.
7. Fixture changes must require `docs/POLICY_CHANGE.md` or equivalent when expected outputs change.

## Acceptance gate

```bash
cargo run -p scr-cli -- generate-schemas schemas/generated
cargo run -p scr-cli -- verify-fixtures fixtures/audit/cases fixtures/audit/expected policies/audit_policy_v1.toml
cargo run -p scr-cli -- explain-receipt fixtures/audit/expected/high_hazard_uncertain.json >/tmp/scr-explain.txt
```

If expected fixture path format changes, update the command and docs; do not leave stale examples.
