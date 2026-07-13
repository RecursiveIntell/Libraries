# Current AiDENs Run

**Source of truth:** `CURRENT_RUN.json`
**Active run:** P32
**Parent run:** P31B
**Last certified run:** P30
**Certification:** blocked
**Support label:** supported-local-build-verified

P32 has a verified local Rust build, lint, test, ownership, and current-run gate bar. The active run remains blocked because package/extracted-replay work has not been rerun, autonomous-cycle receipt history is explicitly process-owned and non-durable, and no authenticated OpenAI-compatible HTTP endpoint was configured for a live boundary receipt.

Fresh live local evidence exists for Ollama `llama3.2:3b`: discovery, chat completion, and a native function-call response. The sanitized receipt is classified at `.codex_evidence/live-provider/ollama-live-2026-07-13.json`.

Historical P31B/P32 evidence is provenance only, not current authority. See `docs/HOSTILE_AUDIT_2026-07-13.md` and `docs/plans/2026-07-13-completion-hardening-plan.md`.
