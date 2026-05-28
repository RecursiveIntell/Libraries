# Phase 00 — Preflight, source truth, run identity

## Goal

Establish exact repo state and prevent P30/P31/P32 drift.

## Tasks

1. Run:
   ```bash
   pwd
   git status --short || true
   git rev-parse --show-toplevel || true
   git rev-parse HEAD || true
   find . -maxdepth 3 -type f | sort | sed 's#^./##' | head -300
   python3 scripts/scr_superpass_preflight.py before || true
   ```
2. Confirm whether `cargo` exists. If missing, record as blocker, but continue static prep only.
3. Set current run to `P32-SCR-RUNTIME-SUPERPASS`.
4. Inventory all P31/P30 docs; archive stale active run docs instead of overwriting silently.
5. Create `docs/P32_COMMAND_RECEIPTS.md` immediately and append every command from this point forward.
6. Create `docs/P32_CHANGED_FILES.md` and maintain it after each phase.

## Acceptance gate

- Current run ID is unambiguous.
- Existing stale run docs are archived or marked non-authoritative.
- Preflight result is recorded.
- No implementation edits before preflight report exists.
