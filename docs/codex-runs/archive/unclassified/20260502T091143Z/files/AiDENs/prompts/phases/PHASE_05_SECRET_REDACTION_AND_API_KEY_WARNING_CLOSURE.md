# Phase 05 — Secret Redaction and API-Key Warning Closure

## Tasks

The current certifier warns on Rust field-copy lines like `api_key: provider.api_key.clone()`. Fix this precisely without weakening secret detection.

Required behavior:

- z.py must still warn/error for literal secrets, hardcoded tokens, `.env`, credential filenames, and high-risk assignments.
- z.py should not warn for non-literal Rust field forwarding/copying patterns where no secret value is present.
- Provider/tool reports must redact API keys and never print them in logs, receipts, archive manifests, or package examples.
- Add fixture tests for allowed field-copy and disallowed literal secret.

## Acceptance Gate

```bash
python3 z.py --root . --profile aidens --mode codex-context --strict --dry-run
python3 scripts/p22_secret_scan_fixture_test.py  # create if needed
```

Findings must not include the two previous false-positive `secret-content-named-secret-assignment` warnings, and literal-secret fixtures must still fail/warn as designed.
