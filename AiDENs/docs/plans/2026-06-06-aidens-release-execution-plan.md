# AiDENs Release Execution Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Finish AiDENs as an honest P32 supported-local release candidate by closing release-gate, package, replay, evidence, and doctrine blockers without widening claims beyond proven evidence.

**Architecture:** AiDENs remains an adapter/orchestration layer. It must delegate canonical truth to sibling crates, run strict typed boundaries for material tool/provider paths, emit receipts for every material transition, and preserve candidate/certified/non-claim distinctions in generated evidence.

**Tech Stack:** Rust workspace, Cargo, Bash/Python gate scripts, `z.py`, AiDENs P32 run ledger, sibling path crates under `/home/sikmindz/Coding/Libraries`.

---

## Current State Snapshot

**Date:** 2026-06-06T00:01:59Z
**Repo:** `/home/sikmindz/Coding/Libraries/AiDENs`
**Workspace packages:** 34

**Observed status:**

- AiDENs target tree has one untracked docs directory from planning work: `?? docs/plans/`.
- Parent workspace has dirty sibling `../semantic-memory` files:
  - `results/provekv_pool_benchmark_receipt.json`
  - `src/provekv_pool.rs`
  - `src/search.rs`
  - `tests/pool_generation_types.rs`
  - `tests/search_tests.rs`
  - untracked `examples/provekv_vs_usearch_benchmark.rs`
  - untracked `results/provekv_vs_usearch_benchmark.json`
- Prior observed checks:
  - `cargo check --workspace --locked --all-targets` passed.
  - `cargo clippy --workspace --locked --all-targets -- -D warnings` passed for AiDENs; sibling `semantic-memory` emitted warnings.
  - `scripts/verify_current.sh .` failed at `cargo_fmt` because dirty sibling `semantic-memory` was unformatted.
  - `python3 scripts/assert_package_validation.py` failed because `target/p32/package` was missing.
  - `python3 scripts/assert_super_pass_docs_evidence_closure.py` failed because `target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json` was missing.

**Completion line:** AiDENs is not complete until release gates pass from a clean/controlled path-dependency state and package/replay/evidence sidecars exist. Until then it remains a P32 schema-compat candidate, not a certified release and not v11B/v11C/production/cloud-ready.

---

## Missing Work by Severity

### P0 — Blocks honest release certification

1. Dirty/unformatted sibling `semantic-memory` blocks workspace fmt gate.
2. No canonical `scripts/verify_release.sh` that runs all existing relevant gates.
3. Missing P32 package directory and sidecars under `target/p32/package/`.
4. Package self-replay is not discoverable from the release gate.
5. Missing generated audit hash manifest: `target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json`.
6. Run truth docs do not yet cite a fresh release-gate receipt bundle.

### P1 — Blocks “complete enough to trust”

7. Material paths still contain silent defaults / panic-adjacent patterns.
8. Dynamic JSON still appears in runtime/tool/provider/receipt paths and needs classification.
9. `aidens-cli/src/lib.rs` and `aidens-tool-kit/src/lib.rs` remain too monolithic for safe review.
10. `aidens-delegation-kit` has 0 tests and must be implemented or quarantined.
11. Scaffold-only profile crates must stay excluded from support claims unless implemented.

### P2 — Product completeness after P32 certification

12. Supported-local E2E receipt-bearing run is not yet a single release proof.
13. High-impact sibling capabilities are not fully surfaced: memory integrity, search replay, governance case creation, assurance release readiness.
14. Boundary compiler strict-vs-lenient policy needs one enforced path for material tool/provider execution.

---

# Phase 1: Stabilize the Baseline

## Task 1.1: Capture current scope before edits

**Objective:** Prove what belongs to AiDENs and what belongs to sibling `semantic-memory`.

**Files:**
- Create: `target/release-execution/baseline/aidens-status.txt`
- Create: `target/release-execution/baseline/full-status.txt`
- Create: `target/release-execution/baseline/package-list.txt`

