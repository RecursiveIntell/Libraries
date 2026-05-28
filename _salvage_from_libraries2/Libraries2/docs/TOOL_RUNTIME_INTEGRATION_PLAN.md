# Tool Runtime Integration Plan

## Goal

Turn the tool/runtime/orchestration ecosystem into an evidence-bearing control plane.

## In-scope crates

- `llm-tool-runtime`
- `forge-pilot`
- `LLM-Pipeline`
- `agent-graph`
- `job-queue`
- `AI-Batch-Queue`
- `Tauri-Queue`

## Target boundary

These crates should communicate through typed execution artifacts rather than ad hoc logs.

### Minimum receipt family

- tool-dispatch receipt
- tool-call result receipt
- queue-hop receipt
- retry-family receipt
- deadline/budget propagation receipt
- replay linkage receipt
- cancellation / failure receipt

## Required fields

- stable receipt id
- attempt / trial / family ids
- trace context
- workload class
- budget / deadline context
- producer crate / version
- result summary or failure taxonomy
- replay parent if applicable

## Integration order

1. Define the canonical receipt shapes.
2. Land sinks/adapters in `llm-tool-runtime` and `forge-pilot`.
3. Bridge queue/orchestrator satellites to the same artifact family.
4. Expose receipts to the evidence / audit path without promoting them into source truth.

## Why this matters

The stack now has a real control ecosystem. Without receipts, execution lineage becomes folklore.
