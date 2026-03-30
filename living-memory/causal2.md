# Verification-First Causal Systems for Code Change: Deep Research Synthesis for Forge

## Problem framing: from “patch attempt” to audited causal claim
A verification-aware Forge wants something stronger than “the patch fixed it (trust me).” The research question is how to encode a **patch attempt** as a *causal intervention* with durable, replayable evidence—so that every attribution edge comes with (a) the assumptions you made, (b) the data you observed, (c) the estimate you computed, and (d) refutation attempts that tried to break your story. This “causal claims are falsifiable artifacts” posture aligns cleanly with the design of **entity["organization","DoWhy","causal inference library"]**, which explicitly frames causal analysis as **model → identify → estimate → refute**, and treats refutation/robustness checks as a core step rather than a decorative appendix. citeturn8view0turn25view0

To map this into your UPDATE_SPEC vocabulary—treatment = edit/patch; outcome = failure signature/metric drift; confounders = environment drift, test selection, adjacent edits, flags, workload—you can use the standard causal inference abstraction of **units**, **treatments**, and **potential outcomes**, including Average Treatment Effects (ATE) as a target estimand when appropriate. citeturn26view0turn3view0

Two adjacent research threads matter here:

- Causal inference is increasingly discussed in software contexts (including fault localization and diagnosis), but is still often applied informally (e.g., “causality graphs” without rigorous identification/refutation). citeturn27view0  
- Production debugging settings already motivate causal reasoning on observational telemetry (logs) because engineers usually can’t fully verify code correctness and must reason backwards from failures. citeturn3view0

The endgame: Forge edges should stop being vibes and start being **evidence bundles** that can survive audit, replay, and later contradiction.

## Verification-aware causal inference for code changes
### Formalizing the causal object without “DAG worship”
The key is to define a causal object that is **operational**. DoWhy’s paper is explicit that analysts can provide **partial causal graphs** and that causal modeling is about making assumptions explicit and testable, not about drawing a perfect universe map. citeturn8view0

A practical minimal formalization for Forge looks like this:

- **Unit (causal instance)**: a test run, benchmark run, canary slice, or replay episode (choose one and stay consistent).
- **Treatment**: “patch applied” vs “baseline” (binary), or a categorical treatment for multiple candidate patches.
- **Outcome**: (a) pass/fail with structured failure signature; (b) numeric drift (latency, error rate, memory); or (c) composite score.
- **Measured confounders (covariates)**: environment fingerprint, dependency versions, config flags, workload/hash, test selection, and “adjacent edits included.” These match the confounder list you gave; the research problem is how to measure and control them tightly enough to make causal claims plausible.

This maps straight onto standard potential-outcomes notation (treatment assignment \(W_i\), observed outcome \(Y_i\), counterfactual outcomes \(Y_i(1), Y_i(0)\)) and ATE-style questions. citeturn26view0

### Baseline-vs-patched experiments as the backbone, not a bolt-on
Your instinct—**paired baseline-vs-patched trials**—is exactly how you stop causal attribution from becoming interpretive dance. A particularly relevant analogue is canary analysis: you expose only part of production to the change (“canary”) and compare against the remainder (“control”) to decide whether to proceed. citeturn22view0

What matters for causal robustness is not just “we ran both,” but the discipline of:

- **Paired trials on the same workload slice** (or as close as you can get).
- **Repeated trials** to estimate variance and fragility.
- **Stratification** by major covariates (e.g., workload class, environment family).
- **Contamination control**: the SRE literature is blunt that overlapping canaries increase risk of signal contamination, and that before/after comparisons are noisy because time itself is a huge source of metric variation. citeturn22view0

This is directly aligned with your confounders list. “Time drift” and “workload drift” aren’t theoretical—they’re the default state of reality.

### Refutation as a first-class verification ladder
DoWhy is unusually explicit about the epistemic stance: observational causal analyses **cannot be proved correct**, but can be **refuted** via robustness tests; failing a refutation means the analysis (or assumptions) need repair. citeturn25view0

For Forge, the best move is to treat refutations as *verification tasks* with escalating cost—your “cheap verification ladder.” DoWhy’s refuters are particularly translatable because they are essentially “nullifying checks”:

