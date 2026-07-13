# AiDENs Hostile-Audit Remediation Plan

> **For Hermes:** Execute controller-owned TDD and verification; do not expand quarantined or non-goal capability work.

**Goal:** Make current AiDENs release truth reproducible, repair the stale Phase 08 canonical-boundary fixture, and harden the compatibility-ledger gate.

**Architecture:** Preserve canonical owner behavior. The bridge continues to reject claims lacking canonical IDs. Root documentation becomes a concise mirror of `docs/codex-runs/CURRENT_RUN.json`; it is not a historical run catalog. The ownership gate recognizes an empty Markdown table correctly and still rejects actual shim rows.

**Tech stack:** Rust 2021/Cargo, Python 3, Bash, current AiDENs gates.

## Evidence-backed current state

- Target root was clean at audit start: `feat/full-integration`, `9d73fae1204062e24beb284c9972779ad69cded6`.
- `cargo metadata --no-deps --format-version 1` passed and reports 37 members.
- `bash scripts/verify_current.sh` failed at `current_run_truth`: missing `SUPPORT_PROFILE.md`; root status lacked P32/P30/current-ledger references and included prohibited historical IDs.
- `cargo test -p aidens-integration-tests --test phase_08_kernel_oracle_integration --locked --no-fail-fast` failed: 2/2 tests panic on `MissingCanonicalClaimId` because the fixture has `claim_id: None`.
- `bash scripts/assert_no_compatibility_ledgers.sh` failed even though the contract ledger contains only Markdown header/separator rows.
- Audit document: `docs/HOSTILE_AUDIT_2026-07-13.md`.

## Scope boundary

**Must fix:** HA-001 through HA-005 in the audit matrix.  
**Do not do:** provider expansion, new daemon/scheduler/kernel/UI capabilities, creating compatibility shims, weakening canonical bridge admission, or activating the quarantined delegation kit.  
**Certification boundary:** do not call P32 certified until the fresh command bar passes.

---

## Phase 1 — P0: Canonical fixture correctness

### Task 1.1: Require a real canonical claim ID in the Phase 08 test fixture

**Files:**
- Modify: `crates/aidens-integration-tests/tests/phase_08_kernel_oracle_integration.rs:11,21`

**Step 1 — RED:** Existing receipt is the focused failing test:

```bash
cargo test -p aidens-integration-tests --test phase_08_kernel_oracle_integration --locked --no-fail-fast
```

Expected: both tests fail with `MissingCanonicalClaimId`.

**Step 2 — GREEN:** Import `ClaimId` from `stack_ids` and make `claim_id` deterministic from namespace and suffix:

```rust
claim_id: Some(ClaimId::new(format!("claim-{namespace}-{suffix}"))),
```

Do not create IDs in `forge-memory-bridge` or `aidens-kernel-kit`.

**Step 3 — Verify:** rerun the exact command; expected 2 passed.

**Phase gate:** `cargo test -p aidens-integration-tests --test phase_08_kernel_oracle_integration --locked --no-fail-fast`.

## Phase 2 — P0/P1: Release truth restoration

### Task 2.1: Decertify stale current-run claims before repairing gates

**Files:**
- Modify: `docs/codex-runs/CURRENT_RUN.json`

Set the source-of-truth status to blocked/non-certified and build certification false until this remediation’s fresh command bar is complete. Preserve historical evidence as historical—not current proof.

**Verify:** JSON parses and README/status/support all mirror the resulting active and last-certified IDs.

### Task 2.2: Create a concise support profile

**Files:**
- Create: `SUPPORT_PROFILE.md`

State active run, last certified run, candidate/block status, supported-local boundary, non-claims, and explicit quarantines. Cite `CURRENT_RUN.json`. Do not include a historic run catalog.

### Task 2.3: Replace stale root status catalog with a current truth mirror

**Files:**
- Modify: `STATUS.md`

Remove historical P00–P19 catalog and claims that disabled delegation is implemented. Keep only current run truth, concise crate posture, active support limitations, and pointers to historical artifacts.

### Task 2.4: Make `CURRENT_RUN.md` a valid concise mirror

**Files:**
- Modify: `docs/codex-runs/CURRENT_RUN.md`

Cite the JSON ledger and use only active, last-certified, and parent run IDs.

### Task 2.5: Fix README active links and non-claims

**Files:**
- Modify: `README.md`

Remove the unreproducible “all gates pass” claim, link only files that exist, and state that current verification is blocked pending the remediation command bar.

**Phase gate:**

```bash
python3 scripts/assert_current_run_truth.py
bash scripts/verify_current.sh
```

Expected: current-run truth passes; `verify_current.sh` may proceed to remaining gates.

## Phase 3 — P1: Compatibility-gate correctness

### Task 3.1: Add a testable parser for compatibility data rows

**Files:**
- Create: `scripts/tests/test_assert_no_compatibility_ledgers.py`
- Modify: `scripts/assert_no_compatibility_ledgers.sh`

**Step 1 — RED:** Add Python subprocess tests that construct a temporary minimal repo with: (a) header + separator only and expect exit 0; (b) one data row and expect exit 1.

**Step 2 — GREEN:** Change the shell gate to count rows only after the Markdown table separator. Preserve its existing file-name scan for `compat`, `shim`, and `legacy` files in `aidens-contracts`.

**Step 3 — Verify:**

```bash
python3 -m unittest scripts/tests/test_assert_no_compatibility_ledgers.py
bash scripts/assert_no_compatibility_ledgers.sh
```

## Phase 4 — Documentation claim closure

### Task 4.1: Mark stale P20 audit as historical

**Files:**
- Modify: `docs/CURRENT_AIDENS_AUDIT.md`

Put a first-line historical/superseded notice pointing to the 2026-07-13 audit and current-run ledger. Do not rewrite historical observations.

### Task 4.2: Cross-surface closure scan

Search active root docs for `P00`–`P29`, `all gates pass`, non-existent `BUILD_SCOPE.md`, and delegation-as-implemented language. Every remaining hit must be a deliberately historical/archive file.

**Phase gate:** root documentation scans return no active stale claim.

## Phase 5 — Final verification and evidence

Run sequentially, record exit status and output under `.codex_evidence/contract_ownership/hostile-audit-2026-07-13/`:

```bash
cargo fmt --all -- --check
bash scripts/assert_no_compatibility_ledgers.sh
bash scripts/phase_verify_contract_ownership.sh final
bash scripts/verify_current.sh
cargo check --workspace --locked --all-targets
cargo test --workspace --locked --all-targets --no-fail-fast
cargo clippy --workspace --locked --all-targets -- -D warnings
```

Only after all pass may `CURRENT_RUN.json` be advanced from blocked to candidate. If any command fails, retain the blocked state and append the exact failed command to `docs/HOSTILE_AUDIT_2026-07-13.md`.

## Final claim boundary

A green command bar proves current local source/gates and tests. It does **not** prove production, cloud readiness, broad autonomy, external federation, or activation of quarantined delegation helpers.
