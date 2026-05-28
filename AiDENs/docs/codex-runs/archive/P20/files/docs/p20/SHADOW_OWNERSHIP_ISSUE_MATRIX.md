# Shadow Ownership Issue Matrix

P20 Phase 03 status: this matrix records the ownership risks that drove the
phase. The current resolved inventory is
`docs/p20/CONTRACT_OWNERSHIP_INVENTORY.md`; renamed/deleted type names there
supersede the historical evidence names below.

| ID | Severity | Finding | Evidence | Canonical owner | Required fix | Acceptance proof |
|---|---|---|---|---|---|---|
| P0-001 | P0 | Local duplicate `AttestationEnvelopeV1` | `aidens-contracts/src/lib.rs` line ~2481 | `attestation-exchange` | Delete local definition; add dependency; explicit `pub use` if needed | duplicate gate passes; local def absent |
| P0-002 | P0 | Local duplicate `SharedDispositionV1` | `aidens-contracts/src/lib.rs` line ~2843 | `federated-settlement` | Delete local definition; add dependency; explicit `pub use` if needed | duplicate gate passes; local def absent |
| P0-003 | P0 | Local duplicate `SettlementCaseV1` | `aidens-contracts/src/lib.rs` line ~2891 | `federated-settlement` | Delete local definition; add dependency; explicit `pub use` if needed | duplicate gate passes; local def absent |
| P0-004 | P0 | Local duplicate `TheoryRefuterSuiteV1` | `aidens-contracts/src/lib.rs` line ~3508 | `mechanism-runtime` | Delete local definition; add dependency; explicit `pub use` if needed | duplicate gate passes; local def absent |
| P0-005 | P0 | Local duplicate `TheoryVersionV1` | `aidens-contracts/src/lib.rs` line ~3603 | `mechanism-runtime` | Delete local definition; add dependency; explicit `pub use` if needed | duplicate gate passes; local def absent |
| P0-006 | P0 | Local duplicate `HypothesisLibraryV1` | `aidens-contracts/src/lib.rs` line ~3815 | `mechanism-runtime` | Delete local definition; add dependency; explicit `pub use` if needed | duplicate gate passes; local def absent |
| P0-007 | P0 | Local digest/canonicalization law | `stable_json_digest`, `stable_text_digest`, `deterministic_artifact_id`, `CanonicalDigestV1` | `stack-ids` | Replace with `stack-ids` or demote to display-only with non-authoritative name | digest gate passes |
| P0-008 | P0 | AiDENs schema registry emits canonical family schemas | `ArtifactFamilyRegistryV1`, `GeneratedSchemaManifestV1`, schema docs | `contract-schema-gen` + owner crates | Remove canonical family generation from AiDENs; keep AiDENs-only display schemas | schema scope gate passes |
| P1-001 | P1 | Tool contracts too close to canonical tool runtime | `ToolDescriptorV1`, `ToolCallRequestV1`, `ToolCallResultV1`, reports | `llm-tool-runtime` | Use canonical runtime types; local wrappers only with backpointers | tool delegation gate passes |
| P1-002 | P1 | Repair/schema validation reports may become local verification truth | `BoundaryRepairReportV1`, `JsonRepairReportV2`, `SchemaValidationReportV1` | `verification-control` | Reference canonical repair records; display-only reports | wrapper/backpointer gate passes |
| P1-003 | P1 | Runtime view/widening/degradation local semantics | `RuntimeViewRequestV1`, `QueryWideningReportV1`, `DegradationEventV1` | `knowledge-runtime`, Forge | Canonical backpointers; display-only wrappers | wrapper/backpointer gate passes |
| P1-004 | P1 | Future kernel/region/subtraction DTOs risk becoming runtime law | `RegionContractV1`, `SyndromeV1`, `SubtractionPlanV1`, etc. | kernel/region/subtraction crates | Display-only wrappers or quarantine | quarantine ledger has owner decision |
| P2-001 | P2 | Stale docs/source basis | old archive names or obsolete counts | repo docs | Update to 2026-04-28 basis | docs gate passes |
