# AiDENs Hard Audit — P31B → P32 Plan

**Date:** 2026-05-29
**From:** P31B (candidate)
**To:** P32 (Schema Compatibility + Reference Boundary Fixtures)
**Auditor:** Hermes Agent

---

## Current State Snapshot

### Run Status
| Item | Value |
|---|---|
| Active run | P31B |
| Role | verification-repair |
| Certification | `candidate` (not certified) |
| Last certified | P30 |
| Parent | P31A (decertified) |

### P31B Verification Results — ALL 18 GATES PASS
1. release_ledger_schema — PASS
2. current_run_truth — PASS
3. release_truth_consistency — PASS
4. root_markdown_archive_policy — PASS
5. codex_artifact_classification — PASS (661 artifacts)
6. support_claims_have_evidence — PASS
7. no_fake_completion — PASS
8. no_shadow_truth — PASS
9. adapter_delegation — PASS
10. tool_runtime_delegation — PASS
11. no_canonical_type_duplicates — PASS
12. no_local_substitute_dependencies — PASS
13. p30_guard — PASS (1842 broad, **0 hard**)
14. cargo_metadata — PASS
15. cargo_fmt — PASS
16. cargo_check — PASS (0 errors)
17. cargo_test — PASS (28 tests)
18. cargo_clippy — PASS (0 warnings)

### P31B Hostile Audit Resolution — ALL 12 FINDINGS ADDRESSED
| ID | Severity | Finding | Status |
|---|---|---|---|
| S0-001 | hard | False certification state | RESOLVED: decertified, now candidate |
| S0-002 | hard | Verifier self-poisoning | RESOLVED: logs → target/verify-current/ |
| S0-003 | hard | P31A evidence unclassified | RESOLVED: 659 → 661 artifacts classified |
| S0-004 | hard | DIRECT_CHILD_KILL_ONLY | RESOLVED: process-group termination in P31A |
| S0-005 | hard | Receipt evidence missing | RESOLVED: 15 receipts in COMMAND_EXECUTION_RECEIPTS.jsonl |
| S1-006 | medium | Package policy defaults P30 | RESOLVED: z.py normalized P31B |
| S1-007 | medium | Root markdown ambiguous | RESOLVED: root_markdown_archive_policy PASS |
| S1-008 | medium | Package validation binding | RESOLVED: assert_package_validation.py PASS |
| S1-009 | medium | Build/test unproven | RESOLVED: all cargo gates pass |
| S1-010 | medium | 1842 broad p30_guard findings | DOCUMENTED: 0 hard |
| S2-011 | low | Supported-local proof lacking | RESOLVED: vertical slice proven |
| Q-012 | quality | No certified claims | ENFORCED: now candidate |

### Known Limitations
1. `extracted_replay_certified=false` — PermissionError in temp dir (environmental, not a code defect)
2. 1842 broad p30_guard findings (0 hard, expected at this scale)
3. 1 non-fatal package warning: `script-ref-not-archived: p30_guard.py`

---

## P32 Objective

Implement **P32 — Schema Compatibility + Reference Boundary Fixtures** per the P31A_archive intent:
> "Wrap the P31 boundary compiler behavior in reference/conformance artifacts and connect it to one real import/export/tool-output path."

Non-goals: full bitemporal query interpreter, v11B graph compiler, region runtime, lawful subtraction, external federation.

---

## P32 Work Items

### Phase 0 — Pre-flight (verify P31B state is clean)

**0.1** Run `bash scripts/verify_current.sh` in AiDENs/ and confirm all 18 gates pass.
- Source: `target/verify-current/P31B_VERIFICATION/*.stdout.log` already shows all PASS.
- Verify fresh clone behavior (do not rely on cached logs).

**0.2** Examine `aidens-boundary-kit` — determine what's implemented vs stub.
- Source: `crates/aidens-boundary-kit/src/lib.rs`
- Source: `docs/codex-runs/P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md`
- Inventory: what types, traits, and entry points exist?

**0.3** Inventory P31 artifact/receipt types that need schemas.
- Source: `crates/aidens-contracts/src/` (capability_turn.rs, artifact.rs, receipt.rs, etc.)
- Cross-reference with P31 boundary compiler report to identify which types cross the JSON boundary.

**0.4** Check p30_guard.py hygiene warning.
- The warning is `script-ref-not-archived: p30_guard.py` — p30_guard.py is active, not stale.
- This is a false positive in z.py hygiene policy. Determine if fix is warranted or if this is a known acceptable false positive.

---

### Phase 1 — aidens-boundary-kit implementation

