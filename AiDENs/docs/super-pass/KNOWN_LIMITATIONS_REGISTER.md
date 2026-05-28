# Super-Pass Known Limitations Register

Record date: `2026-05-07`

This register is the active limitation ledger for the current hardening super-pass. It does not grant a release label. Final package sidecars and extracted-package self-replay remain required before any package support claim.

| ID | Linked rows | Limitation | Status | Support effect |
|---|---|---|---|---|
| SP-LIM-001 | `CLAUDE-F-001`, Gate 13 | Final package sidecars and extracted-package self-replay were regenerated after the current tree modifications. | closed by Phase 20 | Package/replay evidence is limited to the generated P29 sidecars and replay receipt. |
| SP-LIM-002 | `AHD-0901`, `AHD-0911`, `AHD-0921`, `AHD-0931` | `matrices/P29_MASTER_ISSUE_MATRIX.csv` is historical and superseded by `matrices/SUPER_PASS_BACKLOG_1020.csv`. | classified | Super-pass closure must use the 1020-row backlog and phase reports. |
| SP-LIM-003 | `AHD-0902`, `AHD-0912`, `AHD-0922`, `AHD-0932` | Final command bar, package sidecars, and extracted replay passed for the supported-local scope. | closed by Phase 20 | Only the allowed local/replay/seed/reserved labels may be used. |
| SP-LIM-004 | `AHD-0904`, `AHD-0914`, `AHD-0924`, `AHD-0934`, `CLAUDE-F-003` | Historical root Markdown remains evidence/reference material unless listed as an active truth doc in `SOURCE_BASIS.md`. | quarantined | Root Markdown ambiguity cannot widen support labels. |
| SP-LIM-005 | `CLAUDE-F-004` | Historical codex/package sidecars are retained as evidence artifacts and are not active current-run sidecars. | quarantined | Stale artifacts cannot satisfy current package gates. |
| SP-LIM-006 | `CLAUDE-F-017` | External citations embedded in research/reference docs are unresolved local artifacts unless reverified during a later research pass. | deferred | Research docs cannot be used as executable evidence. |
| SP-LIM-007 | `CLAUDE-F-020` | Audit logs are hashed into `target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json` after the Phase 20 package/replay commands. | closed by Phase 20 | The hash command's own console log is external evidence; the hash manifest is the queryable digest ledger. |
| SP-LIM-008 | `AHD-0903`, `AHD-0913`, `AHD-0923`, `AHD-0933` | The operator-created source bundle is accepted as clean source basis, but source-basis validity is not product conformance. | classified | Clean source basis does not imply release readiness. |
| SP-LIM-009 | `AHD-0908`, `AHD-0918`, `AHD-0928`, `AHD-0938` | Package sidecar identities distinguish zip-byte hashes from canonical content manifests. | closed by Phase 20 | Final package report states both identities explicitly. |
| SP-LIM-010 | `AHD-0909`, `AHD-0919`, `AHD-0929`, `AHD-0939` | The skipped post-bundle operator gate is evidence hygiene, not a product defect in the source basis. | classified | It still requires regenerated sidecars and extracted replay after this pass. |
| SP-LIM-011 | `CLAUDE-F-015` | Historical P29 residual BUG IDs are now represented as `fixed`, `quarantined`, `deferred`, or `open-blocking` classification evidence instead of a flat ambiguous open/quarantine list. | fixed by classification | Marker-only or flat bug buckets cannot satisfy hard gates. |
| SP-LIM-012 | `CLAUDE-F-016` | `forge-pilot`, `effect-runtime`, verification pipeline, federation, attestation, `authority-delegation`, and `recursive-kernel-core` are quarantined from AiDENs supported-local claims pending separate layer audits. | quarantined | These layers cannot widen AiDENs labels or serve as AiDENs-owned correctness/authority surfaces. |

## Query Rule

Every degradation, fallback, widening, repair, quarantine, waiver, or deferred item must be represented in this register, the issue matrix, or a phase report. Marker-only evidence is not sufficient for hard gates.
