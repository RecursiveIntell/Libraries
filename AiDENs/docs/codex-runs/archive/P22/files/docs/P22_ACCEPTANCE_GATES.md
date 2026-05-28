# P22 Acceptance Gates

## Hard gates

P22 is failed if any hard gate fails.

```bash
python3 scripts/p22_zpy_archival_selftest.py
python3 scripts/assert_p22_zpy_archive_contract.py z.py
python3 scripts/assert_p22_codex_archival_hygiene.py .
P22_REQUIRE_CARGO=1 bash scripts/p22_verify.sh
bash scripts/p22_verify_release_archive.sh target/p22/aidens-p22-release-context.zip
python3 z.py --root . --profile aidens --mode codex-context --strict
python3 z.py --root . --profile aidens --mode audit-full --include-codex-archive --strict --dry-run
```

## Cargo gates

```bash
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Package gates

Normal package must:

- include AiDENs source and external path dependency roots required by Cargo;
- exclude `docs/codex-runs/archive/**` unless explicit audit mode;
- exclude `.codex_evidence/**` from normal context;
- exclude old `docs/p20`, `docs/p21`, `prompts/p21`, `handoffs/p21` active paths;
- include `docs/codex-runs/CODEX_RUN_INDEX.md` and `CURRENT_RUN.md` if present;
- emit manifest/report/excluded/findings sidecars.

## Documentation gates

Docs must not state unsupported features as supported. Known partial/deferred surfaces must remain disclosed.
