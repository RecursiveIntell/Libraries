# Migration to 0.2.0

`claim-ledger` 0.2.0 intentionally changes deterministic identifiers and the
ledger integrity API.

- Stable IDs now use the `claim-ledger.stable-id.v2` binary preimage. It binds
  the scheme version, prefix, part count, and byte lengths. Persisted IDs from
  0.1.x will change when regenerated; treat them as a distinct ID generation
  epoch and retain old IDs as historical references.
- Ledger builders, digest computation, and JSONL serialization/parsing now
  return `Result<_, ClaimLedgerError>`. Handle failures rather than accepting
  an empty serialization fallback.
- `verify_ledger` now requires an `ExpectedLedgerHead`. Use
  `ExpectedLedgerHead::empty()` only for a deliberately empty ledger; otherwise
  bind the expected final sequence and digest with `ExpectedLedgerHead::new`.
- Ledger digests use the documented canonical binary preimage in
  `ledger::entry_digest_preimage`, rather than serializer-dependent JSON.
- A proof-debt waiver no longer credits a budget. It is now explicitly split
  into public receipt data (`ProofDebtWaiverReceipt`) and a private-field,
  non-serializable verified authorization (`VerifiedProofDebtWaiver`). Raw,
  forged, or deserialized receipt data cannot produce `Waived` directly.
  Call `verify_proof_debt_waiver(&budget, receipt, authorize_operator)` first,
  supplying the caller's operator-authority check, then pass only the returned
  `VerifiedProofDebtWaiver` to `evaluate_proof_debt_gate_with_waiver` or
  `ProofDebtSummaryV1::from_budget_with_waiver`.
- Waiver identities now bind the schema and authorization domain, budget ID,
  scope, budget ceiling, captured outstanding debt, waived amount, operator,
  rationale, and canonical RFC 3339 UTC recorded time. Legacy waiver receipts
  are not valid under this authorization model and must be re-issued and
  re-authorized against the current budget/debt snapshot.
- A verified waiver never reduces debt and becomes inapplicable when its bound
  budget ID, scope, ceiling, or outstanding-debt snapshot no longer matches.
