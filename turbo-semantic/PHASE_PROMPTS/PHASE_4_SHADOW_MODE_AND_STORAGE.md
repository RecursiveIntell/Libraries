# Phase 4 — Shadow Mode Encode/Persist/Evaluate Path

## Goal

Allow semantic-memory to encode TurboQuant sidecar artifacts without changing authoritative search/write behavior.

## Required changes

1. Add sidecar storage:
   - codec profiles table;
   - encoded vector artifacts table;
   - optional eval runs table or JSON report output.
   - idempotent migration.

2. Hook write paths:
   - on fact/document/chunk/message/episode embedding write, optionally shadow-encode if enabled;
   - raw embedding write remains authoritative;
   - shadow encode failure records degradation and does not fail write unless strict config exists and is enabled.

3. Add encode receipts:
   - entity type/key;
   - profile digest;
   - encoded length;
   - checksum;
   - status;
   - error/degradation if any;
   - recorded time.

4. Tests:
   - default config writes no sidecar rows;
   - shadow config writes sidecar rows;
   - sidecar rows do not replace raw embedding;
   - injected shadow failure does not break write;
   - migration idempotent.

## Non-goal

Do not use TurboQuant scores for production ranking yet.
