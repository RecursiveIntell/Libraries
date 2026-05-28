# P21 Source Basis and Code-First Audit

## Inputs supplied by operator

- AiDENs latest: `libraries-source-clean-20260430.zip`
- Canonical libraries: `libraries 4/28.zip`
- Legacy/satellite libraries: `libraries2 4/28.zip`
- Recall: `recall 4/26.zip`
- Recall-Coding: `recall-coding 4/25/26.zip`
- Research corpus: `Full Provenance+ Research 4/26/26.zip`

## Code-first inspection summary

The latest AiDENs archive contains a real workspace with these relevant signals:

- root directories include `scripts/`, `evals/`, `fixtures/`, `tests/fixtures/`, `crates/`, `examples/`, `schemas/`, `prompts/`, and `handoffs/`;
- `evals/p20_agency_eval_cases.jsonl` exists;
- `crates/aidens-integration-tests` exists;
- `crates/aidens-testkit` is now separate from production-heavy integration tests;
- `scripts/p20_2_verify.sh` and package-integrity scanners exist;
- `fixtures/test-agent/basic-agent.toml` and runner expected event fixtures are referenced by integration tests;
- CLI already has many surfaces: `new`, `run`, `provider-check`, `tools inspect`, `plan compile`, `plan validate`, `package`, `schemas`, `coding`, `daemon/queue`, `memory`, and `receipts`.

## Current actual next target

The project should now move from repair to **usable agent-builder proof**.

The next high-ROI target is not more library wiring. It is:

- top-level `run-test-agent` operator command;
- generated agent projects that run;
- plan/profile system made product-usable;
- provider/tool truth made impossible to fake;
- agency governance upgraded from v0.1 heuristic to v0.2 eval-backed gate;
- Recall/Recall-Coding patterns extracted into AiDENs profiles and templates without importing application-specific assumptions;
- release archive replay certification.

## Immediate risk

Do not expand into multi-agent fanout or native cloud providers until the usable-agent and archive-replay gates pass. If the test-agent command and generated-agent command cannot run cleanly, provider expansion will multiply ambiguity.
