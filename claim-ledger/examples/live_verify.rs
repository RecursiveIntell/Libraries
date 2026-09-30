//! Live end-to-end verification against operator-provisioned trust root
//! (decision 1a/3a first proof, 2026-09-30).
//!
//! Reads the public trust-root file — NEVER a seed file.

use claim_ledger::{ArtifactEnvelopeV1, EnvelopeVerificationStatus};

const ACTIVE_ARTIFACT: &str = "/home/sikmindz/.ares/runtime/releases/d2ead80bd21fafbd895f38797413b70ab3aa58bc/source/apps/desktop/release/linux-unpacked/resources/app.asar";
const LIVE_ENVELOPE: &str = "/home/sikmindz/.ares/trust/envelopes/active-runtime-envelope.json";

fn main() {
    let envelope: ArtifactEnvelopeV1 =
        serde_json::from_str(&std::fs::read_to_string(LIVE_ENVELOPE).expect("envelope readable"))
            .expect("envelope parses");

    let artifact = std::fs::read(ACTIVE_ARTIFACT).expect("artifact readable");

    let root = claim_ledger::trust_root::load_trust_root(
        &claim_ledger::trust_root::default_trust_root_path(),
    )
    .expect("operator trust root loads");

    let status = root.verify(&artifact, &envelope);
    println!("status: {status:?}");
    match status {
        EnvelopeVerificationStatus::FullyVerified => println!("LIVE CHAIN: FULLY VERIFIED"),
        other => {
            eprintln!("FAILED at stage: {other:?}");
            std::process::exit(1);
        }
    }
}
