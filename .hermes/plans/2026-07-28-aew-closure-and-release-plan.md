# Agent Evidence Workbench Closure and Release-Readiness Plan

> **For Hermes:** Use `subagent-driven-development` task-by-task. Preserve the current dirty worktree. Do not commit, reset, stage, publish, register a Hermes hook, or promote to a live semantic-memory store without an explicit subsequent instruction.

**Planning checkpoint:** 2026-07-28, `/home/sikmindz/Coding/Libraries`.

**Goal:** Close the remaining correctness, evidence-storage, verification, packaging, and dependency-quality gaps so AEW can make narrow, reproducible local-first accountability claims without relying on unverified subagent assertions or a red dependency-tree lint gate.

**Architecture:** Keep `agent-evidence-workbench` as an independent Rust CLI and projection layer. `claim-ledger` remains the claim/evidence type owner; `semantic-memory` remains the promotion owner; `llm-pipeline` receipts remain LLM-specific. AEW owns only run reports, immutable evidence blobs, signed workbench receipts, and adapters/backpointers.

**Tech stack:** Rust 2021, Tokio, Clap, Serde, SHA-256/HMAC-SHA256, claim-ledger, optional semantic-memory, maturin/PyO3, CPython 3.14.

---

## 0. Current evidence and claim boundary

### Observed baseline

| Surface | Evidence | State |
|---|---|---|
| Main workspace | `cargo metadata --no-deps --format-version 1` | Observed: 64 workspace members |
| AEW default tests | `cargo test -p agent-evidence-workbench` | Verified: 3 passed |
| AEW semantic-memory feature tests | `cargo test -p agent-evidence-workbench --features semantic-memory` | Verified: 3 passed |
| AEW own all-feature lint lane | `cargo clippy -p agent-evidence-workbench --no-deps --all-features -- -D warnings` | Verified: clean |
| LLM provenance e2e | `cargo test -p llm-pipeline --test e2e_provenance --all-features` | Verified: 1 passed |
| AEW disposable demo | `/tmp/aew-final-demo` | Verified: `tests pass` supported; `no regressions` unsupported; valid receipt passes; invalid receipt/promotion exit nonzero |
| AEW full dependency-tree Clippy | `cargo clippy -p agent-evidence-workbench --all-features -- -D warnings` | Blocked: five semantic-memory lint failures |
| PyO3 wheel | `maturin build --release --auditwheel skip` then isolated venv import | Verified: native CPython 3.14 wheel built and imported; not portable/repaired |

### Known blockers and defects

1. **P0 release gate blocker — dependency Clippy:** AEW’s literal all-feature Clippy command fails in `semantic-memory`, not in AEW:
   - `semantic-memory/src/episodes.rs:810` and `:875`: `clone_on_copy`
   - `semantic-memory/src/hnsw_ops.rs:253`: unnecessary `as u128`
   - `semantic-memory/src/journal.rs:64`: redundant closure
   - `semantic-memory/src/projection_import.rs:377-411`: needless `Ok(...?)`
2. **P1 evidence-retention gap:** AEW records evidence digests and 200-character summaries but does not yet prove durable, content-addressed blob persistence/readback for command output, transcript imports, or Git snapshots. A digest without durable local content is an incomplete witness.
3. **P1 test-coverage gap:** current AEW suite has only three unit tests. The user-facing imports, verification persistence, graph-cost absence behavior, receipt CLI nonzero behavior, and promotion metadata path are not independently covered as executable behavior tests.
4. **P1 receipt-chain gap:** `aew sign` currently passes `None` for `previous_receipt_digest`; receipt chaining exists in type shape but has no defined continuity policy.
5. **P1 promotion idempotency gap:** repeated promotion can create duplicate semantic-memory facts. The current flow has no durable AEW promotion receipt binding each verified claim to the fact ID returned by the canonical store.
6. **P2 graph-import precision gap:** recursive discovery accepts generic numeric fields named `cost`, `estimated_cost`, or token fields anywhere in arbitrary JSON. It must retain JSON pointers and distinguish recognized Agent Graph event schema from untyped observations.
7. **P2 Hermes observer operational gap:** the opt-in observer intentionally swallows all exceptions. That preserves nonblocking behavior, but gives operators no bounded health/error signal.
8. **P2 wheel portability blocker:** standard maturin repair failed because `patchelf` is absent. The existing wheel is valid for the local machine only; it must not be represented as a portable distribution artifact.

