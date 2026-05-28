# projection_storage v13 notes

## Read path changes
- current-state lookups should use transaction interval openness, not only `recorded_at DESC`
- recorded-as-of queries should bind against `tx_from` / `tx_to`
- valid-as-of queries should continue to bind against `valid_from` / `valid_to`

## Write path changes
- claim-version insertion should write `tx_from`
- supersession / retraction should close `tx_to`
- support-set and contradiction-witness artifacts should be inserted before claim-version references are committed

## Backpointer rule
Any `ProjectionClaimVersionV13` should be able to answer:
- which support set it uses
- which contradiction witness, if any, exists
- which retraction record closed its currentness, if any
