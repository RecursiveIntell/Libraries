# AiDENs Support Profile

**Source of truth:** `docs/codex-runs/CURRENT_RUN.json`
**Active run:** P32
**Parent run:** P31B
**Last certified run:** P30
**Certification:** blocked
**Support label:** supported-local-build-verified
**Disclosure:** supported-local-candidate, not production-cloud-ready

## Supported boundary

AiDENs provides bounded local orchestration for configured, receipt-bearing paths. Canonical memory, governance, kernel, stable-ID, tool-runtime, and federation semantics remain owned by their sibling crates.

### Evidence exercised in P32

- Fresh local workspace format, Clippy, test, ownership, truth, and P30 guard bars.
- Local Ollama `llama3.2:3b` discovery, chat completion, and native function-call response at `127.0.0.1:11434`.

## Explicit non-claims

- This is not package/extracted-replay certification, production/cloud readiness, or broad autonomy certification.
- Autonomous cycle history is non-durable process memory. It cannot be used as restart-safe canonical audit history.
- OpenAI Codex OAuth is available for coding work; no authenticated OpenAI-compatible HTTP runtime endpoint/key was configured, so no such runtime claim is certified.
- Federation, remote oracle, attestation, settlement, cancellation, rate-limit, and production network behavior remain unexercised.

## Quarantines and deferred surfaces

- `aidens-delegation-kit` is disabled pending canonical-owner wiring.
- Daemon, desktop, memory, and research profile crates are scaffold-only where capability status says so.
- The current issue matrix and release bar are in `docs/plans/2026-07-13-completion-hardening-plan.md`.
- Historical package/status artifacts are history only and do not override this ledger.
