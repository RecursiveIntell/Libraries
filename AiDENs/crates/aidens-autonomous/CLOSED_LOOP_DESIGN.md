# Closed-Loop Autonomous Research-to-Implementation Engine
# Implementation Design — Missing Pieces + Integration

Date: 2026-06-25
Author: Hermes Agent (for Josh Stevenson / RecursiveIntell)

## EXECUTIVE SUMMARY

The autonomous loop already exists as `aidens-autonomous` with a working
detect → enqueue → execute → capture → evaluate cycle (683 lines in
loop_driver.rs). The semantic-memory substrate already has decoder (998
lines), subtraction (429 lines), compression_governor (303 lines),
temporal (391 lines), provenance (1456 lines), contradiction_detect,
discord, factor_graph, community, topology, and pipeline modules.

Four control pieces are missing that would make the loop self-regulating
instead of human-driven. Two additional pieces are easy wins that improve
overall functionality. All six are designed here against the actual
codebase, not against memory.

## WHAT EXISTS (verified against source)

### aidens-autonomous crate (7 source files)
- loop_driver.rs (683 lines) — AutonomousLoop with run() cycle
- gap_detector.rs (1233 lines) — 7 gap types, HTTP calls to SM server
- task_generator.rs — converts gaps to JobV1 entries
- executor.rs — plan-act-verify loop via Ollama
- capture.rs — stores results in SM with dedup
- evaluation.rs (399 lines) — quality scoring, Promote/Quarantine/Reject
- lib.rs — public API

LoopState tracks: iteration, gaps_detected, tasks_generated,
tasks_completed, tasks_failed, facts_captured, facts_rejected,
consecutive_failures, safe_mode, current_job, last_error.

LoopConfig has: max_iterations, gap_detection_interval,
sleep_between_iterations_ms, max_consecutive_failures, ollama_url,
ollama_model, memory_dir, queue_dir, http_base_url.

### semantic-memory crate (56 source files)
- decoder.rs (998 lines) — Syndrome, SyndromeType, SyndromeSeverity,
  CorrectionOperation, hyperedges, refutation testing
- subtraction.rs (429 lines) — SubtractionCandidate, CompactionStrategy,
  SubtractionReceipt, structuring scores, invariant checks
- compression_governor.rs (303 lines) — ImportanceScore,
  QuantizationLevel, RequantizationReceipt, weighted scoring
- temporal.rs (391 lines) — TemporalConfig, compute_temporal_weight,
  age decay, supersession, support boost, contradiction penalty
- provenance.rs (1456 lines) — 4 semirings, ProvenanceReceiptV1,
  append-plus-supersession, combine operations
- contradiction_detect.rs — content-level contradiction detection
- factor_graph.rs — belief propagation
- community.rs — Leiden community detection
- topology.rs — Betti numbers, structural gaps
- pipeline.rs — staged retrieval
- discord.rs — second-order retrieval
- routing.rs + rl_routing.rs — retrieval routing with RL

### aidens-governance-kit
- verification.rs — VerificationPlanV1, RiskClass, GovernanceDecisionV1
- Re-exports verification_control, verification_policy,
  verification_adjudication, verification_calibration from canonical stack

## THE FOUR MISSING PIECES

### 1. ViscosityController (FEUT-005)

Location: new file `aidens-autonomous/src/viscosity.rs`
Lines: ~280

PURPOSE: Adaptive strictness controller. Raises/lowers gate strictness
based on observed loop signals. Replaces the fixed
max_consecutive_failures / safe_mode binary with a continuous control
signal.

DESIGN:

