# P25 Phase Gate Protocol

## Gating model

P25 uses **every-other-phase human gates**.

The operator manually injects guardrail prompts after:
- Phase 01, before Phase 02
- Phase 03, before Phase 04
- Phase 05, before Phase 06
- Phase 07, before Phase 08
- Phase 09, before final closure

## Why every-other-phase is allowed

Every gate must validate all work since the previous gate. This reduces operator friction while still preventing context drift.

## Gate contract

At a gate, Codex must:

1. stop execution;
2. emit a phase report;
3. list changed files;
4. list commands run and command outputs;
5. revalidate all active invariants;
6. identify unresolved risks;
7. state whether it is safe to proceed;
8. wait for the operator’s pasted phase-injection prompt.

## Required injection language

Every active P25 phase-injection file must include:

```text
STOP. Do not proceed until this injection is pasted by the operator.
```

## Violation handling

If Codex proceeds without the gate injection:
- mark a run violation,
- quarantine post-gate work,
- emit a violation record,
- wait for operator approval.

## Machine-check requirement

P25 must add or update a verifier that fails if:
- active phase-injection files reference stale prior-run paths or run IDs,
- active phase-injection files omit STOP/WAIT language,
- configured gate sequence and phase plan disagree,
- final handoff lacks gate compliance evidence.
