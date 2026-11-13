use crate::tools::*;
use crate::trusted_head::{load_projected_entries, TrustedHeadError, TrustedHeadProjection};
use claim_ledger::{LedgerEntry, LedgerEvent, SupportState};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router, ErrorData, Json, ServerHandler,
};
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct Output {
    pub data: Value,
}
fn out(v: Value) -> Json<Output> {
    Json(Output { data: v })
}
fn err(e: impl ToString) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}
fn load(path: &PathBuf) -> Result<Vec<LedgerEntry>, ErrorData> {
    let text = match std::fs::read_to_string(path) {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(err(e)),
    };
    claim_ledger::parse_ledger_entries(&text).map_err(err)
}
fn internal_chain_check(
    entries: &[LedgerEntry],
) -> Result<claim_ledger::LedgerVerification, claim_ledger::ClaimLedgerError> {
    // This verifies consistency only; the expected head is derived from the same file.
    let head = match entries.last() {
        Some(last) => {
            claim_ledger::ExpectedLedgerHead::new(last.sequence, last.entry_digest.clone())
        }
        None => claim_ledger::ExpectedLedgerHead::Empty,
    };
    claim_ledger::verify_ledger(entries, &head)
}
fn claim_rows(entries: &[LedgerEntry]) -> Vec<Value> {
    let mut rows = Vec::new();
    for e in entries {
        if let LedgerEvent::ClaimAdded {
            claim_id,
            source_id,
            span_id,
            normalized_claim,
        } = &e.event
        {
            let state = entries
                .iter()
                .rev()
                .find_map(|x| match &x.event {
                    LedgerEvent::SupportJudgment {
                        claim_id: id,
                        support_state,
                        ..
                    } if id == claim_id => Some(*support_state),
                    // Legacy admissions are retained as events, not projected as support.
                    // The canonical V1 snapshot folds SupportJudgment, not SupportAdmission.
                    _ => None,
                })
                .unwrap_or(SupportState::Unknown);
            rows.push(json!({"claim_id":claim_id,"source_id":source_id,"span_id":span_id,"claim":normalized_claim,"support_state":state}));
        }
    }
    rows
}

