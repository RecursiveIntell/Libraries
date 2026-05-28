# P30 Acceptance Gates — v11B-centered

P30 does not pass because docs exist. P30 passes only if code, tests, receipts, and handoff evidence prove the claim.

## Gate 0 — Source and build honesty

- Package cleanliness is not build certification.
- Workspace path dependencies must be present and checked.
- If `cargo check/test/clippy/fmt` cannot run, the final report must say so and downgrade release claims.

## Gate 1 — v11A prerequisite gate

The pass must prove or explicitly debt/quarantine:

- material operation receipts;
- execution context envelopes or admitted equivalents;
- deterministic material artifact identity;
- proof profile / proof debt / waiver honesty;
- boundary compiler strictness;
- no material done-state without receipts.

## Gate 2 — Boundary safety gate

The harness must prove:

- malformed tool calls are rejected/quarantined, not dropped;
- parser repair emits repair/degradation/treatment-integrity receipts;
- serialization failure cannot become empty provider success;
- patch missing/unreadable input cannot be treated as empty content;
- rollback errors are durable failures;
- command execution is bounded and sandboxed enough for this pass.

## Gate 3 — v11B right-graph gate

The harness must prove:

- graph surface declarations exist for region execution;
- storage/retrieval/inference/repair/subtraction/control graph surfaces are not silently collapsed;
- storage-as-inference shortcut is rejected or degraded with proof;
- retrieval expansion cannot promote as causal evidence;
- control/receipt graph cannot become hidden truth.

## Gate 4 — v11B region protocol gate

The harness must prove:

- region contracts exist;
- boundary messages are typed;
- boundary receipts record accept/reject/quarantine;
- region state snapshot distinguishes observed, latent, nuisance, contradiction, proof, budget, and boundary state where applicable;
- replay slice exists or non-replayability reason is explicit.

## Gate 5 — v11B convergence/residual/syndrome gate

The harness must prove:

- recursive/iterative operators declare stop law;
- convergence reports emit stop reason, residual summary, exactness/degradation impact;
- oscillation/non-convergence does not look successful;
- residuals/syndromes are typed artifacts, not logs;
- unresolved severe syndromes block or degrade promotion.

## Gate 6 — v11B repair/subtraction gate

The harness must prove:

- repair candidates declare blast radius, proof obligations, rollback path, and semantic diff;
- applied repairs preserve append-plus-supersession;
- subtraction operators declare `SUBTRACTS_STRUCTURE`;
- support core, removal frontier, invariant-preservation receipt, and historical-loss budget exist where applicable;
- protected as-of queries still work or degrade according to declared budget;
- subtraction challenge path exists or release debt is explicit.

## Gate 7 — v11B causal/interventional gate

The harness must prove:

- causal/blame/attribution claims use causal bundles or are degraded/refused;
- treatment, outcome, unit, confounder/nuisance, assumptions, refuters, and evidence refs are present where causal language is used;
- proximity-only blame cannot promote as causal attribution.

## Gate 8 — Hostile audit absorption gate

Every row in the absorption matrix must be:

- fixed;
- explicitly quarantined with owner/blocker/next-pass target;
- or recorded as accepted release debt with allowed uses and release-claim downgrade.

## Gate 9 — Release claim gate

Final release label must be honest:

- `v11A-conformant-core` only if v11A gates pass.
- `v11B-draft-runtime` only if executable v11B spine exists with tests and debt ledger.
- `v11B-conformant-runtime` only if full v11B release bar passes. This is not expected unless proven.
