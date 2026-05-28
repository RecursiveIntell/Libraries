# Phase 06 — Assertion Suite and CI Gates

## Tasks

1. Install/adapt all P22 assertion scripts.
2. Add P22 verifier scripts.
3. Ensure CI or local gate documentation includes P22 commands.
4. Ensure `P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh` runs cargo fmt/check/test/clippy.
5. Preserve P21 verifier only for historical archive replay if needed; do not use it as the P22 final gate.

## Acceptance Gate

```bash
python3 scripts/assert_p22_zpy_archive_contract.py z.py
python3 scripts/assert_p22_codex_archival_hygiene.py .
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
```