pub struct ClaimLedgerServer {
    path: PathBuf,
    /// Optional operator trust-root JSON text (anchored mode).
    trust_root: Option<String>,
    /// Optional independent trusted-head JSON text (anchored mode).
    expected_head: Option<String>,
    tool_router: ToolRouter<Self>,
}
impl ClaimLedgerServer {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            path: dir.join("claim_ledger.jsonl"),
            trust_root: None,
            expected_head: None,
            tool_router: Self::tool_router(),
        }
    }

    /// Anchored-capable constructor. `trust_root` + `expected_head` texts are
    /// pre-validated by the CLI; projection happens per tool call against the
    /// current ledger file.
    pub fn with_trust(
        dir: PathBuf,
        trust_root: Option<String>,
        expected_head: Option<String>,
    ) -> Self {
        Self {
            path: dir.join("claim_ledger.jsonl"),
            trust_root,
            expected_head,
            tool_router: Self::tool_router(),
        }
    }

    /// Per-call projection: anchored when both texts are provisioned
    /// (fail-closed on any defect), otherwise the #43/#45 unanchored
    /// contract (empty snapshot, support Unknown).
    fn project(&self, entries: &[LedgerEntry]) -> Result<TrustedHeadProjection, ErrorData> {
        let text = entries
            .iter()
            .map(|e| claim_ledger::serialize_entry(e).map_err(err))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        match load_projected_entries(
            &text,
            self.trust_root.as_deref(),
            self.expected_head.as_deref(),
        ) {
            Ok(p) => Ok(p),
            Err(TrustedHeadError::MissingExpectedHead(reason)) => {
                // Unreachable via CLI pairing; typed defensive branch.
                Err(err(format!("trusted head unavailable: {reason}")))
            }
            Err(e) => Err(err(e.to_string())),
        }
    }
}
#[tool_router]
impl ClaimLedgerServer {
    #[tool(description = "Return claim ledger status and counts")]
    async fn claim_ledger_status(&self) -> Result<Json<Output>, ErrorData> {
        let e = load(&self.path)?;
        match self.project(&e) {
            Ok(mut p) if p.anchored => {
                let Some(anchor) = p.anchor.take() else {
                    return Err(err("anchored projection missing anchor"));
                };
                Ok(out(json!({
                    "ledger_path": self.path,
                    "entry_count": e.len(),
                    "snapshot_state": "anchored",
                    "ok": true,
                    "verification_status": "anchored",
                    "anchor_admission_id": anchor.admission_id,
                    "anchor_signer_id": anchor.signer_id,
                    "anchor_envelope_digest": anchor.envelope_digest,
                    "anchor_envelope_verification": anchor.envelope_verification_name,
                    "digest_chain_valid": true
                })))
            }
            Ok(_) => Ok(out(
                json!({"ledger_path":self.path,"entry_count":e.len(),"snapshot_state":"none","ok":false,"verification_status":"unanchored","reason":"independent_expected_head_unavailable","digest_chain_valid":internal_chain_check(&e).is_ok()}),
            )),
            Err(e) => Err(e),
        }
    }
    #[tool(description = "Verify hash chain and snapshot integrity")]
    async fn claim_ledger_verify(&self) -> Result<Json<Output>, ErrorData> {
        let e = load(&self.path)?;
        match self.project(&e) {
            Ok(p) if p.anchored => Ok(out(
                json!({"ok":true,"entry_count":e.len(),"last_sequence":p.last_sequence,"digest_chain_valid":true,"snapshot_valid":true,"verification_status":"anchored","anchor_admission_id":p.anchor.as_ref().map(|a|a.admission_id.clone())}),
            )),
            Ok(_) => match internal_chain_check(&e) {
                Ok(v) => Ok(out(
                    json!({"ok":false,"entry_count":e.len(),"last_sequence":v.last_sequence,"digest_chain_valid":true,"snapshot_valid":false,"verification_status":"unanchored","reason":"independent_expected_head_unavailable"}),
                )),
                Err(x) => Ok(out(
                    json!({"ok":false,"entry_count":e.len(),"digest_chain_valid":false,"verification_status":"unanchored","error":x.to_string()}),
                )),
            },
            Err(e2) => Err(e2),
        }
    }
    #[tool(description = "Query claims by text and support state")]
    async fn claim_ledger_query(
        &self,
        Parameters(p): Parameters<QueryParams>,
    ) -> Result<Json<Output>, ErrorData> {
        let entries = load(&self.path)?;
        let mut r = claim_rows(&entries);
        let (verification_status, anchored) = match self.project(&entries)? {
            proj if proj.anchored => {
                // Only anchored (verified + fold-projected) support states are
                // surfaced. Claims absent from the projected support stay
                // Unknown.
                for row in &mut r {
                    let claim_id = row["claim_id"].as_str().unwrap_or("").to_owned();
                    let state = proj
                        .claim_support
                        .get(&claim_id)
                        .copied()
                        .unwrap_or(SupportState::Unknown);
                    row["support_state"] = json!(state);
                }
                ("anchored", true)
            }
            _ => {
                // Unanchored mode: a self-consistent JSONL chain cannot
                // authorize support or state filters (stay Unknown).
                for row in &mut r {
                    row["support_state"] = json!(SupportState::Unknown);
                }
                ("unanchored", false)
            }
        };
        if let Some(t) = p.text {
            let t = t.to_lowercase();
            r.retain(|x| {
                x["claim"]
                    .as_str()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&t)
            });
        }
        if let Some(s) = p.state {
            r.retain(|x| x["support_state"].as_str().unwrap_or("") == s);
        }
        if let Some(ns) = p.namespace {
            r.retain(|x| x["source_id"].as_str().unwrap_or("").contains(&ns));
        }
        r.truncate(p.limit.unwrap_or(50).min(200));
        Ok(out(
            json!({"claims":r,"verification_status":verification_status,"anchored":anchored}),
        ))
    }
    #[tool(description = "Get a claim and its related ledger events")]
    async fn claim_ledger_get(
        &self,
        Parameters(p): Parameters<GetParams>,
    ) -> Result<Json<Output>, ErrorData> {
        let e = load(&self.path)?;
        let verification_status = match self.project(&e) {
            Ok(p) if p.anchored => "anchored",
            Ok(_) => "unanchored",
            Err(e2) => return Err(e2),
        };
        let events: Vec<&LedgerEntry> = e
            .iter()
            .filter(|x| {
                serde_json::to_value(&x.event)
                    .map(|v| v.to_string().contains(&p.claim_id))
                    .unwrap_or(false)
            })
            .collect();
        if events.is_empty() {
            return Ok(out(
                json!({"found":false,"claim_id":p.claim_id,"verification_status":verification_status,"raw_untrusted":true}),
            ));
        }
        Ok(out(
            json!({"found":true,"claim_id":p.claim_id,"events":events,"verification_status":verification_status,"raw_untrusted":true}),
        ))
    }
    #[tool(description = "Evaluate proof debt gate for claim IDs")]
    async fn claim_ledger_evaluate_proof_debt(
        &self,
        Parameters(p): Parameters<ProofDebtParams>,
    ) -> Result<Json<Output>, ErrorData> {
        let entries = load(&self.path)?;
        let rows = claim_rows(&entries);
        let ids = if p.claim_ids.is_empty() {
            rows.iter()
                .filter_map(|r| r["claim_id"].as_str().map(str::to_owned))
                .collect()
        } else {
            p.claim_ids
        };
        // Anchored mode attaches the verified anchor but the gate itself
        // still has no budget instrument in MCP scope: decision stays
        // conservative (block).
        let (verification_status, anchor) = match self.project(&entries) {
            Ok(mut p) if p.anchored => match p.anchor.take() {
                Some(a) => ("anchored".to_string(), Some(a)),
                None => return Err(err("anchored projection missing anchor")),
            },
            Ok(_) => ("unanchored".to_string(), None),
            Err(e) => return Err(e),
        };
        Ok(out(
            json!({"claim_ids":ids,"budget_micros":p.budget_micros,"debt_weight_micros":null,"gate_decision":"block","verification_status":verification_status,"anchor_admission_id":anchor.as_ref().map(|a|a.admission_id.clone()),"reason":"budget_instrument_not_provisioned_in_mcp_scope"}),
        ))
    }
    #[tool(description = "Generate a binding export receipt")]
    async fn claim_ledger_export_receipt(
        &self,
        Parameters(p): Parameters<ExportParams>,
    ) -> Result<Json<Output>, ErrorData> {
        let mut r =
            claim_ledger::ExportReceipt::new(&p.operation, p.claim_ids.clone(), p.attempt_id);
        let output_bytes = serde_json::to_vec(&p.claim_ids).map_err(err)?;
        r.bind_output(
            "claim_ids".into(),
            claim_ledger::sha256_bytes(&output_bytes),
        );
        r.mark_success();
        let mut value = serde_json::to_value(r).map_err(err)?;
        let verification_status = match self.project(&load(&self.path)?) {
            Ok(p) if p.anchored => "anchored",
            Ok(_) => "unanchored",
            Err(e2) => return Err(e2),
        };
        value["verification_status"] = json!(verification_status);
        value["receipt_scope"] = json!("provided_claim_ids_only");
        Ok(out(value))
    }
}
#[tool_handler(router=self.tool_router, name="claim-ledger-mcp", version="0.1.0")]
impl ServerHandler for ClaimLedgerServer {}

