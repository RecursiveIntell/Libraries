# Optional Codex Config and Commands

These are optional. Do not use them if they conflict with your local setup.

## Git checkpoint

```bash
git status --short
git switch -c p31-v11a-boundary-compiler
# or: git checkout -b p31-v11a-boundary-compiler
```

If the repo has uncommitted work, stash or commit intentionally before the Codex run.

## Basic Codex run

From the repository root:

```bash
codex
```

Then paste `00_P31_CODEX_SUPER_PASS_PROMPT.md`.

You can also launch with an initial prompt:

```bash
codex "$(cat /path/to/P31_CODEX_SUPER_PASS_CLIPBOARD.txt)"
```

## Goal mode variant

If your Codex CLI supports `/goal`, paste `01_P31_GOAL_MODE_PROMPT.md`.

Use goal mode only when you are comfortable letting Codex continue across validation loops. The stopping condition is deliberately concrete: targeted tests pass and the report exists.

## Suggested local validation commands

After Codex finishes, run the commands it reported. Typical forms:

```bash
cargo test --manifest-path <chosen-crate>/Cargo.toml
cargo fmt --manifest-path <chosen-crate>/Cargo.toml --check
```

If the crate is integrated cleanly into a workspace:

```bash
cargo test -p <crate-name>
cargo fmt -p <crate-name> --check
```

Optional:

```bash
cargo clippy --manifest-path <chosen-crate>/Cargo.toml --all-targets -- -D warnings
```

## Review command set

```bash
rg -n "BoundaryCompilerProfileV1|ParseReceiptV1|RepairReceiptV1|TreatmentIntegrityReceiptV1|BoundaryCompileResultV1" .
rg -n "duplicate|last.write|canonical|NoRepair|RepairedAccept|treatment" <chosen-crate>
cat docs/codex-runs/P31_BOUNDARY_COMPILER_MICROKERNEL_REPORT.md
```

## Optional `.codex/config.toml` idea

Project-local config is useful when you want repeatable behavior. Do not add a project config unless it is already normal for the repo.

```toml
# .codex/config.toml
# Example only. Check your installed Codex version before relying on exact keys.

# Use a profile or model setting appropriate for implementation-heavy work.
# Keep permissions tight enough that shell commands remain reviewable.
```
