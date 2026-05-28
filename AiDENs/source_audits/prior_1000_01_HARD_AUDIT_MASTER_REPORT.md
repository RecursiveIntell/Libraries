# 01 — Hard Audit Master Report

## Security & sandbox boundary (70 findings)

**Gate:** v11A Boundary/Execution Evidence

**Risk:** Risk that filesystem/tool authority exceeds declared least-exposure boundary.

**Representative findings:**
- `AHD-0001` Sandbox denylist does not cover one common developer-secret path or metadata surface. (`crates/aidens-security-kit/src/lib.rs`)
- `AHD-0002` Filesystem path validation does not prove race-free handling for the target path at write time. (`crates/aidens-tool-kit/src/lib.rs`)
- `AHD-0003` Hidden file/directory access policy is weaker than the future application-builder threat model requires. (`crates/aidens-cli/src/lib.rs`)
- `AHD-0004` Path display can leak host-specific absolute paths when relative rendering fails. (`crates/aidens-runner/src/lib.rs`)
- `AHD-0005` Symlink, hardlink, mountpoint, or case-folding behavior lacks an explicit hostile fixture. (`crates/aidens-security-kit/src/lib.rs`)

**Required acceptance pattern:** Add hostile sandbox tests and deny/quarantine receipts for .git/.env/symlink/hardlink/unicode/TOCTOU cases.

## Tool exposure & permit policy (45 findings)

**Gate:** v11A Material Operation / Permit

**Risk:** Risk that declared tools, exposed tools, permitted tools, and executable tools diverge.

**Representative findings:**
- `AHD-0071` Default coding tool registry exposes or enables a capability broader than least-exposure requires. (`crates/aidens-tool-kit/src/lib.rs`)
- `AHD-0072` Declared tool, registered tool, executable tool, exposed tool, and provider-schema tool states can diverge. (`crates/aidens-permit-kit/src/lib.rs`)
- `AHD-0073` Tool descriptor persistence can be ephemeral even when the tool materially affects artifacts. (`crates/aidens-contracts/src/capability_turn.rs`)
- `AHD-0074` Permit-use evidence is not proven to be durable and queryable for every side-effecting tool path. (`crates/aidens-tool-kit/src/lib.rs`)
- `AHD-0075` Provider route policy can hide a tool for one route but not prove disabled tools are nonexistent. (`crates/aidens-permit-kit/src/lib.rs`)

**Required acceptance pattern:** Add tool exposure parity harness: declared vs registered vs executable vs exposed vs provider schema vs permit-used.

## Patch transactionality & treatment integrity (70 findings)

**Gate:** v11A Boundary Compiler / Treatment Integrity

**Risk:** Risk that code changes are applied without atomicity, rollback, or semantic receipts.

**Representative findings:**
- `AHD-0116` Patch application is not a full unified-diff interpreter and can mislead users/tools expecting patch semantics. (`crates/aidens-tool-kit/src/lib.rs`)
- `AHD-0117` Patch hunk anchoring is too weak for repeated-content files. (`crates/aidens-contracts/src/tool_artifacts.rs`)
- `AHD-0118` Patch context is not enforced strongly enough to prove the intended treatment was applied. (`crates/aidens-runner/src/lib.rs`)
- `AHD-0119` Multi-file patch application is not transactionally atomic. (`crates/aidens-tool-kit/src/lib.rs`)
- `AHD-0120` Patch failure after partial write lacks a rollback/quarantine path. (`crates/aidens-contracts/src/tool_artifacts.rs`)

**Required acceptance pattern:** Replace or constrain patch_apply; require transaction receipt, before/after digests, rollback/quarantine, and post-write verification.

## Command execution & environment control (45 findings)

**Gate:** v11A Execution Evidence

**Risk:** Risk that command execution is not replayable, bounded, or environment-fingerprinted.

**Representative findings:**
- `AHD-0186` Command arguments are parsed with whitespace-oriented logic instead of structured argv. (`crates/aidens-tool-kit/src/lib.rs`)
- `AHD-0187` Child process timeout does not prove grandchildren are killed or quarantined. (`crates/aidens-cli/src/lib.rs`)
- `AHD-0188` Command output volume is not proven bounded by byte caps in every path. (`crates/aidens-cli/src/package.rs`)
- `AHD-0189` Command execution receipts lack sufficient environment/toolchain fingerprinting for replay. (`crates/aidens-tool-kit/src/lib.rs`)
- `AHD-0190` Allowed-command policy is narrower than actual verification needs and can cause bypass workarounds. (`crates/aidens-cli/src/lib.rs`)

**Required acceptance pattern:** Require structured argv, process-group kill, output caps, environment/toolchain fingerprints, and replay mismatch taxonomy.

## Provider route & local/mocking honesty (55 findings)

