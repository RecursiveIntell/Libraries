# Phase 04 Receipt - Downstream Dependency Repair

Date: 2026-05-25

## Repairs

Recall:

- Rewrote `agent-graph` and `job-queue` from `_vendor/Libraries2/*` to `../Libraries/*`.
- Rewrote stale `_vendor/Libraries/*` path dependencies in `Cargo.toml` to canonical `../Libraries/*`.
- Rewrote nested `deps/llm-pipeline` path dependencies to canonical `../../../Libraries/*`.
- Rewrote `recall-session` and `recall-contracts` `verification-policy` paths to canonical `../../Libraries/verification-policy`.
- Updated build/preflight/verification scripts to use `Libraries` only.

Recall-Coding:

- Rewrote `agent-graph` and `job-queue` from `_vendor/Libraries2/*` to `../Libraries/*`.
- Rewrote stale `_vendor/Libraries/*` path dependencies in `Cargo.toml` to canonical `../Libraries/*`.
- Rewrote nested `deps/llm-pipeline` path dependencies to canonical `../../../Libraries/*`.
- Rewrote `recall-session` and `recall-contracts` `verification-policy` paths to canonical `../../Libraries/verification-policy`.
- Updated preflight/verification scripts to use `Libraries` only.

Gloss:

- Rewrote `src-tauri/Cargo.toml` from local vendors to canonical `../../Libraries` for `llm-pipeline`, `tauri-queue`, and `semantic-memory`.
- Kept vendor directories as historical/local copies for later hygiene; they are no longer active Cargo dependency truth for these crates.

## Validation

- `cargo metadata --manifest-path /home/sikmindz/Coding/Recall/Cargo.toml --format-version 1`: pass.
- `cargo metadata --manifest-path /home/sikmindz/Coding/Recall-Coding/Cargo.toml --format-version 1`: pass.
- `cargo metadata --manifest-path /home/sikmindz/Coding/Gloss/src-tauri/Cargo.toml --format-version 1`: pass after rewriting Gloss canonical group, avoiding duplicate `stack-ids` lockfile collision.
- `cargo check --manifest-path /home/sikmindz/Coding/Recall/Cargo.toml`: pass. Log: `phase04_recall_cargo_check.log`.
- `cargo check --manifest-path /home/sikmindz/Coding/Recall-Coding/Cargo.toml`: pass. Log: `phase04_recall_coding_cargo_check.log`.
- `cargo check --manifest-path /home/sikmindz/Coding/Gloss/src-tauri/Cargo.toml --features semantic-memory-backend`: pass. Log: `phase04_gloss_tauri_cargo_check.log`.
- Static Cargo manifest scan for `_vendor/Libraries`, `_vendor/Libraries2`, `../Libraries2`, and active Gloss vendor semantic paths in modified manifests: no matches.

## Gate

Phase 04 passes for the high-confidence tranche. The modified downstream Cargo surfaces now resolve through canonical `Libraries` crates and pass metadata/check validation.
