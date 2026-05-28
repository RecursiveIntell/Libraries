# 08 — CI and Commands

## Local commands

```bash
cargo fmt --all
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
bash scripts/verify.sh
```

`bash scripts/verify.sh` is the single release gate. It includes the example compile/test smoke through `scripts/check_examples.sh`, so operator-facing examples stay exercised in CI.

## Boundary/dependency commands to add later

```bash
cargo tree -p aidens-contracts
cargo tree -p aidens-runner
cargo tree -p aidens-app-kit
```

Expected findings:

- `aidens-contracts` has no runtime/app/shell deps.
- `aidens-runner` has no Tauri/daemon/UI deps.
- `aidens-tool-kit` has no app-specific Recall tools.
- `aidens-memory-kit` does not depend on provider/tool loop crates unless a very explicit adapter seam is created.

## CI workflow

The included `.github/workflows/ci.yml` runs `bash scripts/verify.sh`. Add nextest only after it can preserve the same fake-ready, scaffold-promotion, schema, dependency-boundary, and example fixture checks.
