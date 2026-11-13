# Cross-Repository Semantic Reconciliation Plan

**Status:** proposal accompanying Semantic Law V1

**Date:** 2026-09-27

**Scope:** 60 repositories inventoried through GitHub metadata; selected source
surfaces inspected. This is not a source audit of all 60 repositories.

## Goal

Reduce semantic drift by making three things explicit and enforceable:

1. one semantic owner per concept family;
2. an explicit source relationship for each duplicated location and one writer
   per shared semantic family, without turning independent implementations into mirrors;
3. one shared vocabulary for identity, authority, receipts, time, replay, evidence, and outcomes.

This plan deliberately separates agreement from migration. The proposal branch changes documentation only.

## Current findings

### Critical — duplicated active source has no consistent direction

The same active packages exist in multiple editable repositories and are not byte-identical.

Observed examples:

| Package | Copy A | Copy B | Manifest state |
|---|---|---|---|
| stack-ids | Libraries/stack-ids | standalone stack-ids | 0.1.3 vs 0.1.1 |
| boundary-compiler | Libraries/boundary-compiler | standalone boundary-compiler | 0.1.0 vs 0.1.1 |
| bitemporal-runtime | Libraries/bitemporal-runtime | standalone bitemporal-runtime | both 0.1.0, source differs |
| semantic-memory | Libraries/semantic-memory | standalone semantic-memory | both 0.5.15, source differs |
| semantic-memory-forge | Libraries/semantic-memory-forge | standalone | both 0.1.1, source differs |
| forge-memory-bridge | Libraries/forge-memory-bridge | standalone | both 0.1.1, source differs |
| llm-pipeline | Libraries/llm-pipeline | standalone | 0.3.0 vs 0.2.1 |
| turbo-quant | Libraries/turbo-quant | standalone | 0.2.0 vs 0.2.3 |
| fib-quant | Libraries/fib-quant | standalone | both 0.1.0-alpha.1, source differs |

Same-name/same-version source divergence is the highest-risk form because version checks cannot reveal it.

### High — canonical digest semantics conflict

Current stack-ids and boundary-compiler both expose ContentDigest semantics.

- stack-ids compute_json recursively sorts JSON object keys and hashes compact serde JSON with BLAKE3.
- boundary-compiler owns RFC 8785 JCS and hashes the resulting canonical bytes.
- the newer standalone boundary-compiler also binds schema/version/domain metadata into its digest profile.

These are deterministic but not equivalent contracts. New cross-repository structured identities must not choose between them implicitly.

### High — receipt means too many things

Across the repos, receipt currently covers:

- ordinary hash-bound records;
- HMAC-authenticated execution records;
- signed transition/control records;
- search/retrieval records;
- benchmark records;
- export records;
- hardware measurement records;
- reported outcomes.

This is workable only if the receipt family and predicate are explicit.

### High — owner and authority are overloaded

Observed meanings include:

- semantic owner;
- SQLite/state writer;
- architecture execution host;
- device/profile owner;
- operator;
- authority issuer;
- current permit holder.

New cross-repository APIs must use qualified terms.

### High — replay is overloaded

Observed meanings include:

- offline recorded-artifact replay;
- graph checkpoint resume;
- replication-journal replay;
- deterministic test rerun;
- recovery after lost acknowledgment.

Effect retry must never be hidden under replay terminology.

### Medium — identity law wording conflicts

stack-ids intentionally permits UUIDv4 opaque object IDs. Some recursive-agent guidance says random UUIDs cannot be material IDs.

Resolution: UUIDs are valid for owner-assigned object/run/record identity; content/material identity must be content-derived.

### Medium — bitemporal truth wording is stronger than the data contract

valid_time should mean the time an assertion/record is declared effective in the domain. It should not imply objective truth without stronger evidence.

The V1 wire fields do not need renaming; the documentation does.

### Medium — bitemporal receipt hashing should get a V2

Current SupersessionReceipt V1 uses SHA-256 over ad-hoc formatted strings and serializes the value with a fallback-to-empty behavior on serialization failure.

Do not mutate V1. Add a fail-closed V2 using the agreed canonical content-binding profile and retain a V1 verifier.

## Recommended source topology

Do not choose a source direction by repository name, version number, or publish
location alone. Classify each pair as a canonical-source/release-mirror relation,
intentional fork, vendored snapshot, independent implementation, stale copy, or
unknown. An unknown pair does not receive a mirror equality gate.

**Operator-selected, not yet synchronized:** Libraries is the editable source for
`semantic-memory`; the standalone repository is a one-way release mirror only
after both dirty deltas are preserved, the exact source revision is admitted,
and a directional sync PR is validated. The nested standalone Git root does not
transfer write authority to its parent or vice versa merely by location.