#[cfg(test)]
#[allow(clippy::expect_used)] // Fixture construction failures should fail the tests, never production reads.
mod tests {
    use super::*;
    use claim_ledger::{LedgerEntryBuilder, SupportState};

    fn claim() -> LedgerEntry {
        LedgerEntryBuilder::new(1, None)
            .add_claim("claim-1", "source-1", "span-1", "claim text")
            .expect("valid test claim")
    }

    #[test]
    fn legacy_admission_alone_does_not_promote_query_support() {
        let first = claim();
        let admission = LedgerEntryBuilder::new(2, Some(first.entry_digest.clone()))
            .add_support_admission(
                "receipt-1",
                "claim-1",
                "old",
                "new",
                SupportState::Supported,
            )
            .expect("valid test admission");
        let rows = claim_rows(&[first, admission]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["support_state"], "unknown");
    }

    #[test]
    fn legacy_admission_does_not_override_latest_judgment() {
        let first = claim();
        let judgment = LedgerEntryBuilder::new(2, Some(first.entry_digest.clone()))
            .add_support_judgment(
                "judgment-1",
                "claim-1",
                "bundle-1",
                SupportState::Unsupported,
                "fixture",
            )
            .expect("valid test judgment");
        let admission = LedgerEntryBuilder::new(3, Some(judgment.entry_digest.clone()))
            .add_support_admission(
                "receipt-1",
                "claim-1",
                "judgment-1",
                "judgment-2",
                SupportState::Supported,
            )
            .expect("valid test admission");
        let rows = claim_rows(&[first, judgment, admission]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["support_state"], "unsupported");
    }

    #[tokio::test]
    async fn admission_only_cannot_clear_proof_debt_gate() {
        let dir = std::env::temp_dir().join(format!("claim-ledger-mcp-{}", claim_ledger::ulid()));
        std::fs::create_dir(&dir).expect("test directory");
        let first = claim();
        let admission = LedgerEntryBuilder::new(2, Some(first.entry_digest.clone()))
            .add_support_admission(
                "receipt-1",
                "claim-1",
                "old",
                "new",
                SupportState::Supported,
            )
            .expect("valid test admission");
        let ledger = format!(
            "{}\n{}\n",
            claim_ledger::serialize_entry(&first).expect("serialize claim"),
            claim_ledger::serialize_entry(&admission).expect("serialize admission")
        );
        std::fs::write(dir.join("claim_ledger.jsonl"), ledger).expect("write test ledger");
        let server = ClaimLedgerServer::new(dir.clone());
        let response = server
            .claim_ledger_evaluate_proof_debt(Parameters(ProofDebtParams {
                claim_ids: vec!["claim-1".into()],
                budget_micros: 0,
            }))
            .await
            .expect("evaluate gate");
        assert_eq!(response.0.data["gate_decision"], "block");
        assert!(response.0.data["debt_weight_micros"].is_null());
        assert_eq!(response.0.data["verification_status"], "unanchored");
        std::fs::remove_dir_all(dir).expect("remove test directory");
    }