```rust
/// Composite signal computed from loop metrics.
/// I(t) = w1*failure_rate + w2*drift_rate + w3*ambiguity_score
///        + w4*contradiction_density
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViscositySignal {
    /// Rolling failure rate (failures / total attempts, window=20).
    pub failure_rate: f64,
    /// Drift rate: fraction of recent captures that duplicate existing
    /// facts (high drift = loop not learning anything new).
    pub drift_rate: f64,
    /// Ambiguity score: fraction of recent evaluations that resulted in
    /// Quarantine (not Promote, not Reject — uncertain).
    pub ambiguity_score: f64,
    /// Contradiction density: contradictions detected per cycle / facts
    /// added per cycle. High = new knowledge conflicts with existing.
    pub contradiction_density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViscosityConfig {
    /// Weight for failure_rate (default 0.30)
    pub failure_weight: f64,
    /// Weight for drift_rate (default 0.20)
    pub drift_weight: f64,
    /// Weight for ambiguity_score (default 0.20)
    pub ambiguity_weight: f64,
    /// Weight for contradiction_density (default 0.30)
    pub contradiction_weight: f64,
    /// Base viscosity (minimum strictness even at zero signal).
    pub base_viscosity: f64,        // default 0.3
    /// Gamma coefficient: ν_eff = ν_base + γ * I(t)
    pub gamma: f64,                 // default 0.7
    /// Window size for rolling metrics.
    pub window_size: usize,         // default 20
}

/// The computed viscosity level for this cycle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StrictnessLevel {
    /// Low viscosity: fast execution, minimal gates, high throughput.
    /// I(t) < 0.2. Accept Promote at score >= 0.6. Skip hostile audit.
    Fast,
    /// Medium viscosity: normal execution, standard gates.
    /// 0.2 <= I(t) < 0.5. Accept Promote at score >= 0.8. Light audit.
    Normal,
    /// High viscosity: slow execution, thorough verification.
    /// 0.5 <= I(t) < 0.8. Accept Promote at score >= 0.9. Full audit.
    Strict,
    /// Critical viscosity: pause new task generation, shift to
    /// subtractive mode. I(t) >= 0.8.
    Frozen,
}

pub struct ViscosityController {
    config: ViscosityConfig,
    /// Rolling window of (success: bool, was_duplicate: bool,
    /// disposition: FactDisposition, contradictions: usize).
    history: VecDeque<CycleRecord>,
}

struct CycleRecord {
    success: bool,
    was_duplicate: bool,
    disposition: FactDisposition,
    contradictions_detected: usize,
    facts_added: usize,
}

impl ViscosityController {
    pub fn new(config: ViscosityConfig) -> Self;

    /// Record a cycle's outcomes.
    pub fn record(&mut self, record: CycleRecord);

    /// Compute current viscosity signal from rolling window.
    pub fn compute_signal(&self) -> ViscositySignal;

    /// Compute effective viscosity: ν_eff = ν_base + γ * I(t).
    pub fn compute_viscosity(&self) -> f64;

    /// Map viscosity to strictness level.
    pub fn current_strictness(&self) -> StrictnessLevel;

    /// Get the promotion threshold for the current strictness.
    /// Facts scoring below this are quarantined instead of promoted.
    pub fn promotion_threshold(&self) -> f64;

    /// Whether to skip gap detection this cycle (Frozen mode).
    pub fn should_generate_tasks(&self) -> bool;

    /// Whether to run a hostile audit this cycle (Strict/Frozen).
    pub fn should_run_audit(&self) -> bool;

    /// Whether to shift to subtractive mode (Frozen).
    pub fn should_shift_to_subtractive(&self) -> bool;
}
```

INTEGRATION INTO loop_driver.rs:

Add `pub viscosity: ViscosityController` to AutonomousLoop struct.

In the run() method, after step 6 (evaluation), add:

```rust
// Record cycle outcome for viscosity.
let record = CycleRecord {
    success: exec_result.success,
    was_duplicate: capture_outcome.facts_skipped_duplicates > 0,
    disposition: /* avg disposition */,
    contradictions_detected: /* from decoder if available */,
    facts_added: capture_outcome.facts_added,
};
self.viscosity.record(record);

// Adjust behavior based on viscosity.
let strictness = self.viscosity.current_strictness();
match strictness {
    StrictnessLevel::Frozen => {
        // Skip task generation, run subtraction instead.
        self.run_subtractive_cycle().await?;
    }
    StrictnessLevel::Strict => {
        // Run hostile audit before accepting results.
        self.run_hostile_audit(&exec_result).await?;
    }
    _ => {}
}
```

