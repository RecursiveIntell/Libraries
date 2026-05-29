# Phase 00 — Preflight and inventory

Run:

```bash
RUN_ID=${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-zpy-hygiene}
mkdir -p .codex-runs/$RUN_ID
pwd
find . -maxdepth 2 -type f | sort | tee .codex-runs/$RUN_ID/root_inventory.txt
git status --short | tee .codex-runs/$RUN_ID/git_status_before.txt
grep -n "ALLOWED_TEXT_EXTENSIONS\|ALLOWED_BASENAMES\|include_decision\|root_markdown\|archive_codex" z.py | tee .codex-runs/$RUN_ID/zpy_surface.txt
```

Record current root residue, excluded package paths, and existing validators. Do not edit yet.