    #[tokio::test]
    async fn unanchored_judgment_cannot_be_queried_as_supported_or_allow_gate() {
        let dir = std::env::temp_dir().join(format!("claim-ledger-mcp-{}", claim_ledger::ulid()));
        std::fs::create_dir(&dir).expect("test directory");
        let first = claim();
        let original = LedgerEntryBuilder::new(2, Some(first.entry_digest.clone()))
            .add_support_judgment(
                "judgment-1",
                "claim-1",
                "bundle-1",
                SupportState::Unsupported,
                "fixture",
            )
            .expect("original judgment");
        let expected_head = claim_ledger::ExpectedLedgerHead::new(2, original.entry_digest);
        let judgment = LedgerEntryBuilder::new(2, Some(first.entry_digest.clone()))
            .add_support_judgment(
                "judgment-1",
                "claim-1",
                "bundle-1",
                SupportState::Supported,
                "fixture",
            )
            .expect("forged self-consistent judgment");
        assert!(
            claim_ledger::verify_ledger(&[first.clone(), judgment.clone()], &expected_head)
                .is_err()
        );
        let ledger = format!(
            "{}\n{}\n",
            claim_ledger::serialize_entry(&first).expect("serialize claim"),
            claim_ledger::serialize_entry(&judgment).expect("serialize judgment")
        );
        std::fs::write(dir.join("claim_ledger.jsonl"), ledger).expect("write test ledger");
        let server = ClaimLedgerServer::new(dir.clone());
        let all = server
            .claim_ledger_query(Parameters(QueryParams {
                text: None,
                state: None,
                namespace: None,
                limit: None,
            }))
            .await
            .expect("query rows");
        assert_eq!(all.0.data["claims"][0]["support_state"], "unknown");
        assert_eq!(all.0.data["verification_status"], "unanchored");
        let strong = server
            .claim_ledger_query(Parameters(QueryParams {
                text: None,
                state: Some("supported".into()),
                namespace: None,
                limit: None,
            }))
            .await
            .expect("filter rows");
        assert_eq!(strong.0.data["claims"], json!([]));
        let gate = server
            .claim_ledger_evaluate_proof_debt(Parameters(ProofDebtParams {
                claim_ids: vec!["claim-1".into()],
                budget_micros: u64::MAX,
            }))
            .await
            .expect("gate");
        assert_eq!(gate.0.data["gate_decision"], "block");
        assert_eq!(gate.0.data["verification_status"], "unanchored");
        let status = server.claim_ledger_status().await.expect("status");
        assert_eq!(status.0.data["verification_status"], "unanchored");
        assert_eq!(status.0.data["ok"], false);
        assert_eq!(
            status.0.data["reason"],
            "independent_expected_head_unavailable"
        );
        let verification = server.claim_ledger_verify().await.expect("verify");
        assert_eq!(verification.0.data["ok"], false);
        assert_eq!(verification.0.data["digest_chain_valid"], true);
        assert_eq!(verification.0.data["verification_status"], "unanchored");
        let found = server
            .claim_ledger_get(Parameters(GetParams {
                claim_id: "claim-1".into(),
            }))
            .await
            .expect("get raw events");
        assert_eq!(found.0.data["found"], true);
        assert_eq!(found.0.data["verification_status"], "unanchored");
        assert_eq!(found.0.data["raw_untrusted"], true);
        let absent = server
            .claim_ledger_get(Parameters(GetParams {
                claim_id: "absent".into(),
            }))
            .await
            .expect("get absent");
        assert_eq!(absent.0.data["verification_status"], "unanchored");
        let receipt = server
            .claim_ledger_export_receipt(Parameters(ExportParams {
                claim_ids: vec!["claim-1".into()],
                operation: "test-export".into(),
                attempt_id: "attempt-1".into(),
            }))
            .await
            .expect("export receipt");
        assert_eq!(receipt.0.data["verification_status"], "unanchored");
        assert_eq!(receipt.0.data["receipt_scope"], "provided_claim_ids_only");
        std::fs::remove_dir_all(dir).expect("remove test directory");
    }
}