Replace the fixed `max_consecutive_failures` / `check_safe_mode()` with
viscosity-driven Frozen detection. The old safe_mode becomes the
Frozen level.

### 2. ProofDebtBudget (FEUT-004)

Location: new file `aidens-autonomous/src/proof_debt.rs`
Lines: ~220

PURPOSE: Tracks cumulative unverified claims. When debt exceeds
threshold, signals the loop to shift from additive to subtractive mode.

DESIGN:

```rust
/// A single proof-debt entry recording a claim that was made but not
/// yet verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofDebtEntry {
    /// Unique entry id.
    pub entry_id: String,
    /// The fact/claim id in semantic-memory.
    pub claim_id: String,
    /// Namespace the claim belongs to.
    pub namespace: String,
    /// When the debt was incurred (ISO-8601).
    pub incurred_at: String,
    /// When the debt was paid (if verified). None = still outstanding.
    pub paid_at: Option<String>,
    /// Risk class of the claim.
    pub risk_class: RiskClass,
    /// How the debt was paid (if paid).
    pub payment_method: Option<PaymentMethod>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskClass {
    /// Low risk: observational, internal. Cheap to verify.
    Low,
    /// Medium risk: cross-domain claim, requires checking.
    Medium,
    /// High risk: public claim, requires falsification attempt.
    High,
    /// Critical risk: requires replay AND falsification.
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentMethod {
    /// Verified by test execution.
    TestPassed,
    /// Verified by hostile audit.
    AuditPassed,
    /// Verified by contradiction check (no contradictions found).
    NoContradictions,
    /// Verified by external evidence.
    ExternalEvidence,
    /// Superseded by a better claim.
    Superseded,
    /// Quarantined (debt forgiven, claim isolated).
    Quarantined,
}

pub struct ProofDebtBudget {
    /// Outstanding debt entries, keyed by entry_id.
    outstanding: HashMap<String, ProofDebtEntry>,
    /// Paid debt entries (for audit trail).
    paid: Vec<ProofDebtEntry>,
    /// Debt threshold per risk class before triggering subtractive mode.
    thresholds: HashMap<RiskClass, usize>,
    /// Total debt incurred since loop start.
    total_incurred: usize,
    /// Total debt paid since loop start.
    total_paid: usize,
}

impl ProofDebtBudget {
    pub fn new() -> Self;

    /// Set debt threshold for a risk class.
    pub fn set_threshold(&mut self, risk_class: RiskClass, threshold: usize);

    /// Incur debt — called when a fact is promoted.
    pub fn incur(
        &mut self,
        claim_id: &str,
        namespace: &str,
        risk_class: RiskClass,
    ) -> String;  // returns entry_id

    /// Pay debt — called when a fact is verified.
    pub fn pay(
        &mut self,
        entry_id: &str,
        method: PaymentMethod,
    ) -> Result<()>;

    /// Pay all debt for a given claim (e.g., when contradiction check
    /// passes for all entries referencing that claim).
    pub fn pay_for_claim(
        &mut self,
        claim_id: &str,
        method: PaymentMethod,
    ) -> usize;  // returns number of entries paid

    /// Current outstanding debt count by risk class.
    pub fn outstanding_by_risk(&self) -> HashMap<RiskClass, usize>;

    /// Total outstanding debt.
    pub fn total_outstanding(&self) -> usize;

    /// Debt ratio: outstanding / total_incurred.
    pub fn debt_ratio(&self) -> f64;

    /// Whether any risk class has exceeded its threshold.
    pub fn exceeds_threshold(&self) -> bool;

    /// Which risk classes have exceeded threshold.
    pub fn exceeded_risk_classes(&self) -> Vec<RiskClass>;

    /// Whether the loop should shift to subtractive mode.
    pub fn should_shift_to_subtractive(&self) -> bool {
        self.exceeds_threshold() || self.debt_ratio() > 0.6
    }

    /// Emit a receipt for the current debt state.
    pub fn debt_receipt(&self) -> ProofDebtReceipt;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofDebtReceipt {
    pub total_incurred: usize,
    pub total_paid: usize,
    pub total_outstanding: usize,
    pub debt_ratio: f64,
    pub outstanding_by_risk: HashMap<String, usize>,
    pub exceeds_threshold: bool,
    pub timestamp: String,
}
```

