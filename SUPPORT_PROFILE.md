# Support profile

Supported closeout lane:
- `contract-schema-gen`
- `forge-memory-bridge`
- `forge-pilot`
- `kernel-conformance`
- `kernel-execution`
- `kernel-oracles`
- `knowledge-runtime`
- `living-memory/living-memory`
- `llm-tool-runtime`
- `recursive-kernel-core`
- `semantic-memory`
- `semantic-memory-forge`
- `stack-ids`
- `verification-adjudication`
- `verification-calibration`
- `verification-control`
- `verification-policy`

This is the narrow release-facing support claim used by `release/closeout_receipt_v1.json`.
It is also the public-doc-certified core checked by `python3 scripts/check_public_api_docs.py`.

Adjacent artifact-owner crates for the demo/benchmark substrate are documented in `SCOPE_NOTES.md`.
They are not part of the narrow build-certified or public-doc-certified claim of the 2026-03-22 hardening receipt.

## Governance crates (V28, build-checked, default-enabled)

The following governance crates now have typed error enums, integration documentation, and are build-checked by `cargo check --workspace`. As of V28, they compile by default (`default = ["governance"]` in forge-pilot) and the governance observation pipeline is live:

- `assurance-runtime`
- `attestation-exchange`
- `authority-delegation`
- `constitutional-memory`
- `continuity-runtime`
- `effect-runtime`
- `mechanism-runtime`
