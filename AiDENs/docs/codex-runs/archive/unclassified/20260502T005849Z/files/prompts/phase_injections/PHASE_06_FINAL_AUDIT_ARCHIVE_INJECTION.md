# Phase 06 injection — final audit and archive integrity

Generate final audit bundle and verify release artifact contents.

Required:

```bash
bash scripts/p20_1_generate_audit_bundle.sh
python3 scripts/p20_1_hard_code_audit.py --fail-on-blocking
```

Final report must include PASS/FAIL for every P20.1 gate and list any remaining limitations.

If target audit files live under ignored `target/`, copy a release-visible summary under `docs/p20_1/final-audit/` or package them explicitly.