### Authority and scope rules

- Do **not** alter `semantic-memory` until its current dirty/untracked state is captured and its owner approves the exact five lint-only edits. Its source tree contains extensive active untracked work.
- Do **not** install or register `aew-observer.py` into global Hermes hooks during this plan. Test it only with disposable JSONL inputs.
- Do **not** run live promotion against a user memory directory. Use a fresh disposable directory plus an explicitly configured real embedder only when the live-test preflight passes.
- Do **not** call an AEW receipt an `llm-pipeline` receipt. They have separate contracts.
- Do **not** use mock embeddings to turn live semantic-memory promotion green.
- Do **not** publish the wheel until repaired-wheel inspection and clean-environment import succeed.

---

## Phase A — preserve ownership and establish reproducible gates

### Task A1: Record a scoped preflight receipt

**Objective:** Prevent accidental overwrite of concurrent work and establish the exact state used for remediation.

**Files:**
- Create: `.hermes/evidence/aew-closure/2026-07-28-preflight.txt`
- Do not modify production source.

**Step 1: Capture worktree state and target hashes.**

```bash
cd /home/sikmindz/Coding/Libraries
mkdir -p .hermes/evidence/aew-closure
{
  date --iso-8601=seconds
  git rev-parse --show-toplevel
  git status --short
  git diff --check
  git diff --stat -- agent-evidence-workbench semantic-memory
  sha256sum agent-evidence-workbench/Cargo.toml \
    agent-evidence-workbench/src/main.rs \
    agent-evidence-workbench/src/receipt.rs \
    semantic-memory/src/episodes.rs \
    semantic-memory/src/hnsw_ops.rs \
    semantic-memory/src/journal.rs \
    semantic-memory/src/projection_import.rs
} | tee .hermes/evidence/aew-closure/2026-07-28-preflight.txt
```

**Expected:** A timestamped receipt exists. `git diff --check` must be clean before source edits proceed.

**Step 2: Create ownership decision record.**

Add a one-line decision to the receipt:

```text
semantic-memory lint-only edits authorized: yes|no; authorizing principal: <name>; observed source hashes above.
```

**Abort rule:** If authorization is not available, skip Phase B; complete only AEW-owned phases C–F and report the full dependency Clippy gate as blocked.

**Completion claim:** “Current source state was captured; no concurrent work was overwritten.”

---

## Phase B — clear the literal dependency-tree Clippy gate

**Prerequisite:** Task A1 recorded `semantic-memory lint-only edits authorized: yes`.

### Task B1: Remove `clone_on_copy` in episode metadata construction

**Objective:** Clear the two behavior-preserving `Option<DateTime<Utc>>` clone lints.

**Files:**
- Modify: `semantic-memory/src/episodes.rs:803-812`
- Modify: `semantic-memory/src/episodes.rs:868-876`
- Test: existing semantic-memory episode tests; add a focused regression only if no outcome-update coverage exists.

**Step 1: Establish behavior baseline.**

```bash
cargo test -p semantic-memory update_episode_outcome -- --nocapture
```

Record whether matching tests execute. If zero tests match, run:

```bash
cargo test -p semantic-memory --lib -- --nocapture
```

**Step 2: Make the minimal changes.**

Replace only:

```rust
valid_time: current_meta.valid_time.clone(),
```

with:

```rust
valid_time: current_meta.valid_time,
```

at both sites. Do not alter ownership of non-`Copy` fields.

**Step 3: GREEN gate.**

