# P28 Final Auditor Handoff

## Summary

P28 passed the declared supported-local v11A material-operation kernel gate. The final evidence manifest is `P28_STATUS_EVIDENCE_MANIFEST.json`; the final audit report is `docs/p28/P28_FINAL_AUDIT_REPORT.md`.

## What To Inspect

- Phase reports: `handoffs/p28/PHASE_00_REPORT.md` through `handoffs/p28/PHASE_15_REPORT.md`
- Final command logs: `target/p28/audit/*p28_final*`
- Final package: `target/p28/package/AiDENs-p28-codex-context.zip`
- Package sidecars: `target/p28/package/AiDENs-p28-codex-context.*`
- Self-replay receipt: `target/p28/audit/package_self_replay_p28_final_receipt.json`
- Failed `/tmp` environment receipt: `target/p28/audit/package_self_replay_p28_final_receipt_tmp_space_failed.json`

## Claims Allowed

- `p28-supported-local-plus`
- `v11A-conformant-core:declared-local-agent-path`
- `v11B-draft` for non-authoritative DTOs/tests only
- `v11C-reserved` with quarantine/default-deny admission only

## Claims Not Allowed

- Production-cloud-ready
- Broad-autonomy-ready
- Active v11B runtime
- Active v11C federation/mechanism/self-hosting
- Canonical memory/governance/kernel/provider/tool/schema truth ownership

## Residual Risks

- Some topic modules remain large after Phase 11 containment.
- Hosted providers, production daemon authority, active federation/admission, and active regional runtime remain deferred/reserved.
- Package replay in constrained `/tmp` environments can fail due disk capacity; use a larger `TMPDIR` and `CARGO_TARGET_DIR` for cargo-backed replay.
