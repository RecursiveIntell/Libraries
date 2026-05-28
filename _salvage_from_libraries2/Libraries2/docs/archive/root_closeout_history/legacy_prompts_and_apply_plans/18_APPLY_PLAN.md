# Apply plan

## Recommended adoption path

1. keep this pack external first
2. apply `patches/0001-ci-hardening.patch`
3. create one branch dedicated to phase 0
4. land split passes as separate PRs
5. land v13 IDs + Forge artifacts next
6. land bridge preservation
7. land semantic-memory migration + additive reads
8. land control consumption
9. land reference fixtures
10. do second-order cleanup only after substrate proof exists

## Suggested branch breakdown

- `next-pass/ci-truth`
- `next-pass/split-verification-control`
- `next-pass/split-constraint-compiler`
- `next-pass/split-kernel-execution`
- `next-pass/split-kernel-oracles`
- `next-pass/v13-forge-contract`
- `next-pass/v13-bridge`
- `next-pass/v13-semantic-memory`
- `next-pass/v13-control`
- `next-pass/v13-reference-fixtures`
- `next-pass/cleanup`

## Merge discipline

Never mix:
- split + semantic change
- storage migration + control behavior
- governance docs + doctrine changes

That is how review stops working.
