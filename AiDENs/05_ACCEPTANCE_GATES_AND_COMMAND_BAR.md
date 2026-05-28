# Acceptance Gates and Command Bar

## Gate 0 — source-basis reconciliation

- The finished user bundle is treated as clean source basis.
- Any stale in-repo `blocked_missing_package` claim is reconciled as evidence hygiene, not source failure.
- Current super-pass changes require a new package and replay at the end.

## Gate 1 — Rust command bar

Run, at minimum, from the final workspace root:

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
```

If feature flags exist, run the strictest supported combination and record exact commands.

## Gate 2 — receipt/done-state gate

- No user-visible final/done state can be written before durable receipts.
- Receipt chains verify under normal and concurrent append cases.
- Corrupt receipt records quarantine instead of poisoning all history.

## Gate 3 — sandbox/security gate

Required hostile fixtures:

- `.git/config`, `.env`, `.npmrc`, `.aws`, `.ssh`, `.git-credentials` read denial.
- Symlink and hardlink read/write escape attempts.
- Unicode normalization and case-folding collisions.
- TOCTOU path swap during write.
- Absolute-path redaction in user-facing reports.

## Gate 4 — transactional patch gate

- Multi-file patch is atomic or explicitly quarantined.
- Before/after digests recorded for every touched path.
- Post-write verification required.
- Rollback/quarantine receipt required on partial failure.
- Narrow string-replacement mode is either removed or relabeled truthfully.

## Gate 5 — command execution gate

- Structured argv, no whitespace parser for shell-like strings unless explicitly shell-mode.
- Process group timeout/kill.
- Output caps.
- Environment/toolchain/source/package fingerprints.
- Replay handle and command receipt.

## Gate 6 — provider honesty gate

- `Local` cannot silently mean `mock`.
- Provider route records exact provider/model/endpoint/tool capability.
- Parser fallback vs native tool mode is disclosed.
- Network permit is explicit.
- Tool results are actually routed to providers that claim tool use.

## Gate 7 — queue/concurrency gate

- Concurrent enqueue/lease/complete tests pass.
- Lease ownership is enforced.
- Late completion after lease expiry is rejected or quarantined.
- Queue logs are lock-safe or single-writer.

## Gate 8 — boundary compiler gate

- Strict parser rejects duplicate keys, unknown fields under strict profiles, invalid schemas, and material repair that changes treatment-critical fields.
- Unsupported schema features are rejected explicitly.
- Repair receipts are durable and queryable.

## Gate 9 — temporal/proof/view gate

- Reference fixture corpus covers valid-time, recorded-time, combined as-of, retroactive correction, supersession, conflict, stale projection, view widening, proof debt, refutation, and proof-waiver-not-proof.

## Gate 10 — v11B minimal region gate

- Distinct right graph declared.
- Region contract exists.
- Convergence, non-convergence, oscillation, residual/syndrome, local repair, support-core protection, and oracle diff fixtures pass.
- Final label remains `v11B-seed` or equivalent, not complete.

## Gate 11 — HNSW/search/pool gate

- HNSW map locking/TOCTOU/dirty-flag ordering/ID exhaustion risks fixed or quarantined.
- Vector scan warning becomes hard circuit breaker/degraded path above configured threshold.
- Timestamp parsing fallback/warning added.
- Reader timeout call sites audited and tested.

## Gate 12 — unaudited surface gate

- forge-pilot, effect-runtime, verification pipeline, federation, attestation, authority-delegation, and recursive-kernel-core are audited, tested, or quarantined from supported labels.

## Gate 13 — final docs and package gate

- Known limitations register populated.
- Final auditor handoff populated.
- Issue matrix contains no raw `open` rows for supported scope.
- Package sidecars generated from exact final tree.
- Extracted-package self-replay passes.

## Forbidden gate substitutions

The following do not satisfy gates:

- Marker-string assertions without behavioral fixtures.
- A green package certifier without extracted replay after changes.
- Docs claiming support while issue matrix has raw `open` blockers.
- Waiver receipts treated as proof.
- Mock/local route ambiguity.
