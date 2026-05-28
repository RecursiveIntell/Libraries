# Truthful Phase Model

## Problem statement

The current repo still uses a phase model that was acceptable for the earlier bootstrap build but is no longer acceptable for a product-looking surface.

Today the worker can:

- queue a run
- capture a baseline
- mark the run `completed`

That is misleading because the intended product still needs:

- candidate generation
- verification
- recommendation/rejection
- memory import
- audit
- approval
- apply

A baseline-only success is **not** a completed repair run.

## Current behavior that needs correction

### Worker order today
`RepairRunJob::execute` currently does this:

1. update phase to `detecting_repo`
2. run baseline capture
3. after success, update phase to `capturing_baseline`
4. persist baseline
5. update phase to `completed`

That is wrong in two ways:

- the phase updates do not match the real order of work
- `completed` is used too early

## Required principles

- phase changes must happen **before** the corresponding work begins
- terminal success must mean **full run success**, not partial progress
- failure states must be specific enough to explain what failed
- the UI badge/tone system must map one-to-one with actual run truth

## Recommended v1 phase model

### Intake and baseline
- `queued`
- `preflighting_repo`
- `detecting_repo`
- `capturing_baseline`
- `baseline_captured`

### Candidate lane
- `retrieving_memory`
- `compiling_mindstate`
- `generating_candidates`
- `validating_candidates`
- `selecting_candidate`

### Verification lane
- `running_verification`
- `scoring_and_explaining`

### Finalization
- `exporting`
- `importing`
- `awaiting_user_decision`
- `applying_approved_patch`
- `completed`
- `rejected`
- `failed`
- `cancelled`

## Meaning of `completed`

`completed` must be reserved for:

- a candidate was generated
- a candidate was verified
- the candidate reached the terminal success path appropriate to the product state
- any required import/apply side effects for that chosen path were completed or intentionally skipped with a durable record

If the product chooses not to apply automatically, a run can still be `completed` after verification and decisioning as long as the terminal state is truthful and durable.

## Transitional rule while the product is still incomplete

Until the full lane exists, baseline-only success must be expressed as one of:

- `baseline_captured`
- `candidate_generation_pending`
- `partial_success`

The best option for this repo is `baseline_captured`.

That makes it honest without inventing product completion too early.

## UI mapping rules

### Phase pill tone
- active/in-flight phases → neutral or warning
- `baseline_captured` → success, but not terminal-complete language
- `completed` → success terminal
- `failed` → danger
- `cancelled` → neutral
- `rejected` → warning or danger depending on final language

### Baseline card status
- `captured` when baseline exists
- `failed before capture` if run failed with no baseline
- `in progress` during capture
- never `pending` on a terminal failed run

### Header metadata
Split provider/model into separate fields:
- Provider
- Model

Do not hide them under a vague `Execution backend` card.

## Timeline event contract

Progress events should carry at least:
- `phase`
- `message`
- optional `failure_class`
- optional `diagnostics`

That is enough for a usable timeline and for later richer debug UX.

## Tests required

- success path yields phase order: queued → preflighting/detecting → capturing_baseline → baseline_captured
- failure before capture yields terminal `failed` with no baseline
- failure after partial work still shows truthful last phase
- `completed` is unreachable from baseline-only worker paths

## Related issues

- `FW-018`
- `FW-019`
- `FW-020`
- `FW-021`
- `FW-022`
