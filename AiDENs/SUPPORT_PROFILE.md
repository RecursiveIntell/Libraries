# Support Profile — P31A Recovery

Record date: `2026-05-29`
Ledger: `docs/codex-runs/CURRENT_RUN.json`
Last certified run: `P30`
Certification status: `blocked`
Support label: `p31a-blocked-release-truth-repair`

This is the active P31A support profile. Claims are bounded to local/operator execution and must trace to P31A evidence before they are treated as release-candidate support. AiDENs remains an orchestration, display, packaging, inspection, fixture, operator, and supported-local runtime layer over canonical sibling crates.

## Supported-Local Candidate

| Surface | Current P31A status | Evidence |
|---|---|---|
| Release-truth ledger | active repair | `docs/codex-runs/CURRENT_RUN.json`; `docs/codex-runs/P31A_RECOVERY/preflight_report.md` |
| Root Markdown archival | active repair | `scripts/assert_root_markdown_archive_policy.py`; Phase 02 evidence |
| Verification gate alignment | active repair | `scripts/assert_adapter_delegation.sh`; `scripts/assert_tool_runtime_delegation.sh`; Phase 03 evidence |
| Static safety hardening | active repair | `scripts/p30_guard.py`; Phase 04 evidence |
| Build/test/package replay | pending | Phase 07–08 evidence |

## Partial / Fixture-Backed

| Surface | Boundary |
|---|---|
| P30 implementation work | candidate evidence only until P31A gates pass |
| Mock-provider Plan-Act-Verify path | fixture-backed local path, not cloud |
| Bitemporal local query/reference behavior | reference fixture and differential check for declared path only |
| Agency/influence classification | heuristic local boundary classifier, not governance truth |
| Memory grounding | canonical adapter/backpointer evidence only; no AiDENs-local truth store |

## Deferred / Reserved

| Surface | Status |
|---|---|
| Hosted/cloud provider execution requiring API keys | deferred-cloud |
| Native tool loops over hosted providers | deferred-cloud |
| Production streaming loops | deferred-cloud |
| Broad autonomous daemon scheduling | deferred-autonomy |
| Production daemon authority | deferred-autonomy |
| v11B regional/subtractive runtime | executable seed only; no active runtime, mutation authority, cross-region admission, or completion claim |
| v11C federation, mechanism, self-hosting, external admission | reserved/quarantine by default |
| Canonical memory/governance/kernel/runtime truth | sibling-owned, not AiDENs-owned |

## Semantic Honesty Rule

Evidence-bearing P31A outputs must expose or link receipts, execution context, manifests, proof/debt/waiver/degradation state, boundary compiler/treatment integrity records, support-tier disclosures, and known limitations. Missing receipts mean the action is not done. Waiver is not proof. Degraded is not exact. Seed is not complete.

## Active Hardening

The active hostile audit finish pack is `aidens_hostile_audit_finish_pack.zip`. Its evidence and plan docs are task material, not source truth.
