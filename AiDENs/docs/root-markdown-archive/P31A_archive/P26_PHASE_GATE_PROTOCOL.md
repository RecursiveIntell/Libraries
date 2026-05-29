# P26 Phase-Gate Protocol

P26 uses every-other-phase gates to reduce overhead while preserving drift control.

## Mandatory gates

- After Phase 01, before Phase 02
- After Phase 03, before Phase 04
- After Phase 05, before Phase 06
- After Phase 07, before Phase 08
- After Phase 09, before final Phase 10

## Required gate behavior

At each gate Codex must:

1. stop execution;
2. emit the phase report;
3. list changed files;
4. list commands and results;
5. revalidate invariants;
6. identify risks and quarantines;
7. wait for the operator’s pasted gate prompt.

Proceeding without the gate prompt is a run violation.

## Gate invariant checklist

- AiDENs remains consumer-only.
- No canonical sibling-truth semantics invented locally.
- No shadow memory/database/cache truth introduced.
- AgentSpec and RunBundle artifacts are support/display/execution evidence unless delegated to canonical crates.
- z.py scope remains bounded.
- Cloud/autonomy/V10+ claims remain deferred.
- Failures are recorded, not papered over.
