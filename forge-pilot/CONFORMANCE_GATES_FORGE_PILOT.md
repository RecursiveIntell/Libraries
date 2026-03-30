# Conformance gates for forge-pilot

## Gate F1 — authority integrity

**Must prove:**

- no direct writes into `semantic-memory` tables,
- no direct write-through compatibility path,
- no pilot-local truth persistence.

**Disqualifying failure:**

- pilot bypasses `export_bundle -> transform_envelope_v3 -> import_projection_batch`.

---

## Gate F2 — observation determinism

**Must prove:**

- the same namespace + import log + kernel payload yields the same `Observation`,
- graph hash and selected primary inputs are stable,
- missing `kernel_payload_json` degrades explicitly.

**Disqualifying failure:**

- observation depends on hidden mutable state or private runtime internals.

---

## Gate F3 — targeting honesty

**Must prove:**

- target extraction is deterministic,
- retry decay and exhaustion are deterministic,
- thin export produces thin-export targets or degradation, not invented hyper-structure.

**Disqualifying failure:**

- target extraction invents unsupported semantics.

---

## Gate F4 — execution-family correctness

**Must prove:**

- at least one kernel-oracle plan executes correctly,
- at least one paired patch plan executes correctly,
- advisory-only plans are visibly advisory-only.

**Disqualifying failure:**

- every target is forced through the patch runner, or oracle plans are reimplemented locally.

---

## Gate F5 — canonical roundtrip

**Must prove:**

- pilot-built bundles export through `ExportEnvelopeV3`,
- bridge transform succeeds,
- import succeeds or records explicit failure,
- subsequent observation sees the new import.

**Disqualifying failure:**

- pilot writes loop results anywhere outside the canonical import lane.

---

## Gate F6 — loop discipline

**Must prove:**

- `max_iterations`, `time_budget_secs`, and `cooldown_secs` are enforced,
- halt reasons are honest,
- advisory-only steps do not inflate success metrics.

**Disqualifying failure:**

- unbounded loop, silent retries, or hidden cooldown bypass.

---

## Gate F7 — optional LLM isolation

**Must prove:**

- default build works without `LLM-Pipeline`,
- enabling LLM refinement does not change authority class,
- LLM output cannot promote truth or choose targets on its own.

**Disqualifying failure:**

- LLM feature becomes required for the crate to compile or function.