INTEGRATION INTO loop_driver.rs:

In step 6 (evaluation), when a fact is Promoted:
```rust
FactDisposition::Promote => {
    // Incur proof-debt for the new fact.
    let risk_class = classify_risk(&exec_result.output, &gap);
    let entry_id = self.proof_debt.incur(
        fact_id, namespace, risk_class
    );
    // Low risk: pay immediately if no contradictions.
    if risk_class == RiskClass::Low {
        self.proof_debt.pay(&entry_id, PaymentMethod::NoContradictions);
    }
}
```

After step 6, check debt:
```rust
if self.proof_debt.should_shift_to_subtractive() {
    self.run_subtractive_cycle().await?;
}
```

The subtractive cycle pays down debt by:
1. Running contradiction detection on outstanding claims
2. Paying debt for claims with no contradictions
3. Quarantining claims that have contradictions (paying debt via Quarantined)
4. Running subtraction on stale/superseded items

RISK CLASSIFICATION HEURISTIC:
- Low: observational facts, internal state, captured from execution
- Medium: cross-domain claims, new patterns, extracted from research
- High: public claims, benchmark claims, novelty claims
- Critical: claims that could influence identity or public positioning

### 3. EntropyGradientSearcher (FEUT-001)

Location: new file `aidens-autonomous/src/entropy_search.rs`
Lines: ~250

PURPOSE: Replaces random gap detection with entropy-gradient-guided
exploration. Computes where knowledge is most uncertain and most
changing, prioritizes those areas.

DESIGN:

```rust
/// Entropy metrics for a namespace or domain area.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEntropy {
    /// Namespace or domain identifier.
    pub domain: String,
    /// Total facts in this domain.
    pub fact_count: usize,
    /// Graph edge count in this domain.
    pub edge_count: usize,
    /// Contradiction count in this domain.
    pub contradiction_count: usize,
    /// Facts added in the last N cycles (growth rate).
    pub recent_growth: usize,
    /// Average structuring score (from subtraction engine).
    pub avg_structuring_score: f64,
    /// Computed entropy: higher = more unknown/uncertain.
    pub entropy: f64,
    /// Computed gradient: how fast knowledge is changing.
    pub gradient: f64,
    /// Exploration priority: entropy / (1 + structuring).
    pub priority: f64,
}

pub struct EntropyGradientSearcher {
    /// HTTP base URL for semantic-memory server.
    http_base_url: String,
    /// Number of recent cycles to consider for growth rate.
    growth_window: usize,  // default 10
    /// Domains that have been declared saturated.
    saturated: HashSet<String>,
    /// Per-domain yield history (candidates found per exploration).
    yield_history: HashMap<String, VecDeque<usize>>,
}

impl EntropyGradientSearcher {
    pub fn new(http_base_url: &str) -> Self;

    /// Query semantic-memory for domain statistics.
    async fn query_domain_stats(&self) -> Result<Vec<DomainStats>>;

    /// Compute entropy for a domain.
    /// entropy = -log2(fact_count + 1) * (1 + contradiction_count)
    ///           / (1 + edge_count)
    /// More facts with fewer edges and more contradictions = higher entropy.
    fn compute_entropy(stats: &DomainStats) -> f64;

    /// Compute gradient for a domain.
    /// gradient = recent_growth / max(growth_window, 1)
    /// Normalized by total facts: gradient / (fact_count + 1).
    fn compute_gradient(stats: &DomainStats, recent_growth: usize) -> f64;

    /// Compute exploration priority.
    /// priority = entropy / (1 + structuring_score)
    /// High entropy + low structuring = explore now.
    /// High entropy + high structuring = implement (well understood but large).
    /// Low entropy = saturated, move on.
    fn compute_priority(entropy: f64, structuring: f64) -> f64;

    /// Rank domains by exploration priority.
    async fn rank_domains(&self) -> Result<Vec<DomainEntropy>>;

    /// Get the top N domains to explore next.
    pub async fn next_targets(&self, n: usize) -> Result<Vec<DomainEntropy>>;

    /// Record exploration yield for a domain (for saturation tracking).
    pub fn record_yield(&mut self, domain: &str, candidates_found: usize);

    /// Check if a domain is saturated.
    /// Saturated if: yield has been below threshold for N consecutive
    /// explorations.
    pub fn is_saturated(&self, domain: &str) -> bool;

    /// Get all saturated domains.
    pub fn saturated_domains(&self) -> Vec<String>;

    /// Mark a domain as manually saturated.
    pub fn mark_saturated(&mut self, domain: &str);

    /// Clear saturation for a domain.
    pub fn clear_saturation(&mut self, domain: &str);
}
```

