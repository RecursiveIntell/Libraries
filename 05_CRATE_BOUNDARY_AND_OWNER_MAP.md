# Crate boundary and owner map

| Crate | Role | Constitutional note |
|---|---|---|
| stack-ids | IDs and typed vocab roots | Keep IDs pure; add permit IDs if needed, but do not turn it into a junk drawer. |
| semantic-memory-forge | Canonical artifact families | Own new v12/vendstate family types and validators. |
| forge-memory-bridge | Export/import transform and import atomicity | Reference interpreter and differential tests for import atomicity. |
| semantic-memory | Projection and queryable truth | Must stay authoritative for projected truth; add subtraction families carefully. |
| knowledge-runtime | View/routing/planning runtime | Must surface widening and consume only admitted artifacts. |
| verification-policy | Policy evaluation and permit minting | Own `CommitToken`/`ApprovalGrant` minting and policy validity. |
| verification-control | Control receipts, proofs, and release-facing evidence | Consume permits and publish control receipts. |
| effect-runtime | Effect artifact contracts | Must gain builders/validators and stop being pure type scaffolding. |
| llm-tool-runtime | Tool execution runtime | Must consume typed grants, not raw approval strings. |
| forge-pilot | Orchestrator | May never bypass permit gating or silently widen to green. |
| kernel-execution | Regional/kernel execution | Needs failure-surface tests and eventual v12 regional artifacts. |
| kernel-oracles | Refutation/oracle slices | Needs stronger failure-surface tests and integration with proof ladders. |
| federated-settlement | Treaty/settlement bounded evaluator | Needs v16 closure on quorum/challenge/publication. |
| mechanism-runtime | Theory/mechanism bounded evaluator | Needs v17 closure on publication/dispute/retirement. |
| spec-execution | Constitution self-hosting helper surfaces | Needs end-to-end admission/veto/challenge closure. |

## Boundary truth

- `semantic-memory-forge` already owns key artifact families such as `EpisodeBundleV1` and `ExecutionContextV1`.
- `forge-memory-bridge` already proves at least one hard invariant (missing canonical `episode_id` fails on the bundle lane).
- `verification-policy` must become the owner of runtime permits, not just boolean policy decisions.
- `llm-tool-runtime` must consume typed grants, not raw strings.
- `effect-runtime` must stop being a schema-only island.

## Naming rule

If a crate is a **surface crate**, say so loudly.  
If a crate is a **runtime**, it must execute law, not just serialize nouns.
