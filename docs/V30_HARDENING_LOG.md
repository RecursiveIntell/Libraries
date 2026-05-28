# Libraries V30 Hardening — Append-Only Execution Log

**Start:** 2026-05-27T00:00 UTC
**Basis:** `LIBRARIES_HARDENING_AND_GAP_AUDIT_2026-05-27.md`
**Plan:** `LIBRARIES_V30_HARDENING_ROADMAP.md`

---

## Phase 0 — Pre-flight & Truth Recovery

### P0-1: Commit all uncommitted files

**Timestamp:** 2026-05-27T00:00 UTC
**Status:** IN PROGRESS

**Action:** Restore deleted docs + stage all dirty tracked files + commit.

```
cd ~/Coding/Libraries
# Recover deleted docs from HEAD
git checkout HEAD -- 02_MASTER_ISSUE_MATRIX.md 06_RISK_REGISTER.md
# Stage all modified tracked files
git add -A
git commit -m "chore: V30 pre-hardening salvage commit — no feature changes
- Restores deleted issue matrix and risk register
- 178 files, +15708/-4306 lines
- All test/conformance/autography fixes from prior session
- No crate-altering changes"
```

**Verification:**
- `git status` → clean working tree
- `git log --oneline -1` → V30 pre-hardening salvage commit

---

### P0-2: Restore deleted tracking docs + fork audit as live artifact

**Timestamp:** 2026-05-27T00:00 UTC
**Status:** PENDING

**Action:**
- Create `LIBRARIES_V30_HARDENING_ROADMAP.md` (done — this spec)
- Copy `LIBRARIES_HARDENING_AND_GAP_AUDIT_2026-05-27.md` → `LIBRARIES_V30_AUDIT_WORKING.md`
- Add P0/P1/P2 tracker section to working doc

---

## Phase 1 — Workspace Integrity

### P1-1: Fix unsafe in Primitives/check-runner

**Timestamp:** PENDING
**Status:** PENDING

**Action:**
- Read `Primitives/check-runner/src/lib.rs`
- Identify 4 unsafe blocks (process forking, libc::kill, signal handling)
- Path A: Replace with safe wrappers (std::process::Command, owned stdin/stdout)
- Path B: If unsafe is structural, add scoped `[lints]` override with documentation
- Verify: `cargo check --workspace` + `cargo test -p Primitives/check-runner`

---

### P1-2: Replace panic! in knowledge-runtime and kernel-oracles

**Timestamp:** PENDING
**Status:** PENDING

**Action:**
- `knowledge-runtime/src/query/classify.rs` → replace 3 `panic!` with `thiserror`
- `kernel-oracles/src/lib.rs` → replace 2 `panic!` with `thiserror`
- Add error variants for "unexpected enum variant at runtime"
- Verify: `grep -rn "panic!" knowledge-runtime/src kernel-oracles/src | grep -v test` returns nothing

---

### P1-3: Add tests to all 10 Primitives

**Timestamp:** PENDING
**Status:** PENDING

**Priority order:** typed-patch → check-runner → cea-sqlite → cea-store → stabilizer-core → sandbox-workspace → forge-policy → mindstate-core → effect-signature → cea-core

**Action per crate:**
- Create `tests/` directory
- Add one integration test with roundtrip: serialize → deserialize → assert
- Run `cargo test -p <crate>` to confirm

---

## Phase 2 — Missing Core Crates

### P2-1: boundary-compiler (RFC 8785 JCS)

**Timestamp:** PENDING
**Status:** PENDING

### P2-2: bitemporal-runtime

**Timestamp:** PENDING
**Status:** PENDING

### P2-3: quant-governor

**Timestamp:** PENDING
**Status:** PENDING

### P2-4: claim-ledger

**Timestamp:** PENDING
**Status:** PENDING

### P2-5: agent-guard

**Timestamp:** PENDING
**Status:** PENDING

### P2-6: receipt-bench, scr-runtime-compression, quant-eval

**Timestamp:** PENDING
**Status:** PENDING

---

## Phase 3 — Doctrinal Compliance

### P3-1: HNSW approximate marking
### P3-2: Episode supersession
### P3-3: Turbo-quant wiring into semantic-memory
### P3-4: Fib-quant wiring
### P3-5: Fibquant KV receipts wired
### P3-6: PolyKV workspace merge
### P3-7: LLM pipeline execution receipts
### P3-8: Governance crates emit receipts
### P3-9: Turbo-quant memory accounting
### P3-10: tauri-queue resolved or removed

---

## Phase 4 — Verification & Benchmark

### P4-1: Full test suite run
### P4-2: Benchmark run under release profile
### P4-3: Performance baselines and scaling curves
### P4-4: Doctrinal conformance self-attestation
### P4-5: Reproduction bundle for external auditors

---

## Phase 5 — Final Audit & Closeout

### P5-1: V31 hostile audit
### P5-2: Closeout receipt
### P5-3: Promote to canonical