- **Placebo treatment**: replace the treatment with an independent random variable; the estimated effect should go to ~0. citeturn25view1  
  *Forge translation*: apply a *placebo patch* (syntactic no-op, or patch applied to irrelevant files) or permute patch assignment labels across runs; if you still “find” a strong effect, your pipeline is hallucinating causality.

- **Random common cause**: add an independent random variable as a (fake) confounder; the estimate should not materially change. citeturn25view2  
  *Forge translation*: inject random “covariates” into the analysis; large estimate shifts suggest overfitting or instability in the estimator/feature set.

- **Data subsampling / bootstrap validation** (in DoWhy’s broader refuter set) checks estimator stability to resampling. citeturn8view0turn25view0  
  *Forge translation*: rerun estimates on subsets of trials; if your causal story flips based on which 20 runs you kept, the edge shouldn’t be promoted to “trusted.”

The meta-point: refutations aren’t a report appendix; they become **stored artifacts** that can later be revisited when you ingest new evidence.

### Evidence bundles: what should be stored per causal edge
If you want forge-audit and semantic-memory to store something real, the evidence bundle should have a stable schema. Based on DoWhy’s step structure and the needs of reproducibility in telemetry + trials, a bundle typically needs:

- **Causal question** (“effect of patch P on outcome O under constraints C”)
- **Unit definition** (what counts as one observation)
- **Treatment specification** (patch ID, content hash, application mode)
- **Outcome specification** (signature hash, metric definition, time window)
- **Covariates/confounders recorded** (environment fingerprint, flags, workload/test IDs)
- **Identification rationale** (why you think confounding is controlled; what is assumed)
- **Estimator and estimate** (including uncertainty intervals where possible)
- **Refutations attempted + results** (placebo, ablation, subsample stability)
- **Raw receipts**: trial logs/metrics pointers, trace IDs, replay handles

This “bundle” perspective is also consistent with production-causal-debugging work: the LOGos paper frames causal inference over logs as a way to quantify effects and assess interventions, acknowledging that production diagnosis frequently runs on imperfect observational data and requires careful modeling. citeturn3view0

## End-to-end trace propagation and observability invariants
### Why trace propagation is not optional
Distributed tracing only works if you can correlate work across boundaries. The **entity["organization","OpenTelemetry","observability project"]** docs are explicit that **context propagation** is the concept that enables distributed tracing and correlation of traces/metrics/logs across services and process/network boundaries; without it, traces fragment and “causal information about a system” can’t be assembled coherently. citeturn9view0

image_group{"layout":"carousel","aspect_ratio":"16:9","query":["OpenTelemetry context propagation diagram distributed tracing","W3C traceparent tracestate header diagram","distributed tracing span links diagram","OpenTelemetry baggage propagation illustration"],"num_per_query":1}

### The standards substrate: W3C Trace Context + baggage
OpenTelemetry’s default propagator uses the headers specified by the **entity["organization","World Wide Web Consortium","web standards body"]** Trace Context specification. citeturn9view0turn10view0 The spec design is explicitly split into:

- `traceparent`: fixed-length portable format describing the position of the incoming request in the trace graph (and must be properly set). citeturn10view0  
- `tracestate`: optional vendor-specific key/value extensions that travel alongside traceparent. citeturn10view0

OpenTelemetry also supports **Baggage**: arbitrary key/value pairs propagated alongside context, with explicit warnings not to put sensitive information there because baggage can cross trust boundaries and be logged or forwarded downstream. citeturn9view0turn23view1

### Observability invariants for Forge-style evidence
To make “one patch attempt, one experiment, one evidence bundle, one episode” line up, you need invariants that are enforceable in code reviews and testable in CI:

- **Every attempt has a root trace**: a stable `attempt_id` (also stored as baggage) that appears in every span/log/metric emitted by the attempt.
- **Every trial has a stable trial identity**: `trial_id`, plus `baseline_or_patch` tag, plus `patch_hash`.
- **Every measurement has a scope boundary**: time window + workload ID + environment fingerprint, so outcome comparisons aren’t accidentally cross-contaminated.
- **Every refutation is trace-linked**: refutation trials must link back to their parent estimate and to the same patch + workload cohort.

OpenTelemetry’s semantics even call out that context propagation enables logs to be correlated with traces by injecting trace/span IDs into log records, and similarly enables richer metric aggregation “in that context.” citeturn9view0