INTEGRATION INTO loop_driver.rs:

In run_gap_detection(), replace the current random detection:

```rust
async fn run_gap_detection(&self) -> Result<()> {
    // Use entropy-gradient to pick which domains to scan.
    let targets = self.entropy_search.next_targets(5).await?;

    // Filter out saturated domains.
    let active_targets: Vec<_> = targets
        .into_iter()
        .filter(|t| !self.entropy_search.is_saturated(&t.domain))
        .collect();

    // For each target domain, detect gaps within that namespace.
    let mut all_gaps = Vec::new();
    for target in &active_targets {
        let gaps = self.detector
            .detect_gaps_in_namespace(30, &attempted, &target.domain)
            .await?;
        all_gaps.extend(gaps);
    }

    // Record yield for saturation tracking.
    for target in &active_targets {
        let yield_count = all_gaps
            .iter()
            .filter(|g| g.namespace == target.domain)
            .count();
        self.entropy_search.record_yield(&target.domain, yield_count);
    }

    // Generate tasks from gaps.
    let job_ids = self.generator.generate_tasks(&all_gaps).await?;
    Ok(())
}
```

REQUIRES: gap_detector.rs needs a new method
`detect_gaps_in_namespace(limit, attempted, namespace)` — a variant of
the existing `detect_gaps` that filters to a specific namespace. This is
a ~30 line addition to the existing HTTP query logic.

### 4. SaturationTracker

Location: same file as entropy_search.rs (companion struct)
Lines: ~80 ( folds into entropy_search.rs)

PURPOSE: Tracks candidate yield per domain per exploration. When yield
drops below threshold for N consecutive explorations, declares the
domain saturated and shifts focus.

Already partially designed above as part of EntropyGradientSearcher
(yield_history, is_saturated, record_yield). The saturation tracker is
not a separate module — it's a natural extension of the entropy searcher
because they share the same domain-level view.

SATURATION LOGIC:
```rust
/// Saturated if last N yields are all below threshold.
fn check_saturation(&self, domain: &str) -> bool {
    let history = match self.yield_history.get(domain) {
        Some(h) => h,
        None => return false,
    };
    if history.len() < self.saturation_window {
        return false;  // not enough data
    }
    let recent = history.iter().rev().take(self.saturation_window);
    recent.all(|&yield_count| yield_count < self.saturation_threshold)
}
```

