# P28 Conformance Fixture Plan

## v11A exact fixtures

| Fixture | Expected result |
|---|---|
| material operation with no operator contract | blocked |
| material operation with undeclared effect | blocked |
| material done state without receipts | blocked |
| risk-bearing artifact without proof profile | blocked or proof debt |
| proof waiver treated as proof | fail |
| degraded surface in release readiness | block unless lawful waiver |
| parser repair changes treatment silently | fail |
| duplicate JSON key in boundary input | reject/quarantine |
| schema mismatch at boundary | reject/quarantine |
| run bundle same run-id overwrite | reject/supersede with receipt |
| tampered event log previous digest | detect fail |
| package degraded subcheck with exact aggregate | fail |

## Tool/patch hostile fixtures

| Fixture | Expected result |
|---|---|
| read through symlink escaping sandbox | blocked |
| create new file under symlinked parent escaping sandbox | blocked |
| failed patch write after parent dir creation | no dirty dirs or rollback receipt |
| repo_list symlink entry | reported as symlink or blocked, not followed silently |
| file_stat read failure | error receipt, not empty digest |
| timeout command output | partial/truncated flag set |
| disallowed command with extra args | blocked with attempted command recorded |

## Temporal/view fixtures

| Fixture | Expected result |
|---|---|
| valid-time only query | deterministic expected state |
| recorded-time only query | deterministic expected belief state |
| combined as_of(valid, recorded) query | deterministic state |
| retroactive correction | previous recorded belief remains queryable |
| stale projection answering current query | degraded disclosure or block |
| timeless fallback forbidden | block or explicit degradation |

## v11B/v11C reserved fixtures

These do not activate v11B/v11C. They prevent incompatible shadows.

| Fixture | Expected result |
|---|---|
| storage graph used as inference graph | fail reserved right-graph test |
| subtraction deletes support core | fail reserved subtraction test |
| external valid-schema artifact with no admission | quarantined |
| generated law self-admits | blocked/human veto required |
| personalized repeated advice without agency classification | blocked or agency receipt required |
