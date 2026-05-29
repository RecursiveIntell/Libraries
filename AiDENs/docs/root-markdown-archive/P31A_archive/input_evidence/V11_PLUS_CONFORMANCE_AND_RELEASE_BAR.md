# v11+ Conformance and Release Bar

**Status:** Companion release/conformance plan  
**Purpose:** Translate the v11A/B/C spec split into implementable gates.

This document is intentionally blunt: a stack that looks architecturally aligned but cannot pass conformance fixtures is not v11+ compliant. Elegant prose is not a release artifact. Receipts or it did not happen.

---

## 1. Release strategy

v11+ SHOULD be implemented in three staged bars:

| Stage | Name | Main question |
|---|---|---|
| v11A | Constitutional Artifact Runtime Core | Can the stack represent material work as typed, receipt-bearing artifact transitions under semantic/proof/boundary law? |
| v11B | Recursive/Subtractive Regional Runtime | Can the stack execute regional recursive/subtractive work with right-graph declarations, convergence law, repair, causal packages, and lawful subtraction? |
| v11C | Self-Hosting/Federated/Mechanism/Agency Horizon | Can the stack reserve and selectively activate future admission surfaces without creating shadow authority? |

---

## 2. Build order

### 2.1 v11A implementation order

1. Artifact identifiers and envelope contracts.
2. Execution context envelope and operator invocation receipt.
3. Material-operation registry and operator effect declarations.
4. Boundary compiler profiles for current import/export/tool-output surfaces.
5. Proof profile, proof obligation set, proof debt ledger, and proof waiver receipts.
6. Semantic product carriers for claim-like state.
7. Bitemporal query reference interpreter and fixtures.
8. View disclosure and degradation record surfaces.
9. Schema compatibility and canonicalization gates.
10. First production path differentially checked against reference behavior.

### 2.2 v11B implementation order

1. Graph surface declarations.
2. Region contract and boundary messages/receipts.
3. Region replay slice and state snapshot.
4. Residual and syndrome envelopes.
5. Convergence report with stop/damping/oscillation law.
6. Local repair candidates and repair execution receipts.
7. Support core and removal frontier.
8. Invariant preservation receipt and historical-loss budget.
9. Delta envelopes and invalidation cones.
10. Causal attribution bundle and intervention/counterfactual replay plan.
11. Oracle slice result and approximate-vs-exact semantic diff.

### 2.3 v11C implementation order

1. Reserve artifact family names and schema namespaces.
2. Add explicit external artifact quarantine default.
3. Add human veto/challenge receipt for spec/governance changes.
4. Add agency-risk classification for user-facing consequential outputs if enabled.
5. Add attested artifact envelope and admission policy stubs.
6. Add mechanism/theory artifact schemas as inactive/advisory surfaces.
7. Add spec bundle/normative AST prototypes.
8. Add generated conformance corpus prototype.
9. Activate individual C surfaces only after reference/policy gates exist.

---

## 3. v11A conformance gates

### 3.1 Artifact lifecycle gate

The test harness MUST verify:

- every material artifact has a family, version, identity, digest/canonical ref, recorded time, authority class, and lifecycle state;
- state transitions produce transition records or receipts;
- promotion cannot occur from non-eligible states;
- supersession preserves history;
- retired/subtracted artifacts remain queryable according to history budget.

### 3.2 Material operation gate

For a sample of production operations, the harness MUST verify:

- operator contract exists;
- effect set is declared;
- input manifest exists;
- output manifest exists;
- execution context exists;
- invocation receipt exists;
- forbidden effects are not performed;
- degradation/proof debt is emitted where required.

### 3.3 Execution evidence gate

The harness MUST verify receipts for:

- tool calls;
- retries;
- deadline/timeouts;
- queue/message hops where present;
- provider/tool route;
- replay/non-replayability;
- budget exhaustion;
- user-visible done states.

A material done state without receipts is release-blocking.

### 3.4 Boundary compiler gate

For every structured boundary, the harness MUST test:

- strict syntax validation;
- schema validation;
- duplicate-key/ambiguous-field policy;
- unknown field policy;
- canonicalization;
- resource ceilings;
- repair receipt emission;
- treatment-integrity receipt emission;
- malformed input rejection/quarantine.

### 3.5 Bitemporal reference gate

A reference interpreter MUST cover:

- valid-time only queries;
- recorded-time only queries;
- `as_of(valid, recorded)` combined queries;
- retroactive corrections;
- supersession;
- conflicting recorded histories;
- projection/index stale-state behavior;
- degraded query disclosure.

Production path MUST be differentially tested against this reference.

### 3.6 Proof economy gate

