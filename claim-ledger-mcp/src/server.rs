use crate::tools::*;
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
    tool_router: ToolRouter<Self>,
}
impl ClaimLedgerServer {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            path: dir.join("claim_ledger.jsonl"),
            tool_router: Self::tool_router(),
        }
    }
}
#[tool_router]
impl ClaimLedgerServer {
    #[tool(description = "Return claim ledger status and counts")]
    async fn claim_ledger_status(&self) -> Result<Json<Output>, ErrorData> {
        let e = load(&self.path)?;
        Ok(out(
            json!({"ledger_path":self.path,"entry_count":e.len(),"snapshot_state":"none","verification_status":claim_ledger::verify_ledger(&e, &claim_ledger::ExpectedLedgerHead::Entry { sequence:e.last().map(|x|x.sequence).unwrap_or(0), entry_digest:e.last().map(|x|x.entry_digest.clone()).unwrap_or_default() }).is_ok()}),
        ))
    }
    #[tool(description = "Verify hash chain and snapshot integrity")]
    async fn claim_ledger_verify(&self) -> Result<Json<Output>, ErrorData> {
        let e = load(&self.path)?;
        let head = match e.last() {
            Some(x) => claim_ledger::ExpectedLedgerHead::new(x.sequence, x.entry_digest.clone()),
            None => claim_ledger::ExpectedLedgerHead::Empty,
        };
        match claim_ledger::verify_ledger(&e, &head) {
            Ok(v) => Ok(out(
                json!({"ok":true,"entry_count":e.len(),"last_sequence":v.last_sequence,"digest_chain_valid":true,"snapshot_valid":true}),
            )),
            Err(x) => Ok(out(
                json!({"ok":false,"entry_count":e.len(),"digest_chain_valid":false,"error":x.to_string()}),
            )),
        }
    }
    #[tool(description = "Query claims by text and support state")]
    async fn claim_ledger_query(
        &self,
        Parameters(p): Parameters<QueryParams>,
    ) -> Result<Json<Output>, ErrorData> {
        let mut r = claim_rows(&load(&self.path)?);
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
        Ok(out(json!({"claims":r})))
    }
    #[tool(description = "Get a claim and its related ledger events")]
    async fn claim_ledger_get(
        &self,
        Parameters(p): Parameters<GetParams>,
    ) -> Result<Json<Output>, ErrorData> {
        let e = load(&self.path)?;
        let events: Vec<&LedgerEntry> = e
            .iter()
            .filter(|x| {
                serde_json::to_value(&x.event)
                    .map(|v| v.to_string().contains(&p.claim_id))
                    .unwrap_or(false)
            })
            .collect();
        if events.is_empty() {
            return Ok(out(json!({"found":false,"claim_id":p.claim_id})));
        }
        Ok(out(
            json!({"found":true,"claim_id":p.claim_id,"events":events}),
        ))
    }
    #[tool(description = "Evaluate proof debt gate for claim IDs")]
    async fn claim_ledger_evaluate_proof_debt(
        &self,
        Parameters(p): Parameters<ProofDebtParams>,
    ) -> Result<Json<Output>, ErrorData> {
        let rows = claim_rows(&load(&self.path)?);
        let ids = if p.claim_ids.is_empty() {
            rows.iter()
                .filter_map(|r| r["claim_id"].as_str().map(str::to_owned))
                .collect()
        } else {
            p.claim_ids
        };
        let mut total = 0u64;
        for id in &ids {
            if let Some(r) = rows.iter().find(|r| r["claim_id"] == *id) {
                let state = r["support_state"].as_str().unwrap_or("unknown");
                if state != "supported" && state != "partially_supported" {
                    total += 250_000;
                }
            }
        }
        Ok(out(
            json!({"claim_ids":ids,"budget_micros":p.budget_micros,"debt_weight_micros":total,"gate_decision":if total>p.budget_micros{"block"}else if total>0{"warn"}else{"allow"}}),
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
        Ok(out(serde_json::to_value(r).map_err(err)?))
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
        assert_eq!(response.0.data["debt_weight_micros"], 250_000);
        std::fs::remove_dir_all(dir).expect("remove test directory");
    }
}
