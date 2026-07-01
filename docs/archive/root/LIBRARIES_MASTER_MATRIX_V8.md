# Libraries Master Issue Matrix — V8

**Date:** 2026-04-02
**Source:** Claude Opus 4.6 full code review (124,934 lines across 40 crates)
**Total:** 10 issues (3 HIGH, 5 MED, 2 LOW)

---

## Assessment

The libraries workspace is materially ahead of Recall. It's a real platform substrate with genuine architectural discipline. The issues are about consumption readiness and process closure, not design problems.

## HIGH (3)

| ID | Title | Action | Owner |
|---|---|---|---|
| LIB-001 | Recall doesn't consume CommitToken/ExecutionPermit | Wire permit chain into Recall session | Recall-side fix (post-CLARA) |
| LIB-002 | forge-governance feature disabled by default | Enable in recall-session Cargo.toml | Libraries prompt Task 1 |
| LIB-010 | Most Recall retrieval paths bypass knowledge-runtime | Recall-side fix (RPD-001) | Recall pack |

## MED (5)

| ID | Title | Action |
|---|---|---|
| LIB-003 | Schema compat checks not CI-enforced | Add scripts to CI |
| LIB-004 | New artifact families need compat gates | Gate on check_schema_compat |
| LIB-005 | Supported-lane manifest may be stale | Verify and update |
| LIB-006 | kernel-conformance surfaces incomplete | Add executable semantic oracles |
| LIB-007 | Bounded repair artifacts need tightening | Type and bound for downstream |

## LOW (2)

| ID | Title | Action |
|---|---|---|
| LIB-008 | Primitives unwraps/unsafe don't match workspace policy | Document the policy |
| LIB-009 | attestation-exchange not consumed by governance observation | Document as V2 scope |

---

## Key Numbers

| Metric | Value |
|---|---|
| Rust lines | 124,934 |
| Crates (workspace) | 30+ |
| Crates (Primitives, excluded) | 10 |
| Tests | 1,320 |
| JSON schemas | 210 |
| Typed ID newtypes | 218 |
| Production unwraps | 0 |
| Production unsafe (excl. Primitives) | 0 |
| Governance surfaces observed | 6 |

## Execution Priority

1. Enable forge-governance feature by default (LIB-002) — 1 line
2. CI-enforce schema compat (LIB-003) — script integration
3. Update supported-lane manifest (LIB-005) — audit and update
4. Document Primitives policy (LIB-008) — README addition
5. Everything else is post-CLARA or Recall-side

The libraries are not the bottleneck. Recall is.