### Deadline propagation and retry semantics: make the causal timeline sane
If you want replayable receipts, you also need consistent semantics for “how long did we try” and “what counts as the same attempt.”

Deadlines are a canonical way to bound work in distributed systems. gRPC’s deadline guide explicitly notes that propagating absolute deadlines is problematic under clock skew; gRPC therefore converts deadlines to **timeouts** with elapsed time deducted when forwarding, shielding systems from unsynchronized clocks. citeturn11view0 This matters for Forge because your verification ladder will otherwise generate misleading “this did/didn’t finish in time” stories when retries hop across machines.

Retries need their own observability semantics. The gRPC retry guide is clear that:
- There is no default retry policy; without one, gRPC can’t safely retry most RPCs. citeturn12view0  
- Even without a retry policy, gRPC may perform “transparent retries” in limited circumstances. citeturn12view0  
- Retry behavior involves saving call history and potentially replaying it on a new attempt. citeturn12view0  

For Forge, the causal implication is straightforward: a “verification attempt” is not a single span; it’s often a *family* of attempts. If you don’t model retries explicitly, you’ll attribute “patch caused latency spike” when the real cause was “retry storm + deadline compression.”

OpenTelemetry semantic conventions for messaging spans describe how **link relationships** (as opposed to strict parent/child) can be used to correlate producer/consumer spans in scenarios where a span can only have a single parent, and where work happens in a different ambient context. citeturn13view0 The same mental model applies to retries, queued jobs, and replay: use links to encode “influenced by” relationships without pretending the runtime was a neat call stack.

### Checkpointing / replay as audit primitives
When you need *hard* replay (not “best-effort reproduce”), there are two relevant tool families:

- **Process/container checkpoint-restore**: CRIU can freeze a running container or application, checkpoint its state, and later restore it to run as it was at freeze time (with explicit caveats about what can change / what cannot be checkpointed). citeturn23view1  
- **Record/replay debugging**: rr records Linux user-space process groups by capturing nondeterministic inputs and can replay while preserving instruction-level control flow and memory/register contents, enabling deterministic debugging and repeated replay of the same failing execution. citeturn23view0  

This isn’t just “debugger nerd stuff.” It is exactly what “replayable receipts” means when someone challenges an attribution edge six weeks later.

## Temporal and episodic memory schemas with stable identity
### Why bitemporality is the right default for “episodes”
Your own notes point to this: contradictions should be handled by **versioning, not deletion**, and episodes need validity intervals plus provenance.

Temporal database research draws a sharp distinction between:
- **Valid time**: when a fact is true in the modeled reality. citeturn14view0turn15view0  
- **Transaction time**: when a fact was stored/recorded in the database; transaction time is append-only in the “can’t change the past” sense. citeturn15view0turn14view0  

A “bitemporal relation” is defined (in the temporal DB glossary literature) as having exactly one system-supported valid time and exactly one system-supported transaction time. citeturn15view0 The classic temporal database taxonomy explicitly argues that supporting both valid and transaction time lets you represent retroactive/postactive changes and query “what was believed then,” not just “what we believe now.” citeturn14view0

This maps directly to Forge evidence:

- Valid time: “this causal claim held for patch hash H on commit range R under workload W between timestamps …”
- Transaction time: “we ingested this evidence bundle on date …; later we ingested a refutation result that weakened it.”

A systematic review protocol on temporal data models in knowledge graphs likewise highlights that timestamps are typically used both for valid time and transaction time, and that capturing both matters for accurate decision-making in evolving data. citeturn17view0

### Stable identity: immutable IDs for episodes, attempts, and entities
A practical outcome of the above is that every stored object needs a stable identity:

- `episode_id`: immutable; never reused.
- `attempt_id`: stable per patch attempt; must align across traces and stored artifacts.
- `artifact_id`: hashes for patch text, binaries, test outputs, dataset snapshots.
- `entity_id`: stable resolved identity for “the thing this is about” (file, module, service, endpoint, test suite, metric definition, failure signature type).

This “IDs first” approach is not theoretical: systems explicitly built for agent memory have adopted bitemporal models to separate event chronology from ingestion chronology. For example, the Zep temporal knowledge graph paper describes a bi-temporal model with separate event-time ordering vs transactional ingestion ordering to support correct handling of time-relative statements. citeturn1search10

