# Governed HyperQuant Eval Receipts Implementation Plan

> For Hermes: implement this plan with TDD. Write the receipt tests first, verify RED, then implement the smallest API that passes.

Goal: turn HyperQuant governance routing into measurable, serializable operational evidence by joining quant-governor decisions with quant-eval HyperQuant fixture metrics and Q8/Q4 baseline estimates.

Architecture: quant-eval owns a crate-local receipt type that is stable for benchmarking and serialization. It calls quant-governor to obtain the policy decision and governance rationale, calls the existing HyperQuant evaluation harness to measure Z1/A2/D4 fixture behavior, then computes simple Q8/Q4 baseline byte estimates and admission/pass/fail fields. It does not claim model quality, paper parity, production deployment, or E8 support.

Tech Stack: Rust 2021, quant-eval, hyperquant, quant-governor, serde, serde_json, cargo test.

---

## Evidence-backed current state

Repo path: /home/sikmindz/Coding/Libraries
Date: 2026-06-27

Observed source state:
- quant-eval has `run_hyperquant_eval` in `quant-eval/src/hyperquant_eval.rs`.
- quant-eval currently depends on `hyperquant`, not `quant-governor`.
- quant-governor has `ContentType::Embedding`, `CodecProfile::Hyperquant`, and `GovernanceDecisionReceipt` routing traces.
- hyperquant evaluates Z1/A2/D4 fixtures and explicitly leaves E8 unsupported.

Previously verified commands in this workstream:
- `cargo test -p quant-governor --all-targets` passed.
- `cargo test -p quant-eval --tests` passed.
- `cargo test -p hyperquant --tests` passed.

Target gap:
- There is no quant-eval receipt that proves a governor policy admitted or blocked HyperQuant for an embedding fixture and ties that policy decision to measured HyperQuant fixture metrics and baseline byte estimates.

Claim boundary:
- Candidate/certified: deterministic fixture-level governed receipt generation, serialization, policy admission/block evidence, Q8/Q4 byte estimate comparison.
- Not claimed: real corpus retrieval quality, production admissibility, CUDA performance, paper parity, E8 implementation, or external benchmark superiority.

---

## Spec

### New public API

Add to `quant_eval::hyperquant_eval` and re-export from `quant_eval::lib`:

- `GovernedHyperQuantEvalConfig`
- `GovernedHyperQuantPolicyPreset`
- `GovernedHyperQuantEvalReceipt`
- `GovernedHyperQuantDecisionTrace`
- `GovernedHyperQuantBaseline`
- `run_governed_hyperquant_eval`

### Config semantics

`GovernedHyperQuantEvalConfig` contains:
- `fixture: HyperQuantEvalConfig`
- `policy_preset: GovernedHyperQuantPolicyPreset`
- `size_bytes: u64`
- `accuracy_requirement: f64`
- `latency_tolerance_ms: u64`
- `admissibility: quant_governor::AdmissibilityClass`

Policy preset variants:
- `Default`
- `StorageEfficient`
- `LowLatency`
- `AccuracyOriented`
- `CustomStrict`

`CustomStrict` maps to `GovernancePolicy::new(0.06, 64, 0.999)` to prove budget rejection of both HyperQuant (0.07) and Q4 (0.10).

### Receipt semantics

`GovernedHyperQuantEvalReceipt` contains:
- original config
- governor trace with selected codec, policy name, content type, admissibility, rationale, blocked profiles, candidate profiles
- `admitted: bool`, true only when selected codec is `Hyperquant`
- `admission_reason: String`, copied/narrowed from the governance rationale
- HyperQuant eval result from `run_hyperquant_eval`
- baselines for `Q8`, `Q4`, and `Hyperquant`
- `selected_hyperquant_profile: Option<HyperQuantProfileEval>` using D4 when available, falling back to A2/Z1
- `claim_boundary: String`

