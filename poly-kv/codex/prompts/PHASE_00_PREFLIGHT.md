# Phase 00 — Preflight and Source Inventory

## Objective

Prove Codex is in the right repo with known git state, toolchain, manifests, and scope.

## Required actions

1. Run `pwd`, `git status --short`, `git rev-parse --show-toplevel`, `git rev-parse HEAD`.
2. Run `rustc --version`, `cargo --version`.
3. List manifests: `find . -name Cargo.toml -print | sort`.
4. Check for existing crates named `poly-kv`, `quant-codec-core`, `turbo-quant`, `fibquant`.
5. Run `python3 scripts/preflight.py` or `bash scripts/preflight.sh` if present.
6. Create `.codex-runs/<run-id>/startup_preflight.md`.
7. Do not edit implementation files until this report exists.

## Acceptance gate

Preflight report exists and no S0 blocker remains unreported.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
