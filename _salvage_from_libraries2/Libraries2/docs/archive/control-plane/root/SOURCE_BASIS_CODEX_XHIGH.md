# Source Basis — Codex XHigh

This document explains **what was treated as truth** when building the XHigh control plane.

## 1. Current implementation truth

For questions about **what the code currently does**, the primary source of truth is the **current library snapshot**:

- `libraries-source.zip` extracted into the audit workspace
- all `Cargo.toml` files
- all Rust source files
- all test files
- all examples

If older docs disagree with current code, the code wins for **current-state assessment**.

## 2. Architectural law

For questions about **what the architecture is supposed to forbid or preserve**, the primary source is:

- `CANONICAL_STACK_SPEC_V5.md`

That document is authoritative for boundary law:
- authority split,
- canonical import path,
- evidence opacity by default,
- temporal / scope semantics as target law,
- runtime non-authority.

If current code violates the canonical spec, the issue matrix should treat that as real debt, not as a new norm.

## 3. Active control-plane truth

For questions about **what Codex XHigh should do next**, the active source is **this bundle**:

- `AGENTS.md`
- `MASTER_ISSUE_MATRIX_CODEX_XHIGH.*`
- `CODEX_XHIGH_EXECUTION_MAP.md`
- `CODEX_XHIGH_ACCEPTANCE_CHECKLIST.md`

These supersede older V6/V7 control docs for execution ordering.

## 4. Historical context and comparison baselines

These were used as historical context, comparison points, or prior research framing:

- `codex_master_control_plane_v7/*`
- `brutally_honest_completion_matrix.md`
- `CANONICAL_STACK_SPEC_V5.md`
- `deep-research-report.md`
- `priority_matrix_research.md`
- `causal-research.md`
- `causal2.md`
- `bitemporal_research.md`
- `bitemp+_research.md`
- `temporal_truth_research.md`
- related research notes in `/mnt/data`

These help explain **why** the issue ordering changed, but they do not outrank the current code snapshot or the canonical spec for present-state truth.

## 5. How conflicts were resolved

Use this rule:

### A. For current behavior
`current code > older docs`

### B. For architectural non-negotiables
`canonical spec > current convenience > older docs`

### C. For execution priority
`XHigh bundle > V7/V6 bundle > historical notes`

## 6. Audit limitations

This was a **static audit**.

That means:
- file inventory was exhaustive,
- source/test/manifests were read and analyzed,
- issue scoring reflects code/doc/package shape,
- but no full build/test execution happened here.

Any claim about runtime correctness that is not statically obvious should be treated as:
- **implemented by source inspection**
- **not yet build-verified in this environment**

## 7. Practical instruction for Codex XHigh

When implementing:
1. trust the current source tree for what exists,
2. trust the canonical spec for what must not be violated,
3. trust the XHigh issue matrix for what to do next,
4. treat older control docs as historical unless the XHigh bundle points back to them deliberately.