**Gate:** v11A Execution Evidence / Support Profile

**Risk:** Risk that provider mode, provider route, and actual behavior disagree.

**Representative findings:**
- `AHD-0231` Local provider mode can map to mock behavior, creating support-profile dishonesty. (`crates/aidens-provider-kit/src/lib.rs`)
- `AHD-0232` Mock provider paths can appear as successful runs without sufficiently sharp exactness/degradation labeling. (`crates/aidens-runner/src/lib.rs`)
- `AHD-0233` Ollama provider does not prove native tool schema/tool-result loop parity. (`crates/aidens-cli/src/lib.rs`)
- `AHD-0234` Provider route fallback can change execution semantics without a strong rejected-alternatives receipt. (`crates/aidens-provider-kit/src/lib.rs`)
- `AHD-0235` Provider endpoint policy does not prove local-only network behavior for local providers. (`crates/aidens-runner/src/lib.rs`)

**Required acceptance pattern:** Make local/mock/Ollama/provider routes honest; prevent local=>mock fallback without explicit degradation and support-label change.

## Receipts, event logs, durability & replay (75 findings)

**Gate:** v11A Execution Evidence / Artifact Lifecycle

**Risk:** Risk that receipts are not durable, ordered, tamper-evident, or replay-linked.

**Representative findings:**
- `AHD-0286` Receipt/event append path is not proven single-writer or file-lock protected. (`crates/aidens-receipts/src/lib.rs`)
- `AHD-0287` Sequence numbers and previous digests can race under concurrent append. (`crates/aidens-runner/src/lib.rs`)
- `AHD-0288` Direct file writes can create visible artifacts before all receipt records exist. (`crates/aidens-cli/src/agent.rs`)
- `AHD-0289` Corrupt trailing NDJSON/event records can poison the whole log instead of entering quarantine. (`crates/aidens-receipts/src/lib.rs`)
- `AHD-0290` Receipt inspection does not prove duplicate IDs are impossible or corruption-blocking. (`crates/aidens-runner/src/lib.rs`)

**Required acceptance pattern:** Introduce file locking/single-writer logs, atomic writes, fsync, hash-chain verification, corruption quarantine, and no done without receipts.

## Queue, scheduler, daemon & concurrency (70 findings)

**Gate:** v11A Execution Evidence / v11B Region Scheduling

**Risk:** Risk that queue/lease/scheduler behavior races or loses lineage.

**Representative findings:**
- `AHD-0361` Queue append computes sequence from snapshot state and can race under concurrent writers. (`crates/aidens-queue-kit/src/lib.rs`)
- `AHD-0362` Queue idempotency suppression can race under concurrent enqueue. (`crates/aidens-daemon-kit/src/lib.rs`)
- `AHD-0363` Lease acquisition is snapshot-plus-append and can double-lease jobs. (`crates/aidens-contracts/src/daemon_queue.rs`)
- `AHD-0364` Job completion can be insufficiently coupled to a verified active lease. (`crates/aidens-queue-kit/src/lib.rs`)
- `AHD-0365` Lease expiry and late completion need adversarial fixtures. (`crates/aidens-daemon-kit/src/lib.rs`)

**Required acceptance pattern:** Make queue append/lease/complete atomic with leases, locks, hop receipts, idempotency proofs, and safe-mode quarantine.

## Boundary compiler, JSON, schema & repair (80 findings)

**Gate:** v11A Boundary Compiler Gate

**Risk:** Risk that structured data is accepted or repaired without full semantic contract.

**Representative findings:**
- `AHD-0431` Boundary repair defaults allow markdown-fence or substring extraction behavior that is too permissive for evidence-grade inputs. (`crates/aidens-boundary-kit/src/lib.rs`)
- `AHD-0432` Handwritten schema validation only covers a subset of JSON Schema features. (`crates/aidens-contracts/src/boundary.rs`)
- `AHD-0433` Unsupported schema keywords may be silently ignored instead of rejected as unsupported semantics. (`crates/aidens-cli/src/lib.rs`)
- `AHD-0434` Duplicate-key detection needs fuzzing against escapes, nesting, unicode, and parser disagreement. (`crates/aidens-boundary-kit/src/lib.rs`)
- `AHD-0435` Treatment-critical paths use simplified addressing that can miss JSON pointer edge cases. (`crates/aidens-contracts/src/boundary.rs`)

**Required acceptance pattern:** Adopt strict boundary compiler profiles, full schema support or unsupported-key rejection, durable repair receipts, and fuzz/property tests.

## Bitemporal, proof, view & semantic state (85 findings)

**Gate:** v11A Bitemporal / Proof Economy / View Disclosure

**Risk:** Risk that temporal truth, proof debt, exactness, or view widening is hidden.

