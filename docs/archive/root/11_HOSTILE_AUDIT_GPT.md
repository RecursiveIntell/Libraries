# Hostile Audit — 10 Lenses + Tensor

## Scope and method

This audit is grounded in **static inspection of the uploaded source archive**. It is not compile-confirmed because the execution environment did not include a Rust toolchain. The result is therefore **code- and artifact-grounded**, not build-confirmed.

## Top-line verdict

The libraries are **architecturally strong** and **operationally messy**.

That is materially better than the reverse.

The dominant risk is not that the code is unserious. The dominant risk is that the repo's **truth machinery, front door, governance artifacts, and release lane are no longer perfectly synchronized with the implementation**. In this stack, that matters more than usual because self-consistency is part of the product claim.

---

## Tensor summary

Scale: **0 = clean**, **5 = severe risk**

Axes:

- **Truth** = repo says contradictory things
- **Code** = structural/code-shape risk
- **Ops** = release/gate/operator risk
- **Scale** = performance/hotspot growth risk
- **Sec** = security/capability risk
- **Human** = maintainer/onboarding/oncall pain

### Current severity

| Lens | Truth | Code | Ops | Scale | Sec | Human |
|---|---:|---:|---:|---:|---:|---:|
| Constitutional truth auditor | 5 | 3 | 5 | 2 | 1 | 5 |
| Cynical architect | 2 | 2 | 2 | 2 | 1 | 2 |
| Blast-radius pessimist | 2 | 5 | 4 | 4 | 2 | 4 |
| Failure-semantics sadist | 3 | 3 | 3 | 2 | 2 | 2 |
| Sleep-deprived oncall engineer | 3 | 3 | 5 | 3 | 2 | 4 |
| Contract/schema lawyer | 4 | 3 | 4 | 2 | 1 | 3 |
| Evidence scientist | 4 | 2 | 4 | 3 | 1 | 3 |
| Security goblin | 2 | 2 | 3 | 2 | 3 | 2 |
| New maintainer from hell | 4 | 4 | 3 | 2 | 1 | 5 |
| DARPA demo reviewer | 5 | 3 | 5 | 3 | 2 | 4 |

**Mean current severity:** **2.95 / 5**

### Severity in 90 days if ignored

| Lens | Truth | Code | Ops | Scale | Sec | Human |
|---|---:|---:|---:|---:|---:|---:|
| Constitutional truth auditor | 5 | 4 | 5 | 3 | 2 | 5 |
| Cynical architect | 2 | 3 | 3 | 3 | 2 | 3 |
| Blast-radius pessimist | 3 | 5 | 5 | 5 | 3 | 5 |
| Failure-semantics sadist | 4 | 4 | 4 | 3 | 3 | 3 |
| Sleep-deprived oncall engineer | 4 | 4 | 5 | 4 | 3 | 5 |
| Contract/schema lawyer | 5 | 4 | 5 | 3 | 2 | 4 |
| Evidence scientist | 5 | 3 | 5 | 4 | 2 | 4 |
| Security goblin | 3 | 3 | 4 | 3 | 4 | 3 |
| New maintainer from hell | 5 | 5 | 4 | 3 | 2 | 5 |
| DARPA demo reviewer | 5 | 4 | 5 | 4 | 3 | 5 |

**Mean severity if ignored:** **3.80 / 5**

### Tensor decomposition

The largest eigenvectors are operationally obvious:

1. **Ops / release truth**
2. **Truth drift**
3. **Human maintainability**

Security is not the main pain right now. Architecture is not the main pain right now. The main pain is that the repo has started acting like a courthouse where three clerks are filing under different calendars.

---

## Ten-lens hostile audit

### 1. Constitutional truth auditor

**Verdict:** this is the biggest problem.

The repo is telling multiple versions of the story at once.

- `README.md` still frames the pack as `finish_pack_2026-03-24` over `libraries-source-clean-20260323.zip`.
- `00_START_HERE.md` indicates a newer V28 state effective 2026-03-30.
- `01_MASTER_ISSUE_MATRIX.md`, `SOURCE_BASIS.md`, `STATUS_DASHBOARD.md`, `PACK_MANIFEST.json`, and related active docs still reference older snapshots and dates.
- `STATUS_EVIDENCE_MANIFEST.json` references `docs/archive/root_closeout_history/manifest.json`, but that file is absent and `scripts/check_root_archive_manifest.py` fails.