### Entity resolution is not optional if you ingest multi-source episodes
If Forge evidence arrives from multiple pipelines (CI, canary, fuzzing harness, manual reports), you will get aliasing: the same service called by multiple names, the same failure signature with multiple formats, etc.

Entity resolution research frames this as “record linkage / entity resolution”: identifying which records refer to the same real-world entity across sources. citeturn16view0 Modern surveys emphasize that deterministic approaches are simple and scalable, while probabilistic approaches (including Fellegi–Sunter descendants and Bayesian variants) better handle noisy data and uncertainty—exactly your situation if you want semantic-memory to carry confidence and reliability rather than a single brittle “truth.” citeturn16view0

A useful Forge-specific framing is: entity resolution is not an ML feature; it’s an integrity constraint. If entity identity is unstable, your episodic + causal views will diverge into parallel inconsistent universes.

## Execution economics and scheduler policy for verification workloads
### The uncomfortable truth: rigor that doesn’t fit the budget becomes folklore
If verification plans are too expensive, they won’t be run, and your causal edges will quietly rot into “we think.” The scheduler is therefore a *scientific instrument*, not a plumbing detail.

Queueing theory gives you a blunt tool to reason about throughput vs latency: **Little’s Law** (\(L = \lambda W\)) relates average number-in-system \(L\), arrival rate \(\lambda\), and average time-in-system \(W\). citeturn19view0 In Forge terms:
- \(L\) = number of ongoing verification trials (WIP)
- \(\lambda\) = incoming patch attempts per unit time
- \(W\) = end-to-end verification latency

If you let WIP grow without bound, latency will follow. No amount of “but we’re being rigorous” will make the wall clock cooperate.

### Backpressure and fairness: borrow from proven overload controllers
A concrete, production-grade pattern is found in **entity["organization","Kubernetes","container orchestration project"]** API Priority and Fairness (APF). APF is designed for overload control: it classifies and isolates requests, introduces a limited amount of queuing for brief bursts, and dispatches from queues using fair queuing to prevent one flow from starving others. citeturn18view0

Even if Forge doesn’t run on a Kubernetes API server, the policy principles transfer almost 1:1:

- Partition verification work into priority classes (e.g., “safety checks,” “cheap refutations,” “expensive canary,” “long fuzz campaigns”).
- Guarantee minimum capacity to each class to prevent starvation.
- Use fair-queuing within a class to prevent a single repo/service from monopolizing resources.

### Deadline-driven escalation and cancellation
Deadlines are not just for RPCs; they are for verification economics. The gRPC deadlines guide emphasizes that deadlines improve utilization and latency: clients don’t wait forever, and servers know when to stop. citeturn11view0 It also emphasizes **deadline propagation** so downstream calls honor the upstream bound. citeturn11view0

Forge can treat “verification plan budget” as a deadline that propagates through:
- test runner calls,
- benchmark runners,
- canary controllers,
- log queries and causal estimators.

If a plan is canceled, the system must stop spawned activity (mirroring gRPC’s point that the server application is responsible for stopping work it spawned). citeturn11view0

### Checkpointing and resume to avoid wasting expensive work
For expensive verification (long fuzzing, long benchmarks), checkpointing can change the economics. CRIU’s core value proposition is freezing a running container/app and restoring later, enabling snapshots and migration. citeturn23view1 This can support a “pause/resume” model for verification campaigns if your workloads are compatible with checkpoint/restore.

Similarly, rr-style record/replay offers a different kind of “resume”: once you’ve recorded a failing run, you can replay it deterministically rather than re-paying the cost of re-triggering the failure. citeturn23view0

### Retry semantics must be priced in, not hand-waved
Retries change both cost and inference. The gRPC retry guide is explicit: retry behavior involves creating new attempts after backoff, and even “transparent retry” can happen without policy. citeturn12view0

Forge should therefore treat retries as:
- a cost center (they consume budget),
- a confounder (they change timing and load),
- and an observability requirement (they must be labeled and linked, or you’ll misattribute).

Also: the SRE canary guidance strongly advises running only one canary at a time in many contexts due to tracking complexity and contamination risk. citeturn22view0 That’s an execution-economics constraint disguised as “process advice.”

