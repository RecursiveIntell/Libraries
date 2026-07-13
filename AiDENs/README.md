# AiDENs

**Source of truth:** `docs/codex-runs/CURRENT_RUN.json`
**Active run:** P32
**Last certified run:** P30
**Certification:** blocked
**Support label:** supported-local-build-verified
**Disclosure:** supported-local-candidate, not production-cloud-ready

## What AiDENs is

AiDENs is a local orchestration, display, packaging, inspection, fixture, and operator layer for the RecursiveIntell stack. It wires, scopes, exposes, validates, and coordinates; sibling crates own canonical memory, governance, kernel, stable-ID, tool, and federation semantics.

## Verified current boundary

P32 has a fresh local Rust format/lint/test and local-gate bar. The local Ollama boundary was also exercised with `llama3.2:3b`: model discovery, a chat completion, and a native function-call response. The tracked summary is in `docs/HOSTILE_AUDIT_2026-07-13.md`; the sanitized machine receipt is classified at `.codex_evidence/live-provider/ollama-live-2026-07-13.json`.

## Why certification remains blocked

- Package and extracted-replay certification were not rerun for this remediation tree.
- Autonomous cycle-receipt history is process-owned and explicitly non-durable; it is not a canonical restart-safe audit log.
- No authenticated OpenAI-compatible HTTP endpoint/key was configured for a live runtime receipt. Codex OAuth is not an API-boundary certification.

## What is not claimed

- Production/cloud readiness, broad autonomy, v11B/v11C completion, or canonical ownership of sibling semantics.
- Federation, attestation, settlement, remote-oracle, and production network/cancellation/rate-limit certification.
- Enabled delegation helpers: `aidens-delegation-kit` remains quarantined/disabled pending canonical-owner wiring.

## Quick start

1. Read `docs/codex-runs/CURRENT_RUN.json` for current identity and claim boundary.
2. Read `docs/plans/2026-07-13-completion-hardening-plan.md` for the issue matrix and gates.
3. Run `bash scripts/verify_current.sh` after material changes.
4. Read `SUPPORT_PROFILE.md` for supported and deferred surfaces.

## Directory guide

- `crates/` — Rust workspace crates
- `scripts/` — verification, packaging, and assertion scripts
- `docs/codex-runs/` — current-run and archival evidence
- `docs/` — audits, plans, and historical evidence
- `scaffold/` — deferred stub material
