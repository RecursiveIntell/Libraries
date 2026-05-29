# Risk Register

| Risk | Status | Mitigation |
|---|---|---|
| Python native wheel not built | Open | `maturin` unavailable; tests skip explicitly. Install maturin and run `maturin build` / `maturin develop` in a later pass. |
| Python native tests skipped | Open | Tests exist and record skip reason; native execution remains next-pass validation. |
| `cargo-semver-checks` unavailable | Open | `scripts/run_rust_gates.sh` records skip; install before release gating. |
| `KvPoolManifestV1` still uses legacy `KvTensorShape` | Open | `KvCacheShapeV2` is staged in `quant-codec-core`; migration is documented in `docs/NEXT_RELEASE_PLAN.md`. |
| Existing dirty baseline | Open | Initial and final git status recorded; user-owned baseline changes were not reverted. |
| Bench harness is synthetic/status-only | Accepted | JSON artifacts contain raw command/status data and no benchmark claims. |