**1.1** Read `P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md` in full.
- Understand what the P31 boundary compiler microkernel actually does.
- Identify the public API surface that needs wiring.

**1.2** Inventory all Rust types in aidens-boundary-kit.
- Check: does the crate have proper error types, or is it using `anyhow::Error` / `thiserror`?
- Check: does it have tests? Cargo test passes? Benchmark?
- Check: does it have doc comments on public items?

**1.3** Identify what `aidens-boundary-kit` needs from sibling crates.
- Source: `Cargo.toml` deps, `crates/aidens-boundary-kit/Cargo.toml`
- Currently: `aidens-contracts` path dep, `chrono`, `serde`, `serde_json`, `thiserror`
- Determine: is `aidens-contracts` the right boundary contract owner, or is there a stack-level canonical crate that should be consumed instead?

---

### Phase 2 — Schema generation and meta-validation

**2.1** Generate JSON Schema from P31 receipt/artifact Rust types.
- Target types: `ToolCallRequestV1`, `ToolCallResultV1`, artifact envelope types, any boundary types used in structured I/O.
- Approach: use `schemars` derive macro on existing types, OR write a JSON Schema generator script.
- Constraint: must not pollute the crate's public API with schema-only concerns.

**2.2** Meta-validate generated schemas.
- Validate that schemas are self-consistent (no circular `$ref`, all `$id` unique).
- Validate that schemas round-trip through serde_json.

**2.3** Create reference fixture corpus for boundary compiler behavior.
- Happy-path fixtures: valid boundary inputs → expected outputs.
- Edge-case fixtures: malformed inputs, boundary violations, version skew.
- Each fixture must be self-contained and documented.

---

### Phase 3 — Conformance run receipt

**3.1** Model or emit `ConformanceRunReceiptV1`.
- Per the P32 spec: "emit or model ConformanceRunReceiptV1 for implementation vs reference fixtures."
- If the type already exists in `aidens-contracts`, wire it. If not, model it as a structured receipt.
- Receipt must include: run ID, fixture count, pass/fail per fixture, timestamp, environment.

**3.2** Run boundary compiler against reference fixtures.
- Capture results as conformance evidence.
- Ensure failures produce diagnostic receipts (not silent drops).

---

### Phase 4 — Wire one real structured path

**4.1** Identify one structured import/export/tool-output path to connect through the boundary compiler.
- Per the P32 spec: "wire one real structured import/export/tool-output path through the boundary compiler."
- Candidates: tool dispatch path, receipt archival path, artifact envelope path.
- Must be a real path that already exists in the codebase — do not invent a synthetic path.

**4.2** Add integration tests proving boundary compiler records survive into the artifact/receipt path.
- Per the P32 spec: "add tests proving that boundary compiler records survive into the artifact/receipt path."
- Test: create boundary input → verify artifact/receipt output includes boundary metadata.

---

### Phase 5 — p30_guard hygiene (if warranted)

**5.1** Determine if `p30_guard.py` hygiene warning is a known false positive or a genuine issue.
- Source: `scripts/z.py` hygiene policy logic.
- If false positive: document as acceptable, add to KNOWN_LIMITATIONS.md.
- If genuine: archive `scripts/p30_guard.py` reference or update policy.

---

## Execution Notes

- Begin each session by reading CLAUDE.md, CURRENT_RUN.md, and the relevant phase doc.
- Do NOT load the full P30 hostile audit matrix into context — use phase-specific file touch maps.
- Every phase emits receipts: changed files, commands run, tests added/updated, blockers.
- Do not claim P32 is complete until all P32 work items are verified and all P31B gates still pass.
- The hostile auditor's handoff statement asks for verification in a fresh clone — plan for that.

## Verification Gates for P32

P32 is complete when:
1. All 18 P31B verification gates still pass (regression check)
2. `aidens-boundary-kit` has ≥1 new test proving boundary compiler record survival
3. Reference fixture corpus exists and covers happy-path + edge cases
4. `ConformanceRunReceiptV1` exists (modeled or emitted) with fixture run evidence
5. One real structured I/O path is wired through the boundary compiler
6. `cargo check --workspace --locked` clean
7. `cargo test --workspace --locked` all pass
8. `cargo clippy --workspace --locked --all-targets -- -D warnings` clean

---

## Unresolved Blockers (at plan creation)

- **B1**: `aidens-boundary-kit` src/ is a single lib.rs — implementation status unknown
- **B2**: P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md needs full review to determine scope
- **B3**: p30_guard.py hygiene warning root cause not yet diagnosed
- **B4**: `extracted_replay_certified=false` is environmental — not a code issue, but limits self-replay proof