# Phase 04 — Manifests, Receipts, and Pool Core

## Objective

Implement `poly-kv` core data model without codecs beyond raw placeholders.

## Required actions

1. Implement `KvPoolManifestV1`, layer manifests, policies.
2. Implement receipt structs.
3. Implement `SharedKvPool` builder skeleton and immutable inner state.
4. Implement digest/serde roundtrips.
5. Run tests.

## Acceptance gate

Manifests/receipts serialize roundtrip; no reader mutation path exists.

## Phase-boundary report must include

- files inspected;
- files changed;
- commands run;
- tests/checks passed/failed/skipped;
- source-of-truth boundary status;
- unresolved blockers;
- rollback notes.
