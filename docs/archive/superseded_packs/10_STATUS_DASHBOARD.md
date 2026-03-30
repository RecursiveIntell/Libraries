# 10_STATUS_DASHBOARD

## Snapshot

- Active assessment date: 2026-03-23
- Review mode: hostile static audit + executable shell/python truth scripts
- Build mode here: not cargo-certified in this environment

## Red

- PACK-001 — front-door pack truth is broken
- PACK-002 — root archive manifest is false
- PACK-003 — dashboard/evidence/receipt drift exists
- SPEC-001 — root canonical specs are stubs passing weak doc checks
- GATE-001 — support lane vs gate scope mismatch
- RUNTIME-001 — degraded reason is lost between kernel and runtime artifacts

## Yellow

- EXEC-001 — execution-evidence convergence is incomplete
- TYPE-001 — duplicate `SurfaceStatus`
- NAME-001 — thin governance/runtime crates still overnamed
- DOC-001 — real doc-comment gaps in governance/kernel crates
- SAFE-001 — supported-lane unwrap hotspots + misleading panic-guard naming
- MOD-001 — oversized hotspot files remain unsplit
- PACK-004 — source-clean still contains `target-*`
- DOC-002 — root/meta clutter still high
- REL-001 — live proof lane still weaker than it should be

## Green or materially strong

- Core architecture in semantic memory / bridge / runtime / living-memory is real
- Repo surface check passes
- Manifest truth passes
- Hotspot budgets pass
- Schema registry uniqueness passes
- Mirror discipline passes
- Public type drift check passes
- Public API docs check passes for the currently monitored set
- Closeout receipt structure check passes
- Episode bundle and execution-context artifact families already exist
