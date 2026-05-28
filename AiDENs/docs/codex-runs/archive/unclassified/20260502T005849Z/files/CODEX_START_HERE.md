# AiDENs P20.2 — Code Package Closure, Test Agent Proof, and v0.1 Certification

Use this bundle for the next Codex run. It supersedes prior P20/P20.1 handoffs.

## Mission

Take AiDENs as far as possible **without destabilizing the canonical foundation**:

1. close the actual code/package blockers;
2. split/purify the test topology;
3. prove a canonical test agent end-to-end;
4. certify v0.1 release readiness;
5. only then execute the guarded stretch lane.

This is a **code-first pass**. Documentation must be corrected, but documents are not proof. Build/test output, package integrity scans, integration tests, and generated audit artifacts are proof.

## Install

```bash
unzip aidens_p20_2_test_agent_v0_1_handoff_20260430.zip
cd aidens_p20_2_test_agent_v0_1_handoff_20260430
bash install_p20_2_overlay.sh /home/sikmindz/Coding/Libraries/AiDENs
cd /home/sikmindz/Coding/Libraries/AiDENs
P20_2_REQUIRE_CARGO=1 bash scripts/p20_2_verify.sh
```

## Run Codex

Paste:

```text
prompts/P20_2_CODEX_RUN_PROMPT.md
```

Between every phase paste:

```text
prompts/phase_injections/GLOBAL_PRE_PHASE_INVARIANT_REVALIDATION.md
```

then paste the matching phase-specific prompt from:

```text
prompts/phase_injections/PHASE_XX_*.md
```

Do not rely on the global invariant alone. The phase injections are the anti-drift mechanism.

## Required end condition

The pass is successful only if it produces a source/package state where:

- every `include_str!`/`include_bytes!` target exists;
- `MANIFEST.txt` entries exist or are corrected;
- `evals/p20_agency_eval_cases.jsonl` exists and validates;
- `aidens-testkit` is pure/reference-only;
- production integration tests live in `aidens-integration-tests` or equivalent;
- the canonical test agent runs through provider/tool/permit/boundary/agency/receipt paths;
- `scripts/p20_2_verify.sh` passes in the real workspace;
- final audit artifacts are generated and included in a handoff/release directory.