**Representative findings:**
- `AHD-0511` Bitemporal valid-time and recorded-time semantics need a pure reference interpreter fixture. (`crates/aidens-contracts/src/semantic.rs`)
- `AHD-0512` Retroactive corrections need fixtures proving belief-history is preserved. (`crates/aidens-contracts/src/proof.rs`)
- `AHD-0513` Projection stale-state behavior needs fixtures proving degraded disclosure. (`crates/aidens-contracts/src/view_runtime.rs`)
- `AHD-0514` Proof waiver paths need fixtures proving waiver is never treated as proof. (`crates/aidens-memory-kit/src/lib.rs`)
- `AHD-0515` Proof debt needs queryable allowed-use restrictions and expiry/escalation behavior. (`crates/aidens-contracts/src/semantic.rs`)

**Required acceptance pattern:** Build bitemporal/proof/view reference fixture corpus and production-path differential tests.

## v11B graph, region, convergence, subtraction & causal runtime (90 findings)

**Gate:** v11B Regional Runtime Gates

**Risk:** Risk that v11B surfaces exist without executable regional/convergence/subtraction proofs.

**Representative findings:**
- `AHD-0596` Right-graph law lacks adversarial tests where storage/retrieval/control graph misuse produces false output. (`crates/aidens-contracts/src/reserved_v11.rs`)
- `AHD-0597` Region contracts are seeded but not proven as executable runtime boundaries. (`crates/aidens-kernel-kit/src/lib.rs`)
- `AHD-0598` Boundary messages and receipts need accept/reject/quarantine fixtures. (`crates/aidens-repair-kit/src/lib.rs`)
- `AHD-0599` Region replay slices and snapshots are not proven end-to-end. (`crates/aidens-contracts/src/reserved_v11.rs`)
- `AHD-0600` Convergence normal/failure/oscillation/budget-exhaustion fixtures are missing. (`crates/aidens-kernel-kit/src/lib.rs`)

**Required acceptance pattern:** Implement one minimal v11B region with right-graph misuse tests, convergence failure, syndrome, repair, support-core, oracle diff.

## Schema, contract governance & generated artifacts (50 findings)

**Gate:** v11A Schema/Contract Gate

**Risk:** Risk that generated schemas drift from Rust contracts or act as hidden law.

**Representative findings:**
- `AHD-0686` Generated schemas may drift from Rust contracts without meta-validation and compatibility gates. (`crates/aidens-contracts/src/schema_catalog.rs`)
- `AHD-0687` Schema identity is not proven content-addressed across generation and packaging. (`schemas/**/v1.schema.json`)
- `AHD-0688` Schema compatibility checks need major/minor migration semantics, not equality-only checks. (`crates/aidens-cli/src/lib.rs`)
- `AHD-0689` Generated schema catalog can become hidden law unless admitted through governance. (`crates/aidens-contracts/src/schema_catalog.rs`)
- `AHD-0690` Schema fixtures need negative cases for incompatible changes and unknown families. (`schemas/**/v1.schema.json`)

**Required acceptance pattern:** Gate generated schema changes with meta-validation, compatibility diffing, canonical digest identity, and migration receipts.

## Artifact lifecycle & operator effect system (55 findings)

**Gate:** v11A Artifact Lifecycle / Operator Effects

**Risk:** Risk that material transitions lack lifecycle receipts or effect enforcement.

**Representative findings:**
- `AHD-0736` Material artifact lifecycle transitions need explicit transition receipts for every path. (`crates/aidens-contracts/src/artifact.rs`)
- `AHD-0737` Operator contracts must block effects absent from the declared effect set. (`crates/aidens-contracts/src/operator.rs`)
- `AHD-0738` Output manifests are not proven mandatory for every material operation. (`crates/aidens-runner/src/lib.rs`)
- `AHD-0739` Input manifests need missing/opaque-ref semantics with degradation records. (`crates/aidens-contracts/src/artifact.rs`)
- `AHD-0740` Promotion eligibility rules need executable enforcement, not only contract fields. (`crates/aidens-contracts/src/operator.rs`)

**Required acceptance pattern:** Enforce OperatorContract effect sets and artifact lifecycle transition receipts for all material paths.

## Code modularity, crate ownership & maintainability (50 findings)

**Gate:** Release Maintainability / Source Ownership

**Risk:** Risk that oversized modules become semantic gravity wells or hidden owners.

**Representative findings:**
- `AHD-0791` Oversized module concentrates too much semantic authority in one file. (`crates/aidens-cli/src/lib.rs`)
- `AHD-0792` Facade module still contains business logic instead of delegating to owner modules. (`crates/aidens-tool-kit/src/lib.rs`)
- `AHD-0793` Test mega-file reduces ability to identify missing semantic coverage. (`crates/aidens-contracts/src/tests.rs`)
- `AHD-0794` Contract crate risks becoming a junk drawer rather than a strict wire-contract owner. (`crates/aidens-runner/src/lib.rs`)
- `AHD-0795` Reserved v11 surfaces should be separated from active runtime surfaces to prevent authority confusion. (`crates/aidens-cli/src/lib.rs`)

