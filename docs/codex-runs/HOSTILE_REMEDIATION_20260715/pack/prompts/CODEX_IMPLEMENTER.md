# Codex implementer prompt template

- Task: `{{TASK_ID}}`
- Issues: `{{ISSUE_IDS}}`
- Repo/worktree/branch: `{{REPO_PATH}}` / `{{WORKTREE_PATH}}` / `{{BRANCH}}`
- Integration base: `{{BASE_COMMIT}}`
- Pack: `{{PACK_PATH}}`

Read global guardrails, assigned issue entries, phase order, current source/tests, then nonconflicting
repository instructions.

Writable: `{{ALLOWED_PATHS}}`
Forbidden: `{{FORBIDDEN_PATHS}}`

If required work leaves scope, stop and propose dependency; do not edit it.

Required changes: `{{REQUIRED_CHANGES}}`
Forbidden shortcuts: `{{FORBIDDEN_SHORTCUTS}}`
Acceptance gates: `{{ACCEPTANCE_GATES}}`
Commands: `{{VALIDATION_COMMANDS}}`

Method: confirm failure; add failing regression; implement smallest boundary-correct repair; run
commands with receipts; inspect diff; write handoff Markdown/JSON; commit intentionally; do not push
unless granted.

Final response only: IDs, changed files, semantic change, commands and receipts, gate status,
residual risks, rollback, commit SHA. Never claim phase/release completion.