```bash
cargo fmt --check -p semantic-memory
cargo test -p semantic-memory update_episode_outcome -- --nocapture
cargo clippy -p semantic-memory --lib -- -D warnings
```

**Rollback:** Restore precisely the two edited lines from the Task A1 hash/baseline if compilation or outcome tests change behavior.

---

### Task B2: Remove the HNSW elapsed-time no-op cast

**Objective:** Preserve receipt precision while clearing the unnecessary cast.

**Files:**
- Modify: `semantic-memory/src/hnsw_ops.rs:242-258`
- Test: existing HNSW receipt/build tests, if feature-supported.

**Step 1: Add or identify a focused receipt assertion.**

The receipt must retain a nonzero `elapsed_ms` type-compatible value. If no test covers this field, add a unit test near the HNSW receipt builder asserting `elapsed_ms` serializes as an integer.

**Step 2: Make the minimal change.**

```rust
elapsed_ms: started.elapsed().as_millis(),
```

**Step 3: GREEN gate.**

```bash
cargo test -p semantic-memory --features hnsw hnsw -- --nocapture
cargo clippy -p semantic-memory --lib --features hnsw -- -D warnings
```

**Rollback:** Restore the single cast if type inference reveals a different declared receipt type; do not coerce unrelated timing fields.

---

### Task B3: Simplify journal error mapping without changing error ownership

**Objective:** Preserve typed `MemoryError::Database(rusqlite::Error)` propagation.

**Files:**
- Modify: `semantic-memory/src/journal.rs:56-64`
- Test: `semantic-memory` journal tests or a new focused query-failure test if one exists.

**Step 1: RED/behavior inspection.**

Verify `MemoryError::Database` remains the canonical error variant in `semantic-memory/src/error.rs`.

**Step 2: Minimal implementation.**

```rust
.map_err(MemoryError::Database)?;
```

**Step 3: GREEN gate.**

```bash
cargo test -p semantic-memory journal -- --nocapture
cargo clippy -p semantic-memory --lib -- -D warnings
```

**Acceptance test:** a deliberately malformed/no-table query, if the existing suite supports it, still returns a typed database error rather than stringification or panic.

---

### Task B4: Simplify projection-import collection without changing row conversion

**Objective:** Clear the needless-question-mark lint while preserving conversion failures and exact `ImportReceipt` values.

**Files:**
- Modify: `semantic-memory/src/projection_import.rs:377-411`
- Test: existing projection import tests.

**Step 1: Write/locate a focused test.**

Cover both:
- a valid row maps to `ImportReceipt`, and
- malformed legacy `envelope_id` returns `MemoryError::Database(FromSqlConversionFailure(...))`.

**Step 2: Minimal implementation.**

Return the collected result directly:

```rust
rows.into_iter()
    .map(/* existing mapping closure unchanged */)
    .collect::<Result<Vec<_>, _>>()
```

**Step 3: GREEN gate.**

```bash
cargo test -p semantic-memory projection_import -- --nocapture
cargo clippy -p semantic-memory --lib -- -D warnings
```

**Rollback:** Restore only the wrapper if error inference changes; do not weaken the legacy `EnvelopeId::from_legacy` conversion boundary.

---

### Task B5: Certify the literal AEW full Clippy command

**Objective:** Close the release gate as the user will actually run it.

**Files:** none expected.

```bash
cd /home/sikmindz/Coding/Libraries
cargo clippy -p agent-evidence-workbench --all-features -- -D warnings
```

**Expected:** Exit 0 with no warnings.

**Evidence:** Save untruncated output to `.hermes/evidence/aew-closure/full-aew-clippy.txt`.

**Completion claim:** “AEW and its enabled dependency graph satisfy this specific Clippy gate.”

---

## Phase C — make evidence truly durable and replayable

### Task C1: Add content-addressed blob storage

**Objective:** Make every AEW `EvidenceItem.digest` refer to an immutable local artifact that can be read back and rehashed.

