# SCR-P0A Non-Goals

SCR-P0A is a deterministic reference control evaluator over proposed actions.
It is not an integration pass and not a generic risk scoring library.

P0A does not implement:

- memory governance;
- retrieval governance;
- tool exposure governance;
- Recall integration;
- AiDENs integration;
- learned calibration;
- storage or database layers;
- UI layers;
- automatic repair execution;
- network-backed verification;
- LLM, model, embedding, or stochastic scoring paths;
- source-of-truth replacement for IDs, artifacts, evidence, provenance,
  policies, schemas, repository state, or execution state.

P0A may produce future integration docs only. Those docs must describe sequence
and boundaries; they must not wire SCR into Recall, AiDENs, memory, retrieval,
or tools in this pass.

The evaluator answers only:

```text
Given this proposed action under this authority and evidence basis, what
control outcome is allowed?
```

It must not answer:

```text
This issue is true.
This patch is safe.
This memory is correct.
This source is canonical.
```
