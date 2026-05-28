# P20 Implementation Playbook

## Phase 0 — Install and baseline

```bash
bash install_p20_overlay.sh /home/sikmindz/Coding/Libraries/AiDENs
cd /home/sikmindz/Coding/Libraries/AiDENs
python3 scripts/p20_scan_aidens.py --root . --out target/p20-scan || true
```

Then run the full verification set. Fix compile failures first; do not edit docs to dodge code failures.

## Phase 1 — Truthful docs

Patch docs in this order:

1. `README.md`
2. `STATUS.md`
3. `SOURCE_BASIS.md`
4. `MASTER_ISSUE_MATRIX.md`
5. `NEXT_CODEX_TASK_MATRIX.md`
6. `docs/CURRENT_AIDENS_AUDIT.md`
7. `docs/AUDITOR_HANDOFF.md`

Use exact readiness labels from `P20_FINISHLINE_SCOPE.md`.

## Phase 2 — Contract ownership

Run:

```bash
python3 scripts/p20_scan_aidens.py --root . --out target/p20-scan
```

Use its public type list to create `docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md`.

For ambiguous local types:

- rename to `Aidens...ReportV1`, `...DisplayV1`, `...AdapterReceiptV1`, or `...DraftV1`; or
- replace with canonical re-export; or
- quarantine/defer.

## Phase 3 — Provider truth

Update provider docs/tests so native tool-loop support cannot be inferred from provider name alone. Only executable proof may set native tool calls true.

## Phase 4 — Agency/influence layer

Implement `aidens-agency-kit` or equivalent module.

Recommended crate layout:

```text
crates/aidens-agency-kit/
  Cargo.toml
  src/lib.rs
  tests/agency_policy.rs
```

Minimum exported functions:

```rust
classify_influence(input: &AgencyPolicyInputV1) -> InfluenceClassV1
evaluate_agency_policy(input: AgencyPolicyInputV1) -> AgencyPolicyDecisionV1
build_advice_envelope(input: ..., decision: ...) -> AdviceEnvelopeV1
emit_influence_receipt(...) -> InfluenceReceiptV1
```

Minimum runner seam:

```text
AiDENsRunner::run
  -> build AgencyPolicyInputV1
  -> evaluate agency policy
  -> if blocked, return governed refusal/redirect with receipt
  -> if allowed/degraded, carry decision id into RunReportV1/TurnReportV1
  -> emit influence/advice receipt when high-impact, personalized, repeated, or tool-influenced
```

## Phase 5 — Evals

Load `evals/p20_agency_eval_cases.jsonl` in tests. Each eval must assert expected policy outcome, required receipt families, and forbidden behavior.

## Phase 6 — Reference interpreter closeout

Search for:

```bash
grep -R "deferred.*true\|reference-deferred\|tempor.*deferred" -n crates tests docs
```

For any supported feature, implement the reference behavior. If not implementing, demote docs to `partial` or `deferred`.

## Phase 7 — Final audit

Run:

```bash
bash scripts/p20_verify.sh
bash scripts/p20_generate_audit_bundle.sh
```

Fix until clean or mark P20 failed.