Default: saturation_window = 3, saturation_threshold = 2.
If 3 consecutive explorations of a domain each find fewer than 2
candidates, the domain is saturated.

When ALL tracked domains are saturated, the loop shifts from exploration
to implementation (generate implementation tasks from the accumulated
patterns rather than searching for new gaps).

## TWO ADDITIONAL EASY WINS

### 5. HostileAuditGate (improves loop quality)

Location: new file `aidens-autonomous/src/hostile_audit.rs`
Lines: ~180

PURPOSE: The loop currently accepts facts based on a content quality
score (evaluation.rs). But it never cross-checks with a different model.
The Codex Super-Pass Protocol uses hostile audits as the verification
gate — the autonomous loop should too.

DESIGN:

```rust
/// A hostile audit cross-checks a captured fact using a different LLM
/// than the one that generated it. If the auditor disagrees or finds
/// problems, the fact is downgraded from Promote to Quarantine.
pub struct HostileAuditGate {
    /// Auditor provider URL (different from executor's Ollama).
    auditor_url: String,
    /// Auditor model name (should be different from executor's model).
    auditor_model: String,
    /// HTTP client.
    client: reqwest::Client,
}

impl HostileAuditGate {
    pub fn new(auditor_url: &str, auditor_model: &str) -> Self;

    /// Audit a captured fact. Returns true if the fact survives audit.
    pub async fn audit(
        &self,
        claim: &str,
        context: &str,
    ) -> Result<AuditResult>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditResult {
    /// Whether the fact survived audit.
    pub survived: bool,
    /// Auditor's assessment.
    pub assessment: String,
    /// Specific issues found (if any).
    pub issues: Vec<String>,
    /// Confidence score from auditor (0.0-1.0).
    pub confidence: f64,
}
```

The audit prompt is simple:
"You are a hostile auditor. Given this claim and its context, find
every reason it might be wrong. If you cannot find any issues, say
SURVIVES. If you find issues, list them."

INTEGRATION: Only runs when viscosity is Strict or Frozen. When the
audit fails, the fact is downgraded to Quarantine and proof-debt is not
incurred (or is incurred at High risk class).

### 6. LoopReceiptEmitter (improves observability)

Location: new file `aidens-autonomous/src/receipt.rs`
Lines: ~120

PURPOSE: The loop currently tracks state in LoopState but doesn't emit
typed receipts per cycle. The RecursiveIntell doctrine requires
receipt-bearing operations. Every loop cycle should produce a receipt.

