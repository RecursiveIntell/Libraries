# Receipt Schema Pack — Phase 1 Specification

**Priority:** P0  
**Window:** 3-10 days  
**Owner:** semantic-memory, claim-ledger, stack-ids, quant-governor  

---

## Objective

Create the lowest-level schema and validation layer that prevents later compression, graph, and security work from drifting into hidden truth stores. All material operations must emit typed, receipt-bearing artifacts with canonical hash rules and supersession indexing.

---

## Required Receipt Families

### 1. SemanticResidualReceiptV1

**Purpose:** Records when semantic-memory projection degrades from exact source representation, including what was lost, why, and how to recover.

**Schema:**
```json
{
  "schema_version": "SemanticResidualReceiptV1",
  "receipt_id": "smrr:<blake3_hash>",
  "source_id": "<source_id>",
  "chunk_id": "<chunk_id>",
  "projection_mode": "exact" | "compressed" | "degraded",
  "codec_used": "raw" | "q8" | "q4" | "turbo" | "fib" | "delta_kv",
  "raw_digest": "<blake3_hash_of_exact_source>",
  "compressed_digest": "<blake3_hash_of_compressed_artifact>",
  "residual_error": {
    "mse": <f64>,
    "max_absolute_error": <f64>,
    "cosine_distortion": <f64>,
    "inner_product_distortion": <f64>
  },
  "degradation_reason": null | "context_length_exceeded" | "budget_constraint" | "codec_unavailable" | "fallback_triggered",
  "exact_fallback_available": <bool>,
  "exact_fallback_retained": <bool>,
  "rollback_pointer": null | "<receipt_id_of_exact_source>",
  "recorded_time": "<ISO8601_utc>",
  "valid_time": "<ISO8601_utc>",
  "superseded_by": null | "<receipt_id>"
}
```

**Validation Rules:**
- If `projection_mode == "exact"`, then `residual_error` must be null or all zeros
- If `projection_mode == "compressed"` or `"degraded"`, then `residual_error` must be present and non-zero
- `exact_fallback_retained` can only be true if `exact_fallback_available` is true
- `rollback_pointer` must reference a valid receipt with matching `raw_digest`
- `recorded_time` must be monotonically increasing within a supersession chain

---

### 2. CapabilityArgumentContractV1

**Purpose:** PACT-style argument-level authority binding for tool invocation. Records what arguments require elevated trust, what was denied, and what was approved.

**Schema:**
```json
{
  "schema_version": "CapabilityArgumentContractV1",
  "contract_id": "cact:<blake3_hash>",
  "tool_name": "<tool_name>",
  "semantic_role": "read_only" | "mutation" | "shell" | "network" | "filesystem" | "package_management",
  "arguments": [
    {
      "arg_name": "<name>",
      "arg_value_hash": "<blake3_hash>",
      "trust_level": "benign" | "elevated" | "dangerous",
      "authority_required": null | "user_consent" | "policy_approval" | "dry_run_first",
      "decision": "approved" | "denied" | "escalated" | "dry_run_scheduled",
      "denial_reason": null | "<reason_string>"
    }
  ],
  "overall_decision": "approved" | "denied" | "partially_approved" | "escalated",
  "approved_arguments": ["<arg_name>", ...],
  "denied_arguments": ["<arg_name>", ...],
  "lineage_pointer": null | "<argument_lineage_receipt_id>",
  "recorded_time": "<ISO8601_utc>",
  "operator_id": null | "<operator_identifier>"
}
```

**Validation Rules:**
- `semantic_role` must match the tool's declared capability surface
- Any argument with `trust_level == "dangerous"` must have `authority_required != null`
- `overall_decision` must be consistent with individual argument decisions
- If any argument is denied, `denial_reason` must be present for that argument

---

### 3. ArgumentLineageReceiptV1

**Purpose:** Tracks the provenance of tool arguments through transformations, user edits, model suggestions, and policy filters.

