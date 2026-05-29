# Remaining Delta

- Install `maturin` and run native Python build validation.
- Run Python native receipt parity, shape rejection, and no-silent-copy tests against an installed `poly_kv._native`.
- Decide if/when `poly-kv` manifests migrate from `KvTensorShape` to `KvCacheShapeV2`.
- Install and run `cargo-semver-checks` before any release gate.
- Add real CPU buffer validation before making broader Python data-interchange claims.
- Keep TurboQuant and FibQuant adapters unsupported until external APIs are inspected and compiled.
- Keep all model-backed benchmark claims out of README until local receipts exist.
