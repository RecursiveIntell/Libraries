# Phase 04 — Authority, evidence, owner-boundary, rollback basis

## Goal

Make all material bases explicit without inventing external truth.

## Tasks

1. Implement `AuthorityCheckV1` or equivalent:
   - result enum: `verified`, `declared_by_adapter`, `missing`, `insufficient`, `unknown`.
   - basis refs and reason codes.
2. Implement `EvidenceCheckV1` or equivalent:
   - result enum: `sufficient`, `insufficient`, `uncertain`, `conflicting`, `unknown`.
   - confidence bps if used.
3. Implement owner-boundary basis:
   - `known_owner`
   - `unknown_owner`
   - `adapter_declared_owner`
   - `external_owner_unavailable`
4. Implement rollback/containment basis:
   - present/missing/not_required/unknown.
5. Hard rules:
   - mutation with unknown owner => quarantine or require repair packet.
   - destructive/apply/release without rollback basis => deny/quarantine.
   - release without sufficient evidence => verify/deny.
   - missing authority for material mutation/release => deny/quarantine.
6. Do not call any basis verified unless the input or adapter explicitly provides verification status.

## Acceptance gate

- Receipts state whether authority/evidence were verified, declared, or unknown.
- Tests cover missing authority, insufficient evidence, missing rollback, unknown owner.
