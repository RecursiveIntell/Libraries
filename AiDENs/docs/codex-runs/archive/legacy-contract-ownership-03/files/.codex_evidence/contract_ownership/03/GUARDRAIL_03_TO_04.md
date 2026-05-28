# GUARDRAIL_03_TO_04

## 1. Required Canonical Owner Crates Added Where Needed

Current manifest wiring:

- `Cargo.toml`
  - `attestation-exchange = { version = "0.1.0", path = "../attestation-exchange" }`
  - `contract-schema-gen = { version = "0.1.0", path = "../contract-schema-gen" }`
  - `federated-settlement = { version = "0.1.0", path = "../federated-settlement" }`
  - `mechanism-runtime = { version = "0.1.0", path = "../mechanism-runtime" }`
  - `remote-oracle-admission = { version = "0.1.0", path = "../remote-oracle-admission" }`
  - `verification-calibration = { version = "0.1.0", path = "../verification-calibration" }`
- `crates/aidens-contracts/Cargo.toml`
  - `attestation-exchange.workspace = true`
  - `federated-settlement.workspace = true`
  - `mechanism-runtime.workspace = true`
  - `remote-oracle-admission.workspace = true`
- `crates/aidens-cli/Cargo.toml`
  - `contract-schema-gen.workspace = true`
- `crates/aidens-governance-kit/Cargo.toml`
  - `verification-calibration.workspace = true`

`forge-pilot` is not a direct AiDENs dependency because the current AiDENs code has no typed pilot/control-loop receipt surface. The only direct AiDENs code mention found was a report string in `crates/aidens-repair-kit/src/lib.rs`; `forge-pilot` appears in cargo metadata through canonical `contract-schema-gen`.

## 2. No Libraries2 Dependency

Revalidation command:

```text
rg -n 'Libraries2|libraries2|Recall|Recall-Coding' Cargo.toml crates/*/Cargo.toml
```

Result:

```text
PASS: no Libraries2/Recall/Recall-Coding dependency paths in AiDENs Cargo manifests
```

## 3. No Local Substitute Module

Revalidation command:

```text
bash scripts/assert_no_local_substitute_dependencies.sh
```

Result:

```text
PASS: no local substitute dependency red flags detected.
```

`bash scripts/phase_verify_contract_ownership.sh 03` also passed and saved gate output.

## 4. Cargo Metadata And Check

Revalidation outputs:

- `.codex_evidence/contract_ownership/03/guardrail_03_to_04_outputs.txt`
- `.codex_evidence/contract_ownership/03/guardrail_03_to_04_cargo_metadata.json`

Current cargo proof:

```text
stack-ids: /home/sikmindz/Coding/Libraries/stack-ids/Cargo.toml
attestation-exchange: /home/sikmindz/Coding/Libraries/attestation-exchange/Cargo.toml
federated-settlement: /home/sikmindz/Coding/Libraries/federated-settlement/Cargo.toml
mechanism-runtime: /home/sikmindz/Coding/Libraries/mechanism-runtime/Cargo.toml
remote-oracle-admission: /home/sikmindz/Coding/Libraries/remote-oracle-admission/Cargo.toml
contract-schema-gen: /home/sikmindz/Coding/Libraries/contract-schema-gen/Cargo.toml
verification-calibration: /home/sikmindz/Coding/Libraries/verification-calibration/Cargo.toml
forge-pilot: /home/sikmindz/Coding/Libraries/forge-pilot/Cargo.toml
```

Affected AiDENs dependency edges:

```text
aidens-contracts
  attestation-exchange: path=/home/sikmindz/Coding/Libraries/attestation-exchange, source=None
  federated-settlement: path=/home/sikmindz/Coding/Libraries/federated-settlement, source=None
  mechanism-runtime: path=/home/sikmindz/Coding/Libraries/mechanism-runtime, source=None
  remote-oracle-admission: path=/home/sikmindz/Coding/Libraries/remote-oracle-admission, source=None
  stack-ids: path=/home/sikmindz/Coding/Libraries/stack-ids, source=None
aidens-cli
  contract-schema-gen: path=/home/sikmindz/Coding/Libraries/contract-schema-gen, source=None
aidens-governance-kit
  stack-ids: path=/home/sikmindz/Coding/Libraries/stack-ids, source=None
  verification-calibration: path=/home/sikmindz/Coding/Libraries/verification-calibration, source=None
```

`cargo check --workspace` result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
```

## 5. Dependency Source-Of-Truth Matrix

Updated matrix:

```text
docs/contract-ownership/DEPENDENCY_SOURCE_OF_TRUTH.md
```

The matrix records the 2026-04-28 source basis, the canonical owner crate for each surfaced concept, the AiDENs dependency edge, and the Phase 03 decision.

## Ambiguity/Quarantine Decision

No dependency ownership ambiguity was found in this guardrail. No new quarantine item is required. Phase 04 may start only after this guardrail is accepted.
