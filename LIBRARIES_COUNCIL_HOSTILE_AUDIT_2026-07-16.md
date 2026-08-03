# Libraries Council Hostile Audit Closeout

Date: 2026-07-16

> **Remediation update (2026-07-16):** The actionable source findings in this
> audit were implemented and the full workspace test and formatting lanes now
> pass. See `LIBRARIES_COUNCIL_REMEDIATION_PLAN_2026-07-16.md` for the executed
> plan and verification. This audit's original no-go remains historically
> correct for the audited snapshot; a fresh release record is still deferred
> because the shared worktree contains unrelated changes.

## Scope and conclusion

This was a read-only audit of the current `Coding/Libraries` tree after the
AiDENs closeout. The tree was already materially dirty, so no source or
evidence artifact was changed and no historic receipt was treated as current.

**Conclusion: NO-GO for a workspace release or current-state readiness claim.**
There are two independent P0 blockers: the root workspace does not compile and
the release-evidence verifier cannot bind the existing closeout to the current
source.

The root `AGENTS.md` requires sequential implementation, so this closeout uses
one evidence pass rather than the draft plan's multi-agent fan-out.

## Validation receipts

| Command | Result | Interpretation |
|---|---|---|
| `cargo test --workspace --all-targets --no-fail-fast` | Failed while compiling `llm-pipeline` | A workspace/CI release lane is blocked by `E0282` at `llm-pipeline/src/pipeline.rs:363`. |
| `cargo test -p agent-graph -p semantic-memory -p claim-ledger -p job-queue -p ai-batch-queue -p forge-pilot -p verification-control -p verification-policy --all-targets --no-fail-fast` | Passed | Focused core tests passed; they do not cover failing checkpoint-store behavior. |
| `cargo fmt --all -- --check` | Failed | CI-required formatting is not satisfied in modified `llm-pipeline` and `semantic-memory` files. |
| `make gate` | Failed closed | Existing evidence is stale/inconsistent: snapshot mismatch, captured-at mismatch, gate-result mismatch, and missing `source_binding`. |
| `git diff --check` | Failed | The already-dirty AiDENs generated ownership CSVs contain trailing whitespace, so the worktree is not clean-closeout ready. |

The all-workspace clippy/doc lanes were not run after the compile gate failed.

## Severity-ranked findings

| ID | Severity | Finding | Evidence | Required remediation |
|---|---:|---|---|---|
| LBR-001 | P0 | The workspace cannot compile. | `llm-pipeline/src/pipeline.rs:363` invokes generic `PayloadOutput::parse_as()` without constraining its result type; Rust reports `E0282`. The root CI runs `cargo test --workspace --all-targets --all-features`. | Restore type inference explicitly (the value is the pipeline's `T`), add a regression compile/test for `execute_streaming`, format, then rerun root fmt/clippy/test/doc lanes. |
| LBR-002 | P0 | The release gate has no current, source-bound proof. | `make gate` reports snapshot/capture/gate-result mismatch and missing `source_binding`; `STATUS_EVIDENCE_MANIFEST.json` is 2026-05-13 while `release/closeout_receipt_v1.json` is 2026-03-30. | Do not edit evidence to force consistency. After the tree is clean and all gates pass, use the explicit recorder, regenerate the derivative closeout receipt from that manifest, and verify read-only. |
| LBR-003 | P1 | Configuring a checkpoint store does not guarantee a receipt-bearing run. | `agent-graph/src/graph.rs:89-100` replaces `create_run` failure with a UUID. `agent-graph/src/engine.rs:264-274`, `372`, `465`, `496`, and `975-1021` drop run, snapshot, and attempt persistence errors while execution can still return success. Existing tests cover successful stores but not a failing store. | Make configured checkpoint persistence a typed failure (or an explicit, surfaced degraded mode that cannot be reported as receipt-backed success). Add failing-store tests for run creation, attempt recording, snapshot persistence, and terminal status persistence. |
| LBR-004 | P1 | Formatting is red in files that are part of the current runtime changes. | `cargo fmt --all -- --check` reports diffs in `llm-pipeline/src/pipeline.rs`, `llm-pipeline/src/tool_loop.rs`, and `semantic-memory/src/config.rs`. | Run the formatter only after reconciling ownership of the dirty files, then keep fmt in the same bounded change set as LBR-001. |
| LBR-005 | P2 | Run/job identities still bypass the canonical-ID owner. | `ai-batch-queue/src/queue.rs:73-74` carries an explicit TODO and emits `Uuid` strings. `agent-graph/src/graph.rs:91-99`, `checkpoint_store.rs:253,281`, and `job-queue/src/types.rs:142` likewise use raw UUID strings despite the declared stack-ID lineage model. | Freeze the ID contract first: distinguish external/storage IDs from canonical run/attempt/trial IDs, then migrate through typed `stack-ids` constructors with serialization-compatibility fixtures. Do not make a mechanical UUID replacement. |
| LBR-006 | P2 | The current worktree cannot qualify for a clean closeout. | `git diff --check` reports trailing whitespace in pre-existing modified `AiDENs/docs/contract-ownership/*_TYPE_INVENTORY.csv` artifacts. | Regenerate or normalize the owning inventory output in its own scoped change; do not bulk-edit unrelated dirty artifacts as part of the runtime fixes. |

## Recommended order

1. Fix and format LBR-001; rerun the full CI-equivalent compile/test lane.
2. Close LBR-003 before treating graph results as receipt-bearing or restarting MCP adoption work.
3. Once the tree is clean and green, record fresh release evidence and verify LBR-002.
4. Resolve LBR-006 in its owning generated-artifact change, then schedule LBR-005 as a contract migration with compatibility tests, not a release-evidence patch.

## Non-findings

The targeted core suite passed, including the existing graph interruption,
queue lineage, semantic-memory integrity/import, claim-ledger, and verification
control tests. This does not override the two P0 blockers or certify any
installed MCP/service runtime.
