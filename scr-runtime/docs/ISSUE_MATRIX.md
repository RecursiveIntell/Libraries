# P31 Issue Matrix

| ID | Severity | Surface | Issue | Required fix | Acceptance evidence |
|---|---:|---|---|---|---|
| P31-B001 | Blocker | Packaging | Report/manifest may describe planned files, not actual ZIP bytes. | Add ZIP/manifest parity verifier and make certifier verify post-write. | `verify_archive_manifest_parity.py` passes on produced ZIP. |
| P31-B002 | Blocker | Package surface | `.codex`/`.agents` claimed but absent or absent but expected. | Make inclusion/exclusion explicit and verified. | `assert_required_archive_paths.py` passes. |
| P31-B003 | Blocker | Owner boundary | Local SCR types may duplicate canonical owner crates. | Inspect Libraries/Libraries2; create boundary map and adapter seams. | `EXTERNAL_CRATE_BOUNDARY_MAP.md`, `assert_existing_crate_boundaries.py`. |
| P31-B004 | Blocker | Source surface | `target_files` duplicates active source. | Archive/delete/mark non-authoritative. | `assert_no_stale_surfaces.py` passes. |
| P31-B005 | Blocker | Release proof | No local Cargo certification in audited environment. | Run full local Rust/test/clippy gates. | command receipts. |
| P31-H001 | High | Signals | Opaque refs are token-scanned for control facts. | Only explicit signals/adapters may produce signals. | Rust tests + grep gate. |
| P31-H002 | High | Policy | Unknown hard rules accepted but ignored. | Registry validation rejects unknowns. | negative test. |
| P31-H003 | High | Policy | Wrong domain/algorithm not rejected. | Enforce header compatibility. | negative test. |
| P31-H004 | High | Schema | Schemas allow unknown fields / weak versions / unbounded scores. | Strict serde + schema postprocess/manual schema. | strict schema script + negative tests. |
| P31-H005 | High | Input | Invalid input flows through normal evaluator. | Reject or route to dedicated raw rejection receipt. | negative tests. |
| P31-H006 | High | Evidence refs | Malformed refs replaced with synthetic refs. | Preserve invalidity or reject; do not substitute. | tests. |
| P31-H007 | High | Provenance | `evaluator_algorithm_hash` hashes ID string. | Rename or compute honest digest. | schema/docs/tests. |
| P31-H008 | High | CLI | Generation and verification conflated. | Split commands; update docs. | CLI tests. |
| P31-H009 | High | Replay | No receipt explanation path. | Add `explain-receipt`. | command output. |
| P31-H010 | High | Receipts | Losing candidates incompletely recorded. | Add full candidate/rejection model. | unit tests. |
| P31-H011 | High | Raw digest | Raw unknown fields can be dropped before hashing. | Preserve raw digest in raw path. | tests. |
| P31-H012 | High | Docs | SOURCE_BASIS stale. | Update to current state. | grep gate. |
| P31-M001 | Medium | Root hygiene | Root certifier ambiguous. | Move to scripts or classify. | source tree check. |
| P31-M002 | Medium | Junk | `testtmp` included. | Delete and assert absent. | stale-surface gate. |
| P31-M003 | Medium | Manual gates | Manual injections active. | Archive or mark legacy; automate gates. | stale-surface gate. |
| P31-M004 | Medium | Schema diff | No semantic schema diff/proof. | strict validation + generated diff. | scripts. |
| P31-M005 | Medium | Canonical JSON | Canonicalization custom and under-specified. | Declare `scr-canonical-json-v1` or use canonical owner. | docs/tests. |
| P31-M006 | Medium | Time | Time fields opaque but named like temporal semantics. | Rename/document or implement RFC3339. | tests/docs. |
| P31-M007 | Medium | Public API | `evaluate()` always unavailable. | Remove/rename/fix. | API tests. |
| P31-M008 | Medium | Scripts | grep gates depend on `rg`. | Add preflight. | run checks. |
| P31-M009 | Medium | Fixture policy | Golden update rationale weak. | require `POLICY_CHANGE` artifact on expected changes. | fixture verification gate. |