**Steps:**

1. Run:
   ```bash
   mkdir -p target/release-execution/baseline
   git status --short -- . > target/release-execution/baseline/aidens-status.txt
   git status --short > target/release-execution/baseline/full-status.txt
   cargo metadata --no-deps --format-version=1 \
     | python3 -c 'import json,sys; d=json.load(sys.stdin); print(len(d["packages"])); print("\n".join(sorted(p["name"] for p in d["packages"])))' \
     > target/release-execution/baseline/package-list.txt
   ```

2. Verify:
   ```bash
   cat target/release-execution/baseline/aidens-status.txt
   cat target/release-execution/baseline/full-status.txt
   ```

**Expected:** AiDENs-only status shows only intended plan/source changes. Full status may show sibling changes.

## Task 1.2: Resolve the sibling fmt blocker

**Objective:** Make AiDENs release verification independent of accidental sibling formatting drift.

**Files:**
- Sibling only if chosen: `/home/sikmindz/Coding/Libraries/semantic-memory/*`
- No AiDENs source changes in this task.

**Steps:**

1. Inspect sibling state:
   ```bash
   git -C ../semantic-memory status --short
   ```

2. If sibling work is intended active work, format it:
   ```bash
   cargo fmt --manifest-path ../semantic-memory/Cargo.toml --all
   ```

3. If sibling work must not be touched, stop and mark release gate as blocked by dirty path dependency in `CURRENT_RUN.json` later. Do not fake a clean gate.

4. Re-test from AiDENs root:
   ```bash
   cargo fmt --all --check
   ```

**Gate:** `cargo fmt --all --check` exits 0, or release remains blocked with an explicit path-dependency blocker.

---

# Phase 2: Build the Release Gate

## Task 2.1: Add `scripts/verify_release.sh`

**Objective:** Create one top-level gate that runs static assertions, cargo gates, package validation, package replay, and evidence closure.

**Files:**
- Create: `scripts/verify_release.sh`

**Implementation:**

Create a Bash script with this shape:

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT="${1:-.}"
cd "$ROOT"
RUN_ID="${RUN_ID:-P32}"
LOG_DIR="${AIDENS_VERIFY_LOG_DIR:-target/verify-release/$RUN_ID}"
mkdir -p "$LOG_DIR"

