# MASTER_ISSUE_MATRIX_FORGE_PILOT.md
## Overview
| Metric | Value |
|---|---:|
| Total issues | 21 |
| P0 issues | 11 |
| P1 issues | 8 |
| P2 issues | 2 |
| Phase span | 0–5 |
| Goal | closed-loop orchestrator, not a new authority plane |

## Priority view

| ID | Priority | Phase | Stream | Title | Status |
|---|---|---|---|---|---|
| SURF-001 | P0 | Phase 0 | Surface | Add the forge-pilot crate and default feature-safe workspace wiring | PLANNED |
| ARCH-001 | P0 | Phase 0 | Architecture | Enforce forge-pilot as a consumer-only orchestrator over existing v6/v7 planes | PLANNED |
| OBS-001 | P0 | Phase 1 | Observe | Reconstruct latest kernel context from public projection import logs and kernel payload JSON | PLANNED |
| OBS-002 | P1 | Phase 1 | Observe | Compute scope health summary from public projection query APIs | PLANNED |
| OBS-003 | P1 | Phase 1 | Observe | Lift runtime advisory/explanation/risk gate outputs into the pilot observation model | PLANNED |
| TGT-001 | P0 | Phase 2 | Targeting | Define target taxonomy, stable target keys, and dedupe/exhaustion policy | PLANNED |
| TGT-002 | P1 | Phase 2 | Targeting | Implement urgency scoring, retry decay, and deterministic tie-break rules | PLANNED |
| DEC-001 | P0 | Phase 2 | Decision | Implement deterministic candidate selection and halt threshold behavior | PLANNED |
| DEC-002 | P2 | Phase 5 | Decision | Add optional feature-gated LLM refinement for plan text and check hints only | PLANNED |
| ACT-001 | P0 | Phase 3 | Act | Implement kernel-oracle plan execution | PLANNED |
| ACT-002 | P1 | Phase 3 | Act | Implement paired patch plan execution via PairedExperimentRunner | PLANNED |
| ACT-003 | P0 | Phase 3 | Act | Build a local EvidenceBundle builder for oracle and experiment outputs | PLANNED |
| EXP-001 | P0 | Phase 3 | Export | Implement canonical V3 export -> bridge -> import roundtrip | PLANNED |
| HIST-001 | P1 | Phase 4 | Loop | Implement in-memory target history, retry accounting, and exhaustion rules | PLANNED |
| LOOP-001 | P0 | Phase 4 | Loop | Implement loop runner budgets, cooldown, halt reasons, and report surfaces | PLANNED |
| LOOP-002 | P1 | Phase 4 | Loop | Expose CLI / JSON report surfaces | PLANNED |
| TEST-001 | P0 | Phase 5 | Testing | Add unit tests for observation, targeting, scoring, and tie-breaks | PLANNED |
| TEST-002 | P0 | Phase 5 | Testing | Add end-to-end oracle and paired-patch roundtrip tests | PLANNED |
| TEST-003 | P1 | Phase 5 | Testing | Add thin-export and missing-kernel-payload degradation tests | PLANNED |
| PERF-001 | P2 | Phase 5 | Performance | Add bounded performance smoke tests for loop budgets and no-runaway behavior | PLANNED |
| DOC-001 | P1 | Phase 5 | Docs | Refresh root surface docs only after the crate is real and tested | PLANNED |

## Full matrix

### SURF-001 — Add the forge-pilot crate and default feature-safe workspace wiring

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 0 |
| Stream | Surface |
| Status | PLANNED |
| Acceptance | Crate builds without optional LLM dependencies; workspace wiring is truthful. |

### ARCH-001 — Enforce forge-pilot as a consumer-only orchestrator over existing v6/v7 planes

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 0 |
| Stream | Architecture |
| Status | PLANNED |
| Acceptance | No new schema, no bridge invention, no direct truth writes. |

### OBS-001 — Reconstruct latest kernel context from public projection import logs and kernel payload JSON

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 1 |
| Stream | Observe |
| Status | PLANNED |
| Acceptance | Deterministic observation from public APIs. |

### OBS-002 — Compute scope health summary from public projection query APIs

| Field | Value |
|---|---|
| Priority | P1 |
| Phase | Phase 1 |
| Stream | Observe |
| Status | PLANNED |
| Acceptance | ScopeHealthSummary exists and is covered by tests. |

