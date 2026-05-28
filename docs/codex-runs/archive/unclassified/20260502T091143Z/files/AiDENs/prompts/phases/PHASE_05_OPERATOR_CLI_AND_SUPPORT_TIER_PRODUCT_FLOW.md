# Phase 05 — Operator CLI and Support-Tier Product Flow

## Goal
Make AiDENs usable as an operator-facing local agent builder/doctor, not just a library pile.

## Required actions

1. Add or harden CLI commands for:
   - `aidens doctor`
   - `aidens package doctor` or equivalent
   - `aidens agent plan`
   - `aidens agent run` / `aidens test-agent run` / equivalent local fixture path
   - `aidens run inspect <run-dir>`
2. Each command must emit `--json` support-tier output where appropriate.
3. Support-tier output must distinguish:
   - supported
   - fixture-supported
   - partial
   - scaffold
   - deferred
   - failed
4. Docs must teach the supported local path first.
5. Unsupported/cloud/native/autonomous paths must fail honestly.

## Required tests

- CLI help snapshot or smoke tests,
- support-tier JSON schema/shape tests,
- run inspect tests,
- no-fake-ready tests.

## Acceptance gate

An operator should be able to run one local fixture-backed agent lane and inspect its receipts without reading internal phase docs.
