# Changelog

All notable changes to `quant-governor` are documented here.

## [Unreleased]

## [0.1.0] — 2026-06-02

Initial crates.io release.

### Added

- `GovernancePolicy` — declares codec profiles and admissibility
  classes for governed compression routing.
- `GovernanceRequest` — the request struct passed to `evaluate()`.
  Carries `ContentType`, size, accuracy requirements, and the
  caller-supplied `PolicyId`.
- `CodecProfile` — enum over `Raw`, `Q8`, `Q4`, `Turbo`, `Fib`
  compression strategies.
- `AdmissibilityClass` — `Exact`, `Lossless`, `Approximate`.
- `CodecDecision` — output of `evaluate()`. Carries the selected
  profile, a BLAKE3 `profile_digest`, and optional `degradation`
  metadata if the policy had to fall back.
- `ExactFallbackReceipt` — emitted when a caller forced an Exact
  class but the policy selected a Lossless or Approximate codec.
  Carries the original `ContentType`, the selected profile, and
  the reason string.
- `DegradationReceipt` — emitted when a codec downgrades from
  higher fidelity to lower fidelity (e.g. Q8 → Q4). Carries the
  pre/post profiles, the `accuracy_loss_estimate`, and the
  trigger.
- `evaluate(request, &policy) -> CodecDecision` — the routing
  function. Stateless: given the same request and policy, always
  returns the same decision.

### Doctests

- 3 doctests in the lib.rs doc-comment covering the default
  policy, a `ContentType::Embedding` request, and the receipt
  round-trip.

### Lints

- `#![forbid(unsafe_code)]` at the lib level.
- `#![deny(missing_docs)]` — every public item is documented.
- `#![deny(rustdoc::broken_intra_doc_links)]` — doc links
  are verified at compile time.
- `cargo clippy --all-targets -- -D warnings` clean.

### Test coverage

- 22 integration tests in `tests/` covering: policy evaluation
  determinism, profile digest stability, admissibility class
  transitions, fallback receipt generation, degradation
  triggers, and ContentType routing.
- 1 example (`examples/basic_policy.rs`).

### Notes

- Extracted from `poly-kv`'s `governance` module in the V29
  audit. The crate was promoted to a workspace peer so it
  could be reused outside the poly-kv-stack.

[Unreleased]: https://github.com/nousresearch/Libraries/compare/quant-governor-v0.1.0...HEAD
[0.1.0]: https://github.com/nousresearch/Libraries/releases/tag/quant-governor-v0.1.0
