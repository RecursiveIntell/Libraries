# P27 11A Alignment — Proof-Governed Semantic Guardrail

The operator requested that this super-pass follow the latest 11A-style spec direction. P27 should use 11A as a **semantic guardrail**, not as permission to implement the entire post-v10 runtime.

## 11A principles applied to P27

### 1. Explicit semantic carrier

Any evidence-bearing P27 output must say what kind of thing it is:

- local operator evidence;
- canonical backpointer;
- display-only abstention/repair note;
- verification receipt;
- durable run receipt;
- advisory score;
- exact check result;
- approximate/degraded result.

### 2. Exact/approx correspondence

If a path is approximate, heuristic, fixture-backed, or display-only, label it. Approximate paths may prioritize, suggest, or display. They may not promote truth.

### 3. Proof/check obligations

For each new or touched artifact family, record the cheapest check that can refute it:

- schema validation;
- digest check;
- replay recipe;
- cargo test/check;
- patch dry-run;
- permit receipt;
- canonical owner backpointer;
- duplicate-key rejection;
- support-tier assertion.

### 4. Repair law

Repair/abstention artifacts are allowed as AiDENs-local display/operator evidence. They must not become canonical repair truth unless routed through canonical owners.

### 5. Learned/advisory boundary

Provider/model outputs are advisory until verified. Coding-agent proposals are not accepted patches until permit, patch check, application, and verification receipts exist.

### 6. Reference semantics

P27 should not attempt to build the full reference interpreter. It should create hooks and labels that make later reference-semantics work possible:

- stable artifact schemas;
- replay recipes;
- exact/approx labels;
- failure taxonomy;
- receipt store;
- verifier entrypoints.

## Forbidden 11A misuse

Do not write speculative V11/V12 claims into support docs. Do not implement regional fixpoint runtime, hypergraph kernel, federation, or mechanism search as part of P27 unless explicitly authorized as a separate stretch pass.
