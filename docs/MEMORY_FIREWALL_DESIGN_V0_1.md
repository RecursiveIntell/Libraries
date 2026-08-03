# RecursiveIntell Memory Firewall v0.1

**Status:** Design target, not a release claim  
**Date:** 2026-07-16  
**Scope:** `semantic-memory` + `semantic-memory-mcp` + host kits, with governed writes explicitly deferred

## Executive decision

The Memory Firewall is the correct final integration boundary for the memory/plugin system, but it is **not** a final feature to add on top of an already-complete product. It is the product boundary that makes the existing pieces supportable and honest.

The v0.1 target should be a small local-first appliance:

```text
Claude / Codex / Hermes / other host
        |
        | bounded MCP profile; no direct database access
        v
Memory Firewall: policy + witness + provenance boundary
        |
        +--> SQLite authoritative state
        +--> exact/FTS retrieval
        +--> optional generation-bound ANN sidecar
        +--> durable receipts and replay metadata
```

**Recommendation:** freeze breadth, ship a lean read-only vertical slice first, and treat governed writes as a separate product phase. Do not make the firewall an umbrella crate or a second memory database.

## Evidence basis and confidence

| Evidence | Classification | Finding |
|---|---|---|
| `semantic-memory-mcp` source and tests at current checkout | Current executable evidence | Lean/standard expose four read-only MCP tools; agent exposes a bounded read-only surface; writes require a trusted issuer in tested paths; witnessed retrieval and opt-in replay exist. |
| `semantic-memory` source and tests | Current executable evidence | SQLite-backed facts, provenance, supersession, embedding-purpose separation, integrity/rebuild helpers, vector artifact identity, and many hostile/invariant tests exist. |
| `agent-memory-kits` and Hermes kit docs | Current integration evidence | Host integrations distinguish hooked hosts from MCP+rule hosts; canonical Hermes hook wiring is external host configuration. |
| Workspace test run | Current execution evidence | `cargo test --workspace --locked --all-targets` failed in `check-runner` with 3 environment-sensitive host-backend tests. The semantic-memory package passed its standalone suite. |
| `semantic-memory-mcp` standalone test run | Current execution evidence | `cargo test --offline --all-targets` passed: 32 unit tests, 7 binary tests, 27 integration tests. It regenerated its lockfile because the checked lock was stale; this is a release-provenance issue to resolve, not a pass for clean reproducibility. |
| 2026-07-16 execution packet/reference profile | Dated internal proposal | Defines the intended gates, exclusions, acceptance path, and nonclaims. It is not proof that those gates have been executed. |
| MCP tool-annotation guidance | External ecosystem evidence | MCP annotations provide risk vocabulary (read-only, destructive, idempotent, open-world) but are hints to clients, not an authorization system. |
| OWASP Agent Security guidance | External threat evidence | Prompt injection, tool misuse, data leakage, and memory/context poisoning are relevant threat classes. |
| Letta, Graphiti, Mem0 documentation | External comparative evidence | Adjacent systems demonstrate archival/core memory, temporal context graphs, and memory extraction/update patterns; none substitutes for this firewall's local authority and receipt contract. |

## Product contract

### Supported v0.1

- Local SQLite is the sole canonical memory state.
- FTS/exact retrieval is always available as the fallback lane.
- ANN/vector, sparse, graph, cache, and quantized artifacts are derived state only.
- MCP over stdio is required; Unix-socket transport is optional after stdio parity.
- HTTP is disabled by default. If enabled later, it is loopback/token-gated and profile-bound.
- The autonomous profile is read-only.
- Every supported retrieval returns a durable witness/receipt reference.
- Complete replay is opt-in because it retains query/filter inputs.
- Results include honest source provenance or are omitted from injection-capable responses.
- Snapshot, restore, corruption detection, sidecar deletion/rebuild, doctor, and replay are first-class operations.
- Host kits invoke the firewall; hooks cannot bypass it or write directly to SQLite.

### Explicitly excluded from v0.1

- Autonomous memory mutation or self-promotion.
- Generic agent execution, scheduling, delegation, wake/daemon/queue surfaces.
- Direct host access to the database or sidecars.
- Quantization in the default supported path.
- Enterprise/security certification, compliance, benchmark superiority, adoption, revenue, or production-support claims.
- A new umbrella crate merely to aggregate existing crates.

## Trust boundaries

1. **Host boundary:** host hooks, MCP clients, prompts, skills, packages, and retrieved text are untrusted inputs.
2. **Request boundary:** validate principal, audience, purpose, namespace/resource scope, request ID, normalized arguments, and size limits before retrieval.
3. **Authority boundary:** only trusted server-side constructors/resolvers may create authority decisions, permits, leases, or trusted claim state. Caller-carried structures are claims to validate, never authority.
4. **Canonical-state boundary:** only SQLite transactions can create or supersede canonical memory. Sidecars cannot create facts, claims, permissions, or receipts.
5. **Response boundary:** no response is reported as witnessed until all reranking, enrichment, source hydration, policy filtering, and serialization-affecting stages have completed.
6. **Effect boundary:** v0.1 has no effectful memory path. Future writes require admission, review, capability validation, durable outbox/preflight/outcome, and replay-safe idempotency.

