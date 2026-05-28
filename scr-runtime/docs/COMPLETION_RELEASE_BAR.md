# SCR P0 Completion Release Bar

The completion pass is done only when a fresh checkout/unzip passes:

```bash
python -m pytest -q
bash scripts/run_all_checks.sh
python scripts/validate_codex_pack.py
python scripts/assert_codex_active_pack.py
bash scripts/run_completion_checks.sh
```

If an archive is produced:

```bash
python scripts/assert_archive_includes_codex.py <archive.zip>
```

Release-blocking failures:
- `.codex/` missing from active repo.
- `.codex/` missing from archive while tests require it.
- manual phase gating required by workflow.
- auto phase runner missing or not receipt-emitting.
- unexpected legacy packager script names remain active in the source package.
- final report claims completion without command evidence.
