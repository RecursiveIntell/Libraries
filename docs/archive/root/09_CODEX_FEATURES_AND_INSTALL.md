# 09 — Codex Features and Install Guidance

## Use these Codex surfaces

1. `AGENTS.md` / repo-local skill.
   - Codex reads AGENTS guidance before work, and repo-local skill instructions can be activated for this task.
2. Plan mode.
   - Use planning before implementation because this is a high-risk math task.
3. Hooks.
   - Use project-local hooks to catch forbidden file changes and missing closeout evidence.
4. `/review` or custom final review.
   - Review uncommitted changes against the acceptance gates.
5. MCP only if needed.
   - The bundle includes enough source-basis text for the pass; MCP can be used for paper/doc lookup but must not become a hidden source of truth.

## Install overlay

From repo root after unpacking this bundle:

```bash
bash /path/to/bundle/scripts/install_fibquant_codex_bundle.sh /path/to/bundle
```

Then open Codex and use `/hooks` to review hook scripts. Approve only:

- `.codex/hooks/fibquant_guard.py`
- `.codex/hooks/fibquant_stop_check.py`

## Hook warning

Hooks are guardrails, not proof. Final acceptance still requires the final assertion script and test receipts.
