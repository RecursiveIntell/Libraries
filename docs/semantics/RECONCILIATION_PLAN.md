# Cross-Repository Semantic Reconciliation Plan

**Status:** proposal accompanying Semantic Law V1  
**Date:** 2026-09-27  
**Scope:** 60 owned repositories inspected through the connected GitHub account.

## Goal

Reduce semantic drift by making three things explicit and enforceable:

1. one semantic owner per concept family;
2. one editable source owner per duplicated package;
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

There should not be a global rule that every standalone repository wins or every monorepo copy wins. The rule is **one editable source per package**, recorded in the registry.

Recommended target:

### Libraries canonical development spine

Prefer Libraries as the editable source for contracts that are deeply composed by the Libraries workspace and current sibling consumers:

- stack-ids
- boundary-compiler
- bitemporal-runtime
- claim-ledger (Rust)
- semantic-memory
- semantic-memory-forge
- forge-memory-bridge
- authority-delegation
- effect-runtime
- verification-control
- verification-policy
- continuity-runtime
- profile-runtime
- attestation-exchange
- assurance-runtime
- contract-schema-gen

Before declaring this final, import and semantically reconcile any newer standalone delta. Do not simply overwrite it.

Standalone repositories for these packages may remain release mirrors/front doors, but after reconciliation they become one-way mirrors, not independent development roots.

### Dedicated standalone canonical owners

Prefer dedicated repositories where the current ecosystem already treats them as independent released domain products:

- ri-agent-graph — core graph engine
- agent-graph-mcp — MCP graph service
- turbo-quant — TurboQuant codec/research surface
- fib-quant — FibQuant codec/research surface
- proveKV — shared KV pool
- mnemes — multi-device memory control plane
- RecProv — RecProv protocol
- recursive-agent — execution kernel
- Ares — Ares runtime

For these, copies under Libraries/utility/other collection repos become pinned mirrors or are removed from active build surfaces.

### Requires an explicit product decision

- llm-pipeline: Libraries has 0.3.0 while standalone publishes 0.2.1 and agent-graph-mcp consumes the published line. Reconcile the 0.3 delta first, then choose one source.
- ClaimLedger Python vs Libraries/claim-ledger Rust: both expose claim/evidence vocabulary. Recommend Rust claim-ledger as canonical stack semantics; Python becomes a conforming adapter/reference implementation or is renamed to make its noncanonical status explicit.
- agent-graph copies under Libraries and utility: dedicated ri-agent-graph should win unless a newer Libraries-only semantic delta is intentionally promoted first.

## Migration phases

## P0 — agree and freeze the vocabulary

Changes:

- merge/adopt Semantic Law V1 and Owner Registry V1;
- no runtime behavior change;
- add no new semantic owner while source conflicts are unresolved.

Acceptance:

- every active repo has a role;
- every strong term has a defined cross-repo meaning;
- unresolved owner decisions are explicitly marked reconciliation_required.

Rollback:

- revert the docs-only governance commit/PR.

## P1 — resolve source ownership

For every source_conflicts entry:

1. compute exact file/tree deltas;
2. classify each delta as behavior, contract, tests, docs, release-only, or stale;
3. merge unique valid semantics into the chosen owner;
4. run that package's full owner gate;
5. record owner commit;
6. regenerate/publish the mirror from that owner;
7. make mirror CI reject local semantic divergence.

Acceptance:

- every active duplicated package has one canonical editable source;
- same package+version cannot exist with different source digests;
- collection repos are nonauthoritative.

Rollback:

- each package ownership transition is a separate PR; previous source remains readable until the mirror cutover is proven.

## P2 — normalize identity and digest semantics

Target contract:

- stack-ids owns shared typed identity/digest references;
- boundary-compiler owns structured canonicalization and cross-repo JSON digest computation;
- new structured digest profile = strict JSON + RFC 8785 + schema/version + domain + BLAKE3-256;
- legacy digest families remain versioned and readable.

Concrete work:

- add a tagged V2 digest reference or equivalent shared type;
- add explicit conversion from boundary-compiler canonical digest result;
- deprecate stack-ids compute_json for new cross-repo structured identity, without removing V1;
- prohibit new bare hex digest fields at cross-repo boundaries.

Acceptance:

- same semantic JSON value yields one agreed V2 digest across owner crates;
- duplicate keys fail;
- schema/domain changes change the digest;
- V1 golden vectors remain unchanged.

## P3 — normalize receipt, authority, and effect outcomes

Ownership chain:

- authority-delegation: grants/leases/delegation chains;
- verification-policy: policy decisions, approvals, execution permits;
- effect-runtime: effect intent -> preflight -> commit -> execution -> observation -> compensation;
- verification-control: verification cases/plans/control receipts;
- domain runtimes: may define domain-specific receipts but may not redefine the generic concepts above.

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

Apply documentation/API aliases across Ares, Gloss, Mnemes, semantic-memory-mcp, agent-memory-kits, benchmark, Recursive-Linux, web, ESP32 projects, proveKV, and other consumers.

Rules:

- no consumer redefines an owner enum/state machine locally;
- public claims use evidence-level qualifiers;
- source presence != activation;
- retrieval/model output != authority;
- hardware/benchmark receipt != general performance proof.

Upstream forks are exempt from vocabulary rewrites inside upstream-owned code. RecursiveIntell-authored integration boundaries still follow the law.

## P7 — enforce drift in CI

Add a repository-independent semantic drift checker.

Minimum checks:

1. package name/version/source digest collision detection;
2. mirror-vs-owner source comparison;
3. artifact schema-ID uniqueness;
4. cross-repo digest-profile allowlist;
5. duplicate semantic type-name inventory;
6. owner registry coverage for newly introduced cross-repo contracts;
7. forbidden new shadow owner patterns;
8. unknown/unversioned semantic widening;
9. owner-specific conformance vectors.

The checker should produce evidence, not mutate repositories.

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
- every duplicated active package has exactly one editable source owner;
- all mirrors are one-way and verifiable;
- cross-repo JSON content identity has one V2 canonical profile;
- receipt predicates and authority boundaries are explicit;
- generation/epoch/revision/incarnation and replay/recovery terms no longer collide;
- no consumer can become a shadow truth/authority store without failing CI;
- legacy V1 artifacts remain verifiable under their original semantics.
