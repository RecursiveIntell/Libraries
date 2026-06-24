# P27 Evidence and Reporting Requirements

## Directory conventions

```text
handoffs/p27/
  PHASE_00_REPORT.md
  ...
  FINAL_AUDITOR_HANDOFF.md

docs/p27/
  P27_FINAL_AUDIT_REPORT.md
  P27_KNOWN_LIMITATIONS.md

target/p27/audit/
  phase00_*.log
  verifier_refs.txt
  cargo_*.log
  package_self_replay.log

target/p27/receipts/
  *.json
```

## Phase report minimum fields

Each phase report must include:

- phase name and date;
- files inspected;
- files changed;
- commands run;
- command result summaries;
- evidence artifacts emitted;
- support-tier changes;
- 11A semantic disclosure impact;
- unresolved issues;
- stop/continue/quarantine decision.

## Final evidence manifest

`P27_STATUS_EVIDENCE_MANIFEST.json` should include:

- created timestamp;
- current run;
- verifier outputs;
- cargo outputs;
- package sidecars;
- self-replay status;
- issue matrix closure status;
- support profile digest;
- known limitations digest;
- final auditor handoff path.
