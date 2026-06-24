# P29 Acceptance Gates

## Gate 0 — P28 repair gate

- Current run locked to P29.
- P29 docs/handoffs/scripts active.
- Verifier active.
- No current-run archive drift.

## Gate 1 — Package replay gate

- Package generated in strict mode.
- Extracted package verifies.
- Manifest paths resolve.
- Package self-replay receipt included.

## Gate 2 — Audit absorption gate

- 200 Claude audit bugs imported into matrix.
- P0/P1 items assigned phase/owner/status.
- Unfixed high-risk items quarantined.

## Gate 3 — Runtime correctness gate

- HNSW lock ordering/deleted snapshot/keymap/load bugs addressed or quarantined.
- SQLite migration atomicity bugs addressed or quarantined.
- Search/ranking/dedup bugs addressed or quarantined.

## Gate 4 — v11A local release gate

- Material operations use operator contracts.
- Execution context envelope present.
- Receipts emitted and persisted.
- Proof/debt/waiver/degradation semantics enforced.
- Boundary compiler profiles enforced.
- Semantic/view disclosure visible.

## Gate 5 — v11B seed gate

- Right-graph misuse tests exist.
- Region contract seed exists.
- Convergence/residual/syndrome seed exists.
- Lawful subtraction seed exists.
- No v11B-complete claim.

## Gate 6 — Final command gate

All pass:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
bash scripts/p29_verify.sh
python3 scripts/assert_p29_package_self_replay.py --package target/p29/package/AiDENs-p29-codex-context.zip
```