### Fail-closed laws

- Missing, malformed, stale, revoked, or unavailable governance cannot increase authority.
- Unknown profile/tool/codec/backend is an explicit error, never a permissive fallback.
- Failure, cancellation, interruption, budget exhaustion, and corruption have typed terminal states and cannot become `Complete`.
- An unavailable codec cannot return encoded bytes under an exactness label.
- Unverified claims cannot be promoted to trusted memory.
- A receipt that does not bind the final ordered result is not a valid response witness.

## Read path

```text
validate request
  -> classify purpose and query
  -> resolve namespace/audience policy
  -> read canonical current state
  -> exact/FTS candidates
  -> optional vector/sparse candidates
  -> deterministic fusion/rerank
  -> filter superseded or unauthorized results
  -> hydrate source spans/provenance
  -> attach authority state and nondeterminism declaration
  -> persist final receipt
  -> return results + receipt reference
```

The receipt must bind at minimum:

```json
{
  "schema": "memory_firewall_witness_v1",
  "request_id": "...",
  "principal": "...",
  "purpose": "recall",
  "scope": {"namespace": "...", "domain": null, "workspace_id": null, "repo_id": null},
  "policy_digest": "...",
  "canonical_state_digest": "...",
  "index_generations": [{"kind": "fts", "generation": "..."}],
  "query_input_digest": "...",
  "ordered_result_ids": ["..."],
  "ordered_result_digest": "...",
  "source_span_digests": ["..."],
  "terminal_status": "complete",
  "nondeterminism": [],
  "created_at": "..."
}
```

`terminal_status` is a closed enum such as `complete`, `failed`, `interrupted`, `cancelled`, `corrupt`, `policy_denied`, `not_replayable`, or `backend_unavailable`. The server must finalize the receipt only after the result returned to the client is fixed.

## Minimal MCP surface

The lean profile should expose exactly:

| Tool | Effect | Purpose |
|---|---|---|
| `sm_search_witnessed` | Read | Current/hybrid/FTS/vector retrieval with durable witness |
| `sm_replay_search` | Read | Replay a stored-input witness, or return explicit non-replayable |
| `sm_decide_assertion_authority` | Read | Fixed-purpose assertion decision; never returns memory content |
| `sm_decide_action_authority` | Read | Fixed-purpose action decision; never performs the action |

The agent/operator profile may additionally expose fact hydration, receipt lookup, namespace listing, graph path, conversation search, and stats. It remains read-only until a trusted authenticated authority issuer is actually present.

Do not rely on MCP `readOnlyHint` or `destructiveHint` as the firewall. MCP annotations are useful client-facing risk metadata; enforcement belongs in the server's typed profile and policy boundary.

## Canonical schema responsibilities

### Authoritative SQLite

- facts/episodes/messages and immutable source metadata
- provenance and source spans
- bitemporal validity/recorded-time fields
- supersession/admission state
- request and response witnesses
- policy/authority decision records
- snapshot and migration metadata
- idempotency/replay records where applicable

### Derived and disposable

- FTS5 indexes
- ANN/HNSW/usearch files
- sparse/vector caches
- graph acceleration projections
- quantized candidate artifacts
- hot query caches

Every derived artifact needs: schema/profile digest, source authority generation, embedding model/version/purpose/dimensions, creation receipt, and a rebuild path. A mismatched or tampered artifact is disabled and rebuilt; it is never silently treated as canonical.

## Future governed-write path

Writes are **not** part of the v0.1 support claim. The later path is:

```text
candidate proposal
  -> operator review/admission inbox
  -> trusted Permit V2 resolution
  -> canonical argument digest + scope/purpose check
  -> durable preflight
  -> durable outbox
  -> one-shot effect dispatch
  -> authoritative SQLite transaction
  -> derived-index update
  -> durable terminal outcome
  -> response-bound receipt
  -> supersession/admission linkage
```

Permit V2 must bind subject, method, canonical arguments, scope, policy version/digest, issuer, nonce, expiry, revocation reference, idempotency key, and one-shot consumption. Crash tests must cover every boundary and prove no duplicate or unauthorized effect after retry.

## Host-kit rules

- One canonical firewall launcher and one canonical store owner per host/profile.
- Hooks call `sm_search_witnessed`; they do not call raw search or write files/database rows.
- Retrieved memory is visibly labeled as recall, not ground truth; current source and user messages outrank it.
- Injection-capable hooks must require source/provenance completeness and apply namespace/sensitivity policy.
- Hook failure behavior is a product decision: retrieval may fail open for host availability, but the firewall itself must report a typed failure in its receipt and never claim witnessed success. Effectful paths must fail closed.
- `doctor` checks binary identity, profile, config, schema, database integrity, sidecar generations, receipt persistence, token permissions, and host routing.
- Uninstall removes host wiring without deleting canonical data unless the operator explicitly requests a separately witnessed data deletion.

