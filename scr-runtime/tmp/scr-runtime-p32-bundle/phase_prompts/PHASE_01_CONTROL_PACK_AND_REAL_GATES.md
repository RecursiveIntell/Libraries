# Phase 01 — Control pack and real gates

## Goal

Replace weak/inert governance surfaces with executable, self-testing gates.

## Tasks

1. Replace null `.codex/hooks.json` with real hook wiring after reviewing current Codex hook docs.
2. Add robust hook scripts:
   - `.codex/hooks/user_prompt_submit_blockers.py`
   - `.codex/hooks/pre_tool_use_policy.py`
   - `.codex/hooks/post_tool_use_receipt.py`
   - `.codex/hooks/stop_final_gate.py`
3. Add/upgrade `.agents/skills/scr-runtime-superpass/SKILL.md`.
4. Replace 108-byte auto gate stubs with gates that name actual commands and invariant checks.
5. Add scripts:
   - `scripts/scr_superpass_preflight.py`
   - `scripts/scr_superpass_static_gates.py`
   - `scripts/scr_superpass_run_all.sh`
6. Hook scripts must self-test with:
   ```bash
   for f in .codex/hooks/*.py; do
     printf '{"hook_event_name":"SelfTest","cwd":"%s","session_id":"selftest"}' "$PWD" | python3 "$f" || exit 1
   done
   ```

## Acceptance gate

- Hooks are not null.
- Hook scripts exist and self-test.
- Phase gates fail a seeded blocker.
- Final gate checks for P32 final artifacts.
