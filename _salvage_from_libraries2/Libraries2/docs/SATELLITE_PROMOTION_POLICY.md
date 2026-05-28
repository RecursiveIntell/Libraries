# SATELLITE_PROMOTION_POLICY.md

This repo ships a supported core lane and an excluded satellite lane.

## Promotion law

- Core crates are the only default ship bar.
- Satellites stay excluded unless they prove canonical `stack-ids` ownership, durable `TraceCtx` flow, and packaging/doc truth.
- Compatibility shims are allowed at satellite boundaries only when the canonical `TraceCtx` / `AttemptId` / `TrialId` fields remain the source of truth.
- Utilities that are not operationally connected to the supported lane remain smoke-checked only.

## Current classification

| Tier | Crates | Policy |
|---|---|---|
| Supported release lane | `forge-pilot`, `stack-ids`, `llm-tool-runtime`, `recursive-kernel-core`, `constraint-compiler`, `kernel-execution`, `kernel-oracles`, `kernel-conformance`, `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, `knowledge-runtime`, `living-memory/living-memory`, `contract-schema-gen` | This is the root workspace `default-members` lane. It must pass `make gate`, and release claims may rely on it only when the recorded evidence exists. |
| Operational satellites | `LLM-Pipeline`, `agent-graph`, `job-queue`, `Tauri-Queue`, `AI-Batch-Queue` | Excluded from the default ship bar, but must preserve canonical id/trace flow and pass `bash scripts/check_excluded_ecosystem_smoke.sh`. |
| Utility satellites | `ComfyUI-RS`, `Ollama-Vision-RS` | Build-smoke only; no supported-lane authority claims. |

## Proof posture

- `TRACE-001` is proven by supported-lane roundtrip evidence plus the excluded ecosystem smoke lane.
- `SAT-001` does not promote satellites by default. It fences them with an explicit policy and a concrete smoke command.