The harness MUST verify:

- risk-bearing artifacts have proof profiles;
- missing proof creates proof debt or waiver;
- proof waiver is not treated as proof;
- proof debt restricts allowed use;
- refuted artifacts cannot promote unless repaired/superseded;
- proof eligibility rules are enforced.

### 3.7 View disclosure gate

The harness MUST verify:

- retrieval/query answers disclose view family;
- widening emits `ViewDisclosureV1`;
- guarantee weakening emits `DegradationRecordV1`;
- degraded answers cannot masquerade as exact answers;
- user/audit surfaces can inspect degradation.

### 3.8 Schema/contract gate

The harness MUST verify:

- versioned artifact schemas exist;
- schema meta-validation passes;
- compatibility checks run;
- incompatible changes are blocked or migration-gated;
- canonicalization/digest behavior is stable;
- schema identity is content-addressed or otherwise stable.

---

## 4. v11B conformance gates

### 4.1 Right-graph gate

The harness MUST create tests where:

- storage graph shortcut would produce wrong inference;
- retrieval graph expansion would imply false causal evidence;
- repair graph would overreach into truth state;
- subtraction graph would remove load-bearing support;
- control graph would accidentally become authoritative.

A conforming implementation must detect/disclose/reject the misuse.

### 4.2 Region protocol gate

The harness MUST verify:

- region contract exists;
- graph surfaces are declared;
- incoming/outgoing artifact families are declared;
- boundary messages are typed;
- boundary receipts record acceptance/rejection/quarantine;
- replay slice exists;
- state snapshot distinguishes observed, latent, nuisance, contradiction, proof, budget, and boundary state.

### 4.3 Convergence gate

For recursive/iterative operators, the harness MUST test:

- normal convergence;
- non-convergence;
- oscillation;
- budget exhaustion;
- residual above threshold;
- oracle escalation;
- degradation on failed convergence.

The output must not claim stability when convergence failed.

### 4.4 Residual/syndrome gate

The harness MUST verify:

- violated constraints emit syndrome envelopes;
- unresolved numeric/semantic residuals emit residual envelopes;
- syndromes link to affected artifacts and regions;
- promotion is blocked or degraded where unresolved syndrome severity requires;
- local repair is attempted before global rebuild where localizable.

### 4.5 Repair gate

The harness MUST verify:

- repair candidate bundle exists;
- blast radius is declared;
- proof obligations are attached;
- treatment-integrity impact is tracked;
- applied repair emits before/after semantic diff;
- rollback handle exists;
- append-plus-supersession is preserved.

### 4.6 Subtraction gate

The harness MUST verify:

- subtraction operators declare `SUBTRACTS_STRUCTURE`;
- support core exists for protected targets;
- removal frontier exists;
- invariant preservation receipt exists;
- historical-loss budget exists when history/query fidelity weakens;
- protected as-of queries still work or degrade according to budget;
- subtraction challenge path works.

### 4.7 Delta/incremental gate

The harness MUST verify:

- deltas carry valid and recorded time;
- invalidation cones are emitted;
- affected regions recompute, quarantine, degrade, or prove unaffectedness;
- retroactive corrections preserve belief-history semantics;
- blind global rebuilds are disclosed if used.

### 4.8 Causal/interventional gate

The harness MUST verify:

- causal claims use causal bundles;
- treatments/outcomes/units are defined;
- confounders/nuisance variables are declared or degradation is emitted;
- refuters are listed and run where required;
- counterfactual/baseline replay plans exist for high-risk claims;
- proximity-only blame cannot promote as causal attribution.

### 4.9 Oracle slice gate

The harness MUST verify:

- exact/conservative oracle slices can run on bounded cases;
- approximate-vs-oracle semantic diff is emitted;
- oracle failures block or degrade promotion where required;
- exactness class is visible.

---

## 5. v11C conformance gates

### 5.1 Reservation gate

The harness MUST verify that implementation does not create incompatible shadow families for:

- spec bundles;
- normative ASTs;
- external admission;
- attested artifacts;
- cross-runtime equivalence;
- treaties/settlement;
- mechanisms/theories;
- agency/influence.

### 5.2 External admission gate

If external artifacts are used, the harness MUST verify:

- external artifacts default to quarantine/rejection without admission;
- admission policy exists;
- trust root or source policy exists;
- schema/digest/attestation checks run where required;
- external artifacts cannot directly write local truth;
- revocation/dispute path exists if activated.

### 5.3 Federation/equivalence gate

If federation is enabled, the harness MUST verify:

- equivalence is explicit and typed;
- remote claims do not collapse into local claims by text/embedding alone;
- dissent is preserved;
- count-only quorum is insufficient;
- local authority remains local;
- settlement has replay and challenge path.

