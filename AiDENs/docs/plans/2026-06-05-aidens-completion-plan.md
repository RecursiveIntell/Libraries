# AiDENs Completion Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Move AiDENs from P32 schema-compat candidate to a clean, replayable, release-certifiable local orchestration layer with no false completion claims, no missing evidence gates, and a concrete path toward v11A/v11B compliance.

**Architecture:** AiDENs remains an orchestration/adapter/control surface. It must wire sibling crates, emit and preserve receipts, package evidence, expose strict boundaries, and avoid owning canonical semantic truth. Completion is defined by passing local verification, package/replay gates, evidence closure, and doctrine hardening; not by claiming production cloud or broad autonomy.

**Tech Stack:** Rust workspace, Cargo, sibling path crates under `/home/sikmindz/Coding/Libraries`, `z.py` source package generator, `scripts/verify_current.sh`, Python assertion gates, AiDENs contract/schema crates.

---

## Current examined state — 2026-06-05

Repository: `/home/sikmindz/Coding/Libraries/AiDENs`

Observed with real commands:

- `cargo metadata --no-deps --format-version=1` sees 34 workspace packages.
- `cargo check --workspace --locked --all-targets` passes.
- `cargo clippy --workspace --locked --all-targets -- -D warnings` passes for AiDENs workspace, while sibling `semantic-memory` emits dependency warnings only.
- `scripts/verify_current.sh .` currently stops at `cargo fmt --all --check` because sibling path crate `../semantic-memory` has unformatted local changes.
- `python3 scripts/assert_package_validation.py` fails before package generation because `target/p32/package` is absent.
- `python3 scripts/assert_super_pass_docs_evidence_closure.py` fails because `target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json` is absent.
- AiDENs repo itself is clean with `git status --short -- .`; parent status includes dirty sibling `../semantic-memory` files.
- Largest production files remain monolithic: `crates/aidens-cli/src/lib.rs` 4,996 lines; `crates/aidens-tool-kit/src/lib.rs` 3,396 lines.
- Static doctrine-risk counts from `crates/**/*.rs`: 731 `.unwrap()` occurrences, 197 `.expect(` occurrences, 62 `unwrap_or_default` occurrences, and many `serde_json::Value` uses. A large share is tests/CLI, but material paths still contain risks.
- Crates with lowest maturity signal: `aidens-delegation-kit` has 29 LOC and 0 tests; profile crates are scaffold-only by design; `boundary-compiler-core` is partial but has 28 tests.

Important caveat: I generated a temporary P32 package during examination to verify the packaging failure mode, then removed it and restored files touched by `z.py` so the repo was not left with packaging side effects. The generation proved package validation can pass once sidecars exist, with 0 errors and 2 warnings (`scripts/p30_guard.py` and `fib-quant/scripts/fibquant_final_assert.py` script-ref-not-archived warnings).

---

## Missing to call AiDENs “complete”

### P0 — Certification blockers

1. Clean verification run is blocked by dirty/unformatted sibling `semantic-memory`.
   - Evidence: `scripts/verify_current.sh .` fails at `cargo_fmt` with diffs in `/home/sikmindz/Coding/Libraries/semantic-memory/src/provekv_pool.rs` and `tests/search_tests.rs`.
   - This is not an AiDENs source edit, but AiDENs cannot truthfully claim a clean current gate while path dependencies are dirty/unformatted.

2. P32 package sidecars are missing from the expected location.
   - Evidence: `python3 scripts/assert_package_validation.py` -> `FAIL: missing package dir: /home/sikmindz/Coding/Libraries/AiDENs/target/p32/package`.
   - Completion needs a reproducible package generation command and retained sidecars.

3. Package self-replay is not wired into the default verification path and needs a real package argument.
   - Evidence: `python3 scripts/assert_package_self_replay.py` with no args -> `FAIL: package path required`.
   - `CURRENT_RUN.json` still records `extracted_replay_certified=false` as an environmental blocker. Completion must either fix the replay environment or explicitly keep this as a non-certified limitation.