### OBS-003 — Lift runtime advisory/explanation/risk gate outputs into the pilot observation model

| Field | Value |
|---|---|
| Priority | P1 |
| Phase | Phase 1 |
| Stream | Observe |
| Status | PLANNED |
| Acceptance | Observation includes advisory + explanation + risk gate surfaces. |

### TGT-001 — Define target taxonomy, stable target keys, and dedupe/exhaustion policy

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 2 |
| Stream | Targeting |
| Status | PLANNED |
| Acceptance | Targets are deterministic and retry-safe. |

### TGT-002 — Implement urgency scoring, retry decay, and deterministic tie-break rules

| Field | Value |
|---|---|
| Priority | P1 |
| Phase | Phase 2 |
| Stream | Targeting |
| Status | PLANNED |
| Acceptance | Scoring and selection are reproducible. |

### DEC-001 — Implement deterministic candidate selection and halt threshold behavior

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 2 |
| Stream | Decision |
| Status | PLANNED |
| Acceptance | Top candidate selection is stable and bounded. |

### DEC-002 — Add optional feature-gated LLM refinement for plan text and check hints only

| Field | Value |
|---|---|
| Priority | P2 |
| Phase | Phase 5 |
| Stream | Decision |
| Status | PLANNED |
| Acceptance | LLM remains advisory-only. |

### ACT-001 — Implement kernel-oracle plan execution

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 3 |
| Stream | Act |
| Status | PLANNED |
| Acceptance | Exact/conservative/delta/temporal/refuter plans are wired through kernel-oracles. |

### ACT-002 — Implement paired patch plan execution via PairedExperimentRunner

| Field | Value |
|---|---|
| Priority | P1 |
| Phase | Phase 3 |
| Stream | Act |
| Status | PLANNED |
| Acceptance | At least one real paired experiment fixture passes. |

### ACT-003 — Build a local EvidenceBundle builder for oracle and experiment outputs

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 3 |
| Stream | Act |
| Status | PLANNED |
| Acceptance | Pilot can emit exportable Forge bundles without inventing a new schema. |

### EXP-001 — Implement canonical V3 export -> bridge -> import roundtrip

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 3 |
| Stream | Export |
| Status | PLANNED |
| Acceptance | Pilot writes only through the canonical lane. |

### HIST-001 — Implement in-memory target history, retry accounting, and exhaustion rules

| Field | Value |
|---|---|
| Priority | P1 |
| Phase | Phase 4 |
| Stream | Loop |
| Status | PLANNED |
| Acceptance | Target loops are bounded and explainable. |

### LOOP-001 — Implement loop runner budgets, cooldown, halt reasons, and report surfaces

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 4 |
| Stream | Loop |
| Status | PLANNED |
| Acceptance | Bounded closed loop runs on fixtures. |

### LOOP-002 — Expose CLI / JSON report surfaces

| Field | Value |
|---|---|
| Priority | P1 |
| Phase | Phase 4 |
| Stream | Loop |
| Status | PLANNED |
| Acceptance | Human and machine-readable outputs exist. |

### TEST-001 — Add unit tests for observation, targeting, scoring, and tie-breaks

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 5 |
| Stream | Testing |
| Status | PLANNED |
| Acceptance | Unit coverage exists for core deterministic logic. |

### TEST-002 — Add end-to-end oracle and paired-patch roundtrip tests

| Field | Value |
|---|---|
| Priority | P0 |
| Phase | Phase 5 |
| Stream | Testing |
| Status | PLANNED |
| Acceptance | Both action families prove canonical roundtrip. |

### TEST-003 — Add thin-export and missing-kernel-payload degradation tests

| Field | Value |
|---|---|
| Priority | P1 |
| Phase | Phase 5 |
| Stream | Testing |
| Status | PLANNED |
| Acceptance | Pilot degrades honestly instead of hallucinating structure. |

### PERF-001 — Add bounded performance smoke tests for loop budgets and no-runaway behavior

| Field | Value |
|---|---|
| Priority | P2 |
| Phase | Phase 5 |
| Stream | Performance |
| Status | PLANNED |
| Acceptance | No accidental runaway loop or deadlock on large fixtures. |

### DOC-001 — Refresh root surface docs only after the crate is real and tested

| Field | Value |
|---|---|
| Priority | P1 |
| Phase | Phase 5 |
| Stream | Docs |
| Status | PLANNED |
| Acceptance | No premature documentation claims. |

