# P22 Expected Final State

## Active repo state

- Source crates remain under `crates/**`.
- Stable docs remain active: `README.md`, `STATUS.md`, `SOURCE_BASIS.md`, `AGENTS.md`, support docs, current package docs.
- Run-specific historical Codex artifacts are archived under `docs/codex-runs/archive/**`.
- Archive index docs remain active:
  - `docs/codex-runs/ARCHIVAL_POLICY.md`
  - `docs/codex-runs/CODEX_RUN_INDEX.md`
  - `docs/codex-runs/CURRENT_RUN.md`
- P22 handoff exists under `handoffs/p22/**` until final packaging; then `z.py` can archive it when preparing a future context.

## `z.py` state

- Supports archival normalization by default.
- Is idempotent.
- Preserves source closure checks.
- Emits archive receipts.
- Excludes archives from normal packages unless explicit.
- Supports audit-full inclusion path.

## Product state

Supported remains conservative:

- `chat-only`
- `coding-agent`
- `mock` provider
- `run-test-agent`
- safe generated `coding-agent` project
- profile/doctor/status/provider/tool/package inspection surfaces

Partial/deferred remains disclosed:

- memory-agent profile;
- autonomous daemon;
- Ollama local-service provider;
- cloud providers;
- native provider tool loops;
- streaming;
- multi-agent fanout;
- federation/remote-oracle/mechanism/research workbench surfaces.

## Forbidden final leftovers

- Active P20/P21 run prompts outside archive.
- Active `.codex_evidence` outside archive or explicit generated target audit output.
- Archive history included in normal `codex-context` package.
- Secret scanner disabled or broadly weakened.
- Any support claim resting only on prose.