run() {
  local name="$1"; shift
  echo "[verify-release] $name: $*"
  set +e
  "$@" >"$LOG_DIR/${name}.stdout.log" 2>"$LOG_DIR/${name}.stderr.log"
  local code=$?
  set -e
  if [[ "$code" -ne 0 ]]; then
    echo "[verify-release] FAIL $name exit=$code" >&2
    tail -80 "$LOG_DIR/${name}.stdout.log" >&2 || true
    tail -80 "$LOG_DIR/${name}.stderr.log" >&2 || true
    exit "$code"
  fi
  echo "[verify-release] PASS $name"
}
```

Then run all existing relevant gates:

```bash
run release_ledger_schema python3 scripts/assert_release_ledger_schema.py
run current_run_truth python3 scripts/assert_current_run_truth.py
run release_truth_consistency python3 scripts/assert_release_truth_consistency.py
run root_markdown_archive_policy python3 scripts/assert_root_markdown_archive_policy.py
run codex_artifact_classification python3 scripts/assert_codex_artifact_classification.py
run support_claims_have_evidence python3 scripts/assert_support_claims_have_evidence.py
run no_fake_completion bash scripts/assert_no_fake_completion.sh .
run no_shadow_truth bash scripts/assert_no_shadow_truth.sh
run adapter_delegation bash scripts/assert_adapter_delegation.sh
run tool_runtime_delegation bash scripts/assert_tool_runtime_delegation.sh
run no_canonical_type_duplicates python3 scripts/assert_no_canonical_type_duplicates.py
run no_local_substitute_dependencies bash scripts/assert_no_local_substitute_dependencies.sh
run p30_guard python3 scripts/p30_guard.py --repo .
run no_scaffold_promoted bash scripts/assert_no_scaffold_promoted.sh
run phase_gate_integrity python3 scripts/assert_phase_gate_integrity.py
run phase19_high_risk_quarantine python3 scripts/assert_phase19_high_risk_quarantine.py
run schema_generation_scope python3 scripts/assert_schema_generation_scope.py
run script_refs_strict python3 scripts/assert_script_refs_strict.py
run sibling_workspace_layout python3 scripts/assert_sibling_workspace_layout.py
run zpy_total_contract python3 scripts/assert_zpy_total_contract.py
run aidens_capability_contract python3 scripts/assert_aidens_capability_contract.py
run docs_match_cargo bash scripts/assert_docs_match_cargo.sh
run super_pass_docs_evidence_closure python3 scripts/assert_super_pass_docs_evidence_closure.py
run cargo_metadata cargo metadata --locked --format-version 1
run cargo_fmt cargo fmt --all --check
run cargo_check cargo check --workspace --locked --all-targets
run cargo_test cargo test --workspace --locked --all-targets
run cargo_clippy cargo clippy --workspace --locked --all-targets -- -D warnings
run package_validation python3 scripts/assert_package_validation.py
LATEST_PACKAGE="$(ls -t target/p32/package/AiDENs-*.zip 2>/dev/null | head -1 || true)"
if [[ -z "$LATEST_PACKAGE" ]]; then
  echo "[verify-release] FAIL package_self_replay: no target/p32/package/AiDENs-*.zip" >&2
  exit 2
fi
run package_self_replay python3 scripts/assert_package_self_replay.py --package "$LATEST_PACKAGE" --expected-run "$RUN_ID" --receipt-out "$LOG_DIR/package_self_replay_receipt.json"
```

**Verification:**

```bash
chmod +x scripts/verify_release.sh
bash scripts/verify_release.sh .
```

**Expected before Phase 3:** Fails only on missing package/evidence artifacts, not on script bugs.

## Task 2.2: Add release-gate receipt manifest

**Objective:** Record which logs belong to a release verification attempt.

**Files:**
- Create: `scripts/generate_release_gate_manifest.py`
- Generated: `target/verify-release/P32/RELEASE_GATE_MANIFEST.json`

**Implementation:**
- Walk `target/verify-release/P32`.
- Hash every `*.stdout.log`, `*.stderr.log`, `*.json`, and `*.jsonl` file.
- Emit JSON with `artifact_kind`, `run`, `created_utc`, `entry_count`, and entries containing path, sha256, bytes.

**Verification:**

```bash
python3 scripts/generate_release_gate_manifest.py --root . --run P32 --log-dir target/verify-release/P32
python3 -m json.tool target/verify-release/P32/RELEASE_GATE_MANIFEST.json >/dev/null
```

---

# Phase 3: Generate Missing Evidence Artifacts

## Task 3.1: Generate P32 package sidecars

**Objective:** Create the package files required by `assert_package_validation.py`.

**Files:**
- Generated: `target/p32/package/AiDENs-aidens-p32-codex-context-*.zip`
- Generated sidecars: `.manifest.json`, `.report.md`, `.findings.json`, `.excluded.json`, `.codex-archive.json`

**Steps:**

```bash
mkdir -p target/p32/package
python3 z.py \
  --root . \
  --profile aidens \
  --mode codex-context \
  --strict \
  --codex-current-run P32 \
  -o "target/p32/package/AiDENs-aidens-p32-codex-context-$(date -u +%Y%m%dT%H%M%SZ).zip"