4. Super-pass evidence closure is missing the Phase 15 audit hash manifest.
   - Evidence: `python3 scripts/assert_super_pass_docs_evidence_closure.py` -> missing `target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json`.
   - Completion needs a deterministic script to generate this from actual logs, not a hand-written placeholder.

5. `verify_current.sh` only runs 17 gates; extra existing assertion scripts are outside the main gate.
   - Missing from main verifier today: package validation, package replay, no scaffold promotion, phase-gate integrity, phase19 quarantine, super-pass docs closure, schema generation scope, script refs strict, sibling workspace layout, z.py contract, capability contract, docs-match-cargo.
   - Completion needs one “release gate” command that runs the whole relevant suite and records receipts.

### P1 — Doctrine hardening gaps

6. Material paths still use failure-erasing patterns.
   - Highest-risk `unwrap_or_default` locations found in `aidens-agency-kit`, `aidens-boundary-kit`, `aidens-receipts`, `aidens-runner`, `aidens-provider-kit`, and `aidens-tool-kit`.
   - Completion needs per-crate migration from silent defaults to typed error/degradation/repair receipts.

7. Dynamic JSON remains in runtime/control/tool/receipt paths.
   - Legitimate areas: CLI display/formatting, tests, schema generation, boundary parsing internals.
   - Risk areas to classify/fix first: `aidens-receipts/src/lib.rs`, `aidens-runner/src/lib.rs`, `aidens-runner/src/provider_tool.rs`, `aidens-tool-kit/src/lib.rs`, `aidens-provider-kit/src/lib.rs`, `aidens-agency-kit/src/lib.rs`.

8. Monoliths remain too large for safe audit.
   - `aidens-cli/src/lib.rs`: 4,996 lines.
   - `aidens-tool-kit/src/lib.rs`: 3,396 lines.
   - Completion should split modules without changing behavior, with tests before/after.

9. `aidens-delegation-kit` is effectively a scaffold.
   - 29 LOC, 0 tests.
   - Either implement a minimal receipt-bearing delegation contract or explicitly quarantine it as not part of supported completion.

10. Profile crates remain scaffold-only.
   - `aidens-profile-daemon`, `aidens-profile-desktop`, `aidens-profile-memory`, `aidens-profile-research` are documented scaffold-only.
   - Completion must either keep them excluded from support claims or implement one supported profile path end-to-end.

### P2 — Product completion gaps

11. Boundary compiler integration is partial and split between strict and lenient paths.
   - Good: `aidens-boundary-kit/src/canonical_boundary.rs` delegates to `boundary_compiler_core`; `aidens-runner/src/provider_tool.rs` uses `boundary_compiler_core`; conformance receipt tests exist.
   - Missing: full strict path adoption for material tool/provider I/O, schema manifest closure, and canonical digest/type replacement where aliases remain.

12. Sibling capabilities are only partially surfaced.
   - Existing: memory/governance wrappers import `semantic_memory`, `knowledge_runtime`, `verification_control`, `assurance_runtime`.
   - Missing high-impact wiring: memory integrity verification, search receipt replay, knowledge-runtime view surfacing, verification-control case creation, assurance-runtime release readiness decisions.

13. There is no single end-to-end “local agent run” release proof.
   - Completion needs one command that executes a representative local coding-agent/daemon-safe path, dispatches a tool through the boundary compiler, records queue/provider/budget/degradation receipts, and validates final receipt chain.

14. Release truth docs contain candidate/non-claim language but no final release handoff for P32+.
   - Completion needs `CURRENT_RUN.json`, `STATUS.md`, `SUPPORT_PROFILE.md`, source basis docs, package sidecars, command receipts, and final auditor handoff updated from the same generated evidence.

---

## Completion definition

AiDENs is complete for the next honest release when all of these are true:

