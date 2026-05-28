# P28 Rollback and Quarantine Plan

## Rollback doctrine

P28 modifies contract/runtime semantics. Any partial implementation that weakens supported-local behavior must be reverted or quarantined before final packaging.

## Quarantine classes

| Class | Meaning | Release effect |
|---|---|---|
| `release_blocking` | unsafe or false claim risk | final P28 cannot claim v11A core |
| `support_downgrade` | local path still works but target claim not met | downgrade support label |
| `draft_only` | v11B/v11C prototype not active | allowed if non-authoritative |
| `doc_only` | planning artifact only | allowed if labeled |

## Required quarantine record fields

- item id
- source issue id
- affected files
- reason
- current behavior
- risk if unquarantined
- release effect
- owner/follow-up phase
- proof/debt/waiver refs if applicable

## Immediate rollback triggers

- material operation can claim done without receipts
- parser repair changes treatment silently
- proof waiver treated as proof
- degraded check aggregates to exact status
- run bundle overwrite silently erases previous evidence
- patch/write path can escape sandbox
- external artifact influences truth without admission
- AiDENs starts owning canonical truth outside its lane

## Safe rollback approach

1. Revert high-risk code change.
2. Preserve tests demonstrating the failure if possible.
3. Add quarantine record and release-blocking status.
4. Update P28 status manifest.
5. Downgrade support claim if necessary.