## Fuzzing and property-based robustness for patching and invariants
### Why this matters: safety layers must be boringly correct
Your notes call out parser reliability and patch application as “least sexy, most profitable.” That’s correct: if patch application can silently misapply edits or escape directories, your whole evidence pipeline becomes untrustworthy—because the “treatment” wasn’t what you claimed.

### Rust fuzzing: cargo-fuzz + structure-aware fuzzing
The Rust fuzzing guidance is unambiguous: the **Rust Fuzz Book** states that `cargo-fuzz` is the recommended tool for fuzz testing Rust code, and that it invokes a fuzzing backend (currently libFuzzer via `libfuzzer-sys`). citeturn20view0

Structure-aware fuzzing is particularly relevant to patching systems because you want to generate *valid-ish* structured inputs (patch hunks, file paths, manifests) rather than only raw bytes. The Rust Fuzz Book describes how `libfuzzer-sys` can fuzz targets over arbitrary structured types as long as they implement `Arbitrary`, enabling well-formed instance generation. citeturn20view1

### Property-based testing: specify invariants and let the generator hunt
Property-based testing research (e.g., QuickCheck) frames testing as checking **properties** over many randomly generated cases; properties act as executable specifications, and generators can be customized. citeturn21view0 This is directly aligned with your need to enforce invariants like:

- Patch application is **idempotent** (reapplying yields same tree state).
- Applying and then reverting yields original state (when a reverse patch is defined).
- Patch application never writes outside workspace root (no path traversal/symlink escape).
- Parser + printer round-trips preserve semantics for supported formats.

In Rust, `proptest` is a commonly used property-testing framework; its docs describe it as a property testing framework in the QuickCheck family and emphasize automatic generation plus shrinking/minimization of failing cases. citeturn6search6turn6search3

There is also active research on combining coverage-guided fuzzing with property-based testing (coverage-guided, property-based testing) to get both “smart exploration” and “semantic oracles.” citeturn6search15

### How fuzzing plugs into causal verification
Fuzzing outcomes can become **negative evidence** and also **confidence boosters**:

- If fuzzing finds that patch application misbehaves on certain path shapes, then any causal claim that relied on a patch application in that region gets its confidence downgraded (the treatment integrity is compromised).
- If fuzzing plus property tests establish strong invariants for the patcher and parsers, causal claims become more defensible because the intervention is well-defined.

In other words: fuzzing doesn’t just harden Forge; it hardens the epistemology of Forge.

## Learned graph scoring after verified edges exist
### Why “learned scoring” is real—if and only if labels are real
Once you have verified edges (your evidence bundles plus refutation results become labels), learned scoring becomes useful for ranking hypotheses, prioritizing verification, and propagating trust/suspicion across neighborhoods.

Graph neural networks (GNNs) are built around **message passing**: node representations are updated by aggregating information from neighbors via permutation-invariant operations (sum/mean/max) and learnable message/update functions. citeturn24view0turn7search4turn7search1

**PyTorch Geometric**’s documentation explicitly provides a `MessagePassing` base class that operationalizes this design: users define message and update functions and choose aggregation, while the framework handles propagation. citeturn24view0

### The label-noise problem: why you’ll “train a smarter liar” with weak edges
The caution you wrote is well-supported by the literature: GNN performance is sensitive to label quality, and label noise is a well-recognized problem because message passing can propagate incorrect information during training. citeturn7search2turn7search19

This is exactly your “don’t prioritize first” argument in technical terms: if initial edge labels are mostly conjecture, a learned model will confidently amplify conjecture.

### A Forge-appropriate path once verified edges exist
A research-backed and pragmatic approach is:

- Start with transparent scoring: rules + calibrated heuristics whose inputs are the evidence bundle fields (refutation pass/fail, effect size stability, environment match quality).
- Use learned graph scoring only after you have:
  - enough verified edges,
  - consistent episode/entity identity,
  - and observability invariants that ensure trace/evidence alignment.

At that point, learned models can help with:
- **Prioritization**: which candidate patches to test next given limited budget.
- **Risk scoring**: which changes are likely to cause regressions under certain environments.
- **Trust propagation**: if a patch touched modules historically associated with fragile outcomes, rank it higher for verification.

The hard requirement remains: your graph must be built on trace-aligned, bitemporal, identity-stable evidence—otherwise the model learns your instrumentation bugs and calls it “insight.”