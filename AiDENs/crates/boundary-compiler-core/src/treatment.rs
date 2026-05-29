use crate::canonical::canonicalize_stable_sorted_json;
use crate::digest::sha256_digest_hex;
use crate::strict_json::StrictJsonValue;
use crate::types::{
    DigestHex, JsonPointerLikePath, TreatmentDifferenceV1, TreatmentIntegrityDecision,
    TreatmentIntegrityReceiptV1,
};
use std::collections::BTreeMap;

pub fn treatment_receipt_for_paths(
    raw_digest: &DigestHex,
    value: &StrictJsonValue,
    paths: &[JsonPointerLikePath],
) -> Option<TreatmentIntegrityReceiptV1> {
    if paths.is_empty() {
        return None;
    }

    let mut after_hashes = BTreeMap::new();
    let mut differences = Vec::new();
    let before_hashes = BTreeMap::new();
    let mut missing = false;

    for path in paths {
        let after = value
            .get_pointer(path)
            .and_then(|v| canonicalize_stable_sorted_json(v).ok())
            .map(|bytes| sha256_digest_hex(&bytes));
        if after.is_none() {
            missing = true;
            differences.push(TreatmentDifferenceV1 {
                path: path.clone(),
                before_digest: None,
                after_digest: None,
                description: "treatment-critical path is missing".to_string(),
            });
        }
        after_hashes.insert(path.clone(), after);
    }

    let decision = if missing {
        TreatmentIntegrityDecision::MissingCriticalPath
    } else {
        TreatmentIntegrityDecision::Preserved
    };

    Some(TreatmentIntegrityReceiptV1 {
        receipt_id: format!(
            "treatment-integrity:{}",
            raw_digest.trim_start_matches("sha256:")
        ),
        treatment_critical_paths: paths.to_vec(),
        before_hashes,
        after_hashes,
        differences,
        decision,
        waiver: None,
    })
}

pub fn treatment_receipt_for_unparsed_boundary(
    raw_digest: &DigestHex,
    paths: &[JsonPointerLikePath],
    description: &str,
) -> Option<TreatmentIntegrityReceiptV1> {
    if paths.is_empty() {
        return None;
    }

    let before_hashes = BTreeMap::new();
    let mut after_hashes = BTreeMap::new();
    let mut differences = Vec::new();

    for path in paths {
        after_hashes.insert(path.clone(), None);
        differences.push(TreatmentDifferenceV1 {
            path: path.clone(),
            before_digest: None,
            after_digest: None,
            description: description.to_string(),
        });
    }

    Some(TreatmentIntegrityReceiptV1 {
        receipt_id: format!(
            "treatment-integrity:{}",
            raw_digest.trim_start_matches("sha256:")
        ),
        treatment_critical_paths: paths.to_vec(),
        before_hashes,
        after_hashes,
        differences,
        decision: TreatmentIntegrityDecision::MissingCriticalPath,
        waiver: None,
    })
}
