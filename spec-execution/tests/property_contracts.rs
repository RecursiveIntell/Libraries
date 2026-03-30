//! Property-based tests for spec-execution artifact types.
//!
//! Validates serde roundtrip fidelity and schema_version constant usage
//! for top-level artifact families. ContentDigest is generated via
//! `ContentDigest::compute()` to guarantee valid 64-char hex values.
//! GeneratedCompanionBundles is skipped (no Serialize/Deserialize).

use proptest::prelude::*;
use spec_execution::*;
use stack_ids::{
    ContentDigest, GeneratedSchemaBundleId, HumanVetoBundleId, MetaChallengeBundleId,
    NormativeAstId, ProofObligationSetId, SpecBundleId, SurfaceStatus,
};

// --- Generators ---

fn arb_string() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{0,30}".prop_map(|s| s)
}

fn arb_id() -> impl Strategy<Value = String> {
    "[a-z]{3}-[a-z0-9]{8}".prop_map(|s| s)
}

fn arb_vec_string() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_string(), 0..3)
}

fn arb_surface_status() -> impl Strategy<Value = SurfaceStatus> {
    prop_oneof![
        Just(SurfaceStatus::AdvisoryOnly),
        Just(SurfaceStatus::NonAdmitted),
        Just(SurfaceStatus::Degraded),
        Just(SurfaceStatus::HorizonOnly),
    ]
}

fn arb_spec_bundle_id() -> impl Strategy<Value = SpecBundleId> {
    arb_id().prop_map(SpecBundleId::new)
}

fn arb_normative_ast_id() -> impl Strategy<Value = NormativeAstId> {
    arb_id().prop_map(NormativeAstId::new)
}

fn arb_proof_obligation_set_id() -> impl Strategy<Value = ProofObligationSetId> {
    arb_id().prop_map(ProofObligationSetId::new)
}

fn arb_generated_schema_bundle_id() -> impl Strategy<Value = GeneratedSchemaBundleId> {
    arb_id().prop_map(GeneratedSchemaBundleId::new)
}

fn arb_human_veto_bundle_id() -> impl Strategy<Value = HumanVetoBundleId> {
    arb_id().prop_map(HumanVetoBundleId::new)
}

fn arb_meta_challenge_bundle_id() -> impl Strategy<Value = MetaChallengeBundleId> {
    arb_id().prop_map(MetaChallengeBundleId::new)
}

fn arb_content_digest() -> impl Strategy<Value = ContentDigest> {
    Just(ContentDigest::compute(b"property-test-fixed-input"))
}

fn arb_generated_admission_state() -> impl Strategy<Value = GeneratedAdmissionState> {
    prop_oneof![
        Just(GeneratedAdmissionState::AdvisoryOnly),
        Just(GeneratedAdmissionState::NonAdmitted),
        Just(GeneratedAdmissionState::HumanVetoed),
    ]
}

fn arb_generated_surface_governance_state() -> impl Strategy<Value = GeneratedSurfaceGovernanceState>
{
    prop_oneof![
        Just(GeneratedSurfaceGovernanceState::AdvisoryBaseline),
        Just(GeneratedSurfaceGovernanceState::ChallengePending),
        Just(GeneratedSurfaceGovernanceState::HumanVetoed),
    ]
}

fn arb_normative_ast_node_v1() -> impl Strategy<Value = NormativeAstNodeV1> {
    (arb_string(), arb_string(), arb_string()).prop_map(|(node_id, kind, normative_text)| {
        NormativeAstNodeV1 {
            node_id,
            kind,
            normative_text,
        }
    })
}

fn arb_generated_schema_file_v1() -> impl Strategy<Value = GeneratedSchemaFileV1> {
    (arb_string(), arb_content_digest(), arb_vec_string()).prop_map(
        |(path, digest, source_node_ids)| GeneratedSchemaFileV1 {
            path,
            digest,
            source_node_ids,
        },
    )
}

// --- Top-level artifact generators ---

fn arb_spec_bundle_v1() -> impl Strategy<Value = SpecBundleV1> {
    (
        arb_spec_bundle_id(),
        arb_string(),
        arb_string(),
        arb_surface_status(),
    )
        .prop_map(
            |(spec_bundle_id, spec_name, canonical_owner, publication_status)| SpecBundleV1 {
                schema_version: SPEC_BUNDLE_V1_SCHEMA.into(),
                spec_bundle_id,
                spec_name,
                canonical_owner,
                publication_status,
            },
        )
}

fn arb_normative_ast_v1() -> impl Strategy<Value = NormativeASTV1> {
    (
        arb_normative_ast_id(),
        arb_spec_bundle_id(),
        prop::collection::vec(arb_normative_ast_node_v1(), 0..3),
        arb_surface_status(),
    )
        .prop_map(
            |(normative_ast_id, spec_bundle_id, nodes, publication_status)| NormativeASTV1 {
                schema_version: NORMATIVE_AST_V1_SCHEMA.into(),
                normative_ast_id,
                spec_bundle_id,
                nodes,
                publication_status,
            },
        )
}

fn arb_human_veto_bundle_v1() -> impl Strategy<Value = HumanVetoBundleV1> {
    (
        arb_human_veto_bundle_id(),
        arb_generated_schema_bundle_id(),
        arb_string(),
    )
        .prop_map(
            |(human_veto_bundle_id, generated_schema_bundle_id, reason)| HumanVetoBundleV1 {
                schema_version: HUMAN_VETO_BUNDLE_V1_SCHEMA.into(),
                human_veto_bundle_id,
                generated_schema_bundle_id,
                reason,
            },
        )
}

// --- Property tests ---

proptest! {
    #[test]
    fn spec_bundle_v1_serde_roundtrip(val in arb_spec_bundle_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: SpecBundleV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn spec_bundle_v1_schema_version_constant(val in arb_spec_bundle_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), SPEC_BUNDLE_V1_SCHEMA);
    }

    #[test]
    fn normative_ast_v1_serde_roundtrip(val in arb_normative_ast_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: NormativeASTV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn normative_ast_v1_schema_version_constant(val in arb_normative_ast_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), NORMATIVE_AST_V1_SCHEMA);
    }

    #[test]
    fn human_veto_bundle_v1_serde_roundtrip(val in arb_human_veto_bundle_v1()) {
        let json = serde_json::to_string(&val).unwrap();
        let back: HumanVetoBundleV1 = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&val, &back);
    }

    #[test]
    fn human_veto_bundle_v1_schema_version_constant(val in arb_human_veto_bundle_v1()) {
        prop_assert_eq!(val.schema_version.as_str(), HUMAN_VETO_BUNDLE_V1_SCHEMA);
    }
}