**What it needs:** one authoritative source basis and one authoritative front door. Everything else should derive from it or be archived.

### 2. Cynical architect

**Verdict:** the macro-architecture is real and mostly good.

Observed signals:

- Workspace `Cargo.toml` has **30 members/default members**.
- The pack contains **45 Cargo packages** total.
- No workspace path-dependency cycles were detected.
- The split across `semantic-memory`, `knowledge-runtime`, `kernel-*`, `verification-*`, `forge-*`, `stack-ids`, and governance crates reads like designed system architecture rather than a blob.

**What it needs:** preserve the crate boundaries. The problem is inside some oversized modules, not in the workspace topology.

### 3. Blast-radius pessimist

**Verdict:** hotspot concentration is real and growing.

Large files observed:

- `profile-runtime/src/adapters.rs` — **1791** lines
- `semantic-memory/src/db.rs` — **1608**
- `semantic-memory/src/lib.rs` — **1593**
- `forge-pilot/src/main_support/mod.rs` — **1591**
- `semantic-memory/src/search.rs` — **1586**
- `verification-control/src/lib.rs` — **1580**
- `llm-tool-runtime/src/runtime.rs` — **1399**
- `knowledge-runtime/src/runtime/core.rs` — **1209**
- `forge-pilot/src/loop_runner.rs` — **1062** and already over the hotspot budget

`scripts/check_hotspot_budgets.sh` fails specifically on `forge-pilot/src/loop_runner.rs`.

**What it needs:** split by responsibility, not by arbitrary line count.

### 4. Failure-semantics sadist

**Verdict:** mixed picture; part real risk, part checker drift.

Good:

- No `todo!` or `dbg!` surfaced in `src`.
- `unsafe` usage in source is tiny: **4 hits**, all under `Primitives/check-runner`.

Bad:

- `scripts/check_no_prod_panics.sh` flags `semantic-memory/src/pool.rs`, but the matched `unwrap!/panic!` lines are inside a `#[cfg(test)] mod tests`.
- That means the production-panic checker is not accurately distinguishing test-only code.
- There are still many `unwrap/expect` hits in `src`, though a meaningful subset are test code embedded in source files and utility surfaces.

**What it needs:** make the panic/unwrap audit AST-aware or at least `cfg(test)`-aware.

### 5. Sleep-deprived oncall engineer

**Verdict:** operator experience is rougher than the code quality.

Good:

- There is a real front door: `make gate`.
- There is a real gate list in `scripts/release_gate_set.py`.

Bad:

- Several gates fail for reasons that are not cleanly “the code is broken”:
  - `check_doc_truth.sh`
  - `check_public_api_docs.py`
  - `check_commit_permit_paths.py`
  - `check_root_archive_manifest.py`
  - `check_hotspot_budgets.sh`
  - `check_no_prod_panics.sh` likely false-positive
- `check_commit_permit_paths.py` expects `ExecutionPermit`, while `llm-tool-runtime/src/runtime.rs` now uses `ToolExecutionPermit`.

**What it needs:** every red gate should map cleanly to one bucket: real regression, stale checker, missing artifact, or outdated doc. Right now the gate lane has too much epistemic noise.

### 6. Contract/schema lawyer

**Verdict:** strong bones, weak truth reconciliation.

Good:

- `check_schema_registry_uniqueness.sh` passes.
- `check_public_type_drift.py` passes with **0 allowlisted duplicates**.

Bad:

- `check_public_api_docs.py` fails.
- `SUPPORT_PROFILE.md` claims a public-doc-certified core, but the script reports gaps:
  - missing governance surface decision-table entries for `discovery-portfolio`, `federated-settlement`, `spec-execution`
  - demoted compatibility-name crates still showing **0/N** public docs

**What it needs:** either finish the compatibility-crate doc/governance story or narrow the claims until they are mechanically true.

### 7. Evidence scientist

**Verdict:** strong correctness evidence, thinner performance evidence.

Good:

- Roughly **153 test files**
- property-contract style testing presence
- fixtures and adversarial testing patterns

