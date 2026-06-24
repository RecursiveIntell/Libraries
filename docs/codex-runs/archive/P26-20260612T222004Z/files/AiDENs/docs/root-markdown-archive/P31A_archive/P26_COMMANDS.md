# P26 Command Plan

## Existing commands to preserve

- `aidens run-test-agent`
- `aidens run-coding-agent`
- `aidens inspect-run`
- `aidens memory seam-fixture`
- `aidens coding repo-read`
- `aidens coding repo-list`
- `aidens coding repo-search`
- `aidens coding patch-propose`
- `aidens coding patch-apply`
- `aidens coding run-checks`
- `aidens permit request|approve|deny|revoke`
- `aidens receipts list|inspect|export|verify-digest`

## New/normalized commands to add if safe

- `aidens agent validate --spec <agent.json>`
- `aidens agent run --spec <agent.json> --task <task.md> --out <dir>`
- `aidens agent inspect --run <dir>`
- `aidens agent doctor --spec <agent.json>`
- `aidens agent new --template local-coding --out <dir>`

If adding a top-level `agent` command would destabilize CLI, implement equivalent package/experimental command behind a clearly marked supported-local example path.
