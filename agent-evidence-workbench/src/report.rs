use crate::model::*;

pub fn generate_markdown(r: &RunReport) -> String {
    let mut s = format!(
        "# Agent Evidence Report\n\n**Run:** `{}`\n\n**Verdict:** **{:?}**\n\n## Claims\n\n| Claim | Status |\n|---|---|\n",
        r.run_id, r.verdict
    );
    for c in &r.claims {
        s.push_str(&format!(
            "| {} | {:?} |\n",
            c.text,
            crate::adjudicator::support_state(
                &c.status,
                r.evidence_manifest.iter().any(|e| c
                    .source_location
                    .as_deref()
                    .is_some_and(|s| s.contains(&e.id) || s.contains(&e.source)))
            )
        ));
    }
    s.push_str("\n## Checks\n\n| Command | Exit | Passed |\n|---|---:|---|\n");
    for c in &r.checks {
        s.push_str(&format!(
            "| `{}` | {:?} | {} |\n",
            c.command, c.exit_code, c.passed
        ));
    }
    let manifest =
        serde_json::to_string_pretty(&r.evidence_manifest).unwrap_or_else(|_| "[]".into());
    s.push_str(&format!(
        "\n## Diff\n\n```diff\n{}\n```\n\n## Evidence manifest\n\n{}\n",
        r.diff, manifest
    ));
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_status() {
        let r = RunReport {
            run_id: "x".into(),
            verdict: RunVerdict::Partial,
            claims: vec![],
            checks: vec![],
            diff: String::new(),
            evidence_manifest: vec![],
        };
        assert!(generate_markdown(&r).contains("Partial"));
    }
}
