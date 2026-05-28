# P22 Package Surface Matrix

| Package mode | Include source | Include external Cargo path deps | Include current truth docs | Include Codex archive history | Include target audit logs | Purpose |
|---|---:|---:|---:|---:|---:|---|
| `source-clean` | yes | yes when needed | yes | no | no | clean source handoff |
| `codex-context` | yes | yes | yes | no | no | next Codex/audit context without stale prompts |
| `full-context` | yes | yes | yes | no by default | no by default | broad dev context |
| `research-context` | selected | no/selected | selected | no by default | no | research transfer |
| `audit-full` | yes | yes | yes | yes only by explicit flag | selected/explicit | hostile audit or run-history replay |

## Normal package must include

- Rust source crates.
- Cargo manifests/lockfiles.
- Stable active docs.
- Archive index/current-run docs.
- P22 verifier scripts if current.
- Required examples/fixtures/tests.

## Normal package must exclude

- `docs/codex-runs/archive/**`.
- `.codex_evidence/**`.
- stale `docs/pNN/**`, `prompts/pNN/**`, `handoffs/pNN/**` outside archive.
- old run prompts/tasks.
- generated package sidecars from previous runs.
- target build output.

## Audit-full package may include

- Codex run archive history.
- Archive manifests.
- Historical handoffs.
- Selected target audit logs if copied into an explicit audit artifact path with receipts.