**Files:**
- Modify: `agent-evidence-workbench/src/storage.rs`
- Modify: `agent-evidence-workbench/src/model.rs`
- Modify: `agent-evidence-workbench/src/main.rs`
- Create: `agent-evidence-workbench/tests/evidence_storage.rs`

**Step 1: Write failing integration tests.**

Test cases:
1. `store_blob` writes `.aew/evidence/sha256/<digest>` and returns the exact SHA-256 digest.
2. Storing identical bytes twice is idempotent and does not rewrite semantic content.
3. `load_blob` rehashes content and rejects a manually tampered file with typed `Error::Integrity`.
4. A completed `aew run` report points every persisted transcript/command/Git evidence item to a readable blob.

**Step 2: Run RED.**

```bash
cargo test -p agent-evidence-workbench --test evidence_storage -- --nocapture
```

Expected: compilation or behavior failure because durable blob API does not exist.

**Step 3: Add minimal data shape.**

Extend `EvidenceItem` additively:

```rust
#[serde(default)]
pub blob_path: Option<String>,
#[serde(default)]
pub byte_len: Option<u64>,
```

Old reports must deserialize with `None` fields.

**Step 4: Implement storage helpers.**

```rust
pub fn store_blob(cwd: &Path, bytes: &[u8]) -> Result<StoredBlob>;
pub fn load_blob(cwd: &Path, digest: &str) -> Result<Vec<u8>>;
```

Requirements:
- Use `create_dir_all(.aew/evidence/sha256)`.
- Write a uniquely named temporary file in the same directory.
- `sync_all`, rename atomically, then best-effort parent-directory sync on supported platforms.
- Never accept a path supplied by report JSON; derive path from validated lowercase 64-hex digest.
- Rehash on read.

**Step 5: Wire creation paths.**

`aew run`, `import-transcript`, `import-graph-result`, and Git evidence must call `store_blob`; summaries remain projections, not source of truth.

**Step 6: GREEN gates.**

```bash
cargo test -p agent-evidence-workbench --test evidence_storage -- --nocapture
cargo test -p agent-evidence-workbench
cargo clippy -p agent-evidence-workbench --no-deps --all-features -- -D warnings
```

**Rollback:** Additive fields and new blobs are safe to retain. To revert code, stop creating blobs; old reports still deserialize. Do not delete evidence blobs automatically.

---

### Task C2: Make `aew evidence` evidence-backed

**Objective:** Turn the CLI from a manifest printer into a verifier of local evidence availability.

**Files:**
- Modify: `agent-evidence-workbench/src/cli.rs`
- Modify: `agent-evidence-workbench/src/main.rs`
- Modify: `agent-evidence-workbench/src/storage.rs`
- Test: `agent-evidence-workbench/tests/evidence_storage.rs`

**Step 1: Add CLI contract.**

```text
aew evidence <run-id> [--verify-blobs]
```

Default prints the manifest. `--verify-blobs` must emit one row per artifact with `available`, `digest_matches`, `byte_len`, and an overall nonzero exit if any expected blob is missing/tampered.

**Step 2: RED tests.**

- Missing blob returns nonzero.
- Tampered blob returns nonzero.
- Legacy report whose item has no `blob_path` is reported as `unavailable_legacy`, never as verified.

**Step 3: Implement and GREEN.**

```bash
cargo test -p agent-evidence-workbench --test evidence_storage -- --nocapture
cargo build -p agent-evidence-workbench
```

**Completion claim:** “Evidence verification means the local bytes exist and match their recorded digest.”

---

## Phase D — close AEW behavior coverage and receipt-chain semantics

### Task D1: Add CLI integration coverage for resolved claim status persistence

**Objective:** Prevent regression to the former misleading `NotChecked`-only behavior.

**Files:**
- Create: `agent-evidence-workbench/tests/cli_workflows.rs`
- Modify: `agent-evidence-workbench/src/main.rs` only if the RED test identifies a defect.