**Schema:**
```json
{
  "schema_version": "ArgumentLineageReceiptV1",
  "lineage_id": "alnr:<blake3_hash>",
  "final_argument": {
    "arg_name": "<name>",
    "final_value_hash": "<blake3_hash>",
    "final_value_preview": "<truncated_preview_max_200_chars>"
  },
  "provenance_chain": [
    {
      "step": <u32>,
      "origin": "user_input" | "model_generated" | "policy_transformed" | "user_edited" | "system_injected",
      "value_hash": "<blake3_hash>",
      "value_preview": "<truncated_preview>",
      "transform_reason": null | "<reason>",
      "actor": null | "<actor_identifier>"
    }
  ],
  "integrity_check": {
    "chain_hash": "<blake3_hash_of_entire_chain>",
    "final_matches_chain_end": <bool>
  },
  "recorded_time": "<ISO8601_utc>"
}
```

**Validation Rules:**
- `provenance_chain` must have at least one step
- The last step's `value_hash` must match `final_argument.final_value_hash`
- `integrity_check.final_matches_chain_end` must be true for the receipt to be valid
- Each step's `step` number must be sequential starting from 0

---

### 4. PersistentReasoningSubgraphV1

**Purpose:** Records a persistent reasoning subgraph extracted from ClaimLedger/semantic-memory, including pruning history and rebuild capability.

**Schema:**
```json
{
  "schema_version": "PersistentReasoningSubgraphV1",
  "subgraph_id": "prsg:<blake3_hash>",
  "source_claims": ["<claim_id>", ...],
  "source_evidence": ["<evidence_id>", ...],
  "reasoning_route": {
    "route_type": "deductive" | "inductive" | "abductive" | "analogical" | "mixed",
    "steps": [
      {
        "step_id": "<step_id>",
        "operation": "modus_ponens" | "modus_tollens" | "induction" | "abduction" | "analogy" | "synthesis",
        "inputs": ["<claim_id_or_evidence_id>", ...],
        "output": "<claim_id>",
        "confidence": <f64_0_to_1>
      }
    ]
  },
  "pruning_history": [
    {
      "pruned_at": "<ISO8601_utc>",
      "pruned_nodes": ["<node_id>", ...],
      "prune_reason": "contradiction_resolved" | "evidence_superseded" | "budget_constraint" | "user_request",
      "preserved_as": null | "<archive_receipt_id>"
    }
  ],
  "rebuild_command": "<command_to_rebuild_subgraph_from_source>",
  "contradiction_lineage_preserved": <bool>,
  "recorded_time": "<ISO8601_utc>",
  "valid_time": "<ISO8601_utc>",
  "superseded_by": null | "<subgraph_id>"
}
```

**Validation Rules:**
- `source_claims` and `source_evidence` must reference existing ClaimLedger records
- Each reasoning step's `inputs` must be defined before the step that uses them
- `contradiction_lineage_preserved` must be true if any `pruning_history` entries exist
- `rebuild_command` must be deterministic and reference only canonical sources

---

### 5. CompressionSurvivabilityReportV1

**Purpose:** Records the results of compression survivability testing, including drift metrics, contradiction detection, and exact baseline comparison.

**Schema:**
```json
{
  "schema_version": "CompressionSurvivabilityReportV1",
  "report_id": "csrp:<blake3_hash>",
  "codec_under_test": "<codec_name>",
  "baseline_codec": "exact_fp32" | "exact_fp16" | "scalar_q8" | "<other>",
  "test_corpus": {
    "corpus_id": "<corpus_identifier>",
    "corpus_size_vectors": <u64>,
    "corpus_dimension": <u32>,
    "corpus_hash": "<blake3_hash>"
  },
  "survivability_metrics": {
    "reconstruction_mse": <f64>,
    "cosine_similarity_mean": <f64>,
    "cosine_similarity_std": <f64>,
    "inner_product_distortion_mean": <f64>,
    "rank_correlation_spearman": <f64>,
    "top_k_recall_at_10": <f64>,
    "top_k_recall_at_100": <f64>
  },
  "drift_metrics": {
    "semantic_drift_detected": <bool>,
    "drift_severity": null | "negligible" | "minor" | "moderate" | "severe",
    "contradictions_introduced": <u64>,
    "contradiction_examples": ["<example_id>", ...]
  },
  "exact_baseline_comparison": {
    "baseline_recall_at_10": <f64>,
    "codec_recall_at_10": <f64>,
    "recall_delta": <f64>,
    "baseline_latency_us": <f64>,
    "codec_latency_us": <f64>,
    "latency_delta_us": <f64>,
    "baseline_memory_bytes": <u64>,
    "codec_memory_bytes": <u64>,
    "memory_savings_ratio": <f64>
  },
  "failure_cases": [
    {
      "case_id": "<case_id>",
      "failure_mode": "catastrophic_forgetting" | "accuracy_cliff" | "latency_spike" | "oom" | "non_determinism",
      "description": "<description>"
    }
  ],
  "public_claim_eligible": <bool>,
  "recorded_time": "<ISO8601_utc>",
  "hardware_profile_ref": "<hardware_profile_receipt_id>"
}
```