1. `git status --short -- .` is clean before and after release gates.
2. Sibling path dependencies required by AiDENs are formatted/buildable or pinned to clean published versions.
3. One top-level gate script passes and records receipts for static assertions, cargo gates, package validation, package self-replay, and super-pass evidence closure.
4. P32 package sidecars exist under `target/p32/package/` and validate with 0 errors.
5. Extracted package self-replay either passes or is explicitly marked as an environmental non-certification with a receipt and not counted as release-certified.
6. No hard p30_guard findings.
7. No false “v11B/v11C/production/cloud/broad-autonomy” claims.
8. Supported-local path has a real receipt-bearing run through boundary compile -> tool/provider dispatch -> result -> replay/validation.
9. Doctrine hardening P1 work has reduced or quarantined material-path `unwrap_or_default`, dynamic JSON, and panic risks.
10. Final docs are generated from evidence, not manually asserted.

---

# Implementation Plan

## Phase 0: Freeze and prove the baseline

### Task 0.1: Capture working tree scope

**Objective:** Separate AiDENs completion work from dirty sibling work.

**Files:**
- Read only: repository status.
- Create: `target/completion-baseline/git-status.txt`

**Steps:**
1. Run:
   ```bash
   mkdir -p target/completion-baseline
   git status --short -- . > target/completion-baseline/aidens-status.txt
   git status --short > target/completion-baseline/full-status.txt
   ```
2. Expected:
   - `aidens-status.txt` is empty.
   - `full-status.txt` may show `../semantic-memory` work.
3. Do not edit AiDENs until this is captured.

**Gate:** Baseline files exist and show scope separation.

### Task 0.2: Resolve sibling formatting blocker

**Objective:** Make `cargo fmt --all --check` runnable from AiDENs.

**Files:**
- Modify only if this is the current active sibling task: `/home/sikmindz/Coding/Libraries/semantic-memory/src/provekv_pool.rs`, `/home/sikmindz/Coding/Libraries/semantic-memory/tests/search_tests.rs`, and related dirty files.
- Otherwise coordinate by stashing/committing sibling work before release gates.

**Steps:**
1. Run:
   ```bash
   git -C ../semantic-memory status --short
   cargo fmt --manifest-path ../semantic-memory/Cargo.toml --all
   ```
2. If sibling changes are not yours to modify, stop and quarantine completion gate as blocked-by-dirty-sibling.
3. Re-run:
   ```bash
   cargo fmt --all --check
   ```

**Gate:** `cargo fmt --all --check` exits 0 from AiDENs root.

## Phase 1: Build the missing release gate

### Task 1.1: Create an all-gates release verifier

**Objective:** Add one release gate script that runs every relevant existing assertion, cargo gate, package validation, package replay, and evidence closure in order.

**Files:**
- Create: `scripts/verify_release.sh`
- Modify: `docs/codex-runs/CURRENT_RUN.json` only after gates pass.

**Implementation requirements:**
- Wrap commands like `scripts/verify_current.sh` does.
- Log under `target/verify-release/${RUN_ID:-P32}/`.
- Run these gates before cargo:
  ```bash
  python3 scripts/assert_release_ledger_schema.py
  python3 scripts/assert_current_run_truth.py
  python3 scripts/assert_release_truth_consistency.py
  python3 scripts/assert_root_markdown_archive_policy.py
  python3 scripts/assert_codex_artifact_classification.py
  python3 scripts/assert_support_claims_have_evidence.py
  bash scripts/assert_no_fake_completion.sh .
  bash scripts/assert_no_shadow_truth.sh
  bash scripts/assert_adapter_delegation.sh
  bash scripts/assert_tool_runtime_delegation.sh
  python3 scripts/assert_no_canonical_type_duplicates.py
  bash scripts/assert_no_local_substitute_dependencies.sh
  python3 scripts/p30_guard.py --repo .
  bash scripts/assert_no_scaffold_promoted.sh
  python3 scripts/assert_phase_gate_integrity.py
  python3 scripts/assert_phase19_high_risk_quarantine.py
  python3 scripts/assert_super_pass_docs_evidence_closure.py
  python3 scripts/assert_schema_generation_scope.py
  python3 scripts/assert_script_refs_strict.py
  python3 scripts/assert_sibling_workspace_layout.py
  python3 scripts/assert_zpy_total_contract.py
  python3 scripts/assert_aidens_capability_contract.py
  bash scripts/assert_docs_match_cargo.sh
  ```
