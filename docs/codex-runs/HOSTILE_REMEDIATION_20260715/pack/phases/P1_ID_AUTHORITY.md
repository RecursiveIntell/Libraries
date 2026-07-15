# Phase 1 — Canonical ID authority

Issues: `ID-001`, `ID-002`.

## Inventory fields

Concept, declaration, public/storage/wire usage, construction sites, lifecycle class, canonical owner,
migration compatibility, and removal condition.

## Freeze stack-ids V2

Private representations; typed errors; validating serde/schema; random/deterministic/version/imported/
receipt/local families; domain-separated derivation; explicit legacy adapters; compile-fail tests.

## Migration waves

1. graph and queues;
2. claim-ledger;
3. memory and AiDENs requests/receipts;
4. codec/profile/artifact;
5. residual cross-crate IDs.

Every wave is dual-read/single-write with compatibility receipt.

Exit: no unallowlisted production authority outside stack-ids and no domain policy leaked into it.
