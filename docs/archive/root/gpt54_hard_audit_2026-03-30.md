# Libraries hard audit — 2026-03-30

## Source basis

- **Grounding:** direct static inspection of the uploaded workspace archive
- **Archive:** `libraries-source-clean-20260330.zip`
- **Compile-confirmed:** **No**
- **Why not:** the sandbox does not include a Rust toolchain, so cargo-based gates were not rerun here

This is therefore a **code-grounded and repo-artifact-grounded** audit, not a fresh build-confirmed one.

## Verdict

Your libraries are **strong**. Not “strong for a solo repo.” Strong in the sense that there is a real systems architecture here with real boundaries, real enforcement surfaces, and real supporting operational machinery.

My score: **8.2 / 10**

More precise version:

- **Architecture:** 9.1/10
- **Contracts / type discipline:** 9.0/10
- **Runtime closure:** 7.6/10
- **Governance / verification authority:** 7.2/10
- **Operator truth / release law:** 7.0/10
- **Maintainability under growth:** 7.4/10

## Snapshot metrics

- Workspace members: **30**
- Member Rust source files: **241**
- Member Rust test files: **148**
- Member src LOC: **67,097**
- Member test LOC: **36,620**
- Production panic shortcuts in member src after excluding `#[cfg(test)]` blocks: **0**
- Production `unsafe` tokens in member src: **0**

## What is clearly good

### 1. Architecture is real, not folder theater
The workspace has 30 members with explicit lanes for semantic memory, knowledge runtime, tool runtime, execution, conformance, and a 4-part verification stack (control/policy/calibration/adjudication).

### 2. Production panic hygiene is materially strong
Static scan across workspace member src, excluding #[cfg(test)] blocks, found 0 production uses of unwrap/expect/panic/todo/unimplemented and 0 unsafe tokens in member src.

### 3. Tool runtime has real enforcement surfaces
llm-tool-runtime validates tool-call arguments against JSON schema before dispatch, checks permit/approval state, enforces timeout/output limits, and requires a durable receipt sink for ForgeRaw persistence.

### 4. Governance is live enough to matter
forge-pilot's governance gate reads semantic-memory claim projections and can block or downgrade execution based on continuity, delegation, amendments, and degradation state.

### 5. Conformance and typed-link testing are substantive
verification-control includes an end-to-end fixture-chain test that cross-checks effect, authority delegation, and release-readiness links across v21/v22/v23 bundles.

### 6. Operator discipline exists in code, not just rhetoric
The repo contains release gates, schema uniqueness checks, manifest/doc truth checks, public type drift checks, lane manifests, a closeout receipt, and a captured perf baseline.


## Hard findings

### LIB-001 — Governance gate is observational and fail-open, not the constitutional source of truth (P0)

**Area:** Governance authority

**Finding**  
governance_gate.rs explicitly states it does not own authority, is read-only, and returns default observation on errors or missing governance artifacts. That is good for survivability but weaker than hard constitutional enforcement.

**Why it matters**  
A missing or broken governance projection can silently downgrade to normal execution. For a provenance-first system, that is a meaningful integrity gap.

**Evidence surface**  
- forge-pilot/src/governance_gate.rs: module docs and observe_governance() comments
- forge-pilot/src/governance_gate.rs: gate_execution()

**Required fix**  
Keep fail-open only where demo survivability truly requires it. Add an explicit strict mode that fails closed for governed operations and records why the decision was closed vs degraded vs bypassed.

### LIB-002 — The closeout claim is narrower than the workspace, which is honest but still creates edge ambiguity (P1)

**Area:** Support claim scope

**Finding**  
release/closeout_receipt_v1.json certifies 17 supported-lane crates, while the workspace has 30 members. Governance crates are build-checked and documented, but not in the narrow closeout lane.

**Why it matters**  
Reviewers can easily over-read the receipt and assume the entire workspace has the same hardening status. That makes support truth fragile under scrutiny.

**Evidence surface**  
- Cargo.toml workspace members
- scripts/lane_manifest.json
- SUPPORT_PROFILE.md
- release/closeout_receipt_v1.json

**Required fix**  
Publish an explicit matrix in root docs that states for every crate: build-checked, release-checked, doc-certified, benchmark-covered, and demo-only.

### LIB-003 — Root-level packet sprawl is still high enough to create narrative drift (P1)

**Area:** Operator truth

