# Acceptance Gates

## Gate A — Preflight

Pass only if:

- source layout is documented;
- no absolute Cargo paths are introduced;
- `cargo metadata` succeeds for each modified workspace;
- Codex reports whether `turbo-quant` is sibling, external, or unavailable.

## Gate B — TurboQuant hardening

Pass only if:

- profile/digest type exists;
- encoded vector artifact type exists;
- bitpacked QJL signs exist or an explicit deferred warning says why not;
- storage-accounting tests exist;
- profile mismatch/corruption tests exist;
- existing turbo-quant tests still pass.

## Gate C — semantic-memory abstraction

Pass only if:

- generic vector codec trait exists;
- current SQ8/raw behavior is preserved;
- no TurboQuant math copied into semantic-memory;
- feature gate compiles when enabled;
- default feature set compiles without TurboQuant behavior changes.

## Gate D — Shadow mode

Pass only if:

- TurboQuant sidecar encode can run without becoming authoritative;
- raw embedding path still works when shadow encode fails;
- encode receipt/evaluation artifact exists;
- no existing write/search behavior changes by default.

## Gate E — Search disclosure

Pass only if:

- approximate result metadata is visible;
- f32 rerank status is visible;
- degradation/fallback flags are visible;
- approximate results cannot silently masquerade as exact.

## Gate F — Evaluation

Pass only if evaluation records include at minimum:

- corpus/query count;
- codec profile digest;
- recall@k or top-k agreement;
- byte-size accounting;
- latency summary;
- degradation/failure counts.

## Gate G — Final

Pass only if:

- `cargo fmt --check`;
- `cargo clippy` for modified crates/features as feasible;
- `cargo test` for modified crates/features as feasible;
- assertion scripts pass;
- final report is honest about deferred work.
