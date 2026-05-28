# Quarantine: delegation-kit-attestation-settlement

STATUS: needs human owner decision
DISCOVERED_IN_PHASE: 02
LOCAL FILE/LINE: `crates/aidens-delegation-kit/src/lib.rs`
LOCAL SYMBOLS: `AdmissionPolicyV1`, `TrustRootRegistryV1`, `import_remote_oracle`, `blocked_remote_oracle_receipt`, `open_remote_contradiction_settlement`, `revoke_trust_root_and_downgrade`
SUSPECTED CANONICAL OWNER(S): `attestation-exchange`, `federated-settlement`, `remote-oracle-admission`
SEARCHES PERFORMED:
- `rg -n "\\b(AttestationEnvelopeV1|SharedDispositionV1|SettlementCaseV1|TheoryRefuterSuiteV1|TheoryVersionV1|HypothesisLibraryV1)\\b" crates tests docs scripts examples schemas`
- `cargo check -p aidens-delegation-kit`
- direct comparison of local `AttestationEnvelopeV1`, `SettlementCaseV1`, and `SharedDispositionV1` fields against canonical owner crate fields
WHY AUTOMATIC COLLAPSE IS UNSAFE:
The prior delegation helpers depended on AiDENs-local fields and methods removed in Phase 02, including `envelope_id`, `subject_artifact_id`, `subject_digest`, `producer_id`, `trust_root_id`, `verification_status`, `truth_bearing`, `body`, `is_signed_and_verified`, `AdmissionDecisionV1::from_envelope`, `RemoteOracleReportV1::imported`, and `SettlementCaseV1::disputed`.

Canonical `attestation-exchange::AttestationEnvelopeV1` uses `attestation_envelope_id`, `artifact_family`, `content_digest`, `signer_identity`, `trust_root_set_id`, disclosure/admission policy IDs, replayability class, and revocation/supersession refs. Canonical `federated-settlement` settlement artifacts use settlement and shared-disposition IDs plus v25 constitutional citation/shared-view fields. Mapping the old helper behavior onto these canonical fields would require lossy reinterpretation of subject identity, signature verification status, trust-root semantics, remote-oracle import authority, and settlement state.
TEMPORARY ACTION TAKEN:
The shadow helper implementation was removed from `aidens-delegation-kit`; the crate now exposes only a disabled status and a `CanonicalOwnerRequired` error. This is a quarantine stop surface, not a compatibility adapter.
FORBIDDEN ACTIONS:
- Do not re-create the removed helper methods with synthetic field mappings.
- Do not wrap canonical artifacts into untyped JSON blobs.
- Do not preserve the old local admission/settlement semantics under renamed types.
- Do not use `Libraries2` owner crates when `/home/sikmindz/Coding/Libraries` owners exist.
REQUIRED HUMAN DECISION:
Decide whether delegation/admission helper behavior should be rebuilt against `attestation-exchange`, `federated-settlement`, and `remote-oracle-admission`, or removed from the product surface until a later owner-approved run.
RECOMMENDED NEXT RUN:
Design a canonical delegation facade with explicit backpointers to canonical attestation envelope IDs, content digests, admission policy IDs, settlement case IDs, and remote-oracle admission records. Keep all admission/settlement truth in canonical owner crates.
