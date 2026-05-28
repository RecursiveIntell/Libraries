# Phase Plan

Each phase must produce a `PHASE_<NN>_REPORT.md` using `templates/PHASE_REPORT_TEMPLATE.md`.

## Phase 00 — Source basis, triage, labels, and no-regression frame

**Intent:** Reconcile package-clean fact; normalize issues; classify fixed/quarantined/deferred/open-blocking; do not mark the finished bundle as failed.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** All source-basis docs generated; backlog has statuses; forbidden claims listed.

## Phase 01 — Receipt/log durability and no done without receipts

**Intent:** Make all material operations emit durable receipt artifacts before visible done state; add hash-chain verification, corruption quarantine, file locking/single-writer discipline.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Tests prove final output cannot exist without durable receipt; concurrent append cannot fork chain; corrupt logs quarantine.

## Phase 02 — Security boundary and sandbox hostile corpus

**Intent:** Harden path policy, secret-path denial, symlink/hardlink/TOCTOU/unicode/case-folding/hidden metadata handling.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Hostile sandbox fixtures pass; deny/quarantine receipts emitted.

## Phase 03 — Tool exposure and permit parity

**Intent:** Reduce default tool exposure; ensure disabled tools are unreachable; bind descriptor risk to permit policy.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Default safe plan excludes admin-risk tools; disabled tool routing fails with receipt.

## Phase 04 — Transactional patch engine and treatment integrity

**Intent:** Replace patch_apply narrow string replacement or relabel it; add real transactional patch subsystem, before/after digests, rollback/quarantine.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Multi-file patch atomicity tests; repeated-content hunk tests; read failure cannot create/replace silently.

## Phase 05 — Command execution receipts and environment control

**Intent:** Structured argv; process-group kill; stdout/stderr caps; env/toolchain/package fingerprints; replay handles.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Quoted args, grandchild timeout, output cap, PATH drift fixtures pass.

## Phase 06 — Provider honesty and local route discipline

**Intent:** Local must not mean mock; provider/tool results routed honestly; native vs fallback exactness disclosed; network permits enforced.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Local unavailable does not silently mock; Ollama sees tool results or is explicitly no-tools/degraded.

## Phase 07 — Queue, scheduler, daemon concurrency

**Intent:** Lock/single-writer queue log; race-free idempotency/leasing/completion; safe-mode quarantine; queue-hop receipts.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Concurrent enqueue/lease/complete tests pass; late completion after TTL rejected.

## Phase 08 — Boundary compiler, JSON, schema, and repair

**Intent:** Strict boundary profiles; full schema validation or explicit unsupported-feature rejection; default material repair reject/quarantine; treatment-critical checks.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Duplicate keys, fenced JSON, unknown fields, schema mismatch, critical-field repair fixtures pass.

## Phase 09 — Bitemporal/proof/view semantic reference corpus

**Intent:** Reference interpreter for valid/recorded time, retroactive correction, supersession, stale projections, view widening, proof debt/refutation.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Reference fixture corpus passes; degraded answer cannot masquerade as exact.

## Phase 10 — Minimal v11B regional recursive/subtractive slice

**Intent:** Right-graph declaration; one region contract; convergence/non-convergence/oscillation; syndrome/residual; local repair; support core; oracle diff.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** One deterministic v11B vertical slice passes; v11B-complete remains forbidden.

## Phase 11 — Schema governance and generated artifacts

**Intent:** Generate schemas from canonical types; meta-validate; digest schemas; compatibility diff gates; reject unsupported schema features.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Schema gen/diff/meta-validation gate passes.

## Phase 12 — Artifact lifecycle and operator effect enforcement

**Intent:** Material-operation registry; operator effect declarations; proof profile/debt enforcement; terminal-state budget consumption.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** All material operations have contracts/effects/receipts; proof waiver cannot promote as proof.

## Phase 13 — Module decomposition and canonical ownership

**Intent:** Split large crates/files; enforce owner boundaries; add boundary scanner to verify gate.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Mega-file budgets enforced; owner scanner passes; no shadow truth owners.

## Phase 14 — Replace marker tests with semantic hostile fixtures

**Intent:** Upgrade marker assertion scripts into behavioral tests or retire them; add adversarial fixtures.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Verifier refuses marker-only completion for hard gates.

## Phase 15 — Docs, evidence, known limitations, and label closure

**Intent:** Populate known limitations; final auditor handoff; classify all issues; update support traceability and forbidden labels.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Final docs reconcile with actual package and test evidence.

## Phase 16 — Config, environment, secrets, and redaction

**Intent:** Harden config validation; redact provider/tool/log secrets; record environment fingerprints.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Secret values never appear in receipts/logs; config mismatch degrades with receipt.

## Phase 17 — App/scaffold/profile readiness

**Intent:** Harden generated app/profile templates; atomic scaffold writes; no unsafe defaults.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Scaffold interruption/overwrite/secret tests pass.

## Phase 18 — Search, pool, HNSW, and semantic-memory risks from Claude audit

**Intent:** Fix HNSW TOCTOU/atomic ordering/ID recycling posture; vector scan circuit breaker; timestamp parsing; pool timeout handling.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** HNSW concurrency tests; vector scan hard-block/degrade; parse warnings; pool error fixtures.

## Phase 19 — Unaudited high-risk layers quarantine/audit

**Intent:** Audit or quarantine forge-pilot, effect-runtime, verification pipeline, federation, attestation, authority-delegation, recursive-kernel-core.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Each high-risk layer is fixed, quarantined, or explicitly out-of-scope with guard tests.

## Phase 20 — Final package, extracted replay, and release bar

**Intent:** Run full command bar; generate package sidecars; extracted-package replay; update final labels only if gates pass.

**Required work:**

- Query `matrices/SUPER_PASS_BACKLOG_1020.csv` for `Suggested_Phase` containing this phase number/title.
- Fix or quarantine all P0/Critical/High rows in scope.
- Add semantic/hostile tests, not just marker asserts.
- Update row statuses and notes.
- Produce phase report.

**Exit gate:** Clean package sidecars; extracted replay passes; labels truthful.

