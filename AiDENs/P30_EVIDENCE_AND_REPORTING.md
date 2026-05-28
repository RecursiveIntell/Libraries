# P30 Evidence and Reporting Requirements

## Per phase

Each `handoffs/p30/P30-XX_REPORT.md` must include:

- phase goal;
- issue rows addressed;
- changed files;
- tests/commands run;
- output excerpts or paths to logs;
- invariant checklist;
- unresolved risks;
- next phase go/no-go.

## Final

Final handoff must include:

- issue absorption report;
- command log;
- package/build/conformance/release claim separation;
- exact release claims allowed;
- exact claims forbidden;
- auditor replay instructions;
- unresolved risks and quarantines;
- evidence manifest.

## Claim labels

Use only these labels unless a stronger one is proven:

- `package-certified`: source package certifier passed.
- `build-certified`: cargo command bar passed with logs.
- `conformance-certified`: required reference/conformance fixtures passed.
- `release-certified`: build + conformance + package + final gates passed.
- `static-audit-hardened`: static guard/audit issues reduced but not fully certified.

Never conflate these labels.