**Required acceptance pattern:** Split gravity-well modules by source-of-truth boundary and forbid new semantics in facades.

## Testing quality, marker assertions & hostile fixtures (60 findings)

**Gate:** v11+ Conformance Gates

**Risk:** Risk that tests prove marker presence instead of adversarial semantics.

**Representative findings:**
- `AHD-0841` Assertion script checks marker strings rather than behavior. (`scripts/assert_p29_*.py`)
- `AHD-0842` Positive fixtures deserialize but do not prove negative cases fail correctly. (`crates/aidens-integration-tests/tests/*.rs`)
- `AHD-0843` Conformance tests do not yet cover all v11A gates. (`fixtures/**`)
- `AHD-0844` P29 issue matrix closure checks IDs rather than statuses/resolutions. (`scripts/assert_p29_*.py`)
- `AHD-0845` Hostile tests should include fuzz/property-based inputs, not only hand fixtures. (`crates/aidens-integration-tests/tests/*.rs`)

**Required acceptance pattern:** Replace marker-only assertions with semantic, negative, extracted-package, concurrency, and fuzz fixtures.

## Docs, evidence, source-basis & run hygiene (40 findings)

**Gate:** Release Evidence / Source Basis

**Risk:** Risk that support labels, matrices, docs, and source basis drift.

**Representative findings:**
- `AHD-0901` Issue matrix statuses remain open-like and do not encode closed/quarantined/deferred semantics. (`matrices/P29_MASTER_ISSUE_MATRIX.csv`)
- `AHD-0902` Support labels need exact definitions tied to executable gates. (`docs/p29/**`)
- `AHD-0903` Source basis should distinguish source package validity from product conformance. (`STATUS.md`)
- `AHD-0904` Root markdown ambiguity remains high and should be archived or classified. (`SUPPORT_PROFILE.md`)
- `AHD-0905` Docs must distinguish v11A active, v11B seeded, and v11C reserved states. (`SOURCE_BASIS.md`)

**Required acceptance pattern:** Make docs/support labels/matrix statuses gate-derived and classify active/seeded/reserved surfaces precisely.

## Config, environment, secrets & redaction (35 findings)

**Gate:** v11A Execution Evidence / Security

**Risk:** Risk that config/env/secrets/redaction are not governed as artifacts.

**Representative findings:**
- `AHD-0941` Config redaction should have adversarial fixtures for secret-like keys and values. (`crates/aidens-config/src/lib.rs`)
- `AHD-0942` Environment variables used by execution should be captured or explicitly excluded. (`crates/aidens-cli/src/lib.rs`)
- `AHD-0943` Config apply should produce receipts before changing runtime-visible behavior. (`examples/*.toml`)
- `AHD-0944` Provider secrets and endpoint configs need stricter source/permission checks. (`crates/aidens-config/src/lib.rs`)
- `AHD-0945` Example configs should be classified as mock/local/unavailable/live to avoid overclaims. (`crates/aidens-cli/src/lib.rs`)

**Required acceptance pattern:** Add config receipt, redaction, environment fingerprint, default-policy, and migration fixtures.

## App/scaffold/profile readiness (25 findings)

**Gate:** Application Builder Readiness

**Risk:** Risk that generated apps/profiles overclaim readiness or bypass core receipts.

**Representative findings:**
- `AHD-0976` Generated app scaffold can imply readiness before receipt/proof paths exist. (`crates/aidens-app-kit/src/lib.rs`)
- `AHD-0977` Profile crates still contain scaffolded notes and need supported-surface classification. (`crates/aidens-profile-*/src/lib.rs`)
- `AHD-0978` App builder should inject receipt store, sandbox root, provider route, and permit policy by default. (`crates/aidens-cli/src/agent.rs`)
- `AHD-0979` Generated apps need tests proving final output cannot bypass AiDENs receipts. (`crates/aidens-app-kit/src/lib.rs`)
- `AHD-0980` Generated templates should not use mock/local provider labels ambiguously. (`crates/aidens-profile-*/src/lib.rs`)

**Required acceptance pattern:** Make scaffolds receipt-first and profile-tier honest; generated apps include conformance smoke tests.

## Non-counted operator note

The operator clarified that the package finished fine and that the final post-bundle gate was skipped intentionally. That shortcut is not counted here as a product defect. Any evidence/doc row in this matrix is about durable hardening, support-label clarity, or matrix hygiene, not the mere absence of that manually skipped post-bundle gate.