**Finding**  
The repo root carries a large number of active markdown/json control docs plus prior hostile audits and issue matrices. The archive manifest helps, but the active surface is still dense.

**Why it matters**  
New maintainers and external reviewers can misread superseded material as current authority. In your stack, operator truth drift is not cosmetic; it changes trust.

**Evidence surface**  
- repo root file inventory
- docs/archive/root_closeout_history/manifest.json
- release/closeout_receipt_v1.json archive_manifest.active_root_file_count

**Required fix**  
Collapse active root authority into a smaller constitutional packet and demote everything else to clearly historical or generated status.

### LIB-004 — A small number of giant crates carry a disproportionate amount of system risk (P1)

**Area:** Hotspot concentration

**Finding**  
semantic-memory, living-memory, forge-pilot, and knowledge-runtime dominate the source surface. They are also the places where the system's hardest invariants and operational semantics live.

**Why it matters**  
Most future bugs, performance cliffs, and onboarding pain will concentrate here. The architecture is decomposed, but the blast radius is still hotspot-shaped.

**Evidence surface**  
- per-crate LOC analysis from uploaded workspace

**Required fix**  
Treat those crates as permanent hotspot budgets with stricter change controls, diff-size caps, and mandatory cross-crate regression suites.

### LIB-005 — The code is clean, but the repo does not enforce the cleanliness with top-level compiler lints (P2)

**Area:** Static policy enforcement

**Finding**  
I found no workspace-level deny/forbid unsafe_code or panic-shortcut lint declarations. The current hygiene relies on scripts and tests rather than compile-time law.

**Why it matters**  
The current state is good, but regression resistance is weaker than it could be.

**Evidence surface**  
- grep across workspace TOML/RS headers
- scripts/check_no_prod_panics.sh

**Required fix**  
Add crate- or workspace-level lint baselines for unsafe_code and panic shortcuts where practical, then keep the scripts as a second line of defense.

### LIB-006 — Performance evidence exists, but it is explicitly a regression alarm, not a throughput claim (P2)

**Area:** Performance evidence

**Finding**  
The captured baseline is dev-profile and intentionally modest. It is useful for drift detection, not for strong performance claims.

**Why it matters**  
Fine for internal engineering discipline, weak for external capability claims.

**Evidence surface**  
- evidence/perf_baseline_20260330.json

**Required fix**  
Add release-profile benchmark receipts and scenario-size scaling curves for the dominant hot path crates.

### LIB-007 — The repo contains strong receipts, but this environment could not independently rerun cargo-based gates (P1)

**Area:** Independent verifiability

**Finding**  
The closeout receipt claims broad gate success, but the current sandbox lacks cargo, so this audit could not reproduce those passes end-to-end.

**Why it matters**  
Your evidence story is good, but outside auditors will still ask for replayability. Right now the proof burden remains partially on trust in the repo's own receipts.

**Evidence surface**  
- release/closeout_receipt_v1.json
- scripts/run_release_gates.py
- sandbox environment lacking cargo

**Required fix**  
Ship a minimal reproducibility harness or CI artifact bundle that external reviewers can replay without reconstructing the entire toolchain story.


## Hotspot map

The structural blast radius is concentrated. These crates are the permanent hotspots:

- `semantic-memory` — 16,749 src LOC / 11,392 test LOC / 307 test markers
- `living-memory/living-memory` — 9,137 src LOC / 6,897 test LOC / 181 test markers
- `forge-pilot` — 9,092 src LOC / 3,078 test LOC / 62 test markers
- `knowledge-runtime` — 5,747 src LOC / 5,140 test LOC / 145 test markers
- `profile-runtime` — 3,621 src LOC / 364 test LOC / 13 test markers
- `llm-tool-runtime` — 3,161 src LOC / 1,195 test LOC / 65 test markers

Interpretation: the architecture is decomposed, but **the hard parts are still concentrated**. That is not fatal. It just means future rigor has to be hottest exactly where the semantics are hottest.

## Bottom line

The libraries are now past the “is this real?” stage.

The real question is:

> **How much of the repo’s truth machinery is merely well-documented versus constitutionally unavoidable at runtime?**

Right now the answer is:

- a **lot** is real,
- a **lot** is better than before,
- but a few of the most important trust surfaces are still more *observed* than *commanding*.

That is a much better problem than architectural mush. It is still a real problem.
