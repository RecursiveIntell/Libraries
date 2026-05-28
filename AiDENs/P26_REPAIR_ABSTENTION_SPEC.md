# P26 Repair and Abstention Spec

## Purpose

Advanced agents must be able to stop honestly.

## Abstention cases

- Unsupported provider/cloud path.
- Missing permit.
- Ambiguous tool authority.
- Invalid or hostile JSON/patch.
- Memory seam unavailable.
- Verification failure.
- Budget/deadline exhaustion.
- Package self-replay failure.

## Required output

Each abstention must include:

- reason code,
- blocked action,
- evidence collected,
- what would be required to proceed,
- support-tier impact,
- whether operator action can resume it.

## RepairPlanDisplayV1

This is AiDENs-local display evidence only. It must not claim canonical repair truth.

Required fields:

- repair_id,
- source_run_id,
- failure_kind,
- candidate_repair_actions,
- required permits,
- required verification,
- risk level,
- canonical owner if applicable.
