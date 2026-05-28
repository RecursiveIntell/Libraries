# Phase 09 — Final self-audit and minimization

## Goal

Prevent false completion and unnecessary scope spread.

## Tasks

1. Re-read issue matrix and mark every item:
   - fixed
   - explicitly deferred
   - quarantined
   - not applicable
2. Run static grep for forbidden leftovers:
   ```bash
   grep -R "TODO\|FIXME\|TBD\|@filename\|{feature}" -n . || true
   grep -R "P31" -n docs prompts .codex .agents README.md AGENTS.md || true
   ```
3. Confirm no active docs say no Rust workspace exists.
4. Confirm no final report claims external integration without proof.
5. Confirm no generated/codex archive detritus pollutes active repo root.
6. Produce exact next-pass plan only if there are blockers.

## Acceptance gate

- Final report is honest and bounded.
- Remaining delta is explicit.
- No completion claim outruns evidence.
