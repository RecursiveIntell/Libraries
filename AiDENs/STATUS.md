# AiDENs Current Status

**Source of truth:** `docs/codex-runs/CURRENT_RUN.json`
**Active run:** P32
**Last certified run:** P30
**Certification:** blocked
**Support label:** supported-local-build-verified
**Disclosure:** supported-local-candidate, not production-cloud-ready

## Current state

The P32 supported local lane has fresh workspace build, lint, test, ownership, truth, and P30 guard evidence. The current hostile audit records the repaired receipt, permit, provider, runner, rollback, autonomous-cycle, UI, and documentation defects.

A bounded local Ollama receipt exists for `llama3.2:3b`: discovery, chat completion, and a native tool-call response. It proves only that configured loopback path at the observed time.

## Remaining certification blockers

- Package and extracted-replay certification are historical P31B evidence, not P32 evidence.
- Autonomous cycle receipts are inspectable in-process history, not durable restart-safe receipts.
- No authenticated OpenAI-compatible HTTP endpoint/key was present for a live provider run.

## Deliberate limits

- `aidens-delegation-kit` is disabled/quarantined; it is not an operational delegation surface.
- Daemon, desktop, memory, and research profile crates remain scaffold-only where indicated by the capability surfaces.
- Federation, remote oracle, attestation, settlement, cloud execution, and production fault behavior are not certified.

Historical pass catalogs are retained as evidence only. They do not override the current ledger.