- Then cargo:
  ```bash
  cargo metadata --locked --format-version 1
  cargo fmt --all --check
  cargo check --workspace --locked --all-targets
  cargo test --workspace --locked --all-targets
  cargo clippy --workspace --locked --all-targets -- -D warnings
  ```
- Then package gates using the package path produced in Phase 2.

**Test:**
```bash
bash scripts/verify_release.sh .
```
Expected before Phase 2: fail only on missing package/evidence closure gates.

### Task 1.2: Make package self-replay usable by default

**Objective:** Remove the “package path required” footgun from release verification.

**Files:**
- Modify: `scripts/assert_package_self_replay.py` or call it only from `verify_release.sh` with an explicit package path.

**Preferred implementation:**
- Keep the script strict.
- In `verify_release.sh`, discover latest `target/p32/package/AiDENs-*.zip` matching active run and pass it as:
  ```bash
  python3 scripts/assert_package_self_replay.py --package "$LATEST_PACKAGE" --expected-run "$RUN_ID" --receipt-out "target/verify-release/$RUN_ID/package_self_replay_receipt.json"
  ```

**Gate:** Running the script without package may still fail; release verifier must not call it incorrectly.

## Phase 2: Generate real package and evidence sidecars

### Task 2.1: Generate P32 package into expected directory without archiving root source unexpectedly

**Objective:** Produce package sidecars where `assert_package_validation.py` expects them.

**Files:**
- Generated: `target/p32/package/AiDENs-aidens-p32-codex-context-*.zip`
- Generated sidecars: `.manifest.json`, `.report.md`, `.findings.json`, `.excluded.json`, `.codex-archive.json`

**Command:**
```bash
mkdir -p target/p32/package
python3 z.py \
  --root . \
  --profile aidens \
  --mode codex-context \
  --strict \
  --codex-current-run P32 \
  -o "target/p32/package/AiDENs-aidens-p32-codex-context-$(date -u +%Y%m%dT%H%M%SZ).zip"
```

**Pitfall:** `z.py` can move root package artifacts/source-package archives. Inspect `git status --short -- .` afterward and revert unintended source/doc moves before claiming clean release.

**Gate:**
```bash
python3 scripts/assert_package_validation.py
```
Expected: PASS with 0 errors. Warnings are allowed only if documented in `CURRENT_RUN.json` and non-fatal.

### Task 2.2: Generate Phase 15 audit hash manifest from real logs

**Objective:** Satisfy super-pass docs/evidence closure with actual hashes.

**Files:**
- Create: `scripts/generate_audit_log_hashes.py`
- Generate: `target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json`

**Implementation:**
- Walk these log roots if they exist:
  - `target/verify-current/`
  - `target/verify-release/`
  - `docs/codex-runs/*/`
- Include only actual log/receipt files (`*.log`, `*.json`, `*.jsonl`, `*.txt`) that are evidence, not source.
- Write JSON:
  ```json
  {
    "artifact_kind": "aidens.audit_log_hash_manifest.v1",
    "run": "P32",
    "entry_count": 0,
    "entries": [
      {"path": "relative/path", "sha256": "64 hex", "bytes": 1234}
    ]
  }
  ```

**Test:**
```bash
python3 scripts/generate_audit_log_hashes.py --root . --run P32 --out target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json
python3 scripts/assert_super_pass_docs_evidence_closure.py
```
Expected: PASS.

### Task 2.3: Run package self-replay honestly

**Objective:** Either pass extracted replay or record a precise blocker.

**Command:**
```bash
LATEST_PACKAGE=$(ls -t target/p32/package/AiDENs-*.zip | head -1)
python3 scripts/assert_package_self_replay.py \
  --package "$LATEST_PACKAGE" \
  --expected-run P32 \
  --receipt-out target/verify-release/P32/package_self_replay_receipt.json
```

