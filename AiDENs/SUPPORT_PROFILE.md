# Support Profile - P29

Record date: `2026-05-07`

This is the active P29 support profile. Claims are bounded to local/operator execution and must trace to P29 evidence before they are treated as release-candidate support. AiDENs remains an orchestration, display, packaging, inspection, fixture, operator, and supported-local runtime layer over canonical sibling crates.

## Supported-Local Candidate

| Surface | Current P29 status | Evidence |
|---|---|---|
| Current verifier wrapper path | candidate-pending-final-package | `scripts/p29_verify.sh`; `scripts/verify_current.sh`; `handoffs/p29/PHASE_03_REPORT.md`; `handoffs/p29/PHASE_15_MANUAL_GATE.md` |
| Package/archive classifier repair | candidate-pending-final-package | `scripts/assert_p29_no_archived_current_run.py`; `z.py`; `handoffs/p29/PHASE_02_REPORT.md`; `handoffs/p29/PHASE_03_MANUAL_GATE.md` |
| Manifest path validation | candidate-pending-final-package | `scripts/assert_p29_manifest_paths.py`; `handoffs/p29/PHASE_03_REPORT.md`; `handoffs/p29/PHASE_19_MANUAL_GATE.md` |
| Extracted package self-replay | pending final package | `scripts/assert_p29_package_self_replay.py`; final `target/p29/package/` sidecars |
| v11A local material-operation path | local release-candidate evidence present for declared supported-local path | `handoffs/p29/PHASE_12_REPORT.md` through `handoffs/p29/PHASE_16_REPORT.md`; `handoffs/p29/PHASE_15_MANUAL_GATE.md`; `docs/p29/P29_SUPPORT_TRACEABILITY.md` |
| v11B regional/subtractive surfaces | executable seed only | `handoffs/p29/PHASE_17_REPORT.md` through `handoffs/p29/PHASE_19_REPORT.md`; `handoffs/p29/PHASE_19_MANUAL_GATE.md`; `scripts/assert_p29_v11b_seed_surfaces.py` |

## Partial / Fixture-Backed

| Surface | Boundary |
|---|---|
| P28 material-operation work | candidate implementation evidence only until P29 gates pass |
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

Evidence-bearing P29 outputs must expose or link receipts, execution context, manifests, proof/debt/waiver/degradation state, boundary compiler/treatment integrity records, support-tier disclosures, and known limitations. Missing receipts mean the action is not done. Waiver is not proof. Degraded is not exact. Seed is not complete.

## Current Hardening Super-Pass Overlay

The active support traceability register for this pass is `docs/super-pass/SUPPORT_TRACEABILITY.md`; the active known-limitations register is `docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md`.

The skipped post-bundle operator gate is not counted as a product defect in the clean source basis, but this pass has modified the tree. Regenerated package sidecars and extracted-package self-replay are therefore still required before a package/replay label.

Historical root Markdown and stale codex/package sidecars are reference evidence only unless listed in `SOURCE_BASIS.md` or a super-pass register as active evidence. They do not widen support labels.
