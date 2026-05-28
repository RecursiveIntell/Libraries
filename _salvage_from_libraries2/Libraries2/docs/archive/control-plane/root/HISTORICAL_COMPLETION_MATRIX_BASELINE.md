# Brutally Honest Completion Matrix

## Scope used for scoring

This matrix is based on four inputs:

1. **Live control plane** in the repo:
   - `AGENTS.md`
   - `CODEX_ACCEPTANCE_CHECKLIST_V6.md`
   - `CODEX_EXECUTION_MAP_V6.md`
   - `CODEX_KEEP_TRIM_REVERT_V6.md`
   - `MASTER_ISSUE_MATRIX_CODEX_V6.md`
   - `FILE_AUDIT_INVENTORY_CODEX_V6.md`
2. **Canonical architecture / research docs**:
   - `CANONICAL_STACK_SPEC_V5.md`
   - `LATEST7.md`
   - `priority_matrix_research.md`
   - `bitemp+_research.md`
   - `causal-research.md`
   - `deep-research-report.md`
3. **Actual code snapshot** from `libraries_source_20260308_233103_clean.zip`
4. **Static review limits**: no Rust toolchain in this environment, so this is a source audit, not a build/test execution audit.

## The blunt verdict

- **Core Codex V6 closure:** about **75% complete**
- **Broader research roadmap:** about **35–40% complete**
- **Reality check:** the architecture is strong, but the library stack is **not** “done except for polish.”

The most important update is this: **the code is ahead of parts of the V6 issue matrix**. Several items that the V6 matrix still marks open are now substantially implemented in the snapshot. The biggest remaining hole is no longer “can imported projections be queried?” It is **“does Forge export rich enough structure to make the whole design intellectually worth the trouble?”**

---

## Score legend

- **90–100** = essentially landed
- **75–89** = mostly landed, still bounded or incomplete
- **45–74** = partial / meaningful gap remains
- **15–44** = early / thin / not good enough
- **Unknown** = not enough evidence to score hard

---

## Core target-state closure matrix

| Area | Score | Verdict | Why |
|---|---:|---|---|
| Authority boundaries / ownership map | 95 | Landed | The architectural split is stable and matches the intended target state. |
| Import-path atomicity + idempotency | 90 | Landed | Canonical batch import is the real path; import logs and duplicate detection are in place. |
| Bitemporal import law (`recorded_at` in store, not bridge) | 95 | Landed | The importer now stamps authoritative `recorded_at`/`imported_at`; bridge only carries provenance timestamps. |
| Public projection query APIs in `semantic-memory` | 95 | Landed | Claims, relations, episodes, aliases, and evidence refs all have public query methods now. |
| Runtime retrieval from imported projections | 90 | Mostly landed | `knowledge-runtime` actually uses projection-backed retrieval; imported retrieval proofs no longer need side-loaded facts. |
| Temporal execution on supported projection routes | 80 | Mostly landed | Real `valid_at` / `recorded_at_or_before` filtering exists, but temporal expression support is still narrow. |
| Full-scope enforcement on supported projection routes | 85 | Mostly landed | Domain/workspace/repo filters are pushed down for projection queries; fallback routes still warn or fail. |
| Evidence opacity in normal retrieval | 95 | Landed | Normal query results do not expose evidence refs as first-class hits or leak raw handles. |
| Version-aware claim lineage | 60 | Partial | Schema + bridge can carry `supersedes_claim_version_id`, but Forge export still does not populate it meaningfully. |
| Forge export richness | 40 | Weakest major gap | `living-memory` still emits a thin envelope: one synthetic claim, one episode, one evidence ref. |
| Causal projection consumption in runtime | 70 | Partial-to-mostly | Runtime can consume imported episode projections, but it is limited by the thin export surface. |
| Bounded entity candidate expansion | 85 | Mostly landed | Imported alias candidates are used for bounded fuzzy recovery. |
| Derivation / invalidation breadth | 45 | Partial | Derivation edges exist, but current coverage is still narrow. |
| Compatibility fencing | 65 | Partial | Legacy surfaces are visibly labeled compatibility-only, but they are still public and still loud. |
| Docs / control-plane accuracy | 50 | Partial and messy | Root V6 docs are good law, but the repo still contains too many historical docs and some V6 status docs are already stale. |

### Approximate rollup

Average across the core rows above: **~75/100**

That is **good progress**, not **done**.

---

