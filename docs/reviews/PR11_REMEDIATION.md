# Libraries PR #11: local review and repair

Source: RecursiveIntell/Libraries main `2c63bc4c423ad90fa3603eef685c9d23ef8c049f`, inspected September 7, 2026. PR #11 merged September 6. The unrelated Ares PR #11 merged August 30.

Scope: all 17 Codex review threads. This follow-up publishes the 16 valid source repairs. Runtime activation is a separate gate after merge and downstream integration.

## Finding dispositions

Thread URLs use `https://github.com/RecursiveIntell/Libraries/pull/11#discussion_r<comment-id>`.

| Comment ID | Priority | Surface / root cause | Current verdict and repair | Acceptance |
|---|---|---|---|---|
| 3944968207 | P1 | evidence_join: empty evidence falls through to Pass | Valid; empty joins return Unsupported | Empty branches and empty evidence never pass |
| 3944968210 | P1 | remote_work: attempt-supplied publication key and input-order winner | Valid; require owner-issued logical-work key, reject mixed identities, deterministic selection | Reversed order gives same settlement; separate attempts publish at most once; alternate keys rejected |
| 3944968211 | P1 | applicability: missing source only changes own presence | Valid; invalidate descendants and inspect ancestor presence, including reconstructed absent sources | Descendants revalidate; unrelated work reusable; restoring bytes alone does not clear invalidation |
| 3944968215 | P1 | maintained_audit: one pair gets zero-width interval | Valid; fewer than two pairs is NotIdentified | Single pair cannot produce promotable estimate |
| 3944968217 | P1 | forge_experiment: staging and trial evidence ignored | Valid; require passed staging and completed, usable, uncontaminated passing trials | Failed/pending staging and absent/failed/incomplete trials block publication |
| 3944968221 | P2 | evidence_actions: deferred flag survives safe-mode exit | Valid; release deferral and require fresh owner reconciliation | No dispatch before reconciliation; retained queue dispatches afterward |
| 3944968224 | P1 | applicability: dependency basis ignored during construction | Valid; reject mismatched edge basis; recheck ancestor basis on evaluation | Stale edges rejected; reconstructed stale projections cannot reuse |
| 3944968227 | P1 | profile-runtime: validity strings not parsed or ordered | Valid; parse RFC3339 and reject expired/inverted windows | Malformed and expired fail; exact boundary and equivalent timezone offsets accepted |
| 3944968229 | P1 | semantic-memory: msg/message prefixes differ | Latent inconsistency, reported admitted-message failure unreachable here; NO source change | Default governed search excludes messages; absent-origin evaluation also denies them |
| 3944968233 | P1 | consumer_bridge: returned fact identity unchecked | Valid; typed identity mismatch rejection | Grant for A cannot project owner-returned B |
| 3944968235 | P1 | lifecycle: retained replay restrictions discarded | Valid; conservative restriction aggregation | Forbidden and reconciliation restrictions never weakened |
| 3944968236 | P2 | recipe_optimizer: candidate baseline not compared | Valid; exclude mismatched baseline before qualification/evaluation/execution | No trial execution or qualification-based promotion |
| 3944968238 | P1 | lifecycle: effect starts before durable attempt admission | Valid; required owner begin_effect contract, exclusive durable claim, reconciliation of existing attempts | Child process exits after effect; fresh coordinator/owner reconciles without second effect; failed admission invokes nothing |
| 3944968239 | P1 | maintained_audit: successful records can exceed controls | Valid; validate success against cost/time ceilings before aggregation and revalidate supplied aggregates | Over-budget success rejected; exact ceilings accepted |
| 3944968241 | P1 | forge_experiment: booleans promote causal hypothesis without effect evidence | Valid; use existing AblationEvidence/evaluate_ablation path | Refuted or environment-confounded intervention cannot support diagnosis |
| 3944968244 | P2 | forge_experiment: publication uses immutable original base | Valid; publication requires explicit current-base input | Rebased candidate passes only with current base and fresh staging/trials/interaction checks |
| 3944968247 | P2 | operator_projection: sequence collision silently discarded | Valid; retain distinct events deterministically and mark Unknown | Opposite arrival orders retain both events, same projection; exact duplicates still deduplicate |

The 16 valid findings were confirmed statically against executable source before edits. They affect admission, support, or publication in these library surfaces; this is not proof that each was exercised by a deployed Ares path. Confidence in the unreachable message finding is bounded to the current source: `search_governed_with_view` selects default source types, which exclude messages; it also calls `decide(..., None, ...)` for every non-fact source, and `evaluate_governed_access_v1` rejects absent origin. Enabling governed messages in future must reconcile identities and origin ownership together.

## Execution contract

Canonical owners remain: profile-runtime for the task policy projection; semantic-memory for origin authority; injected native owners for effects, durable attempts, containment, grants, and publication; Agent Graph for orchestration and rebuildable applicability/operator projections. No new truth database, native effect implementation, authority fallback, or compatibility shim was added.

Phases: (1) inspect all findings and current owners; (2) repair projection/identity/invalidation gates; (3) repair lifecycle and experiment boundaries; (4) run regression, package, format/lint and independent source review. Tests use disposable local fixtures. Stop on owner drift, unpartitionable changes, or failed mandatory gates. Original checkout was clean.

Rollback: discard this isolated candidate checkout or reverse the final patch against the pinned base. No live data or services require rollback. Keep the review and test receipts when rejecting a candidate; do not resolve GitHub threads based solely on this local result.

## API and compatibility impact

- `RemoteOwnerPort::canonical_publication_key` is required. The external owner must supply the same stable key for a logical work item across calls; `publish_once` retains atomic publication ownership. A deduplicated settlement does not claim the losing attempt's result was selected.
- `EffectOwner::begin_effect` is required. Implementors must atomically bind the full request to its idempotency key and acknowledge only after durable persistence. Existing/completed attempts return AlreadyStarted, conflicting bindings reject, unavailable persistence fails closed. The filesystem test owner is test-only, not a production adapter. Live owner durability remains an integration obligation.
- `ForgeExperiment::publish_selected` requires the observed current base explicitly, which is also checked by native authorization. The experiment's original base remains historical evidence.
- `record_paired_intervention` accepts the existing `AblationEvidence` instead of two booleans. This is evidence supplied by the caller, not independently verified scientific proof.
- `record_staging` explicitly updates the candidate staging projection. Passing staging never replaces native containment, trials, interaction checks, or authorization.
- New typed rejection variants may require downstream exhaustive-match updates. All in-repository callers were searched and updated. Out-of-repository consumers are not compile-certified.
- Policy validation uses the existing workspace chrono dependency; Cargo.lock adds only that dependency edge. The schema remains unchanged. Missing or malformed expiry now returns a typed projection error instead of accepting placeholder timestamp text.

Validation: 857 tests passed, zero failed, five ignored with `cargo test -p agent-graph -p profile-runtime -p semantic-memory --features semantic-memory/testing --locked -j 2`. Strict Clippy for agent-graph/profile-runtime all targets, changed-file rustfmt, public API docs, production-panic checks and patch forward/reverse checks passed. Independent review found no surviving original defect. A passing local suite does not certify production activation, scientific interval calibration, or out-of-repository implementations of owner ports.