**Step 1: Build a disposable Cargo fixture in `tempfile::TempDir`.**

Fixture requirements:
- initialize Git and configure local identity,
- create a minimal Cargo crate,
- commit baseline,
- run the compiled AEW binary with:

```text
run --name demo sh -c 'cargo test --quiet && echo "tests pass; no regressions"'
```

**Step 2: Assertions.**

- Saved `demo.json` contains `Verified` status for `tests pass`.
- Saved `demo.json` contains `Unsupported` for `no regressions`.
- Command result has exit code 0.
- Transcript/command evidence has a blob path after Phase C.
- Report verdict is `Partial`, never `Clean`.

**Step 3: RED then GREEN.**

```bash
cargo test -p agent-evidence-workbench --test cli_workflows resolved_claims_are_persisted -- --nocapture
```

**Completion claim:** “AEW does not license broad regression claims from one passing test command.”

---

### Task D2: Test transcript and graph import boundaries

**Objective:** Cover the new public import commands with strict absence semantics.

**Files:**
- Modify: `agent-evidence-workbench/tests/cli_workflows.rs`
- Modify: `agent-evidence-workbench/src/main.rs` only if required.

**Tests:**
1. `import-transcript` persists transcript content as a blob and extractable claims.
2. JSON with no recognized metric fields prints `cost_fields=0` and creates no numeric-cost evidence claim.
3. JSON with recognized numeric fields records their JSON Pointer (for example `/events/0/input_tokens`) and exact observed values.
4. Numeric field named `cost` outside the documented Agent Graph result envelope is retained as `untyped_numeric_observation`, not an authoritative cost metric.

**Implementation direction:** Replace `Vec<String>` discovery with a typed observation:

```rust
struct GraphMetricObservation {
    json_pointer: String,
    field: GraphMetricField,
    value: serde_json::Number,
    schema_class: SchemaClass,
}
```

Do not calculate derived cost or fill absent fields with zero.

**Gates:**

```bash
cargo test -p agent-evidence-workbench --test cli_workflows graph_import -- --nocapture
cargo test -p agent-evidence-workbench --test cli_workflows transcript_import -- --nocapture
```

---

### Task D3: Define and implement receipt-chain policy

**Objective:** Either make `previous_receipt_digest` real or remove it until it is real. Do not retain cosmetic chaining.

**Files:**
- Modify: `agent-evidence-workbench/src/receipt.rs`
- Modify: `agent-evidence-workbench/src/main.rs`
- Modify: `agent-evidence-workbench/src/storage.rs`
- Test: `agent-evidence-workbench/tests/receipt_chain.rs`

**Policy decision (adopt this):** A run may be signed repeatedly. Each new receipt records the digest of the prior receipt for the same run, and `latest.json` is an atomic projection; historical receipts are retained by digest.

**Step 1: RED tests.**

- First sign has `previous_receipt_digest: None`.
- Second sign references exact digest of first receipt.
- Tampering any historical receipt causes chain verification failure.
- Same report signed again produces a new timestamped receipt but preserves report digest.
- `verify-receipt` exits nonzero for malformed receipt JSON, digest mismatch, wrong run ID, wrong key, or chain break.

**Step 2: Data layout.**

```text
.aew/receipts/<run-id>/<receipt-digest>.json
.aew/receipts/<run-id>/latest.json
```

Do not overwrite the only signed receipt at `.aew/receipts/<run-id>.json`.

**Step 3: Implement `verify-receipt --chain`.**

When requested, walk predecessor references until `None`, reject loops, missing predecessor blobs, invalid digest filenames, or mismatched run IDs.

**Step 4: GREEN gates.**

```bash
cargo test -p agent-evidence-workbench --test receipt_chain -- --nocapture
cargo test -p agent-evidence-workbench --features semantic-memory
```

**Rollback:** Old single-file receipts remain readable as legacy receipts. New chain receipts are additive; never delete them during rollback.

---

### Task D4: Replace timing-sensitive HMAC tag equality

