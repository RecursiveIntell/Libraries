# P28 Acceptance Gates — v11A Constitutional Material-Operation Kernel

## Release label target

P28 may claim at most:

- `p28-supported-local-plus`
- `v11A-conformant-core:declared-local-agent-path` only if all v11A gates below pass
- `v11B-draft` only for non-authoritative DTOs/tests
- `v11C-reserved` only if no incompatible horizon shadows are introduced

P28 MUST NOT claim:

- production-cloud-ready
- broad-autonomy-ready
- full v11+ release candidate
- canonical memory truth owner
- canonical governance truth owner
- canonical kernel truth owner
- canonical provider/tool contract owner
- canonical schema-generation owner

## Gate A — Source and scope lock

- The pass records source package hash/report sidecar.
- P27 evidence baseline is copied into P28 source basis.
- Claude 72-bug audit is absorbed into `P28_BUG_ABSORPTION_MATRIX.*`.
- All unsupported feature ambitions are listed as non-goals.

## Gate B — P0 bug closure

All P0 Claude findings must be fixed or explicitly quarantined with release-blocking status:

- C05 lease identity timestamp fallback
- C07 `z.py safe_relative` symlink/fallback bug
- C11 profile expansion panic
- C24 dirty directory creation on failed patch
- C25 symlink escape on new file target
- C32 random artifact IDs where deterministic replay is expected
- C53 package manifest/upload naming verification gap
- C54 empty known-limitations register blocks completion
- C55 waiver satisfied despite blocked state
- C59 convergence/degraded inconsistent dual fields
- C66 history preservation digest logic
- C72 top-level exact status with degraded subcheck

## Gate C — Artifact lifecycle

The harness verifies every material artifact on declared paths has:

- family
- version
- identity
- digest or canonical ref
- recorded time
- authority class
- lifecycle state
- transition receipt for state changes

Promotion from non-eligible states is blocked.

## Gate D — Material operation registry

For each declared production operation, verify:

- `OperatorContractV1` exists or an admitted AiDENs facade exists.
- input artifact families are declared.
- output artifact families are declared.
- allowed and forbidden effects are declared.
- preconditions/postconditions exist.
- proof obligations exist.
- replay requirements exist.
- failure taxonomy exists.

Minimum declared production path:

```text
AgentSpecV1 validate/doctor
→ runner Plan/Act/Verify
→ repo read/list/stat/search/propose/apply/check
→ final report/run bundle/package replay
```

## Gate E — Execution evidence

A material done state is release-blocking unless all exist:

- `ExecutionContextEnvelopeV1`
- `OperatorInvocationReceiptV1`
- per-call `ToolCallReceiptV1`
- input/output manifests
- budget/deadline status
- replay handle or non-replayability reason
- degradation/proof debt/waiver refs if relevant

## Gate F — Boundary compiler

Every structured boundary has a profile and tests for:

- strict syntax validation
- duplicate JSON key rejection
- schema validation and schema identity
- unknown-field policy
- canonicalization/digest stability
- resource ceilings
- repair receipt
- treatment-integrity receipt
- malformed input quarantine/rejection

## Gate G — Proof economy

The harness verifies:

- risk-bearing artifacts have proof profiles
- missing proof becomes proof debt or explicit waiver
- waiver is never treated as proof
- proof debt restricts allowed use
- refuted artifacts cannot promote unless repaired/superseded
- release readiness blocks degraded or blocked surfaces unless waived explicitly

## Gate H — View and temporal disclosure

The harness verifies:

- query/retrieval/report surfaces expose exact/degraded/support labels
- `ViewDisclosureV1` is emitted on widening
- `DegradationRecordV1` is emitted on guarantee weakening
- bitemporal reference fixtures exist for declared memory/query paths
- stale projection cannot answer as current without disclosure

## Gate I — Receipt/store immutability

- run bundles are attempt/content addressed, not overwritable by run id alone
- event logs have sequence numbers and previous-record digests
- package hash is clearly labeled as zip-byte hash, not canonical content hash
- P28 status manifest references the actual uploaded package naming scheme

## Gate J — Final verification

Required final commands:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
P28_FINAL_STRICT=1 bash scripts/verify_current.sh
python3 z.py --root . --profile aidens --mode next-codex-context --strict --codex-current-run P28 --output target/p28/package/AiDENs-p28-codex-context.zip
python3 scripts/assert_package_validation.py
python3 scripts/assert_package_self_replay.py --package target/p28/package/AiDENs-p28-codex-context.zip --verifier scripts/verify_current.sh --require-verifier --receipt-out target/p28/audit/package_self_replay_p28_final_receipt.json
```

Skip-cargo replay may exist, but if it is degraded, the aggregate semantic status must not claim exact check.
