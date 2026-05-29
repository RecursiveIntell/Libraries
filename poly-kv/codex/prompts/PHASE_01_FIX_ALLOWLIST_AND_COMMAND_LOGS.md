# Phase 01 — Fix z.py allowlist and command evidence

Implement:

- `.pyi` in `ALLOWED_TEXT_EXTENSIONS`.
- `py.typed` in `ALLOWED_BASENAMES`.
- Context/audit mode inclusion for safe `.codex-runs/**/commands_run.log` or an equivalent command receipt JSONL.

Do not globally include all logs unless operator explicitly approves. Use a narrow context receipt log rule.

Add tests proving `_native.pyi`, `py.typed`, and `commands_run.log` are included in a fixture manifest.
