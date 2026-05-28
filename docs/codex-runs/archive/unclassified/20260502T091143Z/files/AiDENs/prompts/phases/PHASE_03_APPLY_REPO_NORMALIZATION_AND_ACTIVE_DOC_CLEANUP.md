# Phase 03 — Apply Repo Normalization and Active Doc Cleanup

## Tasks

1. Run `python3 z.py --root . --profile aidens --archive-only --strict`.
2. Verify old P20/P21 prompts, handoffs, run docs, `.codex_evidence`, and stale run control files moved into `docs/codex-runs/archive/...`.
3. Update or create:
   - `docs/codex-runs/ARCHIVAL_POLICY.md`
   - `docs/codex-runs/CODEX_RUN_INDEX.md`
   - `docs/codex-runs/CURRENT_RUN.md`
4. Promote any genuinely reusable script to a generic name before archival; otherwise archive old run-specific scripts.
5. Ensure root docs are not moved.

## Acceptance Gate

```bash
python3 scripts/assert_p22_codex_archival_hygiene.py .
python3 z.py --root . --profile aidens --mode codex-context --dry-run --strict
```

No stale active Codex-run artifact may remain outside `docs/codex-runs/archive/**`, except explicitly current P22 run files before final packaging.