**Validation Rules:**
- `survivability_metrics` must all be finite (no NaN or Inf)
- If `drift_metrics.semantic_drift_detected` is true, `drift_severity` must not be null
- `exact_baseline_comparison` must be present for any public-claim-eligible run
- `public_claim_eligible` can only be true if `exact_baseline_comparison` is present and all metrics are finite

---

### 6. EvidenceSufficiencyReceiptV1

**Purpose:** Records whether retrieved evidence was sufficient to support an answer, what was missing, and whether the model reasoned correctly given the evidence.

**Schema:**
```json
{
  "schema_version": "EvidenceSufficiencyReceiptV1",
  "receipt_id": "esrp:<blake3_hash>",
  "query_id": "<query_identifier>",
  "answer_id": "<answer_identifier>",
  "retrieved_evidence": ["<evidence_id>", ...],
  "evidence_sufficiency": {
    "sufficient_for_answer": <bool>,
    "missing_evidence_types": ["<type>", ...],
    "coverage_score": <f64_0_to_1>,
    "redundancy_score": <f64_0_to_1>
  },
  "reasoning_quality": {
    "reasoning_route_valid": <bool>,
    "reasoning_errors": [
      {
        "error_type": "non_sequitur" | "false_dichotomy" | "hasty_generalization" | "circular_reasoning" | "evidence_misinterpretation",
        "description": "<description>",
        "affected_step": "<step_reference>"
      }
    ],
    "answer_supported_by_evidence": <bool>,
    "answer_contradicts_evidence": <bool>
  },
  "graphr b_diagnosis": {
    "answer_in_retrieved_context": <bool>,
    "model_failed_reasoning": <bool>,
    "failure_mode": null | "retrieval_success_reasoning_failure" | "evidence_insufficient" | "evidence_contradictory"
  },
  "recorded_time": "<ISO8601_utc>"
}
```

**Validation Rules:**
- If `evidence_sufficiency.sufficient_for_answer` is true, `reasoning_quality.answer_supported_by_evidence` should typically be true (flag if not)
- If `graphr b_diagnosis.answer_in_retrieved_context` is true AND `graphr b_diagnosis.model_failed_reasoning` is true, this indicates a reasoning bottleneck (not retrieval failure)
- `reasoning_errors` must be empty if `reasoning_quality.reasoning_route_valid` is true

---

## Additional Receipt Families (Phase 1 Extension)

7. **GlossDenseIndexReceiptV1** — Dense indexing completion with coverage metrics
8. **GlossSemanticMemoryProjectionReceiptV1** — Projection completion with link counts and failure summary
9. **GlossRetrievalProbeReceiptV1** — Retrieval probe results with backend decision disclosure
10. **GlossAnswerReceiptV1** — Answer generation with evidence disclosure and degradation markers

---

## Canonical Hash Rules

All receipts must use **blake3** for content hashing with the following rules:

1. **RFC 8785 JCS** — All JSON must be canonicalized using RFC 8785 JSON Canonicalization Scheme before hashing
2. **Duplicate-key rejection** — JSON parsers must reject objects with duplicate keys
3. **Field order independence** — Canonicalization ensures field order does not affect hash
4. **Receipt ID derivation** — `receipt_id = "<prefix>:<blake3_hex_of_canonical_json>"`
5. **Supersession chaining** — Each superseding receipt includes hash of superseded receipt

---

## Unknown-Field Policy

All receipts must use **strict unknown-field rejection**:

```json
{
  "unknown_field_policy": "reject",
  "unknown_field_behavior": "Return error on deserialization if any field is present that is not defined in the schema version"
}
```

This prevents silent schema widening and ensures backward compatibility is explicit.

---

## Supersession Indexing

All receipts that can be updated must support supersession:

```json
{
  "superseded_by": null | "<receipt_id_of_newer_version>",
  "supersedes": null | ["<receipt_id_of_superseded_receipt>", ...]
}
```

**Rules:**
- A receipt can only be superseded once (linear chain, not DAG)
- The superseding receipt must have a later `recorded_time`
- Supersession must preserve `valid_time` semantics
- Supersession chain must be traversable from any receipt to the current version

---

## Files to Create

```text
Libraries/stack-ids/src/receipts/
  mod.rs
  semantic_residual_receipt.rs
  capability_argument_contract.rs
  argument_lineage_receipt.rs
  persistent_reasoning_subgraph.rs
  compression_survivability_report.rs
  evidence_sufficiency_receipt.rs
  gloss_dense_index_receipt.rs
  gloss_semantic_memory_projection_receipt.rs
  gloss_retrieval_probe_receipt.rs
  gloss_answer_receipt.rs

Libraries/stack-ids/tests/
  receipt_roundtrip/
    semantic_residual_receipt.rs
    capability_argument_contract.rs
    argument_lineage_receipt.rs
    persistent_reasoning_subgraph.rs
    compression_survivability_report.rs
    evidence_sufficiency_receipt.rs
  unknown_field_rejection/
    all_receipts.rs
  supersession_chains/
    all_receipts.rs

Libraries/contracts/
  SemanticResidualReceiptV1.schema.json
  CapabilityArgumentContractV1.schema.json
  ArgumentLineageReceiptV1.schema.json
  PersistentReasoningSubgraphV1.schema.json
  CompressionSurvivabilityReportV1.schema.json
  EvidenceSufficiencyReceiptV1.schema.json

Libraries/scripts/
  validate_receipt_schemas.py
  generate_receipt_fixtures.py
  verify_supersession_chains.py
```

---

## Acceptance Gates

1. ✅ Schema files validate against strict test fixtures
2. ✅ Round-trip serialization preserves canonical hashes
3. ✅ Unknown field and silent widening tests fail closed
4. ✅ Ownership map lists canonical owner and forbidden duplicate owners for every schema
5. ✅ A sample mixed-trust tool invocation emits argument lineage and deny/escalate decision
6. ✅ A sample compressed artifact points to raw baseline and has rollback metadata
7. ✅ A sample graph projection rebuilds from source/evidence refs
8. ✅ Final report includes receipt-like evidence and rollback plan

---

## Ownership Map

| Receipt Family | Canonical Owner | Forbidden Duplicate Owners |
|---|---|---|
| SemanticResidualReceiptV1 | semantic-memory | gloss, ai-batch-queue, llm-pipeline |
| CapabilityArgumentContractV1 | agent-guard | agent-graph, forge-pilot, recall-session |
| ArgumentLineageReceiptV1 | agent-guard | agent-graph, llm-pipeline |
| PersistentReasoningSubgraphV1 | claim-ledger | semantic-memory, ai-batch-queue |
| CompressionSurvivabilityReportV1 | quant-eval | turbo-quant, fib-quant, poly-kv |
| EvidenceSufficiencyReceiptV1 | claim-ledger | gloss, llm-pipeline |
| GlossDenseIndexReceiptV1 | gloss (app) | semantic-memory |
| GlossSemanticMemoryProjectionReceiptV1 | gloss (app) | semantic-memory |
| GlossRetrievalProbeReceiptV1 | gloss (app) | semantic-memory |
| GlossAnswerReceiptV1 | gloss (app) | llm-pipeline |

---

## Rollback Plan

- All schema changes are additive; old schema versions remain supported for deserialization
- If receipt emission breaks existing workflows, emit receipts as advisory-only (not blocking) for 1 sprint
- If canonical hash rules cause reproducibility issues, document the divergence and pin hash algorithm version
- No receipt schema change should break existing persisted data; receipts are append-only observations
