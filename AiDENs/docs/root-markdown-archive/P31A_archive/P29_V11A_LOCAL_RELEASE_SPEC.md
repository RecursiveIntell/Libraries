# P29 v11A Local Release Candidate Spec

## Material operation contract

Every material operation on the supported-local path must have:

- `OperatorContractV1`;
- declared inputs;
- declared outputs;
- declared effects;
- forbidden effects;
- preconditions;
- proof obligations;
- degradation behavior;
- replay requirements.

## Execution context

Every material operation must receive an `ExecutionContextEnvelopeV1` containing:

- run id;
- trace id;
- attempt id;
- retry family;
- route;
- environment fingerprint;
- budget/deadline;
- degradation/truncation state;
- replay handle.

## Receipts

Every material operation must emit:

- invocation receipt;
- tool receipt if tool call;
- input/output manifest;
- proof/debt/waiver state;
- degradation record if partial/failed/repaired;
- semantic/view disclosure for user-visible outputs.

## Local path under test

```text
AgentSpecV1
→ validate/doctor
→ runner Plan/Act/Verify
→ repo read/search/propose/apply/check
→ run bundle
→ receipt store/event log
→ inspect/replay
→ final report
```