**Objective:** Use constant-time tag comparison at the local secret boundary.

**Files:**
- Modify: `agent-evidence-workbench/Cargo.toml`
- Modify: `agent-evidence-workbench/src/receipt.rs`
- Test: existing receipt tests plus `receipt_chain.rs`.

**Step 1: Add dependency.**

```toml
subtle = "2"
```

**Step 2: RED test / contract test.**

Keep functional correctness tests; timing cannot be reliably asserted in unit tests. Add a code-level test helper that compares same-length decoded tags and rejects malformed hex before comparison.

**Step 3: Implement.**

Use `subtle::ConstantTimeEq` on decoded 32-byte tags. Reject invalid/lowercase-format violations through typed `Error::Invalid` before comparison.

**Gate:**

```bash
cargo test -p agent-evidence-workbench receipt -- --nocapture
cargo clippy -p agent-evidence-workbench --no-deps --all-features -- -D warnings
```

---

## Phase E — make semantic-memory promotion durable and safe to retry

### Task E1: Add a durable AEW promotion projection

**Objective:** Bind each promoted claim to the canonical semantic-memory fact ID and prevent duplicate promotion attempts from being represented as new success.

**Files:**
- Modify: `agent-evidence-workbench/src/model.rs`
- Modify: `agent-evidence-workbench/src/storage.rs`
- Modify: `agent-evidence-workbench/src/main.rs`
- Create: `agent-evidence-workbench/src/promotion.rs`
- Create: `agent-evidence-workbench/tests/promotion.rs`

