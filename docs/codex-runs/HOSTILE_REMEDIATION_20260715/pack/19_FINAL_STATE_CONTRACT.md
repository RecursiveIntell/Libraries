# Final state contract

- `stack-ids` is the only cross-crate ID definition/construction authority; private validated
  representations and lifecycle-specific constructors.
- Structured digests use domain/version-separated length framing with explicit historical readers.
- Errors/absence/corruption cannot become success; governance defaults fail closed.
- Raw state remains distinct from derived indexes/compression.
- One codec/profile/wire contract with typed score semantics/capabilities is implemented by all backends.
- Queue claims/transitions are atomic; completion requires terminal work; lease/cancellation uncertainty is observable.
- Ledger parsing is strict and completeness is anchored.
- Every required workspace/feature/platform lane is explicit.
- Verification is read-only; receipts are source/environment-bound; final tree is clean.
- Quantitative/readiness claims are classified and receipt-backed.
