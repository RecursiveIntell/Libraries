# 09 — Memory, Kernel, and Runtime Model

## Memory model

`aidens-memory-kit` wraps existing memory libraries:

```text
semantic-memory
semantic-memory-forge
forge-memory-bridge
knowledge-runtime
profile-runtime
```

It must preserve authority boundaries:

```text
Forge = raw verification/evidence truth
Bridge = deterministic transform only
Semantic memory = queryable projected truth
Knowledge runtime = planning/query/merge/provenance, not durable truth
Runner = consumer, not memory DB
```

## Memory modes

```text
disabled
optional
required
```

If memory is required and migration/opening fails, app startup blocks. If optional, runtime starts with explicit degradation.

## Scope model

Memory scopes should include:

```text
repo
project
personal
workspace
organization
external
```

Profiles must opt into cross-scope reads.

Coding profile defaults:

```text
repo/project memory only
personal memory denied unless explicit
```

## Temporal model

Every memory-backed answer should know:

```text
valid_as_of
recorded_as_of
temporal_mode
snapshot_id
view_policy_id
widening_disclosures
```

Reranking cannot widen time or scope silently.

## Memory writes

Memory writes must go through canonical memory crates and may emit library-owned receipts:

```text
Canonical memory write receipt/report
  candidate_id
  content_hash
  dedupe_status
  write_status
  provenance
  scope
  valid_time
  recorded_time
```

Dedupe should distinguish:

```text
exact hash dedupe
retrieval-assisted dedupe
semantic update
new fact
blocked
failed
```

## Kernel model

`aidens-kernel-kit` should wrap graph/kernel libraries but remain optional.

Adapters:

```text
agent-graph
constraint-compiler
recursive-kernel-core
kernel-execution
kernel-oracles
kernel-conformance
```

## Right-graph law

These are different objects:

```text
storage graph
retrieval graph
inference graph
repair graph
control/receipt graph
```

AiDENs must not collapse them by default.

## Snapshot law

Kernel runs should consume immutable inputs:

```text
memory_snapshot_id
contract_schema_id
compiled_graph_digest
app_plan_id
config_generation_id
```

Live streaming/incremental kernel execution must be explicitly declared.

## Region law

Future region runtime should exchange typed artifacts:

```text
deltas
residuals
syndromes
witnesses
certificates
repair proposals
receipts
```

No shared mutable hidden state across regions.

## Convergence law

Loopy or recursive graph execution requires:

```text
damping policy
scheduler policy
residual threshold
iteration cap
budget cap
oscillation handling
oracle escalation path
degradation receipt
```

## Repair model

Repair proposals must not self-promote.

```text
kernel detects syndrome
repair-kit creates repair proposal
control/governance decides
memory/bridge applies via lawful path
receipts record outcome
```
