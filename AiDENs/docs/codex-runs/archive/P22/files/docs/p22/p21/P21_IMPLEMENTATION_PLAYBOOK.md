# P21 Implementation Playbook

## Execution order

P21 is a superpass, but it is still gated. Do not reorder phases unless a phase cannot start due to a prior failure.

1. Phase 00: package/source closure.
2. Phase 01: build certification.
3. Phase 02: `run-test-agent` CLI.
4. Phase 03: generated agent project proof.
5. Phase 04: profile + plan-kit usability.
6. Phase 05: provider/tool capability certification.
7. Phase 06: agency governance v0.2.
8. Phase 07: Recall/Recall-Coding extraction.
9. Phase 08: archive replay certification.
10. Phase 09: guarded stretch work.
11. Phase 10: final audit and handoff.

## Phase 00 details

Run scanners first. If missing files exist, restore them. Do not delete tests. Do not mark missing fixtures as deferred unless the referenced feature is also demoted and no code path references the fixture.

## Phase 01 details

Only compile/test/clippy fixes. No feature expansion. Keep fixes local and minimal.

## Phase 02 details

Implement CLI command:

```text
run-test-agent <config> [--prompt <prompt>] [--out <dir>]
```

The command should reuse existing runner/test-agent fixtures and produce an output directory with:

- `final.txt`
- `run-report.json`
- `turn-report.json`
- `tool-exposure.json`
- `agency-policy-reports.json`
- `event-log.ndjson`
- `summary.md`

Do not create a separate runner path that bypasses real `AiDENsRunner`.

## Phase 03 details

Improve `aidens new` so generated projects are genuinely runnable. The generated app should default to safe/disabled/mock provider mode and should not enable write/admin tools by default.

## Phase 04 details

Make profile/plan state usable:

- `chat-only`: supported, no tools by default, agency gate enabled;
- `coding-agent`: supported, safe read/search tools by default, write/admin permit-gated;
- `memory-agent`: partial/proof-only, canonical memory adapter only;
- `daemon`: partial, no timer storms, safe-mode default;
- `research`: deferred or example-only.

`aidens-plan-kit` should stop being an 18-line placeholder. It should own execution-plan assembly only, not memory/kernel truth.

## Phase 05 details

Provider/tool truth must be explicit:

- mock: executable fixture provider;
- ollama: executable chat-only if configured;
- openai/anthropic/openrouter: unavailable unless implemented and tested;
- no native tool-loop claim without executable tests.

## Phase 06 details

Expand agency evals to cover at least:

- high-impact recommendation;
- medical/legal/financial caution;
- employment/life decisions;
- repeated nudging;
- memory personalization influence;
- fake urgency;
- sycophancy/overvalidation;
- emotional dependence;
- tool-output persuasion risk;
- subagent influence aggregation.

## Phase 07 details

Inspect Recall and Recall-Coding to extract reusable behavior patterns:

- daemon/heartbeat/safe-mode lessons;
- IPC/session startup lessons;
- coding-agent UI/CLI workflows;
- provider fallback pitfalls;
- tool routing and approval queue patterns;
- job/timer storm prevention;
- no app-specific assumptions in AiDENs core.

Produce docs and templates. Only implement code where it is directly useful to AiDENs agent-builder flows.

## Phase 08 details

Create and verify a release archive. This phase exists because prior passes repeatedly produced local states whose zips omitted fixtures/scripts/evals.

## Phase 09 stretch rules

Stretch work is allowed only after Phase 08 passes. Stretch work must be isolated and revertible. Prefer examples/templates over deep provider expansion unless all gates remain green.

## Phase 10 final audit

Final output must be code-first. Docs are audited only to ensure they match actual code and supported commands.
