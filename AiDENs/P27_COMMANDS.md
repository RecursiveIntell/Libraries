# P27 Commands

## Preflight

```bash
mkdir -p target/p27/audit handoffs/p27 docs/p27
find scripts .github/workflows -type f -maxdepth 3 -print | sort | tee target/p27/audit/script_files.txt
grep -RIn "p[0-9][0-9]_verify\|verify_current\|verify.sh" scripts .github/workflows 2>/dev/null | tee target/p27/audit/verifier_refs.txt
```

## Current verifier

```bash
bash scripts/verify_current.sh 2>&1 | tee target/p27/audit/verify_current.log
```

Optional strict cargo mode:

```bash
P27_REQUIRE_CARGO=1 bash scripts/verify_current.sh 2>&1 | tee target/p27/audit/verify_current_require_cargo.log
```

## Rust checks, when toolchain is present

```bash
cargo fmt --all -- --check 2>&1 | tee target/p27/audit/cargo_fmt.log
cargo check --workspace --all-targets 2>&1 | tee target/p27/audit/cargo_check.log
cargo test --workspace --all-targets 2>&1 | tee target/p27/audit/cargo_test.log
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee target/p27/audit/cargo_clippy.log
cargo doc --workspace --no-deps 2>&1 | tee target/p27/audit/cargo_doc.log
```

## Package

```bash
python3 z.py --root . --profile aidens --mode next-codex-context --strict \
  --codex-current-run P27 \
  --output target/p27/package/AiDENs-p27-codex-context.zip \
  2>&1 | tee target/p27/audit/zpy_package.log
```

## Self-replay attempt

```bash
python3 scripts/assert_package_self_replay.py --package target/p27/package/AiDENs-p27-codex-context.zip \
  --verifier scripts/verify_current.sh \
  --receipt-out target/p27/audit/package_self_replay_receipt.json \
  2>&1 | tee target/p27/audit/package_self_replay.log
```

If the package self-replay script does not support this exact interface, update the script first; do not fake the result.

## Useful static checks

```bash
python3 scripts/assert_p27_verifier_surface.py .
python3 scripts/assert_p27_current_run_truth.py .
python3 scripts/assert_p27_ownership_scanner_fail_closed.py .
python3 scripts/assert_p27_agents_md_current.py .
```
