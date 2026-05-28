# Phase 05 Receipt - Semantic-Memory Collision And Gloss Boundary

Date: 2026-05-25

## Canonical Semantic-Memory Identity

Canonical owner:

- `Libraries/semantic-memory/Cargo.toml`

Evidence:

- `phase01_cargo_metadata.json` includes exactly one `semantic-memory` package: `/home/sikmindz/Coding/Libraries/semantic-memory/Cargo.toml`.
- `cargo check --manifest-path /home/sikmindz/Coding/Libraries/Cargo.toml --workspace` passed with canonical `semantic-memory`.

## Duplicate Collision

Unresolved collision:

- `Libraries/turbo-semantic/Cargo.toml` declares `package.name = "semantic-memory"`.

Containment status:

- `turbo-semantic` is not a member/default-member of the `Libraries` workspace.
- No active path dependency in `Libraries`, `Recall`, `Recall-Coding`, or `Gloss` points at `turbo-semantic`.
- Duplicate scan only passes with explicit `--allow semantic-memory`; this is a containment waiver, not a resolution.

Blocked action:

- Rename, merge, or physical quarantine of `turbo-semantic` requires Josh approval and targeted tests because the prior ledger marks this as an unapproved semantic collision.

## Gloss Boundary

Gloss now compiles against canonical `Libraries/semantic-memory`, `Libraries/llm-pipeline`, and `Libraries/tauri-queue` through `Gloss/src-tauri/Cargo.toml`.

Boundary evidence:

- Gloss feature text and code keep `semantic-memory-preview` optional; local Gloss remains default (`src-tauri/src/features.rs`).
- Local fallback/degradation is explicitly represented by settings and receipts (`memory_backend_fallback`, `fallback_used`, `fallback_reason`, and `semantic_memory_projection_status`).
- `docs/SOURCE_OF_TRUTH_MAP.md` states that FTS/BM25 local retrieval is separate from semantic-memory projection.
- `docs/CURRENT_FEATURE_MATRIX.md` marks semantic-memory preview as degraded.

Validation:

- `cargo metadata --manifest-path /home/sikmindz/Coding/Gloss/src-tauri/Cargo.toml --format-version 1`: pass.
- `cargo check --manifest-path /home/sikmindz/Coding/Gloss/src-tauri/Cargo.toml --features semantic-memory-backend`: pass.

## Gate

Phase 05 passes as containment. The canonical semantic-memory identity is clear for builds and downstream dependency truth. The `turbo-semantic` duplicate remains an unresolved, documented collision and must not be promoted as canonical without approval.
