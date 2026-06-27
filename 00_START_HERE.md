# V29 Unified Hostile Audit Remediation Pack

**Effective date:** 2026-03-30
**Supersedes:** All prior numbered packs (V1–V28), all prior issue matrices, all prior playbooks
**Source basis:** `libraries-source-clean-20260330.zip`
**Scope:** Close every confirmed defect from two independent hostile audits and achieve DARPA CLARA submission readiness
**Deadline:** April 10, 2026

Supersession note (2026-03-17): the earlier no-v25 terminal position is superseded by the current v25 repo truth surface and `scripts/check_v25_repo_truth.sh`.

## Genesis

This pack merges findings from two independent hostile audits:

1. **Claude audit (10-angle hostile):** Wire-format integrity, error handling, governance completeness, convention enforcement, documentation coverage, gate/script integrity, hotspot concentration, trust-chain completeness, schema versioning, DARPA submission readiness. Confirmed 0 production `unwrap()` calls, GOV-001/GOV-002 resolved, 56 serde enum violations found.

2. **GPT audit (10-lens hostile + gate execution):** Constitutional truth, cynical architecture, blast-radius, failure semantics, oncall pain, contract law, evidence science, security, new-maintainer, DARPA demo review. Mean severity 2.95/5. Primary diagnosis: "architecturally strong, operationally messy."

**Combined verdict:** The code is real. The architecture is sound. The governance pipeline is wired. The remaining risk is entirely meta-layer: repo truth, gate truth, and presentation polish.

## Reading order

| # | Document | Purpose |
|---|----------|---------|
| 1 | This file | Context and orientation |
| 2 | `01_MASTER_ISSUE_TENSOR.json` | All 16 issues with evidence, fix instructions, acceptance criteria |
| 3 | `02_MASTER_ISSUE_MATRIX.md` | Human-readable issue matrix with priority and phase assignments |
| 4 | `03_IMPLEMENTATION_PLAYBOOK.md` | Phase order, dependency graph, execution rules |
| 5 | `04_EXACT_FILE_TOUCH_MAP.md` | Every file to create or modify, by issue |
| 6 | `05_TEST_AND_CONFORMANCE_PLAN.md` | Required verification per issue |
| 7 | `06_RISK_REGISTER.md` | What can go wrong, mitigations, forbidden shortcuts |
| 8 | `CLAUDE.md` | Agent instructions for implementation |
| 9 | `PROMPT.md` | Execution prompt for agent sessions |
| 10 | `10_HOSTILE_AUDIT_CLAUDE.md` | Full Claude 10-angle audit report |
| 11 | `11_HOSTILE_AUDIT_GPT.md` | Full GPT 10-lens audit report (included from bundle) |

## Issue summary

| Priority | Count | Representative issues |
|----------|------:|----------------------|
| P0 | 3 | TRUTH-001, GATE-001, DOC-002 |
| P1 | 5 | TRUTH-002, TRUTH-003, GATE-002, WIRE-001, DOC-001 |
| P2 | 7 | TRUTH-004, GATE-003, WIRE-002, CONV-001, GOV-001, PERF-001, SAFE-001 |
| P3 | 1 | GOV-002 |

## What is NOT broken

Both auditors independently confirmed:
- Zero production `unwrap()` calls (all 211 are inside `#[cfg(test)]`)
- Zero `unsafe` in workspace member code
- Zero `todo!()` or `unimplemented!()` in production
- Governance observation pipeline is wired and tested (GOV-001/GOV-002 from V28 are resolved)
- 211 JSON schemas generated from type system via `contract-schema-gen`
- OODA loop runner is real: observe → orient → decide → act with verification, calibration, adjudication
- HNSW RwLock ordering is consistent (key_to_id → id_to_key → deleted_ids)
- `thiserror` error types throughout with descriptive payloads