python3 scripts/assert_package_validation.py
```

**Expected:** package validation PASS with 0 errors. Any warnings must be copied into `CURRENT_RUN.json` evidence notes.

**Pitfall:** `z.py` may archive/move package artifacts. Run `git status --short -- .` after generation and revert unintended source/doc churn.

## Task 3.2: Generate super-pass audit hash manifest

**Objective:** Satisfy `assert_super_pass_docs_evidence_closure.py` with real hashes.

**Files:**
- Create: `scripts/generate_audit_log_hashes.py`
- Generate: `target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json`

**Implementation requirements:**
- Hash logs/receipts from:
  - `target/verify-current/`
  - `target/verify-release/`
  - `docs/codex-runs/`
- Use relative paths.
- Do not hash source files as evidence entries.
- JSON format:
  ```json
  {
    "artifact_kind": "aidens.audit_log_hash_manifest.v1",
    "run": "P32",
    "entry_count": 1,
    "entries": [{"path": "target/verify-release/P32/cargo_check.stdout.log", "sha256": "...", "bytes": 123}]
  }
  ```

**Verification:**

```bash
python3 scripts/generate_audit_log_hashes.py --root . --run P32 --out target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json
python3 scripts/assert_super_pass_docs_evidence_closure.py
```

## Task 3.3: Run package self-replay

**Objective:** Prove extracted package replay or record a precise blocker.

**Command:**

```bash
LATEST_PACKAGE=$(ls -t target/p32/package/AiDENs-*.zip | head -1)
python3 scripts/assert_package_self_replay.py \
  --package "$LATEST_PACKAGE" \
  --expected-run P32 \
  --receipt-out target/verify-release/P32/package_self_replay_receipt.json
```

**Acceptance:**
- If PASS: release can claim extracted replay certified.
- If FAIL due to environment/path dependency/permission: release may remain candidate only; `CURRENT_RUN.json` must keep `extracted_replay_certified=false` and cite the receipt.

---

# Phase 4: Close P1 Doctrine Risks

## Task 4.1: Classify silent defaults

**Objective:** Stop treating `unwrap_or_default` as a vague smell; classify every instance.

**Files:**
- Create: `docs/audits/P33_UNWRAP_OR_DEFAULT_CLASSIFICATION.md`
- Generated probe: `target/release-execution/unwrap_or_default.txt`

**Steps:**

```bash
mkdir -p target/release-execution
grep -RIn 'unwrap_or_default' crates --include='*.rs' > target/release-execution/unwrap_or_default.txt
```

Classify every row as one of:

- `test-only-ok`
- `cli-display-ok`
- `schema-generation-ok`
- `material-path-fix-required`
- `boundary-repair-receipt-required`

**Gate:** No unclassified row remains.

## Task 4.2: Fix material-path defaults first

**Objective:** Remove silent defaulting from receipt/provider/tool/runner material paths.

**Files to prioritize:**
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-runner/src/receipts.rs`
- `crates/aidens-provider-kit/src/lib.rs`
- `crates/aidens-tool-kit/src/lib.rs`
- `crates/aidens-boundary-kit/src/canonical_boundary.rs`
- `crates/aidens-agency-kit/src/lib.rs`

**Pattern:**
- Before: serialization/read/parse failure becomes empty/default value.
- After: return typed error or emit typed degraded/failure receipt.

**Verification:**

```bash
cargo test -p aidens-receipts --locked --all-targets
cargo test -p aidens-provider-kit --locked --all-targets
cargo test -p aidens-tool-kit --locked --all-targets
cargo test -p aidens-runner --locked --all-targets
cargo clippy --workspace --locked --all-targets -- -D warnings
```

## Task 4.3: Classify dynamic JSON uses

**Objective:** Keep dynamic JSON only where it is boundary parsing, CLI formatting, tests, or schema generation.

**Files:**
- Create: `docs/audits/P33_DYNAMIC_JSON_CLASSIFICATION.md`

**Steps:**

```bash
grep -RIn 'serde_json::Value\|Value::' crates --include='*.rs' > target/release-execution/dynamic-json.txt
```