## Why the V6 issue matrix is now partially stale

The V6 issue matrix still says several core gaps are open. In this snapshot, that is no longer fully true.

### Items the matrix still marks as open that are now largely implemented

#### BRG-101 — authoritative `recorded_at` in the importing store
Code now shows the importer assigning a single `imported_at` timestamp to all imported rows and to the import log:

- `semantic-memory/src/lib.rs:3396-3435`
- `forge-memory-bridge/src/batch.rs:26-31`
- `forge-memory-bridge/src/transform.rs:423-428`

#### SM-101 — public projection read APIs
Public query methods now exist:

- `semantic-memory/src/lib.rs:3547-3589`
- `semantic-memory/src/types.rs:121-164`
- `semantic-memory/src/types.rs:165-266`

#### KR-101 — runtime retrieval from imported projections
Projection-backed retrieval is actually wired in:

- `knowledge-runtime/src/adapters/semantic_memory.rs:33-95`
- `knowledge-runtime/src/runtime.rs:362-487`

And the proof path no longer needs side-loaded facts for the imported-projection case:

- `knowledge-runtime/tests/cross_crate_proof.rs:1107-1284`

#### KR-102 — real temporal execution on supported routes
Projection temporal queries now set both valid-time and recorded-time cutoffs:

- `knowledge-runtime/src/adapters/semantic_memory.rs:62-86`
- `semantic-memory/src/projection_storage.rs:780-907`
- `knowledge-runtime/tests/cross_crate_proof.rs:837-886`

#### KR-103 — full-scope enforcement on supported routes
Projection queries filter on namespace + domain + workspace + repo:

- `semantic-memory/src/projection_storage.rs:780-907`
- `semantic-memory/src/projection_storage.rs:1026-1268`
- `knowledge-runtime/tests/cross_crate_proof.rs:1286-1381`

### Items that are still genuinely open

#### LIV-101 — richer Forge export rendering
The export path in `living-memory` is still thin:

- `living-memory/living-memory/src/export.rs:84-184`

It emits:
- one `Claim`
- one `Episode`
- one `EvidenceRef`

But not a richer relation/entity/claim-version lineage story from the available bundle structure.

#### SMF-101 — version-aware claim supersession lineage, end-to-end
The schema and bridge can preserve real version lineage:

- `forge-memory-bridge/src/transform.rs:24-27`
- `forge-memory-bridge/src/transform.rs:476-518`
- `forge-memory-bridge/tests/forge_bridge_memory_proof.rs:231-255`

But the Forge exporter still emits `supersedes_claim_version_id: None`:

- `living-memory/living-memory/src/export.rs:107-110`

#### SM-102 — broader derivation edges / bounded recomputation
Current derivation insertion is still narrow:

- `semantic-memory/src/lib.rs:3462-3480`

Right now the main durable edge inserted during import is evidence-ref -> claim / claim-version. That is useful, but not enough for the fuller projection ecosystem the research points toward.

---

## The `AGENTS.md` situation: useful law, slightly outdated diagnosis

The new root `AGENTS.md` is the right high-level law source. It correctly freezes the authority boundaries and the completion criteria.

But its **status diagnosis** is already a little behind the code:

What `AGENTS.md` still says is missing:
- projection retrieval
- temporal/scope semantics
- richer Forge export
- compatibility/doc drift

What the code now shows:
- projection retrieval is **mostly solved**
- temporal and full-scope semantics are **substantially solved on supported projection routes**
- richer Forge export is **still a real blocker**
- compatibility/doc drift is **still messy**

So `AGENTS.md` is still good as **policy**, but not perfect as a **status dashboard**.

Key files:
- `AGENTS.md:8-25`
- `AGENTS.md:58-91`
- `AGENTS.md:163-176`

---

## The real blockers that keep this from being “complete”

### 1. Forge export is still semantically underpowered
This is the most important unfinished piece.

The rest of the stack can now store, query, and route richer projection types. But the main Forge export path still does not emit enough structure to fully exercise that design.

That means the system risks becoming:

> “beautiful canonical ingestion of semantically thin envelopes.”

That is not what the architecture is aiming for.

### 2. End-to-end version lineage is not truly closed
The bridge correctly refuses to invent lineage. Good.
But that also means your end-to-end version semantics are only as strong as the exporter. Right now the exporter is not feeding enough real version lineage into the lane.

