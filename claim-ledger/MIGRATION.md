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
- A proof-debt waiver no longer credits a budget. Construct it from the budget,
  then pass it to `evaluate_proof_debt_gate_with_waiver`; summaries expose both
  outstanding debt and waiver information.