Classify each file:

- allowed: tests
- allowed: CLI display/report formatting
- allowed: schema catalog generation
- allowed: strict boundary parser internals
- fix: runtime/tool/provider/receipt path should use typed structs

**Gate:** Risk files have follow-up issues/tasks or code fixes.

---

# Phase 5: Reduce Structural Risk

## Task 5.1: Split `aidens-cli/src/lib.rs`

**Objective:** Reduce 4,996-line CLI monolith without behavior changes.

**Files:**
- Modify: `crates/aidens-cli/src/lib.rs`
- Create as needed:
  - `crates/aidens-cli/src/doctor.rs`
  - `crates/aidens-cli/src/support.rs`
  - `crates/aidens-cli/src/profile.rs`
  - `crates/aidens-cli/src/receipt_display.rs`
  - `crates/aidens-cli/src/coding_agent.rs`

**Steps:**
1. Move one cohesive section at a time.
2. Re-export only required items.
3. After each move run:
   ```bash
   cargo test -p aidens-cli --locked --all-targets
   cargo clippy -p aidens-cli --locked --all-targets -- -D warnings
   ```

**Gate:** `aidens-cli/src/lib.rs` below 2,500 lines.

## Task 5.2: Split `aidens-tool-kit/src/lib.rs`

**Objective:** Isolate registry, dispatch, permit, receipt, and repo-read logic.

**Files:**
- Modify: `crates/aidens-tool-kit/src/lib.rs`
- Create:
  - `crates/aidens-tool-kit/src/registry.rs`
  - `crates/aidens-tool-kit/src/dispatch.rs`
  - `crates/aidens-tool-kit/src/permit_gate.rs`
  - `crates/aidens-tool-kit/src/receipts.rs`
  - `crates/aidens-tool-kit/src/repo_read.rs`

**Verification:**

```bash
cargo test -p aidens-tool-kit --locked --all-targets
cargo clippy -p aidens-tool-kit --locked --all-targets -- -D warnings
```

**Gate:** `aidens-tool-kit/src/lib.rs` below 1,500 lines.

## Task 5.3: Implement or quarantine `aidens-delegation-kit`

**Objective:** Eliminate unsupported 0-test ambiguity.

**Option A — implement minimal contract:**

Files:
- Modify: `crates/aidens-delegation-kit/src/lib.rs`
- Add: `crates/aidens-delegation-kit/tests/delegation_receipts.rs`

Minimum types:
- `DelegationRequestV1`
- `DelegationDecisionV1`
- `DelegationReceiptV1`

Rules:
- deterministic receipt ID from content digest
- explicit rejection for missing scope, unsupported target, expired authority
- no random UUID/counter for material IDs

Verification:
```bash
cargo test -p aidens-delegation-kit --locked --all-targets
```

**Option B — quarantine:**
- Keep status `scaffold-only` or `partial` in `STATUS.md`.
- Add a known limitation row.
- Ensure no support claim includes it.

---

# Phase 6: Add Supported-Local Release Proof

## Task 6.1: Add one E2E receipt-bearing path

**Objective:** Prove the supported-local path with receipts, not docs.

**Files:**
- Create: `crates/aidens-integration-tests/tests/supported_local_e2e.rs`
- Optional fixture: `fixtures/supported-local/repo_read_tool_call.json`

**Scenario:**
1. Load minimal supported-local config/profile.
2. Strict-compile JSON tool call through boundary compiler.
3. Dispatch safe deterministic repo-read tool.
4. Emit boundary, tool, provider/runner, budget/permit receipts as applicable.
5. Validate receipt chain and replay determinism.

**Verification:**

```bash
cargo test -p aidens-integration-tests --test supported_local_e2e --locked -- --nocapture
```

**Gate:** Test passes and prints/captures receipt IDs.

## Task 6.2: Enforce strict material boundary policy

**Objective:** Make material executable tool/provider paths reject or degrade explicitly.