**Expected outcomes:**
- Best: PASS.
- Acceptable candidate-only: FAIL/BLOCKED with receipt showing environmental blocker; `CURRENT_RUN.json` must keep `extracted_replay_certified=false` and final release claim must not count this as certified.

## Phase 3: Harden material receipt and boundary paths

### Task 3.1: Classify all `unwrap_or_default` instances

**Objective:** Separate harmless display/test defaults from material-path failure erasure.

**Files:**
- Create: `docs/audits/unwrap_or_default_classification_P33.md`
- Modify after classification: source crates listed below.

**Command:**
```bash
grep -RIn 'unwrap_or_default' crates --include='*.rs' > target/completion-baseline/unwrap_or_default.txt
```

**Classification buckets:**
1. Test-only OK.
2. CLI display OK if not receipt/material.
3. Material path must fix.
4. Boundary parse/repair must emit repair/degradation receipt.

**High-priority fix files:**
- `crates/aidens-agency-kit/src/lib.rs`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-runner/src/receipts.rs`
- `crates/aidens-provider-kit/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-boundary-kit/src/canonical_boundary.rs`

**Gate:** No material-path entry remains unclassified.

### Task 3.2: Replace receipt-path silent defaults with typed errors

**Objective:** Make receipt construction fail loudly or emit degradation evidence.

**Files:**
- Modify: `crates/aidens-receipts/src/lib.rs`
- Modify tests: add/extend tests in same crate or `crates/aidens-integration-tests/`.

**Pattern:**
- Replace:
  ```rust
  serde_json::to_string(value).unwrap_or_default()
  ```
- With a result-returning API:
  ```rust
  pub fn encode_receipt_payload(value: &TypedReceiptPayloadV1) -> Result<String, ReceiptError> {
      serde_json::to_string(value).map_err(ReceiptError::SerializePayload)
  }
  ```

**Gate:** Receipt serialization failures cannot produce empty payloads.

### Task 3.3: Replace provider/tool runtime defaults with degraded receipts

**Objective:** Network/tool/dispatch parse failures must become explicit degraded/failure artifacts.

**Files:**
- Modify: `crates/aidens-provider-kit/src/lib.rs`
- Modify: `crates/aidens-tool-kit/src/lib.rs`
- Modify: `crates/aidens-runner/src/provider_tool.rs`

**Test cases:**
- Provider response body cannot be read -> failure receipt with body_read_failed.
- Tool input JSON invalid -> boundary rejection receipt, not default/empty input.
- Provider mock response missing -> explicit fixture error in tests; explicit runtime degradation in production.

**Gate:** New tests fail before implementation and pass after.

### Task 3.4: Decide strict-vs-lenient boundary policy

**Objective:** Ensure material tool/provider inputs use strict boundary compiler path unless explicitly repair-gated.

**Files:**
- Modify: `crates/aidens-boundary-kit/src/lib.rs`
- Modify: `crates/aidens-boundary-kit/src/canonical_boundary.rs`
- Modify: `crates/aidens-runner/src/provider_tool.rs`
- Modify tests: `crates/aidens-integration-tests/tests/conformance_run_receipt.rs`

**Requirements:**
- Material executable tool calls must go through `boundary_compiler_core` strict profile.
- Lenient repair path is allowed only if output includes explicit repair/degradation provenance.
- Duplicate keys, unknown fields, coercion, future-version skew, and resource ceiling breach must all produce typed receipts.

**Gate:** Conformance receipt test proves every fixture result appears in `ConformanceRunReceiptV1`.

## Phase 4: Split monoliths without behavior change

### Task 4.1: Split `aidens-cli/src/lib.rs`

**Objective:** Reduce CLI monolith risk without semantic changes.

**Files:**
- Modify: `crates/aidens-cli/src/lib.rs`
- Existing modules: `agent.rs`, `package.rs`
- Create modules as needed:
  - `crates/aidens-cli/src/doctor.rs`
  - `crates/aidens-cli/src/profile.rs`
  - `crates/aidens-cli/src/support.rs`
  - `crates/aidens-cli/src/receipt_display.rs`
  - `crates/aidens-cli/src/coding_agent.rs`

**Process:**
1. Run current CLI tests.
2. Move one cohesive block at a time.
3. Re-export only what tests need.
4. Run after each extraction:
   ```bash
   cargo test -p aidens-cli --locked --all-targets
   cargo clippy -p aidens-cli --locked --all-targets -- -D warnings
   ```

**Gate:** `aidens-cli/src/lib.rs` below 2,500 lines and tests unchanged/pass.

### Task 4.2: Split `aidens-tool-kit/src/lib.rs`

**Objective:** Isolate registry, dispatch, permits, receipts, and canonical stack adapters.

**Files:**
- Modify: `crates/aidens-tool-kit/src/lib.rs`
- Existing: `crates/aidens-tool-kit/src/canonical_stack.rs`
- Create:
  - `registry.rs`
  - `dispatch.rs`
  - `permit_gate.rs`
  - `receipts.rs`
  - `repo_read.rs`

**Gate:** `aidens-tool-kit/src/lib.rs` below 1,500 lines; package tests pass.

## Phase 5: Implement or quarantine weak crates/profiles

### Task 5.1: Finish `aidens-delegation-kit` minimal contract

**Objective:** Stop carrying a 0-test scaffold in a “complete” release.

**Files:**
- Modify: `crates/aidens-delegation-kit/src/lib.rs`
- Add tests: same file or `crates/aidens-delegation-kit/tests/delegation_receipts.rs`

**Minimum implementation:**
- `DelegationRequestV1`
- `DelegationDecisionV1`
- `DelegationReceiptV1`
- deterministic receipt ID from content digest, not random UUID or counter
- rejection cases: missing scope, expired authority, unsupported target

**Gate:** At least 6 tests; no promoted support claim unless adapter wired into runner/governance path.

### Task 5.2: Either implement one profile vertical slice or quarantine all profile crates

**Objective:** Make support claims match profile reality.

**Option A — implement one:**
- Pick `aidens-profile-coding` or `aidens-profile-daemon`.
- Wire config -> boundary -> tool dispatch -> receipt chain -> validation.

**Option B — quarantine:**
- Keep profile crates scaffold-only in `STATUS.md`.
- Add explicit known limitation row.
- Ensure `assert_no_scaffold_promoted.sh` catches any promoted claim.

**Gate:** No scaffold-only profile appears in supported release claims.

## Phase 6: Wire high-impact sibling capabilities

### Task 6.1: Memory integrity and search replay

**Objective:** Expose semantic-memory verification without owning memory truth.

**Files:**
- Modify: `crates/aidens-memory-kit/src/lib.rs`
- Add tests: `crates/aidens-integration-tests/tests/memory_integrity_replay.rs`

**Capabilities:**
- `verify_integrity()` adapter if available in `semantic-memory`.
- `replay_search_receipt()` adapter if available.
- Return AiDENs wrapper receipt that points to sibling canonical receipt, not a duplicate truth model.

**Gate:** Test proves AiDENs delegates to semantic-memory type IDs or canonical receipt types.

### Task 6.2: Governance/release readiness adapters

**Objective:** Surface verification-control and assurance-runtime decisions as delegated cases.

**Files:**
- Modify: `crates/aidens-governance-kit/src/lib.rs`
- Add tests: `crates/aidens-integration-tests/tests/governance_release_readiness.rs`

**Capabilities:**
- Create verification-control case for effect review/delegation review.
- Create assurance-runtime release readiness decision wrapper.
- No advisory observation may be reported as verified success.

**Gate:** Tests prove advisory/degraded/failed states remain distinct.

## Phase 7: End-to-end supported-local proof

### Task 7.1: Add one real local agent run proof

**Objective:** Prove AiDENs can run the supported local vertical slice with receipts.

**Files:**
- Add test: `crates/aidens-integration-tests/tests/supported_local_e2e.rs`
- Possibly add fixture: `fixtures/supported-local/valid_tool_call.json`

**Scenario:**
1. Load a profile/config.
2. Strict-compile a JSON tool input through boundary compiler.
3. Dispatch a deterministic repo-read or safe local tool.
4. Emit provider/tool/boundary/budget receipts.
5. Validate receipt chain and replay determinism.

**Gate:**
```bash
cargo test -p aidens-integration-tests --test supported_local_e2e --locked -- --nocapture
```
Expected: PASS and output receipt IDs.

## Phase 8: Final release truth and package closure

### Task 8.1: Regenerate release receipts

**Objective:** Create a single evidence bundle from the release gate.

**Commands:**
```bash
bash scripts/verify_release.sh .
python3 scripts/generate_audit_log_hashes.py --root . --run P32 --out target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json
bash scripts/verify_release.sh .
```

**Gate:** Second run passes all non-environmental gates.

### Task 8.2: Update run truth docs from evidence

**Objective:** Make docs match actual gate outcomes.

**Files:**
- Modify: `docs/codex-runs/CURRENT_RUN.json`
- Modify: `STATUS.md`
- Modify: `SUPPORT_PROFILE.md`
- Modify: `SOURCE_BASIS.md`
- Modify: `handoffs/super-pass/FINAL_AUDITOR_HANDOFF.md`

**Rules:**
- If package self-replay passed: set `extracted_replay_certified=true` and cite receipt path.
- If replay blocked: leave `extracted_replay_certified=false`; do not claim release-certified extracted replay.
- Do not claim v11B/v11C/production/cloud/broad-autonomy readiness.
- Claim only supported-local candidate/certified scope backed by receipts.

**Gate:**
```bash
python3 scripts/assert_current_run_truth.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_support_claims_have_evidence.py
python3 scripts/assert_super_pass_docs_evidence_closure.py
```

### Task 8.3: Final verification and clean tree check

**Objective:** End with a clean, reproducible state.

**Commands:**
```bash
git status --short -- .
bash scripts/verify_release.sh .
git status --short -- .
```

**Expected:**
- First status clean except intentional source/doc changes not yet committed.
- Release gate PASS or explicitly documented environmental replay blocker.
- Final status only includes intended source/docs/scripts; no target artifacts committed unless policy requires.

---

## Suggested commit sequence

1. `docs: add AiDENs completion plan`
2. `test: add release verifier covering package and evidence gates`
3. `chore: generate audit log hash manifest from real logs`
4. `fix: make P32 package validation and replay discoverable`
5. `fix(receipts): remove silent defaults from receipt paths`
6. `fix(runtime): emit degraded receipts for provider/tool parse failures`
7. `refactor(cli): split CLI modules without behavior changes`
8. `refactor(tool-kit): split registry and dispatch modules`
9. `feat(delegation): add receipt-bearing delegation contract`
10. `feat(memory): expose delegated integrity and search replay adapters`
11. `feat(governance): add delegated release readiness cases`
12. `test: add supported-local receipt-bearing e2e proof`
13. `docs: update run truth from release evidence`

---

## Final acceptance commands

Run from `/home/sikmindz/Coding/Libraries/AiDENs`:

```bash
git status --short -- .
cargo fmt --all --check
cargo check --workspace --locked --all-targets
cargo test --workspace --locked --all-targets
cargo clippy --workspace --locked --all-targets -- -D warnings
python3 scripts/assert_package_validation.py
LATEST_PACKAGE=$(ls -t target/p32/package/AiDENs-*.zip | head -1)
python3 scripts/assert_package_self_replay.py --package "$LATEST_PACKAGE" --expected-run P32 --receipt-out target/verify-release/P32/package_self_replay_receipt.json
python3 scripts/assert_super_pass_docs_evidence_closure.py
bash scripts/verify_release.sh .
```

Completion claim allowed only if these pass or any blocker is explicitly classified as non-certified/non-release scope in `CURRENT_RUN.json` and `SUPPORT_PROFILE.md`.
