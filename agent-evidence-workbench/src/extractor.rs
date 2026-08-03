use crate::model::{AgentClaim, ClaimStatus};
use regex::Regex;
use sha2::{Digest, Sha256};
pub fn extract_claims(transcript: &str) -> Vec<AgentClaim> {
    let patterns = [
        ("tests_pass", r"(?i)tests?\s+pass"),
        ("build_succeeds", r"(?i)build\s+succeeds"),
        ("lint_clean", r"(?i)(lint|clippy|format)\s+(clean|passes)"),
        (
            "fixed",
            r"(?i)(fixed|resolved|patched|implemented)\s+[^.!?\n]+",
        ),
        ("added_test", r"(?i)(added|wrote|created)\s+.*test"),
        ("no_regressions", r"(?i)no\s+regressions"),
        ("production_ready", r"(?i)production.ready"),
        ("fully_fixed", r"(?i)fully\s+(fixed|complete)"),
    ];
    let mut out = Vec::new();
    for (pred, pat) in patterns {
        for m in Regex::new(pat).expect("pattern").find_iter(transcript) {
            let text = m.as_str().trim().to_string();
            let id = hex::encode(Sha256::digest(text.to_lowercase().as_bytes()));
            out.push(AgentClaim {
                id,
                text: text.clone(),
                normalized_predicate: pred.into(),
                source_quote: text,
                source_location: Some(format!("transcript:{}-{}", m.start(), m.end())),
                status: ClaimStatus::NotChecked,
            });
        }
    }
    out
}
