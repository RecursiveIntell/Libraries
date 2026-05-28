# P25 Command Plan

Codex should adapt commands to the repo, but default command expectations are:

```bash
python z.py --profile aidens --mode next-codex-context --dry-run
python z.py --profile aidens --mode next-codex-context --archive-root-markdown-noise --root-markdown-archive-dry-run
python scripts/assert_phase_gate_integrity.py
python scripts/assert_root_markdown_archive_policy.py
python scripts/assert_current_run_truth.py
bash scripts/p25_verify.sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
```

If an environment lacks cargo or Python dependencies, Codex must report that limitation explicitly and not mark the gate as passed.