DESIGN:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReceiptV1 {
    /// Cycle number.
    pub iteration: usize,
    /// ISO-8601 timestamp.
    pub timestamp: String,
    /// Gaps detected this cycle.
    pub gaps_detected: usize,
    /// Tasks executed this cycle.
    pub tasks_executed: usize,
    /// Facts captured this cycle.
    pub facts_captured: usize,
    /// Facts rejected this cycle.
    pub facts_rejected: usize,
    /// Viscosity signal.
    pub viscosity_signal: Option<ViscositySignal>,
    /// Strictness level.
    pub strictness: StrictnessLevel,
    /// Proof-debt outstanding.
    pub proof_debt_outstanding: usize,
    /// Mode (Additive or Subtractive).
    pub mode: LoopMode,
    /// Domains explored.
    pub domains_explored: Vec<String>,
    /// Saturated domains.
    pub saturated_domains: Vec<String>,
    /// Errors (if any).
    pub errors: Vec<String>,
    /// Hash of the receipt (for chaining).
    pub receipt_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode {
    Additive,
    Subtractive,
}
```

INTEGRATION: At the end of each cycle in run(), emit a receipt and store
it in semantic-memory as a fact in the "autonomous" namespace. This
gives you a full audit trail of loop behavior over time.

## INTEGRATION SUMMARY

### Changes to AutonomousLoop struct:

```rust
pub struct AutonomousLoop {
    // ... existing fields ...
    pub viscosity: ViscosityController,
    pub proof_debt: ProofDebtBudget,
    pub entropy_search: EntropyGradientSearcher,
    pub hostile_audit: Option<HostileAuditGate>,
    pub receipt_emitter: ReceiptEmitter,
    pub mode: LoopMode,
}
```

### Changes to LoopConfig:

```rust
pub struct LoopConfig {
    // ... existing fields ...
    /// Auditor URL (for hostile audit gate). If empty, no audit.
    pub auditor_url: String,
    /// Auditor model name (should differ from ollama_model).
    pub auditor_model: String,
}
```

### Changes to LoopState:

```rust
pub struct LoopState {
    // ... existing fields ...
    /// Current viscosity signal.
    pub viscosity_signal: Option<ViscositySignal>,
    /// Current strictness level.
    pub strictness: StrictnessLevel,
    /// Current loop mode.
    pub mode: LoopMode,
    /// Outstanding proof-debt.
    pub proof_debt_outstanding: usize,
    /// Saturated domains.
    pub saturated_domains: Vec<String>,
    /// Domains explored this cycle.
    pub domains_explored: Vec<String>,
}
```

### Changes to run() method:

The modified loop cycle becomes:

1. VISCOSITY CHECK — compute signal, determine strictness
2. MODE CHECK — if subtractive mode, run subtractive cycle instead
3. GAP DETECTION (entropy-guided) — only if additive and not frozen
4. JOB ACQUISITION
5. JOB EXECUTION
6. RESULT CAPTURE
7. EVALUATION (with viscosity-adjusted thresholds)
8. HOSTILE AUDIT (if strict/frozen)
9. PROOF-DEBT UPDATE (incur on promote, pay on verify)
10. SUBTRACTIVE CHECK (if debt exceeds threshold, shift mode)
11. RECEIPT EMISSION
12. SATURATION CHECK (update domain saturation)
13. SLEEP (viscosity-adjusted: frozen sleeps longer)

### New run_subtractive_cycle() method:

```rust
async fn run_subtractive_cycle(&self) -> Result<()> {
    // 1. Run contradiction detection on outstanding claims.
    // 2. Pay proof-debt for claims with no contradictions.
    // 3. Quarantine claims with contradictions.
    // 4. Run subtraction engine on low-structuring items.
    // 5. Run compression governor re-evaluation.
    // 6. Emit subtractive cycle receipt.
    // 7. Check if debt is low enough to return to additive mode.
    if !self.proof_debt.should_shift_to_subtractive() {
        self.update_state(|s| s.mode = LoopMode::Additive);
    }
    Ok(())
}
```

## WHAT I ALSO FOUND THAT'S MISSING

### 7. detect_gaps_in_namespace in gap_detector.rs

The current gap_detector queries all priority namespaces at once. The
entropy-gradient searcher needs to target specific domains. This
requires a small addition to gap_detector.rs:

```rust
/// Detect gaps within a specific namespace only.
pub async fn detect_gaps_in_namespace(
    &self,
    limit: usize,
    attempted: &HashSet<String>,
    namespace: &str,
) -> Result<Vec<DetectedGap>>;
```

~40 lines. Extracts the namespace-filtered logic from the existing
detect_gaps method.

### 8. Contradiction check integration

The loop captures facts but doesn't run the decoder against them after
capture. The decoder exists in semantic-memory but isn't called from the
autonomous loop. Adding a post-capture contradiction check:

```rust
// After capture, check for contradictions.
let contradictions = self.check_contradictions(&capture_outcome.fact_ids).await?;
if !contradictions.is_empty() {
    // Record contradictions in proof-debt.
    for c in &contradictions {
        self.proof_debt.incur(
            &c.new_fact_id,
            &c.namespace,
            RiskClass::Medium,
        );
    }
}
```

This requires an HTTP call to the semantic-memory server's contradiction
detection endpoint. ~60 lines in loop_driver.rs.

### 9. Subtraction trigger from proof-debt

The subtraction engine in semantic-memory is computational only — it
doesn't touch SQLite. The loop needs to call it and act on results:

```rust
async fn run_subtraction(&self) -> Result<()> {
    // 1. Get structuring scores for items in the autonomous namespace.
    // 2. Feed them to the subtraction engine.
    // 3. For each candidate, apply the recommended action.
    // 4. Emit receipts.
}
```

~80 lines. Uses the existing SubtractionEngine API.

## FILE MANIFEST

New files (in aidens-autonomous/src/):
1. viscosity.rs — ~280 lines
2. proof_debt.rs — ~220 lines
3. entropy_search.rs — ~330 lines (includes saturation tracker)
4. hostile_audit.rs — ~180 lines
5. receipt.rs — ~120 lines

Modified files:
6. lib.rs — add 5 new module declarations + re-exports
7. loop_driver.rs — integrate all 5 into AutonomousLoop + run() method
8. loop_driver.rs — add run_subtractive_cycle() method (~80 lines)
9. loop_driver.rs — add check_contradictions() method (~60 lines)
10. loop_driver.rs — add run_subtraction() method (~80 lines)
11. gap_detector.rs — add detect_gaps_in_namespace() (~40 lines)

Total new code: ~1130 lines
Total modified code: ~260 lines
Grand total: ~1390 lines

## DEPENDENCY CHECK

All new modules use only dependencies already in aidens-autonomous's
Cargo.toml:
- serde / serde_json (for serialization)
- reqwest (for HTTP calls to SM server)
- chrono (for timestamps)
- uuid (for receipt ids)
- sha2 ( for receipt hashes)
- anyhow (for errors)
- tokio (for async)

No new dependencies needed.

## BUILD ORDER

1. receipt.rs (no deps, pure types)
2. proof_debt.rs (no deps, pure types + logic)
3. viscosity.rs (depends on evaluation::FactDisposition)
4. entropy_search.rs (depends on HTTP calls, standalone)
5. hostile_audit.rs (depends on reqwest, standalone)
6. gap_detector.rs modification (add namespace method)
7. loop_driver.rs integration (depends on all above)

Each step compiles independently. Each can be tested independently.

## VERIFICATION

After implementation:
```bash
cd /home/sikmindz/Coding/Libraries/AiDENs
cargo check -p aidens-autonomous
cargo test -p aidens-autonomous
cargo clippy -p aidens-autonomous -- -D warnings
```

## WHAT THIS ENABLES

The loop after these changes:

1. SELF-DIRECTING: Entropy-gradient search picks what to explore next
   based on where knowledge is most uncertain and most changing.

2. SELF-REGULATING: Viscosity controller adjusts strictness based on
   failure rate, drift, ambiguity, and contradiction density. Fast when
   succeeding, strict when struggling, frozen when failing.

3. SELF-CORRECTING: Proof-debt budget tracks unverified claims. When
   debt exceeds threshold, loop shifts to subtractive mode (verify,
   repair, compact, retire) until debt is paid down.

4. SELF-FOCUSING: Saturation tracker shifts focus away from exhausted
   domains toward productive ones. When all domains are saturated, shifts
   from exploration to implementation.

5. SELF-VERIFYING: Hostile audit gate cross-checks facts with a
   different model before promotion. Contradiction detection runs after
   every capture.

6. SELF-MAINTAINING: Subtractive cycle runs subtraction engine,
   compression governor, and contradiction repair when debt is high.

7. SELF-DOCUMENTING: Every cycle emits a typed receipt with full state
   snapshot, stored in semantic-memory for audit.

The loop still needs human input for:
- Initial domain selection (what to seed it with)
- Pattern extraction across wide domain gaps (your cognitive ability)
- External validation of claims requiring real-world evidence
- Deciding when to stop the loop entirely

But within a seeded domain, the loop runs autonomously: explore →
generate → execute → capture → verify → store → check → repair →
subtract → compact → adjust → repeat.