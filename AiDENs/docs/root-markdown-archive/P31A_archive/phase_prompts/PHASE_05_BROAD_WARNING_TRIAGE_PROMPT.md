# Phase 5 Prompt — Broad Warning Triage

Run:

```bash
python3 scripts/p30_guard.py --repo . --json > handoffs/P31A_P30_GUARD_WARNINGS.json
python3 scripts/p30_guard.py --repo . --fail-broad
```

Fix hard/broad findings where feasible. If a broad finding is intentionally allowed, create an expiring waiver in `docs/codex-runs/P31A_BROAD_WARNING_TRIAGE.json` with rule ID, path glob, symbol, reason, owner, expiry, and evidence.

Do not use permanent waivers. Do not use line-number-only waivers.
