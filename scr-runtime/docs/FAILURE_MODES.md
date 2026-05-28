# Failure Modes

Known failure modes and outcomes:

| Failure | Outcome |
|---|---|
| Invalid input schema | `quarantine_artifact` receipt when evaluation can build a receipt, otherwise explicit parse error. |
| Missing authority basis | `require_approval` by hard rule. |
| Forbidden historical term in production path | `quarantine_artifact` by fixture signal and hostile script failure in scanned production paths. |
| Unknown owner for mutation | `require_owner_resolution`. |
| Source-basis drift | minimum `require_verification`. |
| False completion with missing tests | `generate_repair_packet`. |
| Destructive release without rollback | `block_release`. |
| Durable float score type | hostile script failure. |
| Naked decision boolean API | hostile script failure. |

Parse errors, policy validation errors, and serialization errors return
`ScrError` with stable `kind()` discriminants.
