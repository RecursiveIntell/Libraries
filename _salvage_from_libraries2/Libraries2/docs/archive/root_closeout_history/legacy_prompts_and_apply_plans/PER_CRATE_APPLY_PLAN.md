# Per-Crate Apply Plan

## agent-graph

### Objective
Make the public error surface boring and correct.

### Required outcome
- no public `anyhow` leakage
- no behavior change for existing error kinds
- no checkpointing regression

### Notes
This crate is excluded from the root workspace but still matters because its public API is real.

## forge-pilot

### Objective
Make the pilot crate feel like a finished orchestrator rather than a smart binary with a storage-room main file.

### Required outcome
- exactly one public error type
- binary-only support code moved under `src/main_support/`
- main file becomes a dispatch shell, not the whole application

### Notes
Do not create new public library APIs just to serve the binary.
Keep support logic private to the binary path where possible.

## knowledge-runtime

### Objective
Split the runtime into a stable module tree without changing its doctrine or public API.

### Required outcome
- `runtime/` directory exists
- execution logic, warnings, temporal helpers, verification summary helpers, and cache handle logic are separated
- cross-crate tests still target the same semantics

### Notes
This is a decomposition pass, not a behavior rewrite.

## Primitives/cea-core

### Objective
Make confidence conservative, coverage-aware, and sample-aware.

### Required outcome
- confidence becomes a lower-bounded / uncertainty-aware quantity
- coverage and sample sufficiency are not averaged away into fake certainty
- risk flags need real support, not fuzzy optimism

### Notes
Keep the lane advisory.

## living-memory / forge-engine

### Objective
Stop using a hand-rolled optimistic hypothesis confidence and keep the package/docs story coherent.

### Required outcome
- local heuristic confidence is aligned with the primitive CEA confidence model or clearly downgraded in meaning
- `danger-sm-write` stays fenced and off by default

## semantic-memory-forge

### Objective
Prevent exported confidence classes from laundering advisory CEA outputs into stronger truth classes.

### Required outcome
- export semantics remain explicit about advisory vs verified

## contract-schema-gen and root scripts/docs

### Objective
Close out schema-owner drift, hotspot drift, mirror discipline, and front-door truth.

### Required outcome
- one machine check for duplicate schema ownership
- one machine check for hotspot budgets
- one machine check or explicit doc rule for mirror discipline
- root docs tell the truth about the current state