Less good:

- No strong benchmark surface was found in the pack
- performance proof appears script-driven rather than surfaced as a canonical current artifact

**What it needs:** ship visible canonical performance evidence, not only the machinery that could generate it.

### 8. Security goblin

**Verdict:** surprisingly decent.

Good:

- `forge-policy` includes path normalization checks, symlink rejection, relative-path enforcement, and DB identity checks.
- Environment passthrough is allowlisted.
- `unsafe` surface is tiny.

Concern:

- the stack performs host execution in some paths, so permit discipline has to stay airtight
- checker drift around execution permits could eventually become a security-story problem if semantics and enforcement drift apart

**What it needs:** end-to-end proof that declared permit semantics, runtime enforcement, and checker expectations all name the same thing.

### 9. New maintainer from hell

**Verdict:** a fresh maintainer will lose time in document archaeology.

At repo root alone, there are dozens of markdown and JSON control artifacts, many representing superseded or overlapping authorities:
- multiple issue matrices
- multiple playbooks
- multiple source-basis docs
- multiple status or risk summaries

**What it needs:** a hard archive policy with one active dashboard, one active issue matrix, one active source basis, and one active playbook.

### 10. DARPA demo reviewer

**Verdict:** the code reads as credible systems work; the proof lane still wobbles.

External-facing reaction would likely be:

> This is clearly real systems engineering.  
> Now show one clean, current, mechanically truthful release story.

**What it needs:** less rhetoric, more single-pass truth: current README, current basis, current evidence ledger, current receipt, passing gate lane, and no stale snapshots in active docs.

---

## Hard evidence behind the audit

Load-bearing evidence points from the inspection:

- Workspace `Cargo.toml` exposes **30 members/default members**
- Pack contains **45 Cargo packages**
- Roughly **415 Rust files**
- Roughly **123,715 lines of Rust**
- Roughly **153 test files**
- **0** detected workspace path-dependency cycles
- **4** `unsafe` hits in source, all under `Primitives/check-runner`
- Gate failures observed:
  - `check_doc_truth.sh`
  - `check_public_api_docs.py`
  - `check_commit_permit_paths.py`
  - `check_root_archive_manifest.py`
  - `check_hotspot_budgets.sh`
  - `check_no_prod_panics.sh` likely false-positive on test code
- `README.md` is materially stale relative to `00_START_HERE.md`
- `STATUS_EVIDENCE_MANIFEST.json` references a missing archive manifest
- The pack includes `target-*` directories, which is archive pollution

---

## Hostile conclusion

The stack is not failing because the architecture is weak.

It is failing because the **meta-layer** is lagging:
- repo truth
- gate truth
- source-basis truth
- release-pack truth
- front-door truth

Brutal summary:

> The libraries look like serious software. The repo packaging still looks like it was assembled by three versions of the same person arguing across time.

---

## Immediate priorities

1. **Unify active truth**
   - rewrite `README.md`, `SOURCE_BASIS.md`, `STATUS_DASHBOARD.md`, `PACK_MANIFEST.json`, and the active issue docs so they all point to the same snapshot and date

2. **Fix stale gate logic**
   - update `check_commit_permit_paths.py` for `ToolExecutionPermit`
   - make `check_no_prod_panics.sh` ignore `#[cfg(test)]` modules correctly

3. **Restore missing archive artifact**
   - add or regenerate `docs/archive/root_closeout_history/manifest.json`

4. **Resolve the compatibility-crate truth story**
   - either fully document them and update governance tables, or narrow claims

5. **Split hotspot files**
   - especially `forge-pilot`, `semantic-memory`, `profile-runtime`, `verification-control`, `llm-tool-runtime`

6. **Run the proof triad locally**
   - `cargo check --workspace`
   - `cargo test --workspace`
   - `cargo clippy --workspace -- -D warnings`
   - then regenerate the evidence ledger and receipt from that exact state

## Deliverables in this bundle

- `hostile_audit_10_lenses.md`
- `hostile_audit_tensor.json`
- `hostile_audit_tensor_current.csv`
- `hostile_audit_tensor_future_90d_if_ignored.csv`
- `hostile_audit_summary.txt`
