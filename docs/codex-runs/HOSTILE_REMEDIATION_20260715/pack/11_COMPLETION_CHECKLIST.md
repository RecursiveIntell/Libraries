# Completion checklist

## Correctness
- [ ] AG-001, GOV-001, CMP-001 post-merge validated.
- [ ] Every P1 issue closed with independent review.
- [ ] No error/absence/corruption path becomes success/default/allow.

## Authority
- [ ] stack-ids is sole canonical cross-crate ID authority.
- [ ] Framed versioned digest law.
- [ ] One codec/profile/wire authority.
- [ ] Raw/SQLite authority remains distinct from sidecars/projections/receipts.

## Runtime
- [ ] Atomic queue claims/transitions and lease ownership.
- [ ] Strict/degraded search policy explicit.
- [ ] Strict ledger parse and head anchor.

## Release proof
- [ ] Every required workspace and feature/platform lane passed.
- [ ] Lint inheritance complete or exact expiring exception.
- [ ] Verify is read-only; record is separate.
- [ ] Receipts bind source/environment/logs.
- [ ] Claims manifest covers quantitative prose.
- [ ] Final tree clean.
- [ ] Independent hostile auditor verdict is accept.
