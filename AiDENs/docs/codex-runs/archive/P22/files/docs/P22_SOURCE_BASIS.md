# P22 Source Basis

## Uploaded source package facts used to scope P22

The latest package basis is `AiDENs-aidens-codex-context-20260501.zip`.

Observed from the certifier/report and package inspection:

- `z.py` exists and is the current source certifier script.
- Current strict packaging reported 0 validation errors and 2 warnings.
- Warnings are `secret-content-named-secret-assignment` at:
  - `AiDENs/crates/aidens-app-kit/src/lib.rs` line 370
  - `AiDENs/crates/aidens-cli/src/lib.rs` line 2777
- Package includes active `.codex` prompt/task files from earlier runs.
- Package includes active `.codex_evidence` historical run evidence.
- Package includes P20/P21 prompt/doc/handoff surfaces in active context.
- `z.py` currently defaults `include_codex_artifacts` to true for `codex-context`/`full-context`.
- P21 final audit claims build/test/clippy and release replay passed, but raw target logs are generated artifacts and not necessarily normal source-package content.

## P22 scoping conclusion

The main remaining risk is not missing architecture. It is active-surface contamination and release-truth drift. P22 must make normal packaging clean by construction.
