# P22 Master Issue Matrix

| ID | Priority | Area | Problem | Required action | Gate |
|---|---:|---|---|---|---|
| P22-001 | P0 | `z.py` | Normal packages include stale Codex artifacts | Add archival normalization before zipping | `p22_zpy_archival_selftest.py` passes |
| P22-002 | P0 | Package policy | `codex-context` includes run docs by default | Exclude archive/run history unless explicit | release package clean assertion passes |
| P22-003 | P0 | Repo hygiene | P20/P21 prompts/docs/handoffs remain active | Archive to `docs/codex-runs/archive` with receipts | hygiene assertion passes |
| P22-004 | P0 | Release truth | Root docs may reference older run state | Update README/STATUS/SOURCE_BASIS to P22 truth | docs audit passes |
| P22-005 | P0 | Secret scanning | `api_key` field-copy false positives | Refine scanner; do not weaken literal-secret detection | no false warning, literal fixture catches |
| P22-006 | P0 | Build proof | P21 verifier cargo gates optional | P22 verifier supports required cargo gate | `P22_REQUIRE_CARGO=1` passes |
| P22-007 | P1 | Release archive | P21 release verifier hardcodes old paths | Add P22 release archive verifier | archive replay report `ok=true` |
| P22-008 | P1 | Support claims | Partial/deferred surfaces can drift | Regenerate support-tier docs and tests | status/package tests pass |
| P22-009 | P1 | Assertion coverage | No dedicated active-stale-run detector | Add assertion scripts | scripts installed and run |
| P22-010 | P2 | Operator UX | Packaging modes not obvious | Add docs and CLI report fields | README/status updated |
| P22-011 | P2 | Stretch | Product surfaces can improve | Only low-risk truth/UX upgrades after core gates | no deferred feature promotion |