### 5.4 Mechanism/theory gate

If mechanism/theory artifacts are enabled, the harness MUST verify:

- mechanism bundles have assumptions and evidence refs;
- simulator contracts define replay and validation;
- fit runs emit receipts;
- observational equivalence is not treated as identification;
- refuter suites are required before action-bearing use;
- theory supersession is append-plus-supersession.

### 5.5 Agency preservation gate

If user-facing personalized or consequential advice/action is enabled, the harness MUST verify:

- influence classification exists for triggering outputs;
- advice envelope includes evidence, uncertainty, alternatives, and reversibility;
- memory influence trace exists when memory materially shapes advice;
- persuasion/repetition budgets are enforced where declared;
- false urgency/manipulative personalization is blocked;
- agency receipts are emitted.

### 5.6 Self-hosting gate

If self-hosting/spec-generation is enabled, the harness MUST verify:

- generated schemas/tests/interpreters are traceable to spec clauses;
- generated artifacts do not self-admit;
- human veto/challenge exists;
- amendment simulations produce impact reports;
- generated law cannot rewrite authority without governance.

---

## 6. Acceptance matrix

| Capability | v11A | v11B | v11C |
|---|---:|---:|---:|
| Artifact envelopes/lifecycle | Required | Required | Required |
| Execution receipts | Required | Required | Required |
| Operator effects | Required | Required | Required |
| Proof economy | Required | Required | Required |
| Boundary compiler | Required | Required for region messages | Required for external/spec artifacts |
| Reference interpreters | Required for core surfaces | Required for region/repair/subtraction | Required when activated |
| Right-graph law | Hook only | Required | Extended to federation/mechanism/spec graphs |
| Regions | Hook only | Required | Extended if federated/multi-runtime |
| Convergence governance | Hook only | Required | Required for mechanism/search if activated |
| Lawful subtraction | Hook/effect only | Required | Required for generated specs/history compaction |
| Causal/interventional claims | Hook/proof profile | Required for causal claims | Extended to mechanisms/federation |
| External admission | Reserved | Reserved | Required if external artifacts influence decisions |
| Mechanism/theory | Reserved | Reserved | Required if mechanism search activated |
| Agency preservation | Reserved effect | Reserved effect | Required if consequential personalization/action enabled |
| Constitutional self-hosting | Reserved | Reserved | Required if spec generation/compilation activated |

---

## 7. Minimum issue-matrix columns

Every v11+ implementation issue SHOULD include:

- spec section;
- artifact families affected;
- owner crate/module;
- authority class;
- required schema work;
- required runtime work;
- required reference/conformance work;
- proof obligations;
- execution receipts affected;
- migration impact;
- release gate;
- test fixture IDs;
- known debt/waivers;
- rollback plan.

---

## 8. Failure modes to test deliberately

The test suite SHOULD include adversarial and edge cases for:

- duplicate JSON keys;
- schema version mismatch;
- parser repair changing treatment;
- bitemporal retroactive correction;
- stale projection answering current query;
- retry storm producing contaminated evidence;
- queue hop losing parent lineage;
- model output claiming unsupported completion;
- storage graph used as inference graph;
- loopy region oscillating;
- repair causing wider contradiction;
- subtraction deleting support core;
- causal attribution without confounder controls;
- external artifact with valid schema but untrusted provenance;
- remote equivalence false positive;
- mechanism fitting data but failing invariance;
- personalized advice exploiting memory without disclosure;
- generated conformance test contradicting human-admitted law.

---

## 9. Release labels

Recommended labels:

| Label | Meaning |
|---|---|
| `v11A-draft` | Schemas/prototypes exist; no release claim. |
| `v11A-conformant-core` | A gates satisfied on declared surfaces. |
| `v11B-draft` | Region/subtraction prototypes exist. |
| `v11B-conformant-runtime` | B gates satisfied on declared region/runtime surfaces. |
| `v11C-reserved` | Horizon hooks reserved; no incompatible shadows. |
| `v11C-active:<surface>` | Named C surface activated and conformance-gated. |
| `v11plus-release-candidate` | A + B complete; C reserved; no release-blocking divergence. |

---

## 10. Brutal release rule

Do not ship a v11+ claim because the architecture “looks right.”

Ship it only when the stack can show:

- the artifact;
- the receipt;
- the proof profile;
- the execution context;
- the boundary/compiler record;
- the time coordinates;
- the view disclosure;
- the conformance run;
- the debt/waiver if incomplete;
- and the challenge path.

Otherwise it is not v11+. It is v10 wearing a lab coat.
