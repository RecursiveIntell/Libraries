# Codex Prompt — P09 Canonical memory adapter over Forge, bridge, semantic-memory, and runtime

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P09_EPISODE_FIRST_MEMORY_AND_BITEMPORAL_STORE.md`.

Implement P09 only. Do not start later passes.

## Goal

Turn `aidens-memory-kit` into a thin adapter over the canonical memory libraries.

## Primary crates

- `aidens-memory-kit`
- `aidens-contracts`
- `aidens-receipts`
- `aidens-boundary-kit`
- `aidens-cli`

## Required artifacts

- `semantic_memory_forge::EpisodeBundleV1`
- `semantic_memory_forge::ExportEnvelopeV3`
- `forge_memory_bridge::ProjectionImportBatchV3`
- `semantic_memory::MemoryStore`
- `knowledge_runtime::KnowledgeRuntime`

## Acceptance gates

- Can insert a claim, supersede it retroactively, and answer both “what was true at valid time V” and “what did we believe at recorded time R”.
- MemoryRequired app fails doctor/run if no memory store is configured.
- No destructive update changes historical belief without supersession receipt.

## Forbidden shortcuts

- Do not implement memory as unversioned key-value notes.
- Do not collapse valid time and recorded time into one timestamp.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