### 3. Derivation / invalidation semantics are still shallow
You have the beginnings of bounded recomputation. You do not yet have the richer derivation graph that will matter once the causal/export side gets more expressive.

### 4. The control plane is too noisy
The repo still has:
- many historical V2–V5 docs
- multiple `LATEST*.md` files
- multiple matrix generations
- multiple prompt/control docs
- a V6 issue matrix that already lags the code in several rows

That is not just annoying. It is how future implementation passes get steered wrong.

---

## Research-aligned completion matrix

This is the tougher score. Here I am not grading only “does Codex V6 close the current lane?” I am grading against the broader research thesis in the uploaded documents.

| Research theme | Score | Honest read |
|---|---:|---|
| Evidence substrate + boundary authority | 80 | Strong progress; this is the most mature part. |
| Bitemporal storage / as-of semantics | 75 | Real substrate exists, but not every retrieval path is equally rich. |
| Multi-view runtime retrieval (semantic + temporal + entity + causal) | 60 | Semantic/temporal/entity are meaningfully present; causal is still thin. |
| Opaque evidence + explicit audit discipline | 90 | This is one of the best-aligned parts. |
| Entity resolution as integrity constraint | 70 | Bounded alias expansion exists; long-term identity integrity work is not “finished.” |
| Causal attribution with model / identify / estimate / refute discipline | 20 | Mostly research direction, not delivered system behavior yet. |
| Refutation hooks / falsification-first verification | 20 | Conceptual and document-level, not yet a clearly landed productized subsystem in this snapshot. |
| Verification economics / scheduler policy / deadline propagation | 30 | Some queue/retry infrastructure exists, but the research-grade economics layer is not the present center of gravity. |
| Trace / retry lineage as epistemic control | 55 | Promising in the secondary lane, but not deeply audited end-to-end in this pass. |
| Mechanistic / code-ML / SAE / GNN roadmap | 5 | Research inventory, not current implementation. |

### Broader research rollup

If I grade against the full ambition implied by the research docs, the stack is only around **35–40% complete**.

That is not an insult. It is just the difference between:
- **architecturally serious**
and
- **research-program-complete**

Those are worlds apart.

---

## Secondary execution lane: promising, not fully scored

I did not deeply re-audit the secondary lane because the V6 file audit explicitly scopes it as secondary after core closure.

What I can say from a light scan:

- `job-queue` already models `trace_ctx`, `attempt_id`, and `trial_id` explicitly:
  - `job-queue/src/types.rs:76-118`
- `agent-graph` has retry tests asserting shared `AttemptId` family with fresh `TrialId`s per concrete retry:
  - `agent-graph/tests/retry_tests.rs:165-280`
- `AI-Batch-Queue` has retry lineage tests preserving attempt family and clearing trial IDs on retry:
  - `AI-Batch-Queue/tests/integration_tests.rs:568-648`
- `Tauri-Queue` already re-exports canonical `TraceCtx`:
  - `Tauri-Queue/src/lib.rs:31-40`

That is encouraging. But I am **not** calling the secondary lane “complete” from a light scan.

---

## Final issue-ID map

### Fixed or mostly fixed in code
- **BRG-101** — fixed
- **SM-101** — fixed
- **KR-101** — mostly fixed
- **KR-102** — mostly fixed on supported routes
- **KR-103** — mostly fixed on supported routes
- **KR-105** — mostly fixed
- **DOC-102** — mostly fixed

### Partial
- **SMF-101**
- **KR-104**
- **SM-102**
- **SM-103**
- **SM-104**
- **BRG-102**
- **DOC-101**

### Still the big blocker
- **LIV-101**

---

## Bottom line

### What is genuinely impressive
- The authority map is not decorative.
- Imported projections are now a real retrieval substrate.
- Scope/temporal truthfulness is materially better than the V6 docs still imply.
- Evidence opacity discipline is good.
- The repo has crossed from “migration architecture” into “working target-state substrate.”

### What still stops me from calling it complete
- Forge export is still too thin.
- Version lineage is not really closed end-to-end.
- Derivation breadth is still limited.
- Docs/control-plane drift is still too high.
- The broader research program is still mostly ahead of the implementation.

### The brutally honest sentence
This is **not** “basically complete except for minor stuff.”

It is **a strong core substrate with several major closure wins already landed, plus one very real semantic blocker (Forge export richness) and a messy documentation/control-plane problem that will keep tripping future work unless you clean it up.**
