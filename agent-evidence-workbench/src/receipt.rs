use crate::{
    error::{Error, Result},
    model::RunReport,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReceiptV1 {
    pub receipt_version: String,
    pub receipt_id: String,
    pub run_id: String,
    pub report_digest: String,
    pub previous_receipt_digest: Option<String>,
    pub integrity_tag: String,
    pub recorded_at: DateTime<Utc>,
}

fn canonical(r: &RunReceiptV1) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"receipt_version":r.receipt_version,"receipt_id":r.receipt_id,"run_id":r.run_id,"report_digest":r.report_digest,"previous_receipt_digest":r.previous_receipt_digest,"recorded_at":r.recorded_at})).expect("receipt canonicalization")
}
fn hmac(key: &[u8], data: &[u8]) -> String {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(data);
    let ih = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(ih);
    hex::encode(outer.finalize())
}
pub fn report_digest(r: &RunReport) -> String {
    hex::encode(Sha256::digest(serde_json::to_vec(r).expect("report json")))
}
pub fn sign(
    run_id: &str,
    report: &RunReport,
    key: &[u8],
    previous: Option<String>,
) -> RunReceiptV1 {
    let mut r = RunReceiptV1 {
        receipt_version: "WORKBENCH-RECEIPT-V1".into(),
        receipt_id: String::new(),
        run_id: run_id.into(),
        report_digest: report_digest(report),
        previous_receipt_digest: previous,
        integrity_tag: String::new(),
        recorded_at: Utc::now(),
    };
    r.receipt_id = hex::encode(Sha256::digest(
        format!("{}:{}", run_id, r.recorded_at.to_rfc3339()).as_bytes(),
    ));
    r.integrity_tag = hmac(key, &canonical(&r));
    r
}
pub fn verify(r: &RunReceiptV1, key: &[u8], report: &RunReport) -> bool {
    r.run_id == report.run_id
        && r.report_digest == report_digest(report)
        && r.integrity_tag == hmac(key, &canonical(r))
}

/// SHA-256 digest of the complete signed receipt JSON.
///
/// This is the immutable receipt reference for downstream metadata; it is
/// distinct from the report digest bound inside the receipt.
pub fn receipt_digest(r: &RunReceiptV1) -> String {
    hex::encode(Sha256::digest(
        serde_json::to_vec(r).expect("receipt serialization"),
    ))
}

pub fn parse_key(s: &str) -> Result<Vec<u8>> {
    let k = hex::decode(s).map_err(|_| Error::Invalid("key must be hex".into()))?;
    if k.len() != 32 {
        return Err(Error::Invalid(
            "key must be exactly 64 hex characters".into(),
        ));
    }
    Ok(k)
}
pub fn parse_key_file(path: &Path) -> Result<Vec<u8>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)?.permissions().mode();
        if mode & 0o077 != 0 {
            return Err(Error::Invalid(
                "key file must not be group- or world-readable".into(),
            ));
        }
    }
    let key = std::fs::read_to_string(path)?;
    parse_key(key.trim())
}

pub fn receipt_path(cwd: &Path, id: &str) -> std::path::PathBuf {
    cwd.join(".aew/receipts").join(format!("{id}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn report() -> RunReport {
        RunReport {
            run_id: "r".into(),
            verdict: crate::model::RunVerdict::Clean,
            claims: vec![],
            checks: vec![],
            diff: String::new(),
            evidence_manifest: vec![],
        }
    }
    #[test]
    fn sign_verify_and_tamper() {
        let key = [7u8; 32];
        let r = report();
        let signed = sign("r", &r, &key, None);
        assert!(verify(&signed, &key, &r));
        let mut tampered = signed.clone();
        tampered.run_id = "other".into();
        assert!(!verify(&tampered, &key, &r));
    }
}
