# SCR Runtime P32 super-pass rules

Run id: `P32-SCR-RUNTIME-SUPERPASS`.

## Required behavior

- Inspect current files before edits.
- Use `docs/P32_COMMAND_RECEIPTS.md` from the first command onward.
- Do not claim completion without passing gates.
- Do not scan opaque refs for signal/control truth.
- Do not invent external owner-crate integration.
- Use typed adapter seams where external owners are unavailable.
- Keep generation and verification commands separate.
- Record golden fixture rationale in `docs/P32_POLICY_CHANGE_RECEIPT.md`.
- End with hostile-auditor handoff.

## Required commands before final completion

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/scr_superpass_static_gates.py final
bash scripts/scr_superpass_run_all.sh final
```
