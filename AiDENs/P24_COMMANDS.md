# P24 command plan

Run these in the real repo environment. P24 command evidence is written under `target/p24/audit/` and `target/p24-verifier/`.

## Preflight

```bash
pwd
git status --short || true
python3 --version
rustc --version
cargo --version
python3 z.py --help
```

## Verifier

```bash
bash AiDENs/scripts/p24_verify.sh .
```

## Cargo gates

```bash
cd AiDENs
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

## Product gates

```bash
cd AiDENs
cargo run -p aidens-cli -- run-test-agent fixtures/test-agent/basic-agent.toml --out target/p24/test-agent
cargo run -p aidens-cli -- inspect-run target/p24/test-agent/run-bundle.json
cargo run -p aidens-cli -- run-coding-agent examples/configs/coding-agent.toml --out target/p24/coding-agent
cargo run -p aidens-cli -- inspect-run target/p24/coding-agent/run-bundle.json
```

## Memory seam gate

```bash
cd AiDENs
cargo run -p aidens-cli -- memory seam-fixture --out target/p24/memory-seam
```

## Daemon-safe gate, if promoted

```bash
cd AiDENs
cargo test -p aidens-cli p11_daemon_commands_suppress_duplicates_persist_cancel_and_safe_mode -- --nocapture
```

## Package gates

```bash
python3 z.py --root . --profile aidens --mode codex-context --codex-current-run P24 --strict
P24_PACKAGE_SELF_REPLAY=target/p24/package/AiDENs-p24-codex-context.zip bash scripts/p24_verify.sh .
```

Every command must be reflected in final evidence, even if it fails.
