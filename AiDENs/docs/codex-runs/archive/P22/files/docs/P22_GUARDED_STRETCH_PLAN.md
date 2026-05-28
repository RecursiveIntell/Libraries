# P22 Guarded Stretch Plan

Stretch work is allowed only after P22 core gates are green.

## Allowed

- Improve `doctor`, `status`, `provider-check`, `tools inspect`, `package examples` truth output.
- Add JSON report modes if low-risk.
- Improve operator docs for normal vs audit-full packaging.
- Improve receipt/report redaction.
- Add tests proving support-tier labels.

## Not allowed

- Cloud provider execution promotion.
- Native provider tool loops.
- Full daemon UX/socket/timer loop.
- Multi-agent fanout.
- Federation or mechanism runtime product flows.
- Any new local substitute for canonical stack crates.

## Stretch acceptance

Every stretch improvement must have a test, support-tier doc, and rollback path. Otherwise it is removed or deferred.