**Data model:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionRecordV1 {
    pub schema_version: String,
    pub run_id: String,
    pub claim_id: String,
    pub receipt_digest: String,
    pub fact_id: String,
    pub promoted_at: DateTime<Utc>,
}
```

Store records at:

```text
.aew/promotions/<run-id>/<claim-id>.json
```

**Step 1: RED tests without live embedding.**

- Metadata builder contains run ID, claim ID, signed receipt digest, and `verified: true`.
- Non-verified claim is rejected before store opening.
- Existing matching promotion record returns idempotent result without calling the store adapter.
- Existing promotion record with a different receipt digest fails as a conflict.

Use a trait boundary only for testing:

```rust
#[async_trait]
pub trait FactSink {
    async fn add_fact(&self, namespace: &str, content: &str, metadata: Value) -> Result<String>;
}
```

The production adapter delegates directly to `semantic_memory::MemoryStore`; it must not create a second database.

**Step 2: Implement feature gate.**

The default build returns a clear `semantic-memory feature required` error. The feature build uses `MemoryStore::open` with a real config and has no mock fallback.

**Step 3: Persist after fact creation only.**

- Create canonical fact.
- Atomically write AEW promotion record.
- If record write fails after fact creation, return typed `PromotionPublicationIncomplete` including the fact ID; do not report generic success.

**Step 4: GREEN gates.**

```bash
cargo test -p agent-evidence-workbench --test promotion --features semantic-memory -- --nocapture
cargo test -p agent-evidence-workbench --features semantic-memory
```

**Live gate (explicitly optional):** only run if a real configured embedding provider passes a preflight health check and the target `--memory-dir` is a fresh disposable directory. Record provider/config digest, fact IDs, and cleanup decision. Do not use a user memory directory.

---

### Task E2: Add `aew promotion-status`

**Objective:** Make promotion state observable without querying or reinterpreting semantic-memory internals.

**Files:**
- Modify: `agent-evidence-workbench/src/cli.rs`
- Modify: `agent-evidence-workbench/src/main.rs`
- Modify: `agent-evidence-workbench/src/storage.rs`
- Test: `agent-evidence-workbench/tests/promotion.rs`

**CLI:**

```text
aew promotion-status <run-id>
```

Output per claim: `not_eligible`, `not_promoted`, `promoted`, `publication_incomplete`, or `conflict`, with run/claim/receipt/fact IDs when present.

**Acceptance:** This command reads AEW’s durable projection only. It must not claim semantic-memory fact existence if the AEW record is missing.

---

## Phase F — harden the opt-in Hermes observer

### Task F1: Define observer failure and path-safety contract

**Objective:** Preserve nonblocking behavior while exposing a bounded local diagnostic path.

**Files:**
- Modify: `agent-evidence-workbench/integrations/hermes/aew-observer.py`
- Create: `agent-evidence-workbench/integrations/hermes/test_aew_observer.py`
- Modify: `agent-evidence-workbench/README.md`

**Contract:**
- Observer remains opt-in and never self-registers.
- Input is one JSON object per line.
- `AEW_EVENTS_PATH` must be present; otherwise the observer exits 0 after consuming input and emits one structured `disabled` diagnostic to stderr.
- Refuse paths whose parent directory cannot be created or whose resolved parent escapes the configured workbench root when `AEW_ROOT` is set.
- Each event append is one JSONL write followed by flush; event serialization failure increments an in-process counter and emits a bounded stderr warning (at most one warning per 60 seconds).
- Never write secrets from environment into event records.

**Tests:**
1. valid event appends deterministic JSON with `aew_received_at`;
2. invalid JSON does not crash subsequent valid event handling;
3. missing `AEW_EVENTS_PATH` has no file side effect;
4. write failure returns/records a diagnostic without traceback spam;
5. environment values are not serialized.

**Gate:**

```bash
python3 -m unittest agent-evidence-workbench/integrations/hermes/test_aew_observer.py -v
```

---

## Phase G — produce a portable, inspectable Python wheel

### Task G1: Preflight native-wheel repair tooling without modifying the global Python environment

**Objective:** Make packaging dependencies explicit and isolated.

**Files:**
- Create: `llm-pipeline-python/scripts/build-wheel.sh`
- Create: `llm-pipeline-python/scripts/test-wheel-import.sh`
- Modify: `llm-pipeline-python/README.md` or `pyproject.toml` documentation section.

**Step 1: Create a tool environment.**

Use `uv` with a disposable/managed tool environment; do not use the system `pip`, which targets a mismatched interpreter in this host setup.

The script must fail clearly if `patchelf` is unavailable and explain that `--auditwheel skip` creates a local-only artifact.

**Step 2: Build policy.**

```bash
maturin build --release
```

is the release candidate path. `--auditwheel skip` is permitted only for developer smoke testing and labels output `local-only`.

**Step 3: Test clean import.**

Create an isolated CPython 3.14 environment, install the exact wheel using `uv pip`, then run:

```bash
python -c 'import llm_pipeline; print(llm_pipeline.__file__)'
```

**Step 4: Inspect linkage.**

```bash
unzip -l target/wheels/*.whl
ldd <extracted-extension>.so
```

Record external library references. A release candidate must satisfy the project’s intended portability policy; do not infer portability merely from an import on the build host.

**Gate:** Script returns 0 only for a repaired wheel plus isolated import. A local-only wheel uses a distinct success code/message and cannot enter publish steps.

---

## Phase H — documentation, release packet, and final certification

### Task H1: Expand AEW operational README

**Objective:** Document exact semantics and prevent overclaiming.

**Files:**
- Modify: `agent-evidence-workbench/README.md`

Required sections:
- quickstart with a real test command;
- claim-state meanings: Verified, Supported/PartiallySupported, Unsupported, Contradicted, HeuristicOnly, NotChecked;
- evidence blobs and `--verify-blobs` semantics;
- receipt signing / verification / chain semantics;
- promotion preconditions and idempotency;
- explicit statement that Hermes observer is opt-in and not installed by AEW;
- explicit statement that local wheel import is not portable-wheel certification;
- limitations: regex extraction, no LLM judge, no historical regression proof from a single run, no security/compliance claim.

**Gate:** Every command in README must be run against `/tmp/aew-final-demo` or an equivalent fresh disposable fixture.

---

### Task H2: Build the final evidence packet

**Objective:** Provide rerunnable closure evidence without treating generated reports as canonical source.

**Files:**
- Create: `.hermes/evidence/aew-closure/final-gates.txt`
- Create: `.hermes/evidence/aew-closure/demo-transcript.txt`
- Create: `.hermes/evidence/aew-closure/final-state.json`

**Required command matrix:**

```bash
cd /home/sikmindz/Coding/Libraries
cargo fmt -p agent-evidence-workbench -- --check
cargo test -p agent-evidence-workbench
cargo test -p agent-evidence-workbench --features semantic-memory
cargo clippy -p agent-evidence-workbench --all-features -- -D warnings
cargo test -p llm-pipeline --test e2e_provenance --all-features
```

Then run the fresh disposable AEW demonstration:

```bash
# in a new temporary Git/Cargo repository
aew init .
aew run --name demo sh -c 'cargo test --quiet && echo "tests pass; no regressions"'
aew evidence demo --verify-blobs
aew sign demo --key-hex <test-key>
aew verify-receipt demo --key-hex <test-key> --chain
aew adjudicate demo
```

**Acceptance assertions:**
- `tests pass` is supported/verified only with an observed passing test command.
- `no regressions` remains unsupported.
- every expected evidence blob verifies.
- valid receipt and full chain verify.
- wrong key, tampered blob, tampered receipt, and invalid promotion all exit nonzero.
- no live memory promotion occurs unless the separate live preflight was explicitly authorized and passed.

**Final report must distinguish:**
- locally observed and verified;
- source-reported / agent-reported;
- blocked / skipped;
- safe public claims vs prohibited claims.

---

## Dependency order and rollback map

```text
A preflight/ownership
  -> B literal Clippy unblock
  -> C durable evidence
  -> D behavior + receipt chain
  -> E promotion projection
  -> F observer hardening
  -> G wheel portability
  -> H documentation + certification
```

| Phase | Rollback boundary |
|---|---|
| B | Revert only the five authorized lint hunks; preserve all unrelated semantic-memory work. |
| C | Stop writing new blobs; retain old reports and blobs. Fields are additive/serde-defaulted. |
| D | Preserve all signed receipts; legacy single receipts remain readable. Never delete receipt history. |
| E | Disable promotion command/feature; do not delete canonical semantic-memory facts automatically. Use a separate governed supersession/retraction path if needed. |
| F | Do not register the observer; remove only the optional script/config documentation. |
| G | Keep local-only wheel as a developer artifact; do not publish it. |
| H | Documentation/evidence-only rollback has no runtime side effect. |

---

## Completion definition

This plan is complete only when all of the following are true:

1. The literal AEW full dependency-tree Clippy command passes or an explicit ownership decision records why Phase B was not authorized.
2. Every displayed evidence digest resolves to an immutable local blob that rehashes correctly.
3. CLI workflows covering run, import, graph absence, sign, tamper rejection, receipt chain, and promotion preconditions execute as automated tests.
4. Repeated promotion is idempotent at the AEW projection boundary and records canonical fact IDs after actual canonical writes.
5. The Hermes observer remains opt-in, has tested failure behavior, and is not globally installed.
6. The Python wheel is either repaired and isolated-import tested or clearly retained as local-only; no ambiguous “built” release claim remains.
7. Final rerunnable evidence files record exact commands, features, observed results, skipped live gates, and remaining blockers.

## Safe final claim

After every required gate passes: “AEW is a local-first CLI that captures immutable local evidence artifacts, applies deterministic skeptical claim checks, produces HMAC-verifiable workbench receipts, supports explicit import and optional canonical promotion, and keeps unproven claims unsupported.”

## Claims still prohibited without additional evidence

- enterprise readiness;
- compliance certification;
- security guarantee;
- historical no-regression proof from one run;
- successful live semantic-memory promotion unless the real configured backend and canonical fact IDs were observed;
- portable Python wheel distribution until repaired-wheel checks pass.