**Operator-selected, not yet mapped:** Rust `Libraries/claim-ledger` owns
claim/support event semantics. Python `ClaimLedger` retains extraction,
application, testimony, and export workflows; a typed, versioned adapter or
shared conformance fixture must map its overlapping support events. Forge's
causal/effect verification bundle is a separate evidence family. Do not delete
useful Python functionality or reinterpret a Forge bundle by its name.

`stack-ids`, `boundary-compiler`, `bitemporal-runtime`, `llm-pipeline`,
`semantic-memory-forge`, `forge-memory-bridge`, `turbo-quant`, `fib-quant`,
and Agent Graph copies remain **source-direction review candidates**, not
declared mirrors or automatic Libraries/standalone winners. Upstream-constrained
forks stay outside intrusive stack-wide migrations. A repo may be audited with
no local manifest or code change.

## Migration phases

## P0 — agree and freeze the vocabulary

Changes:

- review this proposal against owner-qualified source and record unresolved
  decisions before any adoption;
- no runtime behavior change;
- add no new semantic owner while source conflicts are unresolved.

Acceptance:

- all 60 repositories have a provisional role or explicit unknown state with an evidence basis;
- the active/relevant subset is justified by current source, integrations, PRs and release work rather than archive status alone;
- material cross-boundary terms are domain-qualified;
- unresolved owner/source decisions stay marked reconciliation_required or mapping_required.

Rollback:

- revert the docs-only governance commit/PR.

## P1 — resolve source ownership

For every source_conflicts entry:

1. compute exact file/tree deltas;
2. classify each delta as behavior, contract, tests, docs, release-only, or stale;
3. choose a source direction only for a shared semantic family or proven mirror;
   preserve justified independent implementations and their boundaries;
4. run that package's full owner gate;
5. record owner commit;
6. for proven mirrors only, regenerate/publish in the selected direction;
7. for proven mirrors only, make CI reject forbidden shared-semantic drift.

Acceptance:

- every P0/P1 duplicate relationship is classified, with one writer for each
  named shared semantic family and explicit exceptions for independent domains;
- same package+version cannot silently identify different shared wire/semantic
  content within a declared release-mirror relationship;
- collection repos are nonauthoritative.

Rollback:

- each package ownership transition is a separate PR; previous source remains readable until the mirror cutover is proven.

## P2 — normalize identity and digest semantics

Target contract:

- stack-ids supplies shared typed identity/BLAKE3 primitives where appropriate;
- boundary-compiler owns its structured canonicalization profile, not every protocol digest;
- each cross-boundary digest declares algorithm, canonicalization/preimage,
  domain, and schema/profile version; strict JSON + RFC 8785 + BLAKE3-256 is
  an opt-in profile, not the universal algorithm;
- legacy digest families remain versioned and readable.

Concrete work:

- add a tagged V2 digest reference only where a named consumer requires it;
- require explicit domain-bound conversion where a boundary-compiler profile is adopted;
- retain stack-ids and protocol-specific V1 contracts without silent reinterpretation;
- prohibit new bare hex digest fields at cross-repo boundaries.

Acceptance:

- within each declared profile, the same semantic JSON value yields the same V2 digest;
- duplicate keys fail;
- schema/domain changes change the digest;
- V1 golden vectors remain unchanged.

## P3 — normalize receipt, authority, and effect outcomes

Ownership chain:

- authority-delegation: grants/leases/delegation chains;
- verification-policy: policy decisions, approvals, execution permits;
- effect-runtime: effect intent -> preflight -> commit -> execution -> observation -> compensation;
- verification-control: verification cases/plans/control receipts;
- domain runtimes: retain justified domain permits/receipts with explicit scope,
  and never implicitly convert approval, permit, observation or outcome across families.

Normalize outcomes:

- reported
- owner_observed
- externally_confirmed
- ambiguous

Acceptance:

- no external effect is settled solely because a participant reported success;
- approvals cannot be reused after the bound generation/policy/scope changes;
- receipts document their predicate and integrity mechanism.

## P4 — normalize time and lifecycle terminology

Apply definitions from Semantic Law V1:

- valid_time
- observed_time
- recorded_time
- schema_version
- revision
- generation
- epoch
- incarnation

Concrete work:

- documentation corrections first;
- wire/schema changes only where existing field meaning is actually wrong;
- add V2 rather than reinterpret V1.

Special gate:

- bitemporal-runtime SupersessionReceipt V2 must fail closed on serialization and bind canonical content.

## P5 — normalize replay/recovery semantics

Every repo using replay/resume/recovery classifies each API as exactly one of:

- recorded_replay
- deterministic_reexecution
- resume
- recovery
- reconciliation
- effect_retry
- rollback
- compensation

Acceptance:

- recorded replay performs no external effects/provider calls;
- effect retry requires current authority;
- recovery does not guess external outcome;
- reconciliation yields a typed disposition before any retry.

## P6 — consumer and public-surface normalization

Audit source behavior and user-facing claims before changing labels in Ares,
Gloss, Mnemes, semantic-memory-mcp, agent-memory-kits, benchmark, web,
proveKV, and other active consumers. Ares PR #85 is a moving draft owned by a
separate session: compare its head with `main` read-only and defer Ares edits
until an explicit stable handoff or merge. Do not rename as a substitute for
owner-controlled behavior.

Rules:

- no consumer redefines an owner enum/state machine locally;
- public claims use evidence-level qualifiers;
- source presence != activation;
- retrieval/model output != authority;
- hardware/benchmark receipt != general performance proof.

Upstream forks are exempt from vocabulary rewrites inside upstream-owned code. RecursiveIntell-authored integration boundaries still follow the law.

## Early gate after owner classification — prevent new drift

Extend existing Libraries/AiDENs ownership inventory and root conformance
tooling; use `contract-schema-gen` for registered wire compatibility. First
snapshot existing debt by ID, path/type, owner, failure mode, temporary reason
and removal condition. CI MUST block **new** P0/P1 shadow semantics without
blanket suppressions while old debt is resolved separately. A new crate or
second schema registry needs a separate proof that existing tooling is
insufficient.

Minimum checks:

1. declared source relationships and same-version *shared-semantic* drift;
2. directional comparison only for proven mirrors;
3. registered wire/schema compatibility through `contract-schema-gen`;
4. algorithm/preimage/domain/version binding for new cross-boundary digests;
5. domain-qualified type ownership, not global same-name rejection;
6. owner registry coverage for newly introduced cross-repo contracts;
7. forbidden new shadow owner patterns;
8. unknown/unversioned semantic widening;
9. owner-specific conformance vectors.

The checker should produce evidence, not mutate repositories. Behavioral owner
tests and cross-stack negative fixtures remain separate required gates.

## Repository rollout groups

### Group A — semantic/core owners

Libraries, stack-ids, boundary-compiler, bitemporal-runtime, ClaimLedger/claim-ledger, semantic-memory, semantic-memory-forge, forge-memory-bridge, RecProv, recursive-agent, ri-agent-graph, agent-graph-mcp, llm-pipeline, mnemes, turbo-quant, fib-quant, proveKV.

These get owner/source reconciliation first.

### Group B — active consumers/integrations

Ares, Gloss, semantic-memory-mcp, agent-memory-kits, benchmark, cpu-router-cargo-automation, Recursive-Linux, recursiveintell-web, esp32-reusable, esp32-s3-lstm-proof, esp32-sensor-hub, esp32-sentinel, VisionForge, Server-Monitor, ClawGuard, PortalDoctor, pal, StableMaster, tiered-edge-ai, Aphelion, Duper, Hootie, local-llm-backend, ares-observatory.

These adopt owner references after Group A stabilizes.

### Group C — active collections or unresolved repositories

utility is a collection mirror and must not independently own duplicated semantics.

esp32-max-lm-training, esp32s3-edge-ai, mine, Sortarr, and torrent-fetch had no observed cross-stack semantic ownership in the inspected surfaces. They remain unclassified_active until they need a shared contract; then they must declare their role before adding one.

Forge-Audit and Rip are tooling.

### Group D — upstream/external-owner repositories

codex-openai, parameter-golf, Personal-AI-Router, awesome-mcp-servers.

Do not rewrite upstream contracts merely for local vocabulary consistency.

### Group E — archived/frozen

Claude-Test-Apps, Coder, Codex, kv-lossless-11x, LivingMemory, Rust-Libraries, Salty.

Preserve history. Do not backport Semantic Law V1 unless a repo is explicitly revived.

## Completion condition

Semantic reconciliation is complete when:

- every active cross-stack concept has exactly one semantic owner;
- every P0/P1 duplicate relationship is classified and shared semantic family
  has one admitted writer, with explicit independent-domain exceptions;
- every *proven* mirror is directional and verifiable;
- every cross-boundary digest identifies its own algorithm, canonicalization/
  preimage, domain and version without a universal digest algorithm;
- receipt predicates and authority boundaries are explicit;
- generation/epoch/revision/incarnation and replay/recovery terms no longer collide;
- no consumer can become a shadow truth/authority store without failing CI;
- legacy V1 artifacts remain verifiable under their original semantics.
