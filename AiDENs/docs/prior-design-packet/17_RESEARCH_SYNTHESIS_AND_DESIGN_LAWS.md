# 17 — Research Synthesis and Design Laws

This packet uses the uploaded provenance/research corpus as design pressure, not as decoration. The strongest recurring signals are below.

## 1. Execution is evidence

Across the Recall-specific research and broader provenance/control-plane research, execution metadata is not telemetry exhaust. It is meaning-bearing evidence.

AiDENs design consequence:

```text
aidens-receipts is foundational, not optional.
```

Every retry, fallback, queue hop, approval, denial, timeout, tool call, boundary repair, provider route, schedule fire, and daemon recovery needs a typed receipt.

## 2. Arbitration must be an artifact

The Recall research specifically frames route selection as a law surface: no-tool, tool loop, parser fallback, delegate, schedule continuation, abstain, and needs-review must be typed choices with evidence references.

AiDENs design consequence:

```text
aidens-arbiter-kit owns ArbiterDecisionV1.
```

A run labeled `no_tools` with tool-attempt receipts is a conformance failure.

## 3. Tool exposure must be minimal and per-turn

Research and current Recall both converge on `ExposedToolSetV1` as necessary. The model should not see every tool by default.

AiDENs design consequence:

```text
aidens-tool-kit owns ToolExposureSetV1 and exposure planning.
```

Disabled tools must be absent, not merely denied after invocation.

## 4. Contract discipline beats rescue heuristics

The Rust/JSON/contract research repeatedly argues for type-owned contracts, generated schemas, schema meta-validation, and compatibility gates.

AiDENs design consequence:

```text
aidens-contracts + aidens-boundary-kit are separate.
```

Contracts define meaning. Boundary-kit enforces input language, structured-output repair, canonicalization, patch semantics, and repair receipts.

## 5. Time is semantic

Bitemporality and as-of retrieval appear throughout the research. Valid time and recorded time are not optional metadata.

AiDENs design consequence:

```text
aidens-memory-kit must preserve valid_as_of and recorded_as_of.
```

Reranking cannot widen temporal/scope boundaries silently.

## 6. Durable schedules are not host timers

The Recall-specific research emphasizes schedule overlap/catchup/misfire semantics. Host wake systems are adapters, not truth.

AiDENs design consequence:

```text
aidens-schedule-kit != aidens-wake-kit != aidens-queue-kit.
```

Host wake backends may arm or wake. They do not define canonical recurrence law.

## 7. Reference interpreters prevent drift

The conformance research recommends simple reference behavior plus differential/metamorphic harnesses for semantic seams.

AiDENs design consequence:

```text
aidens-testkit is a core crate.
```

It should include fixtures for arbiter decisions, exposure planning, boundary repair, permit attenuation, schedule next-fire calculation, queue lease semantics, and run receipt completeness.

## 8. Regions and right-graph law matter later

The decoder/region research warns against one giant graph. It argues for small communicating regions, explicit graph compilation, residuals/syndromes, certificates, and oracle slices.

AiDENs design consequence:

```text
aidens-kernel-kit is optional and later-phase.
```

It should not contaminate v0.1, but the architecture must reserve the seam.

## 9. Learning and heuristics stay downstream

The research consistently warns against learned scores or heuristics outranking verified truth.

AiDENs design consequence:

```text
learned routing/scheduling/ranking may assist but cannot promote truth or erase receipts.
```

## 10. The core law

Everything materially important should be a typed, replayable, inspectable artifact:

```text
app plan
config generation
capability truth
arbiter decision
tool exposure set
permit grant
provider route receipt
tool attempt receipt
run receipt
queue hop receipt
schedule fire receipt
boundary repair receipt
memory write receipt
view disclosure
repair record
```

The root crate should make these feel automatic. The internal crates should make them impossible to skip accidentally.