Baseline byte estimates:
- raw bytes per vector = fixture dim * 4
- Q8 bytes per vector = fixture dim
- Q4 bytes per vector = ceil(fixture dim / 2)
- Hyperquant bytes per vector = selected measured profile `estimated_compressed_bytes_per_vector`
- compression ratio = raw bytes / compressed bytes

### Admission semantics

- `StorageEfficient` + medium embedding fixture should select/admit HyperQuant.
- `LowLatency` + low latency embedding fixture should select Turbo and block HyperQuant.
- `AccuracyOriented` should select Q8 and block HyperQuant by profile policy.
- `CustomStrict` should select Q8 and block HyperQuant and Q4 by budget.

### Tests

Create `quant-eval/tests/hyperquant_governed_receipt.rs`.

Required tests:
1. `governed_storage_efficient_admits_hyperquant_with_measured_receipt`
   - config: storage_efficient, dim 16, vectors 16, size 500_000, accuracy 0.89, latency 200, Standard
   - expect selected codec `hyperquant`, admitted true, D4/A2/Z1 measurement present, Hyperquant baseline present.

2. `governed_low_latency_blocks_hyperquant_with_rationale`
   - config: low_latency, size 2_000_000, accuracy 0.85, latency 60, Standard
   - expect selected codec `turbo`, admitted false, blocked_profiles contains `hyperquant`, rationale mentions latency.

3. `governed_accuracy_oriented_blocks_hyperquant`
   - config: accuracy_oriented, size 500_000, accuracy 0.93, latency 200, Standard
   - expect selected codec `q8`, blocked_profiles contains `hyperquant`.

4. `governed_strict_budget_blocks_hyperquant_and_q4`
   - config: custom_strict, size 2_000_000, accuracy 0.80, latency 500, Standard
   - expect selected codec `q8`, blocked_profiles contains `hyperquant` and `q4`.

5. `governed_hyperquant_receipt_round_trips_json`
   - serialize/deserialize receipt and assert equality.

---

## Implementation tasks

### Task 1: Add failing governed receipt tests

Files:
- Create: `quant-eval/tests/hyperquant_governed_receipt.rs`

Step 1: Add the five tests above using the desired public API.
Step 2: Run `cargo test -p quant-eval --test hyperquant_governed_receipt`.
Expected: FAIL because the API and quant-governor dependency are not wired yet.

### Task 2: Add quant-governor dependency

Files:
- Modify: `quant-eval/Cargo.toml`

Add:
`quant-governor = { version = "0.1.0", path = "../quant-governor" }`

Step 1: Run focused test again.
Expected: still FAIL because receipt API is missing.

### Task 3: Implement receipt structs and runner

Files:
- Modify: `quant-eval/src/hyperquant_eval.rs`
- Modify: `quant-eval/src/lib.rs`

Implementation notes:
- Add public structs/enums with `Debug, Clone, PartialEq, Serialize, Deserialize`.
- Convert quant-governor receipt data into string-backed `GovernedHyperQuantDecisionTrace` to avoid leaking unstable enum internals into the quant-eval receipt schema.
- If the governor returns a non-governance receipt, still populate selected codec and use a conservative rationale.
- Call `run_hyperquant_eval(&config.fixture)` regardless of admission so blocked decisions still carry measured candidate context.
- Pick selected HyperQuant profile as D4, else A2, else Z1.

Step 1: Run focused test.
Expected: PASS.

### Task 4: Run affected suites

Commands:
- `cargo test -p quant-eval --tests`
- `cargo test -p quant-governor --all-targets`
- `cargo test -p hyperquant --tests`

Expected: all PASS.

### Task 5: Diff audit

Commands:
- `git diff -- quant-eval/Cargo.toml quant-eval/src/hyperquant_eval.rs quant-eval/src/lib.rs quant-eval/tests/hyperquant_governed_receipt.rs docs/plans/2026-06-27-governed-hyperquant-eval-receipts.md`

Expected: changes are limited to plan, quant-eval dependency/API/tests, and no unrelated files.
