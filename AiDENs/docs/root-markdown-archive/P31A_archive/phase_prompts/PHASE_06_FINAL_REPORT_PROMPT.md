# Phase 6 Prompt — Final Gate and Auditor Report

Run:

```bash
bash scripts/verify_current.sh | tee handoffs/P31A_FINAL_VERIFY.log
```

Then update `CURRENT_RUN.json` booleans/evidence refs honestly:

- only set `build_certified=true` if cargo metadata/fmt/check/test/clippy passed and logs exist;
- only set `package_certified=true` if sidecars validate;
- only set `extracted_replay_certified=true` if extracted package replay passed;
- only set `certification_status=certified` if all required certification booleans are true.

Write:

- `handoffs/P31A_FINAL_REPORT.md`
- `handoffs/P31A_DEFERRED_RUNTIME_EVIDENCE_ISSUES.md`

Do not claim feature completion. This pass is release truth and final gate repair only.