## Hostile test matrix

### P0 before v0.1 release

- graph error/cancellation/interruption cannot surface as complete;
- governance outage/malformed state denies effectful access;
- unknown/unavailable codec fails explicitly;
- caller-minted authority/lease/permit is rejected;
- query/document embedding purpose and geometry cannot alias cache/index identity;
- duplicate JSON keys are rejected and canonicalization is cross-language stable;
- claim verification lists executed checks and refuses promotion when checks did not run;
- final result digest changes when order, enrichment, source span, or policy changes;
- superseded facts are absent from current retrieval;
- missing source provenance prevents injection-capable result inclusion;
- corrupt/tampered sidecars are rejected and rebuildable;
- replay either reproduces the ordered result or states why it cannot.

### Recovery matrix

| Event | Required behavior |
|---|---|
| clean restart | canonical facts and receipt index reopen |
| process crash during retrieval | no false complete receipt |
| SQLite snapshot restore | restore authority, invalidate incompatible sidecars |
| sidecar deletion | exact/FTS path remains usable; rebuild produces new generation |
| sidecar corruption | disable sidecar, emit diagnostic, rebuild from SQLite |
| interrupted compaction/migration | prior authoritative generation remains usable |
| duplicate request | same idempotent witness, no duplicate write |
| malformed request | bounded typed error; no state change |

## Release gate

A tagged v0.1 release is acceptable only when all are true:

1. canonical branch, tag, source manifest, package, and installed binary identify one commit;
2. release tree is clean and package replay is reproducible;
3. clean-machine runbook completes install → ingest → witnessed retrieve → source inspection → sidecar delete/rebuild → replay → uninstall;
4. lean `tools/list` exactly matches the declared allowlist;
5. SQLite authority and derived-state recovery matrix passes;
6. hostile P0 suite passes;
7. no unauthenticated mutation path exists;
8. every public claim maps to a current receipt, current source/test evidence, or an explicit research/proposal label;
9. full workspace release gate is green, or every blocked/environment-sensitive check is explicitly recorded as blocked and the release is withheld.

## Research comparison

| System | Useful adjacent idea | Boundary for comparison |
|---|---|---|
| Letta | Separates always-visible core memory from out-of-context archival memory and lets agents manage memory through tools. | Its model is agent-managed memory; this design emphasizes a local authority boundary, provenance, witnessed retrieval, and fail-closed admission. Do not claim superiority without identical evaluation. |
| Graphiti/Zep | Temporal context graphs, incremental updates, provenance, and evolving relationships. | Strong comparison for temporal/graph retrieval; it does not establish this design's SQLite/receipt/replay properties. |
| Mem0 | Extract/update memory workflows and hybrid search patterns. | Useful product comparison for memory lifecycle; its hosted/self-hosted modes and extraction behavior are not evidence for this local firewall's guarantees. |
| MCP annotations | Standard risk vocabulary for tool behavior. | Hints help clients choose/confirm tools; annotations are not authorization, capability issuance, or audit proof. |
| OWASP agent security | Threat taxonomy including prompt injection, tool misuse, leakage, and memory poisoning. | Use it to structure hostile fixtures; passing internal tests is not external security certification. |

## Strategic conclusion

The firewall is likely the highest-leverage final integration step because it collapses many existing capabilities into one defensible product promise: **an agent may recall local memory, but only through a bounded, provenance-bearing, replayable boundary; canonical changes are reviewable and governed rather than silently authored by the agent.**

It is not the final step if “final” means every memory feature is complete. It is the final step for making the memory system coherent, distributable, and safe to claim.

The correct build order is:

1. resolve canonical source/release truth;
2. close remaining P0 false-success and provenance gaps;
3. ship read-only firewall vertical slice;
4. prove recovery and third-party reproduction;
5. only then implement governed writes;
6. publish scoped benchmarks and external reproduction.

Until those gates pass, the honest status is: **substantial implementation with a strong firewall target and a working read-only prototype, not a completed production memory appliance.**

## Sources

- Local: `/tmp/recursiveintell_dossier_2026-07-16/RecursiveIntell_Memory_Firewall_Execution_Packet_2026-07-16.md`
- Local: `/tmp/recursiveintell_dossier_2026-07-16/recursiveintell_memory_firewall_reference_profile_v1.json`
- Local: `/home/sikmindz/Coding/Libraries/semantic-memory/README.md`
- Local: `/home/sikmindz/Coding/Libraries/semantic-memory-mcp/README.md`
- Local: `/home/sikmindz/Coding/agent-memory-kits/README.md`
- MCP tool annotations: https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/
- OWASP AI Agent Security: https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html
- Letta archival memory: https://docs.letta.com/guides/ade/archival-memory/
- Graphiti: https://github.com/getzep/graphiti
- Mem0: https://github.com/mem0ai/mem0