**Files:**
- Modify: `crates/aidens-boundary-kit/src/lib.rs`
- Modify: `crates/aidens-boundary-kit/src/canonical_boundary.rs`
- Modify: `crates/aidens-runner/src/provider_tool.rs`
- Modify: `crates/aidens-integration-tests/tests/conformance_run_receipt.rs`

**Tests required:**
- duplicate keys -> rejected receipt
- unknown fields -> rejected/quarantined receipt
- type coercion attempt -> rejected receipt
- resource ceiling breach -> rejected receipt
- lenient repair path -> explicit degradation/repair receipt

**Verification:**

```bash
cargo test -p aidens-boundary-kit --locked --all-targets
cargo test -p aidens-integration-tests --test conformance_run_receipt --locked
```

---

# Phase 7: Final Release Truth

## Task 7.1: Run full release gate twice

**Objective:** Prove release gate is stable and not self-poisoning.

**Commands:**

```bash
bash scripts/verify_release.sh .
python3 scripts/generate_release_gate_manifest.py --root . --run P32 --log-dir target/verify-release/P32
python3 scripts/generate_audit_log_hashes.py --root . --run P32 --out target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json
bash scripts/verify_release.sh .
```

**Gate:** Second run passes all gates, or fails only with an explicitly documented environmental replay blocker.

## Task 7.2: Update docs from evidence only

**Objective:** Prevent false completion claims.

**Files:**
- Modify: `docs/codex-runs/CURRENT_RUN.json`
- Modify: `STATUS.md`
- Modify: `SUPPORT_PROFILE.md`
- Modify: `SOURCE_BASIS.md`
- Modify: `handoffs/super-pass/FINAL_AUDITOR_HANDOFF.md`

**Rules:**
- Cite exact log/receipt paths.
- If self-replay failed, keep `extracted_replay_certified=false`.
- Do not claim v11B, v11C, production, cloud readiness, broad autonomy, or canonical semantic ownership.
- Claim only supported-local scope proven by tests/receipts.

**Verification:**

```bash
python3 scripts/assert_current_run_truth.py
python3 scripts/assert_release_truth_consistency.py
python3 scripts/assert_support_claims_have_evidence.py
python3 scripts/assert_super_pass_docs_evidence_closure.py
bash scripts/assert_no_fake_completion.sh .
```

## Task 7.3: Final clean-tree gate

**Objective:** Finish in a commit-ready state.

**Commands:**

```bash
git status --short -- .
bash scripts/verify_release.sh .
git status --short -- .
```

**Expected:** Only intentional source/docs/scripts changes are present. Generated `target/` artifacts are not accidentally staged unless policy explicitly requires them.

---

## Commit Sequence

1. `docs: add AiDENs release execution plan`
2. `test: add full AiDENs release verifier`
3. `chore: generate release and audit hash manifests`
4. `chore: generate P32 package evidence`
5. `fix: make package replay discoverable in release gate`
6. `fix: remove silent defaults from material receipt paths`
7. `fix: enforce strict material boundary policy`
8. `refactor: split aidens cli modules`
9. `refactor: split aidens tool kit modules`
10. `feat: add delegation receipts or quarantine delegation kit`
11. `test: add supported-local receipt-bearing e2e proof`
12. `docs: update P32 release truth from evidence`

---

## Final Acceptance Command

```bash
git status --short -- .
bash scripts/verify_release.sh .
LATEST_PACKAGE=$(ls -t target/p32/package/AiDENs-*.zip | head -1)
python3 scripts/assert_package_self_replay.py --package "$LATEST_PACKAGE" --expected-run P32 --receipt-out target/verify-release/P32/package_self_replay_receipt.json
python3 scripts/assert_super_pass_docs_evidence_closure.py
git status --short -- .
```

AiDENs can be called complete for P32 supported-local release only when this passes and the docs claim exactly that scope—nothing broader.
