# Phase 03 — Package Mode and Replay Verifier Closure

## Goal
Make package replay a first-class gate, then stop letting packaging dominate the project.

## Required actions

1. Create `scripts/p23_verify.sh`.
2. Create package replay verifier that extracts the emitted package to a temp dir and runs the included verifier.
3. Ensure any package that includes a verifier includes all verifier dependencies.
4. Bind final package docs to actual package role and hash.
5. Fix mismatch between handoff package hash and actual emitted package hash by explicitly naming package roles.
6. Emit `target/p23/audit/package_replay_report.json`.

## Required package roles

- `release-context`: operator/source release, no Codex run control docs.
- `next-codex-context`: package for the next coding pass; includes current truth docs and minimal handoff.
- `codex-run-full`: includes current run prompts/injections/handoffs for auditing current run.
- `audit-full`: includes archived history deliberately.

## Acceptance gate

A freshly extracted `next-codex-context` or equivalent package must pass its own verifier. If a mode intentionally excludes verifier scripts, its manifest must declare `self_replay_verifier: excluded_by_role` and the release docs must not imply otherwise.
